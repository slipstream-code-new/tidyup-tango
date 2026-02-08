use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_returns_200() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.content_length(), Some(0));
}
