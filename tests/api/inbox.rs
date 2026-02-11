use crate::helpers::spawn_app;

/// Helper: creates a reqwest client with cookie store and no redirect following.
fn authenticated_client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

/// Helper: register a user and log them in, returning the cookie-jar client.
async fn register_and_login(address: &str, email: &str, password: &str) -> reqwest::Client {
    let client = authenticated_client();

    // Register
    let resp = client
        .post(format!("{address}/register"))
        .form(&[
            ("email", email),
            ("password", password),
            ("password_confirmation", password),
        ])
        .send()
        .await
        .expect("Failed to register");
    assert_eq!(resp.status().as_u16(), 303, "Registration should redirect");

    // Login
    let resp = client
        .post(format!("{address}/login"))
        .form(&[("email", email), ("password", password)])
        .send()
        .await
        .expect("Failed to login");
    assert_eq!(resp.status().as_u16(), 303, "Login should redirect");

    client
}

#[tokio::test]
async fn post_inbox_captures_item_and_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "capture@example.com", "securepassword123").await;

    // Capture an item
    let response = client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "Buy milk")])
        .send()
        .await
        .expect("Failed to post to inbox");

    assert_eq!(
        response.status().as_u16(),
        303,
        "POST /inbox should redirect"
    );

    let location = response
        .headers()
        .get("location")
        .expect("Missing Location header")
        .to_str()
        .unwrap();
    assert_eq!(location, "/inbox", "Should redirect to /inbox");

    // Verify the item shows on the inbox page
    let response = client
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("Buy milk"),
        "Inbox page should show the captured item"
    );
}
