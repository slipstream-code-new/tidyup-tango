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

/// Helper: get the first context ID from the contexts page
async fn get_first_context_id(client: &reqwest::Client, address: &str) -> String {
    let response = client
        .get(format!("{address}/contexts"))
        .send()
        .await
        .expect("Failed to GET /contexts");

    let body = response.text().await.unwrap();
    // Extract the first context ID from a delete form action
    let action_prefix = "action=\"/contexts/";
    let start = body.find(action_prefix).expect("Missing context action") + action_prefix.len();
    let end = body[start..].find('/').expect("Missing / after context ID");
    body[start..start + end].to_string()
}

// ---- Page Loading ----

#[tokio::test]
async fn next_actions_page_returns_200_for_authenticated_user() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "na1@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET /next-actions");

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn next_actions_page_has_proper_heading() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "na2@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET /next-actions");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("<h1>Next Actions</h1>"),
        "Next Actions page should have <h1>Next Actions</h1>"
    );
}

#[tokio::test]
async fn next_actions_page_shows_empty_state_when_no_actions() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "na3@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET /next-actions");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("No next actions yet"),
        "Empty state should show guidance message"
    );
}

#[tokio::test]
async fn next_actions_page_indicates_current_page_in_nav() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "na4@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET /next-actions");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("aria-current=\"page\""),
        "Next Actions should show current page in nav"
    );
}

// ---- Add Next Action ----

#[tokio::test]
async fn post_next_action_adds_item_and_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "add1@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;

    let response = client
        .post(format!("{}/next-actions", &app.address))
        .form(&[("title", "Buy groceries"), ("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to post next action");

    assert_eq!(
        response.status().as_u16(),
        303,
        "POST /next-actions should redirect"
    );

    // Verify the item shows on the page
    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET /next-actions");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("Buy groceries"),
        "Next Actions page should show the added action"
    );
}

#[tokio::test]
async fn post_next_action_with_empty_title_silently_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "add2@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;

    let response = client
        .post(format!("{}/next-actions", &app.address))
        .form(&[("title", ""), ("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to post next action");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Empty title should silently redirect"
    );
}

#[tokio::test]
async fn post_next_action_with_too_long_title_returns_422() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "add3@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;
    let long_title = "a".repeat(301);

    let response = client
        .post(format!("{}/next-actions", &app.address))
        .form(&[("title", long_title.as_str()), ("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to post next action");

    assert_eq!(
        response.status().as_u16(),
        422,
        "Too-long title should return 422"
    );
}

// ---- Next Action Shows Context ----

#[tokio::test]
async fn next_action_shows_context_name() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "ctx1@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;

    client
        .post(format!("{}/next-actions", &app.address))
        .form(&[("title", "Call dentist"), ("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to post next action");

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET /next-actions");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("Call dentist"),
        "Action should show its title"
    );
    // The context name should appear (one of the default contexts)
    assert!(
        body.contains("@computer")
            || body.contains("@home")
            || body.contains("@errands")
            || body.contains("@phone")
            || body.contains("@anywhere"),
        "Action should show its context name"
    );
}

// ---- Context Filter ----

#[tokio::test]
async fn next_actions_page_shows_context_filter_links() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "filter1@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET /next-actions");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("Filter by context"),
        "Page should have context filter navigation"
    );
    assert!(body.contains("All"), "Filter should have an 'All' option");
    assert!(
        body.contains("@computer"),
        "Filter should show default contexts"
    );
}

#[tokio::test]
async fn next_actions_filter_by_context_shows_only_matching_actions() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "filter2@example.com", "securepassword123").await;

    // Get contexts page to find two different context IDs
    let response = client
        .get(format!("{}/contexts", &app.address))
        .send()
        .await
        .expect("Failed to GET /contexts");

    let body = response.text().await.unwrap();

    // Extract first two context IDs
    let action_prefix = "action=\"/contexts/";
    let start1 = body.find(action_prefix).expect("Missing first context") + action_prefix.len();
    let end1 = body[start1..].find('/').unwrap();
    let context_id_1 = body[start1..start1 + end1].to_string();

    let remaining = &body[start1 + end1..];
    let start2 = remaining
        .find(action_prefix)
        .expect("Missing second context")
        + action_prefix.len();
    let end2 = remaining[start2..].find('/').unwrap();
    let context_id_2 = remaining[start2..start2 + end2].to_string();

    // Add action to context 1
    client
        .post(format!("{}/next-actions", &app.address))
        .form(&[("title", "Context 1 action"), ("context_id", &context_id_1)])
        .send()
        .await
        .expect("Failed to add action");

    // Add action to context 2
    client
        .post(format!("{}/next-actions", &app.address))
        .form(&[("title", "Context 2 action"), ("context_id", &context_id_2)])
        .send()
        .await
        .expect("Failed to add action");

    // Filter by context 1
    let response = client
        .get(format!(
            "{}/next-actions?context={}",
            &app.address, &context_id_1
        ))
        .send()
        .await
        .expect("Failed to GET filtered next-actions");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("Context 1 action"),
        "Filtered view should show matching actions"
    );
    assert!(
        !body.contains("Context 2 action"),
        "Filtered view should NOT show non-matching actions"
    );
}

// ---- Complete ----

#[tokio::test]
async fn complete_next_action_removes_from_list() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "comp1@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;

    client
        .post(format!("{}/next-actions", &app.address))
        .form(&[("title", "Complete me"), ("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to add");

    // Get the action ID
    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET");
    let body = response.text().await.unwrap();

    let action_prefix = "action=\"/next-actions/";
    let start = body.find(action_prefix).expect("Missing action") + action_prefix.len();
    let end = body[start..]
        .find("/complete\"")
        .expect("Missing /complete");
    let action_id = &body[start..start + end];

    // Complete it
    let response = client
        .post(format!(
            "{}/next-actions/{}/complete",
            &app.address, action_id
        ))
        .send()
        .await
        .expect("Failed to complete");

    assert_eq!(response.status().as_u16(), 303);

    // Verify it's gone from the active list
    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET");
    let body = response.text().await.unwrap();
    assert!(
        !body.contains("Complete me"),
        "Completed action should not appear in active list"
    );
}

// ---- Delete ----

#[tokio::test]
async fn delete_next_action_removes_from_list() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "del1@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;

    client
        .post(format!("{}/next-actions", &app.address))
        .form(&[("title", "Delete me"), ("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to add");

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET");
    let body = response.text().await.unwrap();

    let action_prefix = "action=\"/next-actions/";
    let start = body.find(action_prefix).expect("Missing action") + action_prefix.len();
    let end = body[start..]
        .find("/complete\"")
        .expect("Missing /complete");
    let action_id = &body[start..start + end];

    let response = client
        .post(format!(
            "{}/next-actions/{}/delete",
            &app.address, action_id
        ))
        .send()
        .await
        .expect("Failed to delete");

    assert_eq!(response.status().as_u16(), 303);

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET");
    let body = response.text().await.unwrap();
    assert!(
        !body.contains("Delete me"),
        "Deleted action should not appear"
    );
}

#[tokio::test]
async fn delete_nonexistent_next_action_returns_404() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "del2@example.com", "securepassword123").await;

    let fake_id = uuid::Uuid::new_v4();
    let response = client
        .post(format!("{}/next-actions/{}/delete", &app.address, fake_id))
        .send()
        .await
        .expect("Failed to delete");

    assert_eq!(response.status().as_u16(), 404);
}

// ---- Edit ----

#[tokio::test]
async fn edit_next_action_updates_title() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "edit1@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;

    client
        .post(format!("{}/next-actions", &app.address))
        .form(&[("title", "Original title"), ("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to add");

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET");
    let body = response.text().await.unwrap();

    let action_prefix = "action=\"/next-actions/";
    let start = body.find(action_prefix).expect("Missing action") + action_prefix.len();
    let end = body[start..]
        .find("/complete\"")
        .expect("Missing /complete");
    let action_id = &body[start..start + end];

    let response = client
        .post(format!("{}/next-actions/{}/edit", &app.address, action_id))
        .form(&[("title", "Updated title")])
        .send()
        .await
        .expect("Failed to edit");

    assert_eq!(response.status().as_u16(), 303);

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET");
    let body = response.text().await.unwrap();
    assert!(
        body.contains("Updated title"),
        "Updated title should appear"
    );
    assert!(
        !body.contains("Original title"),
        "Original title should be gone"
    );
}

#[tokio::test]
async fn edit_next_action_with_empty_title_returns_422() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "edit2@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;

    client
        .post(format!("{}/next-actions", &app.address))
        .form(&[("title", "Some action"), ("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to add");

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET");
    let body = response.text().await.unwrap();

    let action_prefix = "action=\"/next-actions/";
    let start = body.find(action_prefix).expect("Missing action") + action_prefix.len();
    let end = body[start..]
        .find("/complete\"")
        .expect("Missing /complete");
    let action_id = &body[start..start + end];

    let response = client
        .post(format!("{}/next-actions/{}/edit", &app.address, action_id))
        .form(&[("title", "")])
        .send()
        .await
        .expect("Failed to edit");

    assert_eq!(
        response.status().as_u16(),
        422,
        "Empty title should return 422"
    );
}

// ---- Accessibility ----

#[tokio::test]
async fn next_action_has_accessible_complete_button() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "a11y1@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;

    client
        .post(format!("{}/next-actions", &app.address))
        .form(&[("title", "Accessible task"), ("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to add");

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET");
    let body = response.text().await.unwrap();

    assert!(
        body.contains("aria-label=\"Complete: Accessible task\""),
        "Complete button should have aria-label including item title"
    );
}

#[tokio::test]
async fn next_action_has_accessible_delete_button() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "a11y2@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;

    client
        .post(format!("{}/next-actions", &app.address))
        .form(&[("title", "Delete a11y test"), ("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to add");

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET");
    let body = response.text().await.unwrap();

    assert!(
        body.contains("aria-label=\"Delete: Delete a11y test\""),
        "Delete button should have aria-label including item title"
    );
}

#[tokio::test]
async fn next_action_has_accessible_edit_link() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "a11y3@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;

    client
        .post(format!("{}/next-actions", &app.address))
        .form(&[("title", "Edit a11y test"), ("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to add");

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET");
    let body = response.text().await.unwrap();

    assert!(
        body.contains("aria-label=\"Edit: Edit a11y test\""),
        "Edit link should have aria-label including item title"
    );
}

#[tokio::test]
async fn next_actions_page_has_visible_label_for_add_input() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "a11y4@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("for=\"next-action-title\""),
        "Add form should have a label associated with the title input"
    );
}

// ---- User Isolation ----

#[tokio::test]
async fn users_cannot_see_each_others_next_actions() {
    let app = spawn_app().await;

    let client_a = register_and_login(&app.address, "ua@example.com", "securepassword123").await;
    let client_b = register_and_login(&app.address, "ub@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client_a, &app.address).await;

    client_a
        .post(format!("{}/next-actions", &app.address))
        .form(&[("title", "User A secret"), ("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to add");

    let response = client_b
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET");

    let body = response.text().await.unwrap();
    assert!(
        !body.contains("User A secret"),
        "User B should not see User A's next actions"
    );
}

// ---- HTMX Fragment Responses ----

#[tokio::test]
async fn htmx_post_next_action_returns_fragment_with_announce() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "htmx1@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;

    let response = client
        .post(format!("{}/next-actions", &app.address))
        .header("hx-request", "true")
        .form(&[("title", "HTMX action"), ("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to post");

    assert_eq!(response.status().as_u16(), 200);

    let trigger = response
        .headers()
        .get("hx-trigger")
        .expect("Missing HX-Trigger")
        .to_str()
        .unwrap();
    assert!(
        trigger.contains("Next action added"),
        "HX-Trigger should contain announce"
    );

    let body = response.text().await.unwrap();
    assert!(
        body.contains("HTMX action"),
        "Fragment should contain the action"
    );
}

#[tokio::test]
async fn htmx_complete_next_action_returns_empty_with_announce() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "htmx2@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;

    client
        .post(format!("{}/next-actions", &app.address))
        .form(&[("title", "HTMX complete"), ("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to add");

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET");
    let body = response.text().await.unwrap();

    let action_prefix = "action=\"/next-actions/";
    let start = body.find(action_prefix).unwrap() + action_prefix.len();
    let end = body[start..].find("/complete\"").unwrap();
    let action_id = &body[start..start + end];

    let response = client
        .post(format!(
            "{}/next-actions/{}/complete",
            &app.address, action_id
        ))
        .header("hx-request", "true")
        .send()
        .await
        .expect("Failed to complete");

    assert_eq!(response.status().as_u16(), 200);

    let trigger = response
        .headers()
        .get("hx-trigger")
        .expect("Missing HX-Trigger")
        .to_str()
        .unwrap();
    assert!(trigger.contains("Action completed"));

    let body = response.text().await.unwrap();
    assert!(body.is_empty(), "HTMX complete should return empty body");
}

#[tokio::test]
async fn htmx_delete_next_action_returns_empty_with_announce() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "htmx3@example.com", "securepassword123").await;

    let context_id = get_first_context_id(&client, &app.address).await;

    client
        .post(format!("{}/next-actions", &app.address))
        .form(&[("title", "HTMX delete"), ("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to add");

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET");
    let body = response.text().await.unwrap();

    let action_prefix = "action=\"/next-actions/";
    let start = body.find(action_prefix).unwrap() + action_prefix.len();
    let end = body[start..].find("/complete\"").unwrap();
    let action_id = &body[start..start + end];

    let response = client
        .post(format!(
            "{}/next-actions/{}/delete",
            &app.address, action_id
        ))
        .header("hx-request", "true")
        .send()
        .await
        .expect("Failed to delete");

    assert_eq!(response.status().as_u16(), 200);

    let trigger = response
        .headers()
        .get("hx-trigger")
        .expect("Missing HX-Trigger")
        .to_str()
        .unwrap();
    assert!(trigger.contains("Next action deleted"));

    let body = response.text().await.unwrap();
    assert!(body.is_empty(), "HTMX delete should return empty body");
}

// ---- Unauthenticated Access ----

#[tokio::test]
async fn next_actions_page_redirects_to_login_when_unauthenticated() {
    let app = spawn_app().await;
    let client = authenticated_client();

    let response = client
        .get(format!("{}/next-actions", &app.address))
        .send()
        .await
        .expect("Failed to GET /next-actions");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Unauthenticated user should be redirected"
    );
}
