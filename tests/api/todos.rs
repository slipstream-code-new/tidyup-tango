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
async fn get_todos_redirects_to_login_when_unauthenticated() {
    let app = spawn_app().await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let response = client
        .get(format!("{}/todos", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Unauthenticated request should redirect"
    );

    let location = response
        .headers()
        .get("location")
        .expect("Missing Location header")
        .to_str()
        .unwrap();
    assert_eq!(location, "/login", "Should redirect to /login");
}

#[tokio::test]
async fn get_todos_shows_empty_state_for_new_user() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "empty@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/todos", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);

    let body = response.text().await.unwrap();

    // Page structure
    assert!(body.contains("My Todos"), "Missing page heading");
    assert!(
        body.contains("My Todos -- Todo List"),
        "Missing descriptive page title"
    );

    // Empty state
    assert!(
        body.contains("No todos yet."),
        "Should show empty state message"
    );

    // Add form is present
    assert!(
        body.contains("<label for=\"title\">New todo</label>"),
        "Missing 'New todo' label"
    );
    assert!(
        body.contains("placeholder=\"What do you need to do?\""),
        "Missing placeholder text"
    );

    // Logout button is present
    assert!(
        body.contains("Sign out"),
        "Missing sign out button in navigation"
    );

    // No list rendered
    assert!(
        !body.contains("<ul"),
        "Should not render a list when there are no todos"
    );
}

#[tokio::test]
async fn post_todo_adds_item_and_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "addtodo@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/todos", &app.address))
        .form(&[("title", "Buy groceries")])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Should redirect after adding todo"
    );

    let location = response
        .headers()
        .get("location")
        .expect("Missing Location header")
        .to_str()
        .unwrap();
    assert_eq!(location, "/todos", "Should redirect back to /todos");
}

#[tokio::test]
async fn get_todos_shows_added_items() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "viewtodos@example.com", "securepassword123").await;

    // Add two items
    client
        .post(format!("{}/todos", &app.address))
        .form(&[("title", "Buy groceries")])
        .send()
        .await
        .expect("Failed to add first todo");

    client
        .post(format!("{}/todos", &app.address))
        .form(&[("title", "Walk the dog")])
        .send()
        .await
        .expect("Failed to add second todo");

    // View the list
    let response = client
        .get(format!("{}/todos", &app.address))
        .send()
        .await
        .expect("Failed to get todos page");

    assert_eq!(response.status().as_u16(), 200);

    let body = response.text().await.unwrap();

    // Items are displayed
    assert!(
        body.contains("Buy groceries"),
        "First todo should be visible"
    );
    assert!(
        body.contains("Walk the dog"),
        "Second todo should be visible"
    );

    // Rendered as a semantic list
    assert!(
        body.contains("<ul"),
        "Todos should be rendered in a <ul> list"
    );
    assert!(body.contains("<li"), "Each todo should be a <li> list item");

    // No empty state message
    assert!(
        !body.contains("No todos yet."),
        "Should not show empty state when todos exist"
    );
}

#[tokio::test]
async fn post_todo_with_empty_title_silently_redirects() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "emptytitle@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/todos", &app.address))
        .form(&[("title", "")])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Empty title should silently redirect, not show error"
    );
}

#[tokio::test]
async fn post_todo_with_whitespace_only_title_silently_redirects() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "whitespace@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/todos", &app.address))
        .form(&[("title", "   ")])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Whitespace-only title should silently redirect"
    );
}

#[tokio::test]
async fn post_todo_with_too_long_title_returns_422() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "longtitle@example.com", "securepassword123").await;

    let long_title = "a".repeat(301);

    let response = client
        .post(format!("{}/todos", &app.address))
        .form(&[("title", long_title.as_str())])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status().as_u16(),
        422,
        "Too-long title should return 422"
    );

    let body = response.text().await.unwrap();
    assert!(
        body.contains("too long"),
        "Should show title-too-long error message"
    );
}

#[tokio::test]
async fn post_todo_redirects_to_login_when_unauthenticated() {
    let app = spawn_app().await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let response = client
        .post(format!("{}/todos", &app.address))
        .form(&[("title", "Sneaky todo")])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Unauthenticated POST should redirect"
    );

    let location = response
        .headers()
        .get("location")
        .expect("Missing Location header")
        .to_str()
        .unwrap();
    assert_eq!(location, "/login", "Should redirect to /login");
}

#[tokio::test]
async fn users_cannot_see_each_others_todos() {
    let app = spawn_app().await;

    // User A adds a todo
    let client_a = register_and_login(&app.address, "usera@example.com", "securepassword123").await;
    client_a
        .post(format!("{}/todos", &app.address))
        .form(&[("title", "User A secret task")])
        .send()
        .await
        .expect("Failed to add todo for user A");

    // User B logs in and views their todos
    let client_b = register_and_login(&app.address, "userb@example.com", "securepassword123").await;

    let response = client_b
        .get(format!("{}/todos", &app.address))
        .send()
        .await
        .expect("Failed to get todos for user B");

    let body = response.text().await.unwrap();

    assert!(
        !body.contains("User A secret task"),
        "User B should not see User A's todos"
    );
    assert!(
        body.contains("No todos yet."),
        "User B should see empty state"
    );
}
