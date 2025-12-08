use bits_e2e::{client::BitsClient, fixtures};

#[tokio::test]
async fn user_stays_signed_in_across_requests() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let email = "user@example.com";
    let password = "secure-password";

    let (user, _hash) = ctx
        .create_verified_user(email, password)
        .await
        .expect("Failed to create user");

    let mut client = BitsClient::new(ctx.server.url(""));
    client.fetch_csrf_token().await.unwrap();

    client.login(email, password).await.unwrap();

    let session1 = client.get_session().await.unwrap();
    assert_eq!(
        session1.as_ref().map(|u| u.id),
        Some(user.id),
        "User should be signed in after login"
    );

    let session2 = client.get_session().await.unwrap();
    assert_eq!(
        session2.as_ref().map(|u| u.id),
        Some(user.id),
        "User should still be signed in on second request"
    );
}

#[tokio::test]
async fn password_change_invalidates_all_other_sessions() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let email = "user@example.com";
    let old_password = "old-password";
    let new_password = "new-secure-password";

    let (user, _hash) = ctx
        .create_verified_user(email, old_password)
        .await
        .expect("Failed to create user");

    let mut client1 = BitsClient::new(ctx.server.url(""));
    client1.fetch_csrf_token().await.unwrap();
    client1.login(email, old_password).await.unwrap();

    let mut client2 = BitsClient::new(ctx.server.url(""));
    client2.fetch_csrf_token().await.unwrap();
    client2.login(email, old_password).await.unwrap();

    let session1 = client1.get_session().await.unwrap();
    assert_eq!(
        session1.as_ref().map(|u| u.id),
        Some(user.id),
        "Client 1 should be signed in before password change"
    );

    let session2 = client2.get_session().await.unwrap();
    assert_eq!(
        session2.as_ref().map(|u| u.id),
        Some(user.id),
        "Client 2 should be signed in before password change"
    );

    let response = client1
        .change_password(&serde_json::json!({
            "current_password": old_password,
            "new_password": new_password,
            "confirm_password": new_password
        }))
        .await
        .unwrap();

    assert!(
        response.status().is_success(),
        "Password change should succeed"
    );

    let session1_after = client1.get_session().await.unwrap();
    assert_eq!(
        session1_after.as_ref().map(|u| u.id),
        Some(user.id),
        "Client 1 should remain logged in after changing their own password"
    );

    let session2_after = client2.get_session().await.unwrap();
    assert_eq!(
        session2_after, None,
        "Client 2 session should be invalidated when user changes password on client 1"
    );
}

#[tokio::test]
async fn valid_code_verifies_email() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let email = "user@example.com";
    let password = "secure-password";

    // Create unverified user
    let user = ctx
        .create_user(email, password)
        .await
        .expect("Failed to create user");

    // Generate verification code
    let email_address_id = ctx
        .get_email_address_id(user.id)
        .await
        .expect("Failed to get email address id");

    let code = ctx
        .state
        .email_verification
        .create_code(&ctx.db_pool, email_address_id)
        .await
        .expect("Failed to create verification code");

    // Verify the code
    let result = ctx
        .state
        .email_verification
        .verify_code(&ctx.db_pool, email_address_id, &code)
        .await;

    assert!(result.is_ok(), "Valid code should verify successfully");

    // Check that email_verifications record was created
    let is_verified = sqlx::query_scalar::<_, bool>(
        "select exists(select 1 from email_verifications where email_address_id = $1)",
    )
    .bind(email_address_id)
    .fetch_one(&ctx.db_pool)
    .await
    .expect("Failed to check verification status");

    assert!(is_verified, "Email should be marked as verified");
}

#[tokio::test]
async fn invalid_code_fails_verification() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let email = "user@example.com";
    let password = "secure-password";

    let user = ctx
        .create_user(email, password)
        .await
        .expect("Failed to create user");

    let email_address_id = ctx
        .get_email_address_id(user.id)
        .await
        .expect("Failed to get email address id");

    ctx.state
        .email_verification
        .create_code(&ctx.db_pool, email_address_id)
        .await
        .expect("Failed to create verification code");

    // Try to verify with wrong code
    let result = ctx
        .state
        .email_verification
        .verify_code(&ctx.db_pool, email_address_id, "000000")
        .await;

    assert!(result.is_err(), "Invalid code should fail verification");
}

#[tokio::test]
async fn expired_code_fails_verification() {
    use bits_app::verification::{EmailVerificationConfig, EmailVerificationService};

    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let email = "user@example.com";
    let password = "secure-password";

    let user = ctx
        .create_user(email, password)
        .await
        .expect("Failed to create user");

    let email_address_id = ctx
        .get_email_address_id(user.id)
        .await
        .expect("Failed to get email address id");

    // Create service with immediate expiration
    let test_config = EmailVerificationConfig {
        code_expiry_hours: -1, // Expired immediately
        max_verification_attempts: 3,
        resend_cooldown_secs: 60,
        max_resends_per_hour: 5,
    };
    let test_secret = b"test-secret-for-verification".to_vec();
    let service = EmailVerificationService::new(test_config, test_secret);

    let code = service
        .create_code(&ctx.db_pool, email_address_id)
        .await
        .expect("Failed to create verification code");

    // Try to verify expired code
    let result = service
        .verify_code(&ctx.db_pool, email_address_id, &code)
        .await;

    assert!(result.is_err(), "Expired code should fail verification");
}

#[tokio::test]
async fn too_many_attempts_blocks_verification() {
    use bits_app::verification::{EmailVerificationConfig, EmailVerificationService};

    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let email = "user@example.com";
    let password = "secure-password";

    let user = ctx
        .create_user(email, password)
        .await
        .expect("Failed to create user");

    let email_address_id = ctx
        .get_email_address_id(user.id)
        .await
        .expect("Failed to get email address id");

    // Create service with only 1 attempt allowed
    let test_config = EmailVerificationConfig {
        code_expiry_hours: 1,
        max_verification_attempts: 1,
        resend_cooldown_secs: 60,
        max_resends_per_hour: 5,
    };
    let test_secret = b"test-secret-for-verification".to_vec();
    let service = EmailVerificationService::new(test_config, test_secret);

    service
        .create_code(&ctx.db_pool, email_address_id)
        .await
        .expect("Failed to create verification code");

    // First wrong attempt
    let _ = service
        .verify_code(&ctx.db_pool, email_address_id, "000000")
        .await;

    // Second attempt should be blocked
    let result = service
        .verify_code(&ctx.db_pool, email_address_id, "111111")
        .await;

    assert!(
        result.is_err(),
        "Verification should be blocked after max attempts"
    );
}

#[tokio::test]
async fn resend_respects_cooldown() {
    use bits_app::verification::{EmailVerificationConfig, EmailVerificationService};

    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let email = "user@example.com";
    let password = "secure-password";

    let user = ctx
        .create_user(email, password)
        .await
        .expect("Failed to create user");

    let email_address_id = ctx
        .get_email_address_id(user.id)
        .await
        .expect("Failed to get email address id");

    // Create service with long cooldown
    let test_config = EmailVerificationConfig {
        code_expiry_hours: 1,
        max_verification_attempts: 3,
        resend_cooldown_secs: 3600, // 1 hour
        max_resends_per_hour: 5,
    };
    let test_secret = b"test-secret-for-verification".to_vec();
    let service = EmailVerificationService::new(test_config, test_secret);

    service
        .create_code(&ctx.db_pool, email_address_id)
        .await
        .expect("Failed to create verification code");

    // Try to resend immediately
    let result = service
        .check_resend_limits(&ctx.db_pool, email_address_id, None)
        .await;

    assert!(result.is_err(), "Resend should be blocked by cooldown");
}
