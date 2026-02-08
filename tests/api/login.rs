use crate::helpers::spawn_app;

/// Helper to register a user via the registration endpoint.
async fn register_user(address: &str, email: &str, password: &str) {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let response = client
        .post(format!("{address}/register"))
        .form(&[
            ("email", email),
            ("password", password),
            ("password_confirmation", password),
        ])
        .send()
        .await
        .expect("Failed to register user");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Registration should succeed with 303"
    );
}

#[tokio::test]
async fn get_login_returns_200_with_login_form() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/login", &app.address))
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
    assert!(body.contains("Sign in"), "Missing page heading");
    assert!(
        body.contains("Sign in -- Todo List"),
        "Missing descriptive page title"
    );

    // Form fields with labels
    assert!(
        body.contains("<label for=\"email\">Email</label>"),
        "Missing email label"
    );
    assert!(body.contains("type=\"email\""), "Missing email input");
    assert!(
        body.contains("autocomplete=\"email\""),
        "Missing autocomplete=\"email\" on email field"
    );
    assert!(
        body.contains("<label for=\"password\">Password</label>"),
        "Missing password label"
    );
    assert!(
        body.contains("autocomplete=\"current-password\""),
        "Missing autocomplete=\"current-password\" on password field"
    );

    // Submit button
    assert!(
        body.contains(">Sign in</button>"),
        "Submit button should say 'Sign in'"
    );

    // Link to register
    assert!(
        body.contains("Create account"),
        "Missing link to registration page"
    );
    assert!(
        body.contains("href=\"/register\""),
        "Missing register link href"
    );

    // Error container always in DOM (for aria-describedby) but empty on initial load
    assert!(
        body.contains("role=\"alert\""),
        "Error element with role=\"alert\" should always be in the DOM"
    );
    assert!(
        !body.contains("aria-invalid=\"true\""),
        "Should not have aria-invalid on initial load"
    );
    // The error element should be empty (no error text)
    assert!(
        !body.contains("didn&#39;t work"),
        "Should not show error message text on initial load"
    );
}

#[tokio::test]
async fn post_login_with_valid_credentials_redirects_to_todos() {
    let app = spawn_app().await;
    let email = "login@example.com";
    let password = "securepassword123";

    register_user(&app.address, email, password).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let response = client
        .post(format!("{}/login", &app.address))
        .form(&[("email", email), ("password", password)])
        .send()
        .await
        .expect("Failed to execute request");

    let status = response.status().as_u16();
    let location = response
        .headers()
        .get("location")
        .map(|v| v.to_str().unwrap().to_string());
    let body = response.text().await.unwrap();

    assert_eq!(
        status, 303,
        "Expected 303 redirect after successful login. Body: {body}"
    );
    assert_eq!(
        location.as_deref(),
        Some("/todos"),
        "Should redirect to /todos after login"
    );
}

#[tokio::test]
async fn post_login_sets_session_cookie() {
    let app = spawn_app().await;
    let email = "session@example.com";
    let password = "securepassword123";

    register_user(&app.address, email, password).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let response = client
        .post(format!("{}/login", &app.address))
        .form(&[("email", email), ("password", password)])
        .send()
        .await
        .expect("Failed to execute request");

    let has_session_cookie = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .any(|v| v.to_str().unwrap_or("").contains("id="));

    assert!(
        has_session_cookie,
        "Response should include a session cookie"
    );
}

#[tokio::test]
async fn post_login_returns_422_with_invalid_credentials() {
    let app = spawn_app().await;
    let email = "wrong@example.com";

    register_user(&app.address, email, "securepassword123").await;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/login", &app.address))
        .form(&[("email", email), ("password", "wrongpassword123")])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status().as_u16(),
        422,
        "Expected 422 for invalid credentials"
    );

    let body = response.text().await.unwrap();
    assert!(
        body.contains("didn") && body.contains("t work"),
        "Should show generic error message: got {body}"
    );
    // Must not reveal whether the email or password was wrong
    assert!(
        !body.contains("email not found"),
        "Should not reveal which credential failed"
    );
    assert!(
        !body.contains("wrong password"),
        "Should not reveal which credential failed"
    );
}

#[tokio::test]
async fn post_login_returns_422_for_nonexistent_user() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/login", &app.address))
        .form(&[
            ("email", "nobody@example.com"),
            ("password", "securepassword123"),
        ])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status().as_u16(),
        422,
        "Expected 422 for nonexistent user"
    );

    let body = response.text().await.unwrap();
    assert!(
        body.contains("didn") && body.contains("t work"),
        "Should show same error as wrong password"
    );
}

#[tokio::test]
async fn post_login_preserves_email_on_failed_attempt() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/login", &app.address))
        .form(&[
            ("email", "preserve@example.com"),
            ("password", "wrongpassword"),
        ])
        .send()
        .await
        .expect("Failed to execute request");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("preserve@example.com"),
        "Email should be preserved in the form on error"
    );
}

#[tokio::test]
async fn post_logout_clears_session_and_redirects_to_login() {
    let app = spawn_app().await;
    let email = "logout@example.com";
    let password = "securepassword123";

    register_user(&app.address, email, password).await;

    // Use a cookie jar to maintain session across requests
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    // Login first
    client
        .post(format!("{}/login", &app.address))
        .form(&[("email", email), ("password", password)])
        .send()
        .await
        .expect("Failed to login");

    // Then logout
    let response = client
        .post(format!("{}/logout", &app.address))
        .send()
        .await
        .expect("Failed to logout");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Expected 303 redirect after logout"
    );

    let location = response
        .headers()
        .get("location")
        .expect("Missing Location header")
        .to_str()
        .unwrap();
    assert_eq!(location, "/login", "Should redirect to /login after logout");
}
