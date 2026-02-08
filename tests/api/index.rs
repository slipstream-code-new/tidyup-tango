use crate::helpers::spawn_app;

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
    assert!(body.contains("<header>"), "Missing header landmark");
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
