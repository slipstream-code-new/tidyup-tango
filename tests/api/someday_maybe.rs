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

/// Helper: extract a someday/maybe item ID from the page body.
fn extract_someday_maybe_id(body: &str) -> String {
    let action_prefix = "action=\"/someday-maybe/";
    let start = body
        .find(action_prefix)
        .expect("Missing someday-maybe action in body")
        + action_prefix.len();
    let end = body[start..].find('/').expect("Missing / after UUID");
    body[start..start + end].to_string()
}

// ---- Page Loading ----

#[tokio::test]
async fn someday_maybe_page_returns_200_for_authenticated_user() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm1@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn someday_maybe_page_has_proper_heading() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm2@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("<h1>Someday/Maybe</h1>"),
        "Someday/Maybe page should have <h1>Someday/Maybe</h1>"
    );
}

#[tokio::test]
async fn someday_maybe_page_shows_empty_state_when_no_items() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm3@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("No ideas parked yet"),
        "Should show empty state message"
    );
}

#[tokio::test]
async fn someday_maybe_page_indicates_current_page_in_nav() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm4@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("aria-current=\"page\""),
        "Someday/Maybe nav link should be marked as current page"
    );
}

// ---- Add Someday/Maybe Item ----

#[tokio::test]
async fn post_someday_maybe_adds_item_and_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm5@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/someday-maybe", &app.address))
        .form(&[("title", "Learn Esperanto")])
        .send()
        .await
        .expect("Failed to POST /someday-maybe");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Should redirect after adding someday/maybe item"
    );

    // Verify the item appears in the list
    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");
    let body = response.text().await.unwrap();
    assert!(
        body.contains("Learn Esperanto"),
        "Item title should appear in the list"
    );
}

#[tokio::test]
async fn post_someday_maybe_with_empty_title_silently_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm6@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/someday-maybe", &app.address))
        .form(&[("title", "   ")])
        .send()
        .await
        .expect("Failed to POST /someday-maybe");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Empty title should silently redirect"
    );
}

#[tokio::test]
async fn post_someday_maybe_with_too_long_title_returns_422() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm7@example.com", "securepassword123").await;

    let long_title = "x".repeat(301);

    let response = client
        .post(format!("{}/someday-maybe", &app.address))
        .form(&[("title", long_title.as_str())])
        .send()
        .await
        .expect("Failed to POST /someday-maybe");

    assert_eq!(
        response.status().as_u16(),
        422,
        "Too long title should return 422"
    );
}

// ---- HTMX Add Someday/Maybe ----

#[tokio::test]
async fn htmx_post_someday_maybe_returns_fragment_with_announce() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm8@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/someday-maybe", &app.address))
        .header("hx-request", "true")
        .form(&[("title", "Build a treehouse")])
        .send()
        .await
        .expect("Failed to POST /someday-maybe with HTMX");

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
        body.contains("Build a treehouse"),
        "Response fragment should contain the item title"
    );
}

#[tokio::test]
async fn htmx_post_someday_maybe_with_empty_title_returns_204() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm9@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/someday-maybe", &app.address))
        .header("hx-request", "true")
        .form(&[("title", "   ")])
        .send()
        .await
        .expect("Failed to POST /someday-maybe with HTMX");

    assert_eq!(
        response.status().as_u16(),
        204,
        "HTMX empty title should return 204"
    );
}

// ---- Activate Someday/Maybe Item ----

#[tokio::test]
async fn activate_someday_maybe_moves_to_inbox_and_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm10@example.com", "securepassword123").await;

    // Add an item
    client
        .post(format!("{}/someday-maybe", &app.address))
        .form(&[("title", "Activate me")])
        .send()
        .await
        .expect("Failed to add someday/maybe item");

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");
    let body = response.text().await.unwrap();
    let item_id = extract_someday_maybe_id(&body);

    // Activate
    let response = client
        .post(format!(
            "{}/someday-maybe/{}/activate",
            &app.address, item_id
        ))
        .send()
        .await
        .expect("Failed to activate someday/maybe item");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Should redirect after activating item"
    );

    // Verify it's gone from someday/maybe
    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");
    let body = response.text().await.unwrap();
    assert!(
        !body.contains("Activate me"),
        "Activated item should not appear in someday/maybe list"
    );

    // Verify it appeared in inbox
    let response = client
        .get(format!("{}/inbox", &app.address))
        .send()
        .await
        .expect("Failed to GET /inbox");
    let body = response.text().await.unwrap();
    assert!(
        body.contains("Activate me"),
        "Activated item should appear in inbox"
    );
}

#[tokio::test]
async fn htmx_activate_someday_maybe_returns_empty_with_announce() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm11@example.com", "securepassword123").await;

    client
        .post(format!("{}/someday-maybe", &app.address))
        .form(&[("title", "HTMX activate")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");
    let body = response.text().await.unwrap();
    let item_id = extract_someday_maybe_id(&body);

    let response = client
        .post(format!(
            "{}/someday-maybe/{}/activate",
            &app.address, item_id
        ))
        .header("hx-request", "true")
        .send()
        .await
        .expect("Failed to HTMX activate item");

    assert_eq!(response.status().as_u16(), 200);
    let trigger = response
        .headers()
        .get("hx-trigger")
        .expect("Missing HX-Trigger");
    assert!(trigger.to_str().unwrap().contains("announce"));
    let body = response.text().await.unwrap();
    assert!(body.is_empty(), "HTMX activate should return empty body");
}

#[tokio::test]
async fn activate_nonexistent_someday_maybe_returns_404() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm12@example.com", "securepassword123").await;

    let response = client
        .post(format!(
            "{}/someday-maybe/00000000-0000-0000-0000-000000000000/activate",
            &app.address
        ))
        .send()
        .await
        .expect("Failed to POST activate");

    assert_eq!(response.status().as_u16(), 404);
}

// ---- Delete Someday/Maybe Item ----

#[tokio::test]
async fn delete_someday_maybe_removes_from_list() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm13@example.com", "securepassword123").await;

    client
        .post(format!("{}/someday-maybe", &app.address))
        .form(&[("title", "Delete me")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");
    let body = response.text().await.unwrap();
    let item_id = extract_someday_maybe_id(&body);

    let response = client
        .post(format!("{}/someday-maybe/{}/delete", &app.address, item_id))
        .send()
        .await
        .expect("Failed to delete item");

    assert_eq!(response.status().as_u16(), 303);

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");
    let body = response.text().await.unwrap();
    assert!(
        !body.contains("Delete me"),
        "Deleted item should not appear in list"
    );
}

#[tokio::test]
async fn htmx_delete_someday_maybe_returns_empty_with_announce() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm14@example.com", "securepassword123").await;

    client
        .post(format!("{}/someday-maybe", &app.address))
        .form(&[("title", "HTMX delete me")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");
    let body = response.text().await.unwrap();
    let item_id = extract_someday_maybe_id(&body);

    let response = client
        .post(format!("{}/someday-maybe/{}/delete", &app.address, item_id))
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

#[tokio::test]
async fn delete_nonexistent_someday_maybe_returns_404() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm15@example.com", "securepassword123").await;

    let response = client
        .post(format!(
            "{}/someday-maybe/00000000-0000-0000-0000-000000000000/delete",
            &app.address
        ))
        .send()
        .await
        .expect("Failed to POST delete");

    assert_eq!(response.status().as_u16(), 404);
}

// ---- Edit Someday/Maybe Item ----

#[tokio::test]
async fn edit_someday_maybe_updates_title() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm16@example.com", "securepassword123").await;

    client
        .post(format!("{}/someday-maybe", &app.address))
        .form(&[("title", "Old idea")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");
    let body = response.text().await.unwrap();
    let item_id = extract_someday_maybe_id(&body);

    let response = client
        .post(format!("{}/someday-maybe/{}/edit", &app.address, item_id))
        .form(&[("title", "New idea")])
        .send()
        .await
        .expect("Failed to edit item");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Should redirect after editing"
    );

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");
    let body = response.text().await.unwrap();
    assert!(
        body.contains("New idea"),
        "Updated title should appear in list"
    );
    assert!(
        !body.contains("Old idea"),
        "Old title should not appear in list"
    );
}

#[tokio::test]
async fn htmx_edit_someday_maybe_returns_fragment_with_announce() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm17@example.com", "securepassword123").await;

    client
        .post(format!("{}/someday-maybe", &app.address))
        .form(&[("title", "HTMX edit idea")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");
    let body = response.text().await.unwrap();
    let item_id = extract_someday_maybe_id(&body);

    let response = client
        .post(format!("{}/someday-maybe/{}/edit", &app.address, item_id))
        .header("hx-request", "true")
        .form(&[("title", "Updated HTMX idea")])
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
        body.contains("Updated HTMX idea"),
        "HTMX edit response should contain updated title"
    );
}

// ---- Accessible Labels ----

#[tokio::test]
async fn someday_maybe_has_accessible_activate_button() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm18@example.com", "securepassword123").await;

    client
        .post(format!("{}/someday-maybe", &app.address))
        .form(&[("title", "My parked idea")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("aria-label=\"Move to Inbox: My parked idea\""),
        "Activate button should have accessible label with item title"
    );
}

#[tokio::test]
async fn someday_maybe_has_accessible_delete_button() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm19@example.com", "securepassword123").await;

    client
        .post(format!("{}/someday-maybe", &app.address))
        .form(&[("title", "My parked idea")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("aria-label=\"Delete: My parked idea\""),
        "Delete button should have accessible label with item title"
    );
}

#[tokio::test]
async fn someday_maybe_has_accessible_edit_link() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm20@example.com", "securepassword123").await;

    client
        .post(format!("{}/someday-maybe", &app.address))
        .form(&[("title", "My parked idea")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("aria-label=\"Edit: My parked idea\""),
        "Edit link should have accessible label with item title"
    );
}

// ---- Semantic List ----

#[tokio::test]
async fn someday_maybe_list_uses_semantic_list() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm21@example.com", "securepassword123").await;

    client
        .post(format!("{}/someday-maybe", &app.address))
        .form(&[("title", "Semantic idea")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("<ul") && body.contains("role=\"list\""),
        "Someday/maybe list should use <ul> with role=\"list\""
    );
}

// ---- User Isolation ----

#[tokio::test]
async fn users_cannot_see_each_others_someday_maybe_items() {
    let app = spawn_app().await;
    let user1 = register_and_login(&app.address, "smuser1@example.com", "securepassword123").await;
    let user2 = register_and_login(&app.address, "smuser2@example.com", "securepassword123").await;

    // User1 adds an item
    user1
        .post(format!("{}/someday-maybe", &app.address))
        .form(&[("title", "Secret idea")])
        .send()
        .await
        .expect("Failed to add item");

    // User2 should not see it
    let response = user2
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe as user2");
    let body = response.text().await.unwrap();
    assert!(
        !body.contains("Secret idea"),
        "User2 should not see User1's someday/maybe items"
    );
}

// ---- Add form has visible label ----

#[tokio::test]
async fn someday_maybe_add_form_has_visible_label() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm22@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("Park an idea"),
        "Add form should have a visible label for the title field"
    );
}

// ---- HTMX Get Edit Form ----

#[tokio::test]
async fn htmx_get_edit_returns_inline_form() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm23@example.com", "securepassword123").await;

    client
        .post(format!("{}/someday-maybe", &app.address))
        .form(&[("title", "Editable idea")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");
    let body = response.text().await.unwrap();
    let item_id = extract_someday_maybe_id(&body);

    let response = client
        .get(format!("{}/someday-maybe/{}/edit", &app.address, item_id))
        .header("hx-request", "true")
        .send()
        .await
        .expect("Failed to GET edit form");

    assert_eq!(response.status().as_u16(), 200);
    let body = response.text().await.unwrap();
    assert!(
        body.contains("Editable idea"),
        "Edit form should contain current title"
    );
    assert!(body.contains("Save"), "Edit form should have a Save button");
    assert!(
        body.contains("Cancel"),
        "Edit form should have a Cancel link"
    );
}

// ---- Get single item (for cancel edit) ----

#[tokio::test]
async fn get_single_someday_maybe_item_returns_fragment() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "sm24@example.com", "securepassword123").await;

    client
        .post(format!("{}/someday-maybe", &app.address))
        .form(&[("title", "Single idea")])
        .send()
        .await
        .expect("Failed to add item");

    let response = client
        .get(format!("{}/someday-maybe", &app.address))
        .send()
        .await
        .expect("Failed to GET /someday-maybe");
    let body = response.text().await.unwrap();
    let item_id = extract_someday_maybe_id(&body);

    let response = client
        .get(format!("{}/someday-maybe/{}", &app.address, item_id))
        .send()
        .await
        .expect("Failed to GET single item");

    assert_eq!(response.status().as_u16(), 200);
    let body = response.text().await.unwrap();
    assert!(
        body.contains("Single idea"),
        "Single item response should contain title"
    );
}
