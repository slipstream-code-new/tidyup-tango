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
async fn index_returns_200_with_html_content() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/", &app.address))
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
    assert!(body.contains("<!DOCTYPE html>"), "Missing DOCTYPE");
    assert!(body.contains("<html lang=\"en\">"), "Missing html lang");
    assert!(
        body.contains("class=\"skip-link\""),
        "Skip link must use .skip-link class for focus reveal"
    );
    assert!(
        body.contains("Skip to main content"),
        "Missing skip link text"
    );
    assert!(
        body.contains("id=\"main-content\""),
        "Missing main content landmark"
    );
    assert!(body.contains("<header"), "Missing header landmark");
    assert!(body.contains("<main"), "Missing main landmark");
    assert!(body.contains("<footer"), "Missing footer landmark");
    assert!(
        body.contains("/static/css/main.css"),
        "Missing CSS stylesheet link"
    );
    assert!(
        body.contains("aria-live=\"polite\""),
        "Missing aria-live region for HTMX announcements"
    );
    assert!(body.contains("<h1>Welcome</h1>"), "Missing h1");
}

#[tokio::test]
async fn static_css_is_served() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/static/css/main.css", &app.address))
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
        content_type.contains("text/css"),
        "Expected text/css, got {content_type}"
    );
}

#[tokio::test]
async fn index_shows_register_and_login_links_for_unauthenticated_visitor() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);

    let body = response.text().await.unwrap();

    // Must show a link to the registration page
    assert!(
        body.contains("href=\"/register\""),
        "Missing link to /register"
    );
    assert!(
        body.contains("Create account"),
        "Missing 'Create account' link text"
    );

    // Must show a link to the login page
    assert!(body.contains("href=\"/login\""), "Missing link to /login");
    assert!(body.contains("Sign in"), "Missing 'Sign in' link text");
}

#[tokio::test]
async fn index_redirects_authenticated_user_to_dashboard() {
    let app = spawn_app().await;

    let client = register_and_login(&app.address, "index@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Authenticated user should be redirected from index"
    );

    let location = response
        .headers()
        .get("location")
        .expect("Missing Location header")
        .to_str()
        .unwrap();
    assert_eq!(
        location, "/dashboard",
        "Authenticated user should be redirected to /dashboard"
    );
}
