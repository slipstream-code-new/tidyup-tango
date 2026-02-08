use crate::helpers::spawn_app;

#[tokio::test]
async fn get_register_returns_200_with_registration_form() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/register", &app.address))
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
    assert!(body.contains("Create account"), "Missing page heading");
    assert!(
        body.contains("Create account -- Todo List"),
        "Missing descriptive page title"
    );

    // Form fields with labels
    assert!(
        body.contains("<label for=\"email\">Email</label>"),
        "Missing email label"
    );
    assert!(body.contains("type=\"email\""), "Missing email input");
    assert!(
        body.contains("<label for=\"password\">Password</label>"),
        "Missing password label"
    );
    assert!(
        body.contains("autocomplete=\"new-password\""),
        "Missing autocomplete=\"new-password\" on password fields"
    );
    assert!(
        body.contains("<label for=\"password_confirmation\">Confirm password</label>"),
        "Missing password confirmation label"
    );

    // Submit button with action words
    assert!(
        body.contains(">Create account</button>"),
        "Submit button should say 'Create account'"
    );

    // Link to login
    assert!(body.contains("Sign in"), "Missing link to login page");
    assert!(body.contains("href=\"/login\""), "Missing login link href");

    // Accessibility: no error states on initial load
    assert!(
        !body.contains("aria-invalid"),
        "Should not have aria-invalid on initial load"
    );
    assert!(
        !body.contains("auth-form__error"),
        "Should not show error messages on initial load"
    );
}
