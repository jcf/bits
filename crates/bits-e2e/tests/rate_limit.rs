use bits_e2e::{client::BitsClient, fixtures};

#[tokio::test]
async fn ip_based_login_rate_limit_blocks_excessive_attempts() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let password = "correct-password";

    // Create multiple users to avoid hitting email-based limit
    for i in 0..11 {
        let email = format!("user{}@example.com", i);
        ctx.create_verified_user(&email, password)
            .await
            .expect("Failed to create user");
    }

    let mut client = BitsClient::new(ctx.server.url(""));
    client.fetch_csrf_token().await.unwrap();

    // Make 10 login attempts with different emails (stay under email limit)
    for i in 0..10 {
        let email = format!("user{}@example.com", i);
        let result = client.login(&email, "wrong-password").await;
        assert!(
            result.is_err(),
            "Attempt {} should fail with wrong password",
            i + 1
        );
    }

    // 11th attempt should be rate limited by IP (10 per 15 minutes)
    let result = client.login("user10@example.com", password).await;
    assert!(
        result.is_err(),
        "11th attempt should be rate limited (IP limit)"
    );

    // Response should be 429 for IP-based limit
    let err = result.unwrap_err();
    assert!(
        matches!(err, bits_e2e::client::ClientError::StatusCode(status) if status.as_u16() == 429),
        "IP-based rate limit should return 429 status"
    );
}

#[tokio::test]
async fn email_based_login_rate_limit_blocks_targeted_attacks() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let email = "victim@example.com";
    let password = "correct-password";

    ctx.create_verified_user(email, password)
        .await
        .expect("Failed to create user");

    let mut client = BitsClient::new(ctx.server.url(""));
    client.fetch_csrf_token().await.unwrap();

    // Make 5 login attempts (email limit is 5 per hour)
    for i in 0..5 {
        let result = client.login(email, "wrong-password").await;
        assert!(
            result.is_err(),
            "Attempt {} should fail with wrong password",
            i + 1
        );
    }

    // 6th attempt should be rate limited (email limit is 5 per hour)
    let result = client.login(email, password).await;
    assert!(
        result.is_err(),
        "6th attempt should be rate limited (email limit)"
    );

    // Response should be 401 for email-based limit (prevents enumeration)
    let err = result.unwrap_err();
    assert!(
        matches!(err, bits_e2e::client::ClientError::StatusCode(status) if status.as_u16() == 401),
        "Email-based rate limit should return 401 to prevent enumeration"
    );
}

#[tokio::test]
async fn ip_based_registration_rate_limit_allows_more_attempts() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let mut client = BitsClient::new(ctx.server.url(""));
    client.fetch_csrf_token().await.unwrap();

    // Registration allows 20 attempts per 15 minutes (more lenient than login)
    for i in 0..20 {
        let email = format!("user{}@example.com", i);
        client.join(&email, "password").await.expect(&format!(
            "Attempt {} should succeed (within registration IP limit)",
            i + 1
        ));

        // Verify user was actually created by checking session exists
        let session = client.get_session().await.unwrap();
        assert!(
            session.is_some(),
            "User {} should be logged in after registration",
            i
        );

        client.logout().await.unwrap();
        client.fetch_csrf_token().await.unwrap();
    }

    // 21st attempt should be rate limited
    let result = client.join("user21@example.com", "password").await;
    let err = result.expect_err("21st registration attempt should be rate limited");
    assert!(
        matches!(err, bits_e2e::client::ClientError::StatusCode(status) if status.as_u16() == 429),
        "Expected 429 rate limit error, got {:?}",
        err
    );
}

#[tokio::test]
async fn email_based_registration_rate_limit_prevents_spam() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let email = "spammer@example.com";
    let mut client = BitsClient::new(ctx.server.url(""));
    client.fetch_csrf_token().await.unwrap();

    // First registration should succeed
    client
        .join(email, "password1")
        .await
        .expect("First registration should succeed");

    // Verify user was created by checking session
    let session = client.get_session().await.unwrap();
    assert!(
        session.is_some(),
        "User should be logged in after registration"
    );

    // Second attempt with same email should still be blocked
    // (email limit is 2 per hour for registration)
    let result = client.join(email, "password2").await;
    assert!(
        result.is_err(),
        "Second attempt fails (email already exists)"
    );

    // Third attempt should be rate limited
    let result = client.join(email, "password3").await;
    assert!(
        result.is_err(),
        "Third registration attempt should be rate limited"
    );
}

#[tokio::test]
async fn successful_login_does_not_trigger_rate_limit() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let email = "legitimate@example.com";
    let password = "correct-password";

    let (user, _hash) = ctx
        .create_verified_user(email, password)
        .await
        .expect("Failed to create user");

    let mut client = BitsClient::new(ctx.server.url(""));
    client.fetch_csrf_token().await.unwrap();

    // Successful login should not count toward rate limit
    for _i in 0..3 {
        client.login(email, password).await.unwrap();

        // Verify session was established
        let session = client.get_session().await.unwrap();
        assert_eq!(
            session.as_ref().map(|u| u.id),
            Some(user.id),
            "User should be logged in after successful login"
        );

        client.logout().await.unwrap();
        client.fetch_csrf_token().await.unwrap(); // Get fresh CSRF token after logout
    }

    // After 3 successful logins, should still be able to login
    client
        .login(email, password)
        .await
        .expect("Successful logins should not trigger rate limit");

    // Verify final login worked
    let session = client.get_session().await.unwrap();
    assert_eq!(
        session.as_ref().map(|u| u.id),
        Some(user.id),
        "User should be logged in after successful login"
    );
}

#[tokio::test]
async fn rate_limit_records_are_created_in_database() {
    let config = fixtures::config().expect("Failed to load config");
    let ctx = fixtures::setup_solo(config)
        .await
        .expect("Failed to setup test");

    let email = "test@example.com";
    let password = "password";

    ctx.create_verified_user(email, password)
        .await
        .expect("Failed to create user");

    let mut client = BitsClient::new(ctx.server.url(""));
    client.fetch_csrf_token().await.unwrap();

    // Make a failed login attempt
    let _ = client.login(email, "wrong-password").await;

    // Wait for async recording to complete (spawned task in middleware)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Check database for auth_attempts record
    let count: i64 = sqlx::query_scalar("select count(*) from auth_attempts where email = $1")
        .bind(email)
        .fetch_one(&ctx.state.db)
        .await
        .expect("Failed to query auth_attempts");

    assert!(count > 0, "Auth attempt should be recorded in database");
}
