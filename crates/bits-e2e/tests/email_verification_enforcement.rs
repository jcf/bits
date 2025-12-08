use bits_e2e::{client::BitsClient, fixtures};

#[tokio::test]
async fn unverified_user_cannot_change_password() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let email = "unverified@example.com";
    let password = "current-password";
    let new_password = "new-password";

    // Create unverified user
    let password_hash = ctx
        .state
        .password_service
        .hash_password(password)
        .expect("Failed to hash password");

    let user = ctx
        .create_user(email, &password_hash)
        .await
        .expect("Failed to create user");

    // Verify user is NOT verified
    let is_verified = sqlx::query_scalar::<_, bool>(
        "select exists(
            select 1 from email_verifications ev
            join email_addresses ea on ea.id = ev.email_address_id
            where ea.user_id = $1
        )",
    )
    .bind(user.id)
    .fetch_one(&ctx.db_pool)
    .await
    .expect("Failed to check verification status");

    assert!(!is_verified, "User should not be verified");

    // Login as unverified user
    let mut client = BitsClient::new(ctx.server.url(""));
    client.fetch_csrf_token().await.unwrap();
    client.login(email, password).await.unwrap();

    // Attempt to change password
    let response = client
        .change_password(&serde_json::json!({
            "current_password": password,
            "new_password": new_password,
            "confirm_password": new_password
        }))
        .await
        .unwrap();

    // Should be forbidden
    assert_eq!(
        response.status(),
        403,
        "Unverified user should not be able to change password"
    );

    let body = response.text().await.expect("Failed to read response");
    assert!(
        body.contains("Email not verified") || body.contains("not verified"),
        "Response should indicate email not verified, got: {}",
        body
    );
}

#[tokio::test]
async fn verified_user_can_change_password() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let email = "verified@example.com";
    let old_password = "old-password";
    let new_password = "new-password";

    // Create verified user
    let (user, _hash) = ctx
        .create_verified_user(email, old_password)
        .await
        .expect("Failed to create verified user");

    // Verify user IS verified
    let is_verified = sqlx::query_scalar::<_, bool>(
        "select exists(
            select 1 from email_verifications ev
            join email_addresses ea on ea.id = ev.email_address_id
            where ea.user_id = $1
        )",
    )
    .bind(user.id)
    .fetch_one(&ctx.db_pool)
    .await
    .expect("Failed to check verification status");

    assert!(is_verified, "User should be verified");

    // Login
    let mut client = BitsClient::new(ctx.server.url(""));
    client.fetch_csrf_token().await.unwrap();
    client.login(email, old_password).await.unwrap();

    // Change password should succeed
    let response = client
        .change_password(&serde_json::json!({
            "current_password": old_password,
            "new_password": new_password,
            "confirm_password": new_password
        }))
        .await
        .unwrap();

    assert!(
        response.status().is_success(),
        "Verified user should be able to change password, got status {}",
        response.status()
    );
}

#[tokio::test]
async fn full_flow_signup_blocked_verify_success() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let email = "flow@example.com";
    let password = "initial-password";
    let new_password = "new-password";

    // Create unverified user
    let password_hash = ctx
        .state
        .password_service
        .hash_password(password)
        .expect("Failed to hash password");

    let user = ctx
        .create_user(email, &password_hash)
        .await
        .expect("Failed to create user");

    // Login as unverified user
    let mut client = BitsClient::new(ctx.server.url(""));
    client.fetch_csrf_token().await.unwrap();
    client.login(email, password).await.unwrap();

    // Attempt password change - should fail
    let response = client
        .change_password(&serde_json::json!({
            "current_password": password,
            "new_password": new_password,
            "confirm_password": new_password
        }))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        403,
        "Should be blocked before verification"
    );

    // Verify email
    ctx.verify_email(user.id)
        .await
        .expect("Failed to verify email");

    // Verify status changed
    let is_verified = sqlx::query_scalar::<_, bool>(
        "select exists(
            select 1 from email_verifications ev
            join email_addresses ea on ea.id = ev.email_address_id
            where ea.user_id = $1
        )",
    )
    .bind(user.id)
    .fetch_one(&ctx.db_pool)
    .await
    .expect("Failed to check verification status");

    assert!(is_verified, "User should now be verified");

    // Re-login to get updated session with verified status
    let mut client2 = BitsClient::new(ctx.server.url(""));
    client2.fetch_csrf_token().await.unwrap();
    client2.login(email, password).await.unwrap();

    // Retry password change - should now succeed
    let response = client2
        .change_password(&serde_json::json!({
            "current_password": password,
            "new_password": new_password,
            "confirm_password": new_password
        }))
        .await
        .unwrap();

    assert!(
        response.status().is_success(),
        "Should succeed after verification, got status {}",
        response.status()
    );
}
