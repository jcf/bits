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

    let client = BitsClient::new(ctx.server.url(""));

    client.login(email, password).await;

    let session1 = client.get_session().await;
    assert_eq!(
        session1.as_ref().map(|u| u.id),
        Some(user.id),
        "User should be signed in after login"
    );

    let session2 = client.get_session().await;
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

    let client1 = BitsClient::new(ctx.server.url(""));
    client1.login(email, old_password).await;

    let client2 = BitsClient::new(ctx.server.url(""));
    client2.login(email, old_password).await;

    let session1 = client1.get_session().await;
    assert_eq!(
        session1.as_ref().map(|u| u.id),
        Some(user.id),
        "Client 1 should be signed in before password change"
    );

    let session2 = client2.get_session().await;
    assert_eq!(
        session2.as_ref().map(|u| u.id),
        Some(user.id),
        "Client 2 should be signed in before password change"
    );

    client1
        .change_password(&serde_json::json!({
            "current_password": old_password,
            "new_password": new_password,
            "confirm_password": new_password
        }))
        .await;

    let session1_after = client1.get_session().await;
    assert_eq!(
        session1_after,
        None,
        "Client 1 should be logged out after changing their own password"
    );

    let session2_after = client2.get_session().await;
    assert_eq!(
        session2_after,
        None,
        "Client 2 session should be invalidated when user changes password on client 1"
    );
}
