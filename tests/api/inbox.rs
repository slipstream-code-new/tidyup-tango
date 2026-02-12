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
    let delete_action_prefix = "action=\"/inbox/";
    // Find the delete form action specifically (contains /delete")
    let delete_action_pos = body.find("/delete\"").expect("Missing /delete action");
    // Walk backwards to find the start of the action attribute value
    let action_start = body[..delete_action_pos]
        .rfind(delete_action_prefix)
        .expect("Missing delete form action prefix")
        + delete_action_prefix.len();
    let item_id = &body[action_start..delete_action_pos];

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

#[tokio::test]
async fn delete_button_has_accessible_label_with_item_title() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "dellabel@example.com", "securepassword123").await;

    // Capture an item
    client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "Call the dentist")])
        .send()
        .await
        .expect("Failed to capture");

    let response = client
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("aria-label=\"Trash: Call the dentist\""),
        "Trash button should have aria-label including the item title"
    );
}

#[tokio::test]
async fn inbox_page_includes_focus_management_script() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "inboxfocus@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("inbox-focus.js"),
        "Inbox page should include focus management script"
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

    let delete_action_prefix = "action=\"/inbox/";
    let delete_action_pos = body.find("/delete\"").expect("Missing /delete action");
    let action_start = body[..delete_action_pos]
        .rfind(delete_action_prefix)
        .expect("Missing delete form action prefix")
        + delete_action_prefix.len();
    let item_id = &body[action_start..delete_action_pos];

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

/// Helper: get the first context ID from the contexts page
async fn get_first_context_id(client: &reqwest::Client, address: &str) -> String {
    let response = client
        .get(format!("{address}/contexts"))
        .send()
        .await
        .expect("Failed to GET /contexts");

    let body = response.text().await.unwrap();
    let action_prefix = "action=\"/contexts/";
    let start = body.find(action_prefix).expect("Missing context action") + action_prefix.len();
    let end = body[start..].find('/').expect("Missing / after context ID");
    body[start..start + end].to_string()
}

/// Helper: get the first inbox item ID from the inbox page
async fn get_first_inbox_item_id(client: &reqwest::Client, address: &str) -> String {
    let response = client
        .get(format!("{address}/inbox"))
        .send()
        .await
        .expect("Failed to GET /inbox");

    let body = response.text().await.unwrap();
    let action_prefix = "action=\"/inbox/";
    let start = body.find(action_prefix).expect("Missing inbox item action") + action_prefix.len();
    let end = body[start..].find('/').expect("Missing / after item ID");
    body[start..start + end].to_string()
}

// ---- Clarify as Next Action ----

#[tokio::test]
async fn clarify_inbox_item_as_next_action_removes_from_inbox() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "clarify1@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;

    // Capture an item
    client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "Clarify me")])
        .send()
        .await
        .expect("Failed to capture");

    let item_id = get_first_inbox_item_id(&client, &app.address).await;

    // Clarify as next action
    let response = client
        .post(format!("{}/inbox/{}/clarify", &app.address, item_id))
        .form(&[("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to clarify");

    assert_eq!(response.status().as_u16(), 303, "Clarify should redirect");

    // Item should be gone from inbox
    let response = client
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");
    let body = response.text().await.unwrap();
    assert!(
        !body.contains("Clarify me"),
        "Clarified item should no longer appear in inbox"
    );
}

#[tokio::test]
async fn clarify_inbox_item_creates_next_action() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "clarify2@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;

    // Capture an item
    client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "Becomes next action")])
        .send()
        .await
        .expect("Failed to capture");

    let item_id = get_first_inbox_item_id(&client, &app.address).await;

    // Clarify as next action
    client
        .post(format!("{}/inbox/{}/clarify", &app.address, item_id))
        .form(&[("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to clarify");

    // Item should appear in next actions
    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET /next-actions");
    let body = response.text().await.unwrap();
    assert!(
        body.contains("Becomes next action"),
        "Clarified item should appear in next actions list"
    );
}

#[tokio::test]
async fn clarify_nonexistent_inbox_item_returns_404() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "clarify3@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;
    let fake_id = uuid::Uuid::new_v4();

    let response = client
        .post(format!("{}/inbox/{}/clarify", &app.address, fake_id))
        .form(&[("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to clarify");

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn htmx_clarify_returns_empty_body_with_announce() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "clarify4@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;

    client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "HTMX clarify test")])
        .send()
        .await
        .expect("Failed to capture");

    let item_id = get_first_inbox_item_id(&client, &app.address).await;

    let response = client
        .post(format!("{}/inbox/{}/clarify", &app.address, item_id))
        .header("hx-request", "true")
        .form(&[("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to clarify");

    assert_eq!(response.status().as_u16(), 200);

    let trigger = response
        .headers()
        .get("hx-trigger")
        .expect("Missing HX-Trigger")
        .to_str()
        .unwrap();
    assert!(
        trigger.contains("Clarified as next action"),
        "HX-Trigger should contain announce"
    );

    let body = response.text().await.unwrap();
    assert!(
        body.is_empty(),
        "HTMX clarify should return empty body (item removed)"
    );
}

#[tokio::test]
async fn inbox_item_has_clarify_form_with_context_select() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "clarify5@example.com", "securepassword123").await;

    client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "Item with clarify")])
        .send()
        .await
        .expect("Failed to capture");

    let response = client
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");
    let body = response.text().await.unwrap();

    assert!(
        body.contains("/clarify"),
        "Inbox item should have a clarify form action"
    );
    assert!(
        body.contains("Next Action"),
        "Inbox item should have a 'Next Action' radio option"
    );
    assert!(
        body.contains("Project"),
        "Inbox item should have a 'Project' radio option"
    );
    assert!(
        body.contains("Trash"),
        "Inbox item should have a 'Trash' button"
    );
    assert!(
        body.contains("inbox-item__context-select"),
        "Inbox item should have a context select for clarifying"
    );
    assert!(
        body.contains("first_action_title"),
        "Inbox item should have a first action title input for project clarification"
    );
}

#[tokio::test]
async fn inbox_item_clarify_button_has_accessible_label() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "clarify6@example.com", "securepassword123").await;

    client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "A11y clarify test")])
        .send()
        .await
        .expect("Failed to capture");

    let response = client
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");
    let body = response.text().await.unwrap();

    assert!(
        body.contains("aria-label=\"Clarify: A11y clarify test\""),
        "Clarify button should have aria-label including item title"
    );
    assert!(
        body.contains("aria-label=\"Trash: A11y clarify test\""),
        "Trash button should have aria-label including item title"
    );
}

// ---- Clarify as Project ----

#[tokio::test]
async fn clarify_inbox_item_as_project_removes_from_inbox() {
    let app = spawn_app().await;
    let client = register_and_login(
        &app.address,
        "projclarify1@example.com",
        "securepassword123",
    )
    .await;

    let context_id = get_first_context_id(&client, &app.address).await;

    // Capture an item
    client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "Build website")])
        .send()
        .await
        .expect("Failed to capture");

    let item_id = get_first_inbox_item_id(&client, &app.address).await;

    // Clarify as project
    let response = client
        .post(format!("{}/inbox/{}/clarify", &app.address, item_id))
        .form(&[
            ("clarify_type", "project"),
            ("context_id", &context_id),
            ("first_action_title", "Set up hosting"),
        ])
        .send()
        .await
        .expect("Failed to clarify as project");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Clarify as project should redirect"
    );

    // Item should be gone from inbox
    let response = client
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");
    let body = response.text().await.unwrap();
    assert!(
        !body.contains("Build website"),
        "Clarified item should no longer appear in inbox"
    );
}

#[tokio::test]
async fn clarify_inbox_item_as_project_creates_project_and_first_action() {
    let app = spawn_app().await;
    let client = register_and_login(
        &app.address,
        "projclarify2@example.com",
        "securepassword123",
    )
    .await;

    let context_id = get_first_context_id(&client, &app.address).await;

    // Capture an item
    client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "Plan vacation")])
        .send()
        .await
        .expect("Failed to capture");

    let item_id = get_first_inbox_item_id(&client, &app.address).await;

    // Clarify as project
    client
        .post(format!("{}/inbox/{}/clarify", &app.address, item_id))
        .form(&[
            ("clarify_type", "project"),
            ("context_id", &context_id),
            ("first_action_title", "Research destinations"),
        ])
        .send()
        .await
        .expect("Failed to clarify as project");

    // Project should appear in projects list with inbox item's title
    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");
    let body = response.text().await.unwrap();
    assert!(
        body.contains("Plan vacation"),
        "Project should appear in projects list with the inbox item's title"
    );

    // First action should appear in next actions list
    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET /next-actions");
    let body = response.text().await.unwrap();
    assert!(
        body.contains("Research destinations"),
        "First action should appear in next actions list"
    );
}

#[tokio::test]
async fn clarify_as_project_with_empty_first_action_title_redirects_to_inbox() {
    let app = spawn_app().await;
    let client = register_and_login(
        &app.address,
        "projclarify3@example.com",
        "securepassword123",
    )
    .await;

    let context_id = get_first_context_id(&client, &app.address).await;

    // Capture an item
    client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "Needs first action")])
        .send()
        .await
        .expect("Failed to capture");

    let item_id = get_first_inbox_item_id(&client, &app.address).await;

    // Clarify as project with empty first action title (non-HTMX redirects to /inbox)
    let response = client
        .post(format!("{}/inbox/{}/clarify", &app.address, item_id))
        .form(&[
            ("clarify_type", "project"),
            ("context_id", &context_id),
            ("first_action_title", ""),
        ])
        .send()
        .await
        .expect("Failed to clarify as project");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Non-HTMX empty first action should redirect to inbox"
    );

    // Item should still be in inbox
    let response = client
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");
    let body = response.text().await.unwrap();
    assert!(
        body.contains("Needs first action"),
        "Item should still be in inbox after failed clarify"
    );
}

#[tokio::test]
async fn htmx_clarify_as_project_with_empty_first_action_returns_422_with_error() {
    let app = spawn_app().await;
    let client = register_and_login(
        &app.address,
        "projclarify3h@example.com",
        "securepassword123",
    )
    .await;

    let context_id = get_first_context_id(&client, &app.address).await;

    client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "HTMX project error test")])
        .send()
        .await
        .expect("Failed to capture");

    let item_id = get_first_inbox_item_id(&client, &app.address).await;

    let response = client
        .post(format!("{}/inbox/{}/clarify", &app.address, item_id))
        .header("hx-request", "true")
        .form(&[
            ("clarify_type", "project"),
            ("context_id", &context_id),
            ("first_action_title", ""),
        ])
        .send()
        .await
        .expect("Failed to clarify");

    assert_eq!(
        response.status().as_u16(),
        422,
        "HTMX empty first action should return 422"
    );

    let body = response.text().await.unwrap();
    assert!(
        body.contains("Enter a first action for this project"),
        "Should contain inline error message, got: {body}"
    );
    assert!(
        body.contains("HTMX project error test"),
        "Should re-render the inbox item"
    );
}

#[tokio::test]
async fn clarify_nonexistent_inbox_item_as_project_returns_404() {
    let app = spawn_app().await;
    let client = register_and_login(
        &app.address,
        "projclarify4@example.com",
        "securepassword123",
    )
    .await;

    let context_id = get_first_context_id(&client, &app.address).await;
    let fake_id = uuid::Uuid::new_v4();

    let response = client
        .post(format!("{}/inbox/{}/clarify", &app.address, fake_id))
        .form(&[
            ("clarify_type", "project"),
            ("context_id", &context_id),
            ("first_action_title", "Some action"),
        ])
        .send()
        .await
        .expect("Failed to clarify");

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn htmx_clarify_as_project_returns_empty_body_with_announce() {
    let app = spawn_app().await;
    let client = register_and_login(
        &app.address,
        "projclarify5@example.com",
        "securepassword123",
    )
    .await;

    let context_id = get_first_context_id(&client, &app.address).await;

    client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "HTMX project test")])
        .send()
        .await
        .expect("Failed to capture");

    let item_id = get_first_inbox_item_id(&client, &app.address).await;

    let response = client
        .post(format!("{}/inbox/{}/clarify", &app.address, item_id))
        .header("hx-request", "true")
        .form(&[
            ("clarify_type", "project"),
            ("context_id", &context_id),
            ("first_action_title", "First step"),
        ])
        .send()
        .await
        .expect("Failed to clarify");

    assert_eq!(response.status().as_u16(), 200);

    let trigger = response
        .headers()
        .get("hx-trigger")
        .expect("Missing HX-Trigger")
        .to_str()
        .unwrap();
    assert!(
        trigger.contains("Clarified as project"),
        "HX-Trigger should contain 'Clarified as project' announce"
    );

    let body = response.text().await.unwrap();
    assert!(
        body.is_empty(),
        "HTMX clarify as project should return empty body (item removed)"
    );
}

#[tokio::test]
async fn clarify_as_project_first_action_is_linked_to_project() {
    let app = spawn_app().await;
    let client = register_and_login(
        &app.address,
        "projclarify6@example.com",
        "securepassword123",
    )
    .await;

    let context_id = get_first_context_id(&client, &app.address).await;

    // Capture an item
    client
        .post(format!("{}/inbox", &app.address))
        .form(&[("title", "Organize garage")])
        .send()
        .await
        .expect("Failed to capture");

    let item_id = get_first_inbox_item_id(&client, &app.address).await;

    // Clarify as project
    client
        .post(format!("{}/inbox/{}/clarify", &app.address, item_id))
        .form(&[
            ("clarify_type", "project"),
            ("context_id", &context_id),
            ("first_action_title", "Buy storage bins"),
        ])
        .send()
        .await
        .expect("Failed to clarify as project");

    // Find the project ID from the projects page
    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");
    let body = response.text().await.unwrap();

    // Extract project ID from the project detail link
    let link_prefix = "href=\"/projects/";
    let start = body.find(link_prefix).expect("Missing project link") + link_prefix.len();
    let end = body[start..].find('"').expect("Missing closing quote");
    let project_id = &body[start..start + end];

    // Project detail page should show the first action
    let response = client
        .get(format!("{}/projects/{}", &app.address, project_id))
        .send()
        .await
        .expect("Failed to GET project detail");
    let body = response.text().await.unwrap();
    assert!(
        body.contains("Buy storage bins"),
        "Project detail should show the first action created during clarify"
    );
}
