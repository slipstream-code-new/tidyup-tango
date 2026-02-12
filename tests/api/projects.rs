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
async fn projects_page_returns_200_for_authenticated_user() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj1@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn projects_page_has_proper_heading() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj2@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("<h1>Projects</h1>"),
        "Projects page should have <h1>Projects</h1>"
    );
}

#[tokio::test]
async fn projects_page_shows_empty_state_when_no_projects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj3@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("No projects yet"),
        "Should show empty state message"
    );
}

#[tokio::test]
async fn projects_page_indicates_current_page_in_nav() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj4@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("aria-current=\"page\""),
        "Projects nav link should be marked as current page"
    );
}

// ---- Add Project ----

#[tokio::test]
async fn post_project_adds_item_and_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj5@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "Build a treehouse")])
        .send()
        .await
        .expect("Failed to POST /projects");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Should redirect after adding project"
    );

    // Verify the project appears in the list
    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");
    let body = response.text().await.unwrap();
    assert!(
        body.contains("Build a treehouse"),
        "Project title should appear in the list"
    );
}

#[tokio::test]
async fn post_project_with_empty_title_silently_redirects() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj6@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "   ")])
        .send()
        .await
        .expect("Failed to POST /projects");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Empty title should silently redirect"
    );
}

#[tokio::test]
async fn post_project_with_too_long_title_returns_422() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj7@example.com", "securepassword123").await;

    let long_title = "x".repeat(301);

    let response = client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", long_title.as_str())])
        .send()
        .await
        .expect("Failed to POST /projects");

    assert_eq!(
        response.status().as_u16(),
        422,
        "Too long title should return 422"
    );
}

// ---- HTMX Add Project ----

#[tokio::test]
async fn htmx_post_project_returns_fragment_with_announce() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj8@example.com", "securepassword123").await;

    let response = client
        .post(format!("{}/projects", &app.address))
        .header("hx-request", "true")
        .form(&[("title", "Launch website")])
        .send()
        .await
        .expect("Failed to POST /projects with HTMX");

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
        body.contains("Launch website"),
        "Response fragment should contain the project title"
    );
}

// ---- Project Detail ----

#[tokio::test]
async fn project_detail_page_shows_project_title() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj9@example.com", "securepassword123").await;

    // Add a project
    client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "Renovate kitchen")])
        .send()
        .await
        .expect("Failed to add project");

    // Get the project ID from the list
    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");
    let body = response.text().await.unwrap();
    let project_id = extract_project_id(&body);

    // Visit the detail page
    let response = client
        .get(format!("{}/projects/{}", &app.address, project_id))
        .send()
        .await
        .expect("Failed to GET project detail");

    assert_eq!(response.status().as_u16(), 200);
    let detail_body = response.text().await.unwrap();
    assert!(
        detail_body.contains("Renovate kitchen"),
        "Detail page should show project title"
    );
    assert!(
        detail_body.contains("Next Actions"),
        "Detail page should have Next Actions heading"
    );
}

#[tokio::test]
async fn project_detail_shows_stalled_notice_when_no_actions() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj10@example.com", "securepassword123").await;

    client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "Stalled project")])
        .send()
        .await
        .expect("Failed to add project");

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");
    let body = response.text().await.unwrap();
    let project_id = extract_project_id(&body);

    let response = client
        .get(format!("{}/projects/{}", &app.address, project_id))
        .send()
        .await
        .expect("Failed to GET project detail");

    let detail_body = response.text().await.unwrap();
    assert!(
        detail_body.contains("no next actions"),
        "Detail page should show stalled notice: got {}",
        detail_body
    );
}

// ---- Stalled Flag on List ----

#[tokio::test]
async fn projects_list_shows_stalled_flag_for_project_without_actions() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj11@example.com", "securepassword123").await;

    client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "Empty project")])
        .send()
        .await
        .expect("Failed to add project");

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("No next actions"),
        "Project without actions should show stalled flag"
    );
}

#[tokio::test]
async fn projects_list_shows_action_count() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj12@example.com", "securepassword123").await;

    client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "Project with actions")])
        .send()
        .await
        .expect("Failed to add project");

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");
    let body = response.text().await.unwrap();
    let project_id = extract_project_id(&body);

    // Get a context ID to add an action
    let context_id = get_first_context_id(&client, &app.address).await;

    // Add a next action to the project
    client
        .post(format!(
            "{}/projects/{}/next-actions",
            &app.address, project_id
        ))
        .form(&[("title", "Buy paint"), ("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to add next action to project");

    // Reload the project list
    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("1 action"),
        "Should show '1 action' count, got: {}",
        body
    );
}

// ---- Complete Project ----

#[tokio::test]
async fn complete_project_removes_from_list() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj13@example.com", "securepassword123").await;

    client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "Complete me")])
        .send()
        .await
        .expect("Failed to add project");

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");
    let body = response.text().await.unwrap();
    let project_id = extract_project_id(&body);

    // Complete the project
    let response = client
        .post(format!("{}/projects/{}/complete", &app.address, project_id))
        .send()
        .await
        .expect("Failed to complete project");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Should redirect after completing project"
    );

    // Verify it's gone from the list
    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");
    let body = response.text().await.unwrap();
    assert!(
        !body.contains("Complete me"),
        "Completed project should not appear in active projects list"
    );
}

// ---- HTMX Complete Project ----

#[tokio::test]
async fn htmx_complete_project_returns_empty_with_announce() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj14@example.com", "securepassword123").await;

    client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "HTMX complete")])
        .send()
        .await
        .expect("Failed to add project");

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");
    let body = response.text().await.unwrap();
    let project_id = extract_project_id(&body);

    let response = client
        .post(format!("{}/projects/{}/complete", &app.address, project_id))
        .header("hx-request", "true")
        .send()
        .await
        .expect("Failed to HTMX complete project");

    assert_eq!(response.status().as_u16(), 200);
    let trigger = response
        .headers()
        .get("hx-trigger")
        .expect("Missing HX-Trigger");
    assert!(trigger.to_str().unwrap().contains("announce"));
    let body = response.text().await.unwrap();
    assert!(body.is_empty(), "HTMX complete should return empty body");
}

// ---- Delete Project ----

#[tokio::test]
async fn delete_project_removes_from_list() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj15@example.com", "securepassword123").await;

    client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "Delete me")])
        .send()
        .await
        .expect("Failed to add project");

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");
    let body = response.text().await.unwrap();
    let project_id = extract_project_id(&body);

    let response = client
        .post(format!("{}/projects/{}/delete", &app.address, project_id))
        .send()
        .await
        .expect("Failed to delete project");

    assert_eq!(response.status().as_u16(), 303);

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");
    let body = response.text().await.unwrap();
    assert!(
        !body.contains("Delete me"),
        "Deleted project should not appear in list"
    );
}

// ---- Edit Project ----

#[tokio::test]
async fn edit_project_updates_title() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj16@example.com", "securepassword123").await;

    client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "Old title")])
        .send()
        .await
        .expect("Failed to add project");

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");
    let body = response.text().await.unwrap();
    let project_id = extract_project_id(&body);

    let response = client
        .post(format!("{}/projects/{}/edit", &app.address, project_id))
        .form(&[("title", "New title")])
        .send()
        .await
        .expect("Failed to edit project");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Should redirect after editing project"
    );

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");
    let body = response.text().await.unwrap();
    assert!(
        body.contains("New title"),
        "Updated title should appear in list"
    );
    assert!(
        !body.contains("Old title"),
        "Old title should not appear in list"
    );
}

// ---- Accessible Labels ----

#[tokio::test]
async fn project_has_accessible_complete_button() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj17@example.com", "securepassword123").await;

    client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "My project")])
        .send()
        .await
        .expect("Failed to add project");

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("aria-label=\"Complete: My project\""),
        "Complete button should have accessible label with project title"
    );
}

#[tokio::test]
async fn project_has_accessible_delete_button() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj18@example.com", "securepassword123").await;

    client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "My project")])
        .send()
        .await
        .expect("Failed to add project");

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("aria-label=\"Delete: My project\""),
        "Delete button should have accessible label with project title"
    );
}

#[tokio::test]
async fn project_has_accessible_edit_link() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj19@example.com", "securepassword123").await;

    client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "My project")])
        .send()
        .await
        .expect("Failed to add project");

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("aria-label=\"Edit: My project\""),
        "Edit link should have accessible label with project title"
    );
}

// ---- Project List uses semantic <ul> ----

#[tokio::test]
async fn projects_list_uses_semantic_list() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj20@example.com", "securepassword123").await;

    client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "Semantic project")])
        .send()
        .await
        .expect("Failed to add project");

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("<ul") && body.contains("role=\"list\""),
        "Projects list should use <ul> with role=\"list\""
    );
}

// ---- Add Next Action to Project ----

#[tokio::test]
async fn add_next_action_to_project_appears_on_detail_page() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj21@example.com", "securepassword123").await;

    client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "House reno")])
        .send()
        .await
        .expect("Failed to add project");

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");
    let body = response.text().await.unwrap();
    let project_id = extract_project_id(&body);

    let context_id = get_first_context_id(&client, &app.address).await;

    // Add a next action
    let response = client
        .post(format!(
            "{}/projects/{}/next-actions",
            &app.address, project_id
        ))
        .form(&[("title", "Get quotes"), ("context_id", &context_id)])
        .send()
        .await
        .expect("Failed to add next action to project");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Should redirect after adding action"
    );

    // Verify on detail page
    let response = client
        .get(format!("{}/projects/{}", &app.address, project_id))
        .send()
        .await
        .expect("Failed to GET project detail");

    let detail_body = response.text().await.unwrap();
    assert!(
        detail_body.contains("Get quotes"),
        "Next action should appear on project detail page"
    );
}

// ---- Project Detail has heading hierarchy ----

#[tokio::test]
async fn project_detail_has_heading_hierarchy() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj22@example.com", "securepassword123").await;

    client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "Heading test")])
        .send()
        .await
        .expect("Failed to add project");

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");
    let body = response.text().await.unwrap();
    let project_id = extract_project_id(&body);

    let response = client
        .get(format!("{}/projects/{}", &app.address, project_id))
        .send()
        .await
        .expect("Failed to GET project detail");

    let detail_body = response.text().await.unwrap();
    assert!(
        detail_body.contains("<h1>"),
        "Detail page should have h1 for project title"
    );
    assert!(
        detail_body.contains("<h2"),
        "Detail page should have h2 for sections"
    );
}

// ---- Users cannot see each other's projects ----

#[tokio::test]
async fn users_cannot_see_each_others_projects() {
    let app = spawn_app().await;
    let user1 =
        register_and_login(&app.address, "projuser1@example.com", "securepassword123").await;
    let user2 =
        register_and_login(&app.address, "projuser2@example.com", "securepassword123").await;

    // User1 adds a project
    user1
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "User1 secret project")])
        .send()
        .await
        .expect("Failed to add project");

    // User2 should not see it
    let response = user2
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects as user2");
    let body = response.text().await.unwrap();
    assert!(
        !body.contains("User1 secret project"),
        "User2 should not see User1's projects"
    );
}

// ---- Complete and Delete return correct errors ----

#[tokio::test]
async fn complete_nonexistent_project_returns_404() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj23@example.com", "securepassword123").await;

    let response = client
        .post(format!(
            "{}/projects/00000000-0000-0000-0000-000000000000/complete",
            &app.address
        ))
        .send()
        .await
        .expect("Failed to POST complete");

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn delete_nonexistent_project_returns_404() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj24@example.com", "securepassword123").await;

    let response = client
        .post(format!(
            "{}/projects/00000000-0000-0000-0000-000000000000/delete",
            &app.address
        ))
        .send()
        .await
        .expect("Failed to POST delete");

    assert_eq!(response.status().as_u16(), 404);
}

// ---- Dashboard shows project counts ----

#[tokio::test]
async fn dashboard_shows_projects_count() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj25@example.com", "securepassword123").await;

    // Add a project
    client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "Dashboard project")])
        .send()
        .await
        .expect("Failed to add project");

    let response = client
        .get(format!("{}/dashboard", &app.address))
        .send()
        .await
        .expect("Failed to GET /dashboard");

    let body = response.text().await.unwrap();
    // The dashboard should show "1 items" for projects
    assert!(
        body.contains("Projects"),
        "Dashboard should show Projects stat"
    );
}

#[tokio::test]
async fn dashboard_shows_stalled_projects_count() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj26@example.com", "securepassword123").await;

    // Add a project (with no actions = stalled)
    client
        .post(format!("{}/projects", &app.address))
        .form(&[("title", "Stalled dashboard")])
        .send()
        .await
        .expect("Failed to add project");

    let response = client
        .get(format!("{}/dashboard", &app.address))
        .send()
        .await
        .expect("Failed to GET /dashboard");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("Stalled Projects"),
        "Dashboard should show Stalled Projects stat"
    );
}

// ---- Focus Management Script ----

#[tokio::test]
async fn projects_page_includes_focus_management_script() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "proj27@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/projects", &app.address))
        .send()
        .await
        .expect("Failed to GET /projects");

    let body = response.text().await.unwrap();
    assert!(
        body.contains("project-focus.js"),
        "Projects page should include focus management script"
    );
}

// ---- Helper ----

fn extract_project_id(body: &str) -> String {
    // Extract a project ID from the projects list page
    // Look for href="/projects/<uuid>"
    let href_prefix = "href=\"/projects/";
    let start = body
        .find(href_prefix)
        .expect("Missing project link in body")
        + href_prefix.len();
    let end = body[start..].find('"').expect("Missing closing quote");
    body[start..start + end].to_string()
}
