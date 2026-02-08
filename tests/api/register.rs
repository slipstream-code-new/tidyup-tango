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
        !body.contains("aria-invalid=\"true\""),
        "Should not have aria-invalid=\"true\" on initial load"
    );
    // Error containers exist (for aria-describedby targets) but should be empty
    assert!(
        !body.contains("Enter your email address"),
        "Should not show email error text on initial load"
    );
    assert!(
        !body.contains("at least 8 characters</p>"),
        "Should not show password error text on initial load"
    );
}

#[tokio::test]
async fn post_register_with_valid_data_redirects() {
    let app = spawn_app().await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let response = client
        .post(format!("{}/register", &app.address))
        .form(&[
            ("email", "test@example.com"),
            ("password", "securepassword123"),
            ("password_confirmation", "securepassword123"),
        ])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Expected 303 See Other redirect after successful registration"
    );
}

#[tokio::test]
async fn post_register_persists_user_in_database() {
    let app = spawn_app().await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    client
        .post(format!("{}/register", &app.address))
        .form(&[
            ("email", "persist@example.com"),
            ("password", "securepassword123"),
            ("password_confirmation", "securepassword123"),
        ])
        .send()
        .await
        .expect("Failed to execute request");

    let saved = sqlx::query!(
        "SELECT email, password_hash FROM users WHERE email = $1",
        "persist@example.com"
    )
    .fetch_one(&app.pool)
    .await
    .expect("Failed to fetch saved user");

    assert_eq!(saved.email, "persist@example.com");
    assert!(
        saved.password_hash.starts_with("$argon2"),
        "Password should be hashed with Argon2"
    );
}

#[tokio::test]
async fn post_register_returns_422_when_email_is_empty() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/register", &app.address))
        .form(&[
            ("email", ""),
            ("password", "securepassword123"),
            ("password_confirmation", "securepassword123"),
        ])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 422);
    let body = response.text().await.unwrap();
    assert!(
        body.contains("Enter your email address"),
        "Should show email error message"
    );
}

#[tokio::test]
async fn post_register_returns_422_when_email_is_invalid() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/register", &app.address))
        .form(&[
            ("email", "notanemail"),
            ("password", "securepassword123"),
            ("password_confirmation", "securepassword123"),
        ])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 422);
    let body = response.text().await.unwrap();
    assert!(
        body.contains("name@example.com"),
        "Should show email format hint"
    );
}

#[tokio::test]
async fn post_register_returns_422_when_password_too_short() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/register", &app.address))
        .form(&[
            ("email", "test@example.com"),
            ("password", "short"),
            ("password_confirmation", "short"),
        ])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 422);
    let body = response.text().await.unwrap();
    assert!(
        body.contains("at least 8 characters"),
        "Should show password length error"
    );
}

#[tokio::test]
async fn post_register_returns_422_when_passwords_dont_match() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/register", &app.address))
        .form(&[
            ("email", "test@example.com"),
            ("password", "securepassword123"),
            ("password_confirmation", "differentpassword"),
        ])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 422);
    let body = response.text().await.unwrap();
    assert!(
        body.contains("Passwords do not match"),
        "Should show password mismatch error"
    );
}

#[tokio::test]
async fn post_register_returns_422_for_duplicate_email_without_leaking_info() {
    let app = spawn_app().await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    // Register the first user
    client
        .post(format!("{}/register", &app.address))
        .form(&[
            ("email", "duplicate@example.com"),
            ("password", "securepassword123"),
            ("password_confirmation", "securepassword123"),
        ])
        .send()
        .await
        .expect("Failed to execute request");

    // Try to register with the same email
    let response = client
        .post(format!("{}/register", &app.address))
        .form(&[
            ("email", "duplicate@example.com"),
            ("password", "anotherpassword123"),
            ("password_confirmation", "anotherpassword123"),
        ])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 422);
    let body = response.text().await.unwrap();
    // Must NOT reveal that the email exists (account enumeration prevention)
    assert!(
        !body.contains("already exists"),
        "Should not reveal email existence"
    );
    assert!(
        body.contains("signing in"),
        "Should suggest signing in as an alternative"
    );
}

#[tokio::test]
async fn register_page_includes_password_toggle_script() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/register", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    let body = response.text().await.unwrap();

    // The register page should include the password toggle script
    assert!(
        body.contains("password-toggle.js"),
        "Register page should include the password-toggle.js script"
    );

    // The confirm group should have an id for the JS to target
    assert!(
        body.contains("id=\"confirm-group\""),
        "Confirm password group should have id=\"confirm-group\" for JS targeting"
    );
}

#[tokio::test]
async fn password_toggle_script_is_served() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/static/js/password-toggle.js", &app.address))
        .send()
        .await
        .expect("Failed to fetch password-toggle.js");

    assert_eq!(
        response.status().as_u16(),
        200,
        "password-toggle.js should be served as a static file"
    );

    let body = response.text().await.unwrap();
    assert!(
        body.contains("aria-pressed"),
        "Script should set aria-pressed for accessibility"
    );
    assert!(
        body.contains("aria-label"),
        "Script should set aria-label for accessibility"
    );
}

#[tokio::test]
async fn post_register_preserves_email_on_validation_error() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/register", &app.address))
        .form(&[
            ("email", "test@example.com"),
            ("password", "short"),
            ("password_confirmation", "short"),
        ])
        .send()
        .await
        .expect("Failed to execute request");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("test@example.com"),
        "Email should be preserved in the form on error"
    );
}
