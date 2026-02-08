use crate::helpers::spawn_app;

#[tokio::test]
async fn get_forgot_password_returns_200_with_placeholder_page() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/forgot-password", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);

    let content_type = response
        .headers()
        .get("content-type")
        .expect("Missing content-type header")
        .to_str()
        .unwrap();
    assert!(
        content_type.contains("text/html"),
        "Expected text/html, got {content_type}"
    );

    let body = response.text().await.unwrap();

    // Page structure
    assert!(body.contains("Forgot password"), "Missing page heading");
    assert!(
        body.contains("Forgot password -- Todo List"),
        "Missing descriptive page title"
    );

    // Placeholder message explaining feature is coming soon
    assert!(
        body.contains("coming soon"),
        "Should explain that password reset is coming soon"
    );

    // Link back to login
    assert!(
        body.contains("href=\"/login\""),
        "Should have a link back to the login page"
    );
}

#[tokio::test]
async fn login_page_has_forgot_password_link() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/login", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    let body = response.text().await.unwrap();

    assert!(
        body.contains("href=\"/forgot-password\""),
        "Login page should have a link to /forgot-password"
    );
    assert!(
        body.contains("Forgot password?"),
        "Login page should have 'Forgot password?' link text"
    );
}
