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

// ---- Capture Tests ----

#[tokio::test]
async fn post_inbox_captures_item_and_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "capture@example.com", "securepassword123").await;

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

#[tokio::test]
async fn post_inbox_with_empty_title_silently_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "empty@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "")])
        .send()
        .await
        .expect("Failed to post to inbox");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Empty title should silently redirect"
    );
}

#[tokio::test]
async fn post_inbox_with_whitespace_only_title_silently_redirects() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "whitespace@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "   ")])
        .send()
        .await
        .expect("Failed to post to inbox");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Whitespace-only title should silently redirect"
    );
}

#[tokio::test]
async fn post_inbox_with_too_long_title_returns_422() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "toolong@example.com", "securepassword123").await;

    let long_title = "a".repeat(301);
    let response = client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", long_title.as_str())])
        .send()
        .await
        .expect("Failed to post to inbox");

    assert_eq!(
        response.status().as_u16(),
        422,
        "Too-long title should return 422"
    );
}

// ---- Empty State ----

#[tokio::test]
async fn inbox_shows_empty_state_when_no_items() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "emptyinbox@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("Inbox zero"),
        "Empty inbox should show 'Inbox zero' message"
    );
    assert!(
        body.contains("weekly review"),
        "Empty state should mention weekly review"
    );
}

// ---- Delete Tests ----

#[tokio::test]
async fn delete_inbox_item_removes_it_and_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "delete@example.com", "securepassword123").await;

    // Capture an item
    client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "Delete me")])
        .send()
        .await
        .expect("Failed to capture");

    // Get inbox page to find the item's ID from the delete form action
    let response = client
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");

    let body = response.text().await.unwrap();
    assert!(body.contains("Delete me"));

    // Extract the item ID from the delete form action URL
    let action_prefix = "action=\"/inbox/";
    let start = body
        .find(action_prefix)
        .expect("Missing delete form action")
        + action_prefix.len();
    let end = body[start..]
        .find("/delete\"")
        .expect("Missing /delete suffix");
    let item_id = &body[start..start + end];

    // Delete the item
    let response = client
        .post(format!("{}/inbox/{}/delete", &app.address, item_id))
        .send()
        .await
        .expect("Failed to delete");

    assert_eq!(response.status().as_u16(), 303, "DELETE should redirect");

    // Verify item is gone
    let response = client
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");

    let body = response.text().await.unwrap();
    assert!(
        !body.contains("Delete me"),
        "Deleted item should no longer appear"
    );
}

#[tokio::test]
async fn delete_nonexistent_inbox_item_returns_404() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "del404@example.com", "securepassword123").await;

    let fake_id = uuid::Uuid::new_v4();
    let response = client
        .post(format!("{}/inbox/{}/delete", &app.address, fake_id))
        .send()
        .await
        .expect("Failed to delete");

    assert_eq!(response.status().as_u16(), 404);
}

// ---- User Isolation ----

#[tokio::test]
async fn users_cannot_see_each_others_inbox_items() {
    let app = spawn_app().await;

    let client_a =
        register_and_login(&app.address, "user_a@example.com", "securepassword123").await;
    let client_b =
        register_and_login(&app.address, "user_b@example.com", "securepassword123").await;

    // User A captures an item
    client_a
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "User A secret")])
        .send()
        .await
        .expect("Failed to capture");

    // User B should not see it
    let response = client_b
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");

    let body = response.text().await.unwrap();
    assert!(
        !body.contains("User A secret"),
        "User B should not see User A's inbox items"
    );
}

// ---- Inbox Count Badge ----

#[tokio::test]
async fn inbox_count_shows_in_dashboard_after_capture() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "count@example.com", "securepassword123").await;

    // Capture two items
    client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "Item one")])
        .send()
        .await
        .expect("Failed to capture");
    client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "Item two")])
        .send()
        .await
        .expect("Failed to capture");

    // Dashboard should show count of 2
    let response = client
        .get(format!("{}/dashboard", &app.address))
        .send()
        .await
        .expect("Failed to GET /dashboard");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("2 items"),
        "Dashboard should show '2 items' for inbox count"
    );
}

#[tokio::test]
async fn inbox_nav_badge_shows_count() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "badge@example.com", "securepassword123").await;

    // Capture an item
    client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "Badge test")])
        .send()
        .await
        .expect("Failed to capture");

    // Check that the nav shows the badge with count
    let response = client
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("gtd-nav__badge"),
        "Nav should show inbox count badge when items exist"
    );
}

// ---- Accessibility ----

#[tokio::test]
async fn inbox_page_has_visible_label_for_capture_input() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "a11y@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("<label"),
        "Capture form should have a <label>"
    );
    assert!(
        body.contains("for=\"inbox-title\""),
        "Label should be associated with capture input via for/id"
    );
}

#[tokio::test]
async fn inbox_page_has_proper_heading() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "heading@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("<h1>Inbox</h1>"),
        "Inbox page should have <h1>Inbox</h1>"
    );
}

// ---- HTMX Fragment Responses ----

#[tokio::test]
async fn htmx_post_inbox_returns_fragment_with_announce_header() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "htmx@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/inbox", &app.address))
        .header("hx-request", "true")
        .form(&[("title", "HTMX capture")])
        .send()
        .await
        .expect("Failed to post to inbox");

    assert_eq!(
        response.status().as_u16(),
        200,
        "HTMX POST should return 200 with fragment"
    );

    // Should have HX-Trigger header with announce event
    let trigger = response
        .headers()
        .get("hx-trigger")
        .expect("Missing HX-Trigger header")
        .to_str()
        .unwrap();
    assert!(
        trigger.contains("Captured to inbox"),
        "HX-Trigger should contain 'Captured to inbox' announce"
    );

    let body = response.text().await.unwrap();
    assert!(
        body.contains("HTMX capture"),
        "Fragment should contain the captured item"
    );
}

#[tokio::test]
async fn htmx_post_inbox_with_empty_title_returns_204() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "htmxempty@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/inbox", &app.address))
        .header("hx-request", "true")
        .form(&[("title", "")])
        .send()
        .await
        .expect("Failed to post to inbox");

    assert_eq!(
        response.status().as_u16(),
        204,
        "HTMX POST with empty title should return 204"
    );
}

#[tokio::test]
async fn htmx_delete_inbox_item_returns_empty_body_with_announce() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "htmxdel@example.com", "securepassword123").await;

    // Capture an item
    client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "HTMX delete test")])
        .send()
        .await
        .expect("Failed to capture");

    // Get inbox to find item ID
    let response = client
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");
    let body = response.text().await.unwrap();

    let action_prefix = "action=\"/inbox/";
    let start = body
        .find(action_prefix)
        .expect("Missing delete form action")
        + action_prefix.len();
    let end = body[start..]
        .find("/delete\"")
        .expect("Missing /delete suffix");
    let item_id = &body[start..start + end];

    // Delete via HTMX
    let response = client
        .post(format!("{}/inbox/{}/delete", &app.address, item_id))
        .header("hx-request", "true")
        .send()
        .await
        .expect("Failed to delete");

    assert_eq!(response.status().as_u16(), 200);

    let trigger = response
        .headers()
        .get("hx-trigger")
        .expect("Missing HX-Trigger header")
        .to_str()
        .unwrap();
    assert!(
        trigger.contains("Inbox item deleted"),
        "HX-Trigger should contain 'Inbox item deleted' announce"
    );

    let body = response.text().await.unwrap();
    assert!(body.is_empty(), "HTMX delete should return empty body");
}

// ---- Quick Capture From Header ----

#[tokio::test]
async fn quick_capture_form_posts_to_inbox() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "quickcap@example.com", "securepassword123").await;

    // Verify the quick capture form on the dashboard page
    let response = client
        .get(format!("{}/dashboard", &app.address))
        .send()
        .await
        .expect("Failed to GET /dashboard");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("action=\"/inbox\""),
        "Quick capture form on dashboard should post to /inbox"
    );

    // Post via quick capture (same endpoint)
    let response = client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "From quick capture")])
        .send()
        .await
        .expect("Failed to post");

    assert_eq!(response.status().as_u16(), 303);

    // Item should appear in inbox
    let response = client
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("From quick capture"),
        "Item captured via quick capture should appear in inbox"
    );
}
