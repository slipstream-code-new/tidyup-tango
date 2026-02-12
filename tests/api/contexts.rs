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

// ---- Default Contexts Seeded on Registration ----

#[tokio::test]
async fn registration_seeds_default_contexts() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "defaults@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/contexts", &app.address))
        .send()
        .await
        .expect("Failed to GET /contexts");

    assert_eq!(response.status().as_u16(), 200);
    let body = response.text().await.unwrap();

    assert!(body.contains("@computer"), "Should have @computer context");
    assert!(body.contains("@home"), "Should have @home context");
    assert!(body.contains("@errands"), "Should have @errands context");
    assert!(body.contains("@phone"), "Should have @phone context");
    assert!(body.contains("@anywhere"), "Should have @anywhere context");
}

// ---- List Contexts ----

#[tokio::test]
async fn contexts_page_returns_200_for_authenticated_user() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "ctxpage@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/contexts", &app.address))
        .send()
        .await
        .expect("Failed to GET /contexts");

    assert_eq!(response.status().as_u16(), 200);
    let body = response.text().await.unwrap();
    assert!(
        body.contains("<h1>Contexts</h1>"),
        "Contexts page should have <h1>Contexts</h1>"
    );
}

#[tokio::test]
async fn contexts_page_redirects_to_login_when_unauthenticated() {
    let app = spawn_app().await;
    let client = authenticated_client();

    let response = client
        .get(format!("{}/contexts", &app.address))
        .send()
        .await
        .expect("Failed to GET /contexts");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Unauthenticated request should redirect"
    );
}

// ---- Add Context ----

#[tokio::test]
async fn post_context_adds_context_and_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "addctx@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/contexts", &app.address))
        .form(&[("name", "@office")])
        .send()
        .await
        .expect("Failed to post context");

    assert_eq!(
        response.status().as_u16(),
        303,
        "POST /contexts should redirect"
    );

    let response = client
        .get(format!("{}/contexts", &app.address))
        .send()
        .await
        .expect("Failed to GET /contexts");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("@office"),
        "Contexts page should show newly added context"
    );
}

#[tokio::test]
async fn post_context_with_empty_name_silently_redirects() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "emptyctx@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/contexts", &app.address))
        .form(&[("name", "")])
        .send()
        .await
        .expect("Failed to post context");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Empty name should silently redirect"
    );
}

#[tokio::test]
async fn post_context_with_too_long_name_returns_422() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "longctx@example.com", "securepassword123").await;

    let long_name = "a".repeat(51);
    let response = client
        .post(format!("{}/contexts", &app.address))
        .form(&[("name", long_name.as_str())])
        .send()
        .await
        .expect("Failed to post context");

    assert_eq!(
        response.status().as_u16(),
        422,
        "Too-long name should return 422"
    );
}

#[tokio::test]
async fn post_context_with_duplicate_name_returns_422() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "dupctx@example.com", "securepassword123").await;

    // @computer already exists from default seeding
    let response = client
        .post(format!("{}/contexts", &app.address))
        .form(&[("name", "@computer")])
        .send()
        .await
        .expect("Failed to post context");

    assert_eq!(
        response.status().as_u16(),
        422,
        "Duplicate context name should return 422"
    );

    let body = response.text().await.unwrap();
    assert!(
        body.contains("already exists"),
        "Should show duplicate name error message"
    );
}

// ---- Delete Context ----

#[tokio::test]
async fn delete_context_removes_it_and_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "delctx@example.com", "securepassword123").await;

    // Add a custom context to delete
    client
        .post(format!("{}/contexts", &app.address))
        .form(&[("name", "@deleteme")])
        .send()
        .await
        .expect("Failed to add context");

    // Get contexts page to find the context's ID
    let response = client
        .get(format!("{}/contexts", &app.address))
        .send()
        .await
        .expect("Failed to GET /contexts");

    let body = response.text().await.unwrap();
    assert!(body.contains("@deleteme"));

    // Extract the context ID
    // Find the delete form for @deleteme specifically
    let deleteme_pos = body.find("@deleteme").expect("@deleteme should be in body");
    let search_area = &body[..deleteme_pos];
    // Find the last context-item before @deleteme
    let item_start = search_area
        .rfind("id=\"context-item-")
        .expect("context item id");
    let id_start = item_start + "id=\"context-item-".len();
    let id_end = body[id_start..].find('"').expect("closing quote");
    let context_id = &body[id_start..id_start + id_end];

    // Delete the context
    let response = client
        .post(format!("{}/contexts/{}/delete", &app.address, context_id))
        .send()
        .await
        .expect("Failed to delete context");

    assert_eq!(response.status().as_u16(), 303, "DELETE should redirect");

    // Verify context is gone
    let response = client
        .get(format!("{}/contexts", &app.address))
        .send()
        .await
        .expect("Failed to GET /contexts");

    let body = response.text().await.unwrap();
    assert!(
        !body.contains("@deleteme"),
        "Deleted context should no longer appear"
    );
}

#[tokio::test]
async fn delete_nonexistent_context_returns_404() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "del404ctx@example.com", "securepassword123").await;

    let fake_id = uuid::Uuid::new_v4();
    let response = client
        .post(format!("{}/contexts/{}/delete", &app.address, fake_id))
        .send()
        .await
        .expect("Failed to delete");

    assert_eq!(response.status().as_u16(), 404);
}

// ---- Edit Context ----

#[tokio::test]
async fn edit_context_updates_name_and_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "editctx@example.com", "securepassword123").await;

    // Add a custom context
    client
        .post(format!("{}/contexts", &app.address))
        .form(&[("name", "@oldname")])
        .send()
        .await
        .expect("Failed to add context");

    // Get the context ID
    let response = client
        .get(format!("{}/contexts", &app.address))
        .send()
        .await
        .expect("Failed to GET /contexts");

    let body = response.text().await.unwrap();
    let oldname_pos = body.find("@oldname").expect("@oldname should be in body");
    let search_area = &body[..oldname_pos];
    let item_start = search_area
        .rfind("id=\"context-item-")
        .expect("context item id");
    let id_start = item_start + "id=\"context-item-".len();
    let id_end = body[id_start..].find('"').expect("closing quote");
    let context_id = &body[id_start..id_start + id_end];

    // Edit the context
    let response = client
        .post(format!("{}/contexts/{}/edit", &app.address, context_id))
        .form(&[("name", "@newname")])
        .send()
        .await
        .expect("Failed to edit context");

    assert_eq!(response.status().as_u16(), 303, "Edit should redirect");

    // Verify the name was updated
    let response = client
        .get(format!("{}/contexts", &app.address))
        .send()
        .await
        .expect("Failed to GET /contexts");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("@newname"),
        "Updated context name should appear"
    );
    assert!(
        !body.contains("@oldname"),
        "Old context name should not appear"
    );
}

// ---- User Isolation ----

#[tokio::test]
async fn users_cannot_see_each_others_contexts() {
    let app = spawn_app().await;

    let client_a = register_and_login(&app.address, "ctx_a@example.com", "securepassword123").await;
    let client_b = register_and_login(&app.address, "ctx_b@example.com", "securepassword123").await;

    // User A adds a custom context
    client_a
        .post(format!("{}/contexts", &app.address))
        .form(&[("name", "@user_a_only")])
        .send()
        .await
        .expect("Failed to add context");

    // User B should not see it
    let response = client_b
        .get(format!("{}/contexts", &app.address))
        .send()
        .await
        .expect("Failed to GET /contexts");

    let body = response.text().await.unwrap();
    assert!(
        !body.contains("@user_a_only"),
        "User B should not see User A's contexts"
    );
}

// ---- Accessibility ----

#[tokio::test]
async fn contexts_page_has_visible_label_for_add_input() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "ctxa11y@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/contexts", &app.address))
        .send()
        .await
        .expect("Failed to GET /contexts");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("for=\"context-name\""),
        "Label should be associated with context name input"
    );
}

#[tokio::test]
async fn delete_context_button_has_accessible_label() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "ctxdellabel@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/contexts", &app.address))
        .send()
        .await
        .expect("Failed to GET /contexts");

    let body = response.text().await.unwrap();
    // Default contexts should have accessible delete labels
    assert!(
        body.contains("aria-label=\"Delete: @computer\""),
        "Delete button should have aria-label including the context name"
    );
}

#[tokio::test]
async fn edit_context_link_has_accessible_label() {
    let app = spawn_app().await;
    let client = register_and_login(
        &app.address,
        "ctxeditlabel@example.com",
        "securepassword123",
    )
    .await;

    let response = client
        .get(format!("{}/contexts", &app.address))
        .send()
        .await
        .expect("Failed to GET /contexts");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("aria-label=\"Edit: @computer\""),
        "Edit link should have aria-label including the context name"
    );
}

// ---- Navigation ----

#[tokio::test]
async fn contexts_link_appears_in_navigation() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "ctxnav@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/dashboard", &app.address))
        .send()
        .await
        .expect("Failed to GET /dashboard");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("href=\"/contexts\""),
        "Header should include link to contexts page"
    );
}

#[tokio::test]
async fn contexts_page_indicates_current_page_in_nav() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "ctxcurrent@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/contexts", &app.address))
        .send()
        .await
        .expect("Failed to GET /contexts");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("aria-current=\"page\""),
        "Contexts nav link should indicate current page"
    );
}

// ---- HTMX Fragment Responses ----

#[tokio::test]
async fn htmx_post_context_returns_fragment_with_announce_header() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "htmxctx@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/contexts", &app.address))
        .header("hx-request", "true")
        .form(&[("name", "@htmx-test")])
        .send()
        .await
        .expect("Failed to post context");

    assert_eq!(
        response.status().as_u16(),
        200,
        "HTMX POST should return 200 with fragment"
    );

    let trigger = response
        .headers()
        .get("hx-trigger")
        .expect("Missing HX-Trigger header")
        .to_str()
        .unwrap();
    assert!(
        trigger.contains("Context added"),
        "HX-Trigger should contain 'Context added' announce"
    );

    let body = response.text().await.unwrap();
    assert!(
        body.contains("@htmx-test"),
        "Fragment should contain the added context"
    );
}

#[tokio::test]
async fn htmx_delete_context_returns_empty_body_with_announce() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "htmxdelctx@example.com", "securepassword123").await;

    // Add a custom context
    client
        .post(format!("{}/contexts", &app.address))
        .form(&[("name", "@htmx-del")])
        .send()
        .await
        .expect("Failed to add context");

    // Get context ID
    let response = client
        .get(format!("{}/contexts", &app.address))
        .send()
        .await
        .expect("Failed to GET /contexts");

    let body = response.text().await.unwrap();
    let target_pos = body.find("@htmx-del").expect("@htmx-del should be in body");
    let search_area = &body[..target_pos];
    let item_start = search_area
        .rfind("id=\"context-item-")
        .expect("context item id");
    let id_start = item_start + "id=\"context-item-".len();
    let id_end = body[id_start..].find('"').expect("closing quote");
    let context_id = &body[id_start..id_start + id_end];

    // Delete via HTMX
    let response = client
        .post(format!("{}/contexts/{}/delete", &app.address, context_id))
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
        trigger.contains("Context deleted"),
        "HX-Trigger should contain 'Context deleted' announce"
    );

    let body = response.text().await.unwrap();
    assert!(body.is_empty(), "HTMX delete should return empty body");
}
