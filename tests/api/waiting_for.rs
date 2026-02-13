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

/// Helper: extract a waiting-for item ID from the page body.
fn extract_waiting_for_id(body: &str) -> String {
    // Look for action="/waiting-for/<uuid>/complete"
    let action_prefix = "action=\"/waiting-for/";
    let start = body
        .find(action_prefix)
        .expect("Missing waiting-for action in body")
        + action_prefix.len();
    let end = body[start..].find('/').expect("Missing / after UUID");
    body[start..start + end].to_string()
}

// ---- Page Loading ----

#[tokio::test]
async fn waiting_for_page_returns_200_for_authenticated_user() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf1@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn waiting_for_page_has_proper_heading() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf2@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("<h1>Waiting For</h1>"),
        "Waiting For page should have <h1>Waiting For</h1>"
    );
}

#[tokio::test]
async fn waiting_for_page_shows_empty_state_when_no_items() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf3@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("Nothing pending from others"),
        "Should show empty state message"
    );
}

#[tokio::test]
async fn waiting_for_page_indicates_current_page_in_nav() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf4@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("aria-current=\"page\""),
        "Waiting For nav link should be marked as current page"
    );
}

// ---- Add Waiting-For Item ----

#[tokio::test]
async fn post_waiting_for_adds_item_and_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf5@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", "Budget approval"), ("waiting_on", "Finance team")])
        .send()
        .await
        .expect("Failed to POST /waiting-for");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Should redirect after adding waiting-for item"
    );

    // Verify the item appears in the list
    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");
    let body = response.text().await.unwrap();
    assert!(
        body.contains("Budget approval"),
        "Item title should appear in the list"
    );
    assert!(
        body.contains("Finance team"),
        "Waiting-on person should appear in the list"
    );
}

#[tokio::test]
async fn post_waiting_for_with_empty_title_silently_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf6@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", "   "), ("waiting_on", "Someone")])
        .send()
        .await
        .expect("Failed to POST /waiting-for");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Empty title should silently redirect"
    );
}

#[tokio::test]
async fn post_waiting_for_with_empty_waiting_on_silently_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf7@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", "Some item"), ("waiting_on", "   ")])
        .send()
        .await
        .expect("Failed to POST /waiting-for");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Empty waiting_on should silently redirect"
    );
}

#[tokio::test]
async fn post_waiting_for_with_too_long_title_returns_422() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf8@example.com", "securepassword123").await;

    let long_title = "x".repeat(301);

    let response = client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", long_title.as_str()), ("waiting_on", "Someone")])
        .send()
        .await
        .expect("Failed to POST /waiting-for");

    assert_eq!(
        response.status().as_u16(),
        422,
        "Too long title should return 422"
    );
}

#[tokio::test]
async fn post_waiting_for_with_too_long_waiting_on_returns_422() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf9@example.com", "securepassword123").await;

    let long_name = "x".repeat(101);

    let response = client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", "Some item"), ("waiting_on", long_name.as_str())])
        .send()
        .await
        .expect("Failed to POST /waiting-for");

    assert_eq!(
        response.status().as_u16(),
        422,
        "Too long waiting_on should return 422"
    );
}

// ---- HTMX Add Waiting-For ----

#[tokio::test]
async fn htmx_post_waiting_for_returns_fragment_with_announce() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf10@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/waiting-for", &app.address))
        .header("hx-request", "true")
        .form(&[
            ("title", "Delivery confirmation"),
            ("waiting_on", "Shipping dept"),
        ])
        .send()
        .await
        .expect("Failed to POST /waiting-for with HTMX");

    assert_eq!(response.status().as_u16(), 200);

    let trigger = response
        .headers()
        .get("hx-trigger")
        .expect("Missing HX-Trigger header");
    let trigger_str = trigger.to_str().unwrap();
    assert!(
        trigger_str.contains("announce"),
        "HX-Trigger should contain announce event"
    );

    let body = response.text().await.unwrap();
    assert!(
        body.contains("Delivery confirmation"),
        "Response fragment should contain the item title"
    );
    assert!(
        body.contains("Shipping dept"),
        "Response fragment should contain the waiting-on person"
    );
}

#[tokio::test]
async fn htmx_post_waiting_for_with_empty_title_returns_204() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf11@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/waiting-for", &app.address))
        .header("hx-request", "true")
        .form(&[("title", "   "), ("waiting_on", "Someone")])
        .send()
        .await
        .expect("Failed to POST /waiting-for with HTMX");

    assert_eq!(
        response.status().as_u16(),
        204,
        "HTMX empty title should return 204"
    );
}

#[tokio::test]
async fn htmx_post_waiting_for_with_empty_waiting_on_returns_204() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf12@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/waiting-for", &app.address))
        .header("hx-request", "true")
        .form(&[("title", "Some item"), ("waiting_on", "   ")])
        .send()
        .await
        .expect("Failed to POST /waiting-for with HTMX");

    assert_eq!(
        response.status().as_u16(),
        204,
        "HTMX empty waiting_on should return 204"
    );
}

// ---- Complete (Received) Waiting-For Item ----

#[tokio::test]
async fn complete_waiting_for_removes_from_list() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf13@example.com", "securepassword123").await;

    // Add an item
    client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", "Receive package"), ("waiting_on", "Amazon")])
        .send()
        .await
        .expect("Failed to add waiting-for item");

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");
    let body = response.text().await.unwrap();
    let item_id = extract_waiting_for_id(&body);

    // Complete (mark as received)
    let response = client
        .post(format!("{}/waiting-for/{}/complete", &app.address, item_id))
        .send()
        .await
        .expect("Failed to complete waiting-for item");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Should redirect after completing item"
    );

    // Verify it's gone from the list
    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");
    let body = response.text().await.unwrap();
    assert!(
        !body.contains("Receive package"),
        "Completed item should not appear in active list"
    );
}

// ---- HTMX Complete ----

#[tokio::test]
async fn htmx_complete_waiting_for_returns_empty_with_announce() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf14@example.com", "securepassword123").await;

    client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", "HTMX received"), ("waiting_on", "Vendor")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");
    let body = response.text().await.unwrap();
    let item_id = extract_waiting_for_id(&body);

    let response = client
        .post(format!("{}/waiting-for/{}/complete", &app.address, item_id))
        .header("hx-request", "true")
        .send()
        .await
        .expect("Failed to HTMX complete item");

    assert_eq!(response.status().as_u16(), 200);
    let trigger = response
        .headers()
        .get("hx-trigger")
        .expect("Missing HX-Trigger");
    assert!(trigger.to_str().unwrap().contains("announce"));
    let body = response.text().await.unwrap();
    assert!(body.is_empty(), "HTMX complete should return empty body");
}

// ---- Delete Waiting-For Item ----

#[tokio::test]
async fn delete_waiting_for_removes_from_list() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf15@example.com", "securepassword123").await;

    client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", "Delete me"), ("waiting_on", "Someone")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");
    let body = response.text().await.unwrap();
    let item_id = extract_waiting_for_id(&body);

    let response = client
        .post(format!("{}/waiting-for/{}/delete", &app.address, item_id))
        .send()
        .await
        .expect("Failed to delete item");

    assert_eq!(response.status().as_u16(), 303);

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");
    let body = response.text().await.unwrap();
    assert!(
        !body.contains("Delete me"),
        "Deleted item should not appear in list"
    );
}

// ---- HTMX Delete ----

#[tokio::test]
async fn htmx_delete_waiting_for_returns_empty_with_announce() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf16@example.com", "securepassword123").await;

    client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", "HTMX delete me"), ("waiting_on", "Person")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");
    let body = response.text().await.unwrap();
    let item_id = extract_waiting_for_id(&body);

    let response = client
        .post(format!("{}/waiting-for/{}/delete", &app.address, item_id))
        .header("hx-request", "true")
        .send()
        .await
        .expect("Failed to HTMX delete item");

    assert_eq!(response.status().as_u16(), 200);
    let trigger = response
        .headers()
        .get("hx-trigger")
        .expect("Missing HX-Trigger");
    assert!(trigger.to_str().unwrap().contains("announce"));
    let body = response.text().await.unwrap();
    assert!(body.is_empty(), "HTMX delete should return empty body");
}

// ---- Edit Waiting-For Item ----

#[tokio::test]
async fn edit_waiting_for_updates_fields() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf17@example.com", "securepassword123").await;

    client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", "Old title"), ("waiting_on", "Old person")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");
    let body = response.text().await.unwrap();
    let item_id = extract_waiting_for_id(&body);

    let response = client
        .post(format!("{}/waiting-for/{}/edit", &app.address, item_id))
        .form(&[("title", "New title"), ("waiting_on", "New person")])
        .send()
        .await
        .expect("Failed to edit item");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Should redirect after editing"
    );

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");
    let body = response.text().await.unwrap();
    assert!(
        body.contains("New title"),
        "Updated title should appear in list"
    );
    assert!(
        body.contains("New person"),
        "Updated person should appear in list"
    );
    assert!(
        !body.contains("Old title"),
        "Old title should not appear in list"
    );
    assert!(
        !body.contains("Old person"),
        "Old person should not appear in list"
    );
}

#[tokio::test]
async fn htmx_edit_waiting_for_returns_fragment_with_announce() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf18@example.com", "securepassword123").await;

    client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", "HTMX edit item"), ("waiting_on", "Editor")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");
    let body = response.text().await.unwrap();
    let item_id = extract_waiting_for_id(&body);

    let response = client
        .post(format!("{}/waiting-for/{}/edit", &app.address, item_id))
        .header("hx-request", "true")
        .form(&[("title", "Updated HTMX"), ("waiting_on", "New editor")])
        .send()
        .await
        .expect("Failed to HTMX edit item");

    assert_eq!(response.status().as_u16(), 200);
    let trigger = response
        .headers()
        .get("hx-trigger")
        .expect("Missing HX-Trigger");
    assert!(trigger.to_str().unwrap().contains("announce"));
    let body = response.text().await.unwrap();
    assert!(
        body.contains("Updated HTMX"),
        "HTMX edit response should contain updated title"
    );
    assert!(
        body.contains("New editor"),
        "HTMX edit response should contain updated person"
    );
}

// ---- Accessible Labels ----

#[tokio::test]
async fn waiting_for_has_accessible_received_button() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf19@example.com", "securepassword123").await;

    client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", "My waiting item"), ("waiting_on", "Someone")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("aria-label=\"Received: My waiting item — Someone\""),
        "Received button should have accessible label with item title and waiting_on"
    );
}

#[tokio::test]
async fn waiting_for_has_accessible_delete_button() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf20@example.com", "securepassword123").await;

    client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", "My waiting item"), ("waiting_on", "Someone")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("aria-label=\"Delete: My waiting item — Someone\""),
        "Delete button should have accessible label with item title and waiting_on"
    );
}

#[tokio::test]
async fn waiting_for_has_accessible_edit_link() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf21@example.com", "securepassword123").await;

    client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", "My waiting item"), ("waiting_on", "Someone")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("aria-label=\"Edit: My waiting item — Someone\""),
        "Edit link should have accessible label with item title and waiting_on"
    );
}

// ---- Semantic List ----

#[tokio::test]
async fn waiting_for_list_uses_semantic_list() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf22@example.com", "securepassword123").await;

    client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", "Semantic item"), ("waiting_on", "Person")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("<ul") && body.contains("role=\"list\""),
        "Waiting-for list should use <ul> with role=\"list\""
    );
}

// ---- User Isolation ----

#[tokio::test]
async fn users_cannot_see_each_others_waiting_for_items() {
    let app = spawn_app().await;
    let user1 = register_and_login(&app.address, "wfuser1@example.com", "securepassword123").await;
    let user2 = register_and_login(&app.address, "wfuser2@example.com", "securepassword123").await;

    // User1 adds an item
    user1
        .post(format!("{}/waiting-for", &app.address))
        .form(&[
            ("title", "Secret waiting item"),
            ("waiting_on", "Secret person"),
        ])
        .send()
        .await
        .expect("Failed to add item");

    // User2 should not see it
    let response = user2
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for as user2");
    let body = response.text().await.unwrap();
    assert!(
        !body.contains("Secret waiting item"),
        "User2 should not see User1's waiting-for items"
    );
}

// ---- Complete and Delete return correct errors ----

#[tokio::test]
async fn complete_nonexistent_waiting_for_returns_404() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf23@example.com", "securepassword123").await;

    let response = client
        .post(format!(
            "{}/waiting-for/00000000-0000-0000-0000-000000000000/complete",
            &app.address
        ))
        .send()
        .await
        .expect("Failed to POST complete");

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn delete_nonexistent_waiting_for_returns_404() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf24@example.com", "securepassword123").await;

    let response = client
        .post(format!(
            "{}/waiting-for/00000000-0000-0000-0000-000000000000/delete",
            &app.address
        ))
        .send()
        .await
        .expect("Failed to POST delete");

    assert_eq!(response.status().as_u16(), 404);
}

// ---- Dashboard shows waiting-for count ----

#[tokio::test]
async fn dashboard_shows_waiting_for_count() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf25@example.com", "securepassword123").await;

    // Add a waiting-for item
    client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", "Dashboard waiting"), ("waiting_on", "Someone")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/dashboard", &app.address))
        .send()
        .await
        .expect("Failed to GET /dashboard");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("Waiting For"),
        "Dashboard should show Waiting For stat"
    );
    // After adding one item, the count should be 1
    assert!(
        body.contains("1 items"),
        "Dashboard should show 1 items for Waiting For, got: {}",
        body
    );
}

// ---- Focus Management Script ----

#[tokio::test]
async fn waiting_for_page_includes_focus_management_script() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf26@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("waiting-for-focus.js"),
        "Waiting For page should include focus management script"
    );
}

// ---- Add form has visible labels ----

#[tokio::test]
async fn waiting_for_add_form_has_visible_labels() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf27@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("What are you waiting for?"),
        "Add form should have a visible label for the title field"
    );
    assert!(
        body.contains("Who or what?"),
        "Add form should have a visible label for the person field"
    );
}

// ---- HTMX Get Edit Form ----

#[tokio::test]
async fn htmx_get_edit_returns_inline_form() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf28@example.com", "securepassword123").await;

    client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", "Editable item"), ("waiting_on", "Editor person")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");
    let body = response.text().await.unwrap();
    let item_id = extract_waiting_for_id(&body);

    let response = client
        .get(format!("{}/waiting-for/{}/edit", &app.address, item_id))
        .header("hx-request", "true")
        .send()
        .await
        .expect("Failed to GET edit form");

    assert_eq!(response.status().as_u16(), 200);
    let body = response.text().await.unwrap();
    assert!(
        body.contains("Editable item"),
        "Edit form should contain current title"
    );
    assert!(
        body.contains("Editor person"),
        "Edit form should contain current person"
    );
    assert!(body.contains("Save"), "Edit form should have a Save button");
    assert!(
        body.contains("Cancel"),
        "Edit form should have a Cancel link"
    );
}

// ---- Get single item (for cancel edit) ----

#[tokio::test]
async fn get_single_waiting_for_item_returns_fragment() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "wf29@example.com", "securepassword123").await;

    client
        .post(format!("{}/waiting-for", &app.address))
        .form(&[("title", "Single item"), ("waiting_on", "A person")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/waiting-for", &app.address))
        .send()
        .await
        .expect("Failed to GET /waiting-for");
    let body = response.text().await.unwrap();
    let item_id = extract_waiting_for_id(&body);

    let response = client
        .get(format!("{}/waiting-for/{}", &app.address, item_id))
        .send()
        .await
        .expect("Failed to GET single item");

    assert_eq!(response.status().as_u16(), 200);
    let body = response.text().await.unwrap();
    assert!(
        body.contains("Single item"),
        "Single item response should contain title"
    );
    assert!(
        body.contains("A person"),
        "Single item response should contain person"
    );
}
