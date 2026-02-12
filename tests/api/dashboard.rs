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
async fn dashboard_redirects_to_login_when_unauthenticated() {
    let app = spawn_app().await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let response = client
        .get(format!("{}/dashboard", &app.address))
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
async fn dashboard_returns_200_for_authenticated_user() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "dashboard@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/dashboard", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);

    let body = response.text().await.unwrap();

    // Page structure
    assert!(body.contains("<h1>"), "Missing page heading");
    assert!(
        body.contains("Dashboard"),
        "Dashboard heading should be present"
    );
}

#[tokio::test]
async fn dashboard_has_gtd_navigation_links() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "dashnav@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/dashboard", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);

    let body = response.text().await.unwrap();

    // Navigation links for all GTD lists
    assert!(body.contains("href=\"/inbox\""), "Missing link to /inbox");
    assert!(
        body.contains("href=\"/next-actions\""),
        "Missing link to /next-actions"
    );
    assert!(
        body.contains("href=\"/projects\""),
        "Missing link to /projects"
    );
    assert!(
        body.contains("href=\"/waiting-for\""),
        "Missing link to /waiting-for"
    );
    assert!(
        body.contains("href=\"/someday-maybe\""),
        "Missing link to /someday-maybe"
    );
    assert!(body.contains("href=\"/review\""), "Missing link to /review");

    // Nav uses proper landmark
    assert!(
        body.contains("aria-label=\"GTD lists\""),
        "Nav should have aria-label for GTD lists"
    );
}

#[tokio::test]
async fn dashboard_indicates_current_page() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "dashcurrent@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/dashboard", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    let body = response.text().await.unwrap();

    // Dashboard link should have aria-current="page"
    assert!(
        body.contains("aria-current=\"page\""),
        "Active nav link should have aria-current=\"page\""
    );
}

#[tokio::test]
async fn dashboard_has_sign_out_button() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "dashlogout@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/dashboard", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    let body = response.text().await.unwrap();

    assert!(
        body.contains("Sign out"),
        "Dashboard should have a sign out button"
    );
    assert!(
        body.contains("action=\"/logout\""),
        "Sign out form should post to /logout"
    );
}

#[tokio::test]
async fn authenticated_user_index_redirects_to_dashboard() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "indexdash@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status().as_u16(),
        303,
        "Authenticated user should be redirected from index"
    );

    let location = response
        .headers()
        .get("location")
        .expect("Missing Location header")
        .to_str()
        .unwrap();
    assert_eq!(
        location, "/dashboard",
        "Authenticated user should be redirected to /dashboard"
    );
}

#[tokio::test]
async fn dashboard_has_hx_boost_on_body() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "dashboost@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/dashboard", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    let body = response.text().await.unwrap();

    assert!(
        body.contains("hx-boost=\"true\""),
        "Body should have hx-boost=\"true\" for SPA-like navigation"
    );
}

#[tokio::test]
async fn dashboard_has_quick_capture_form() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "dashcapture@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/dashboard", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    let body = response.text().await.unwrap();

    // Quick capture form posts to /inbox
    assert!(
        body.contains("action=\"/inbox\""),
        "Quick capture form should post to /inbox"
    );
    assert!(
        body.contains("quick-capture"),
        "Quick capture form should have quick-capture class"
    );
    // The input should have a label (visually hidden is fine)
    assert!(
        body.contains("Quick capture"),
        "Quick capture input should have a label"
    );
    assert!(
        body.contains("Capture something..."),
        "Quick capture input should have placeholder text"
    );
}

#[tokio::test]
async fn dashboard_shows_getting_started_guidance_when_empty() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "dashstart@example.com", "securepassword123").await;

    let response = client
        .get(format!("{}/dashboard", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    let body = response.text().await.unwrap();

    assert!(
        body.contains("Getting Started"),
        "Dashboard should show getting started guidance when all counts are zero"
    );
    assert!(
        body.contains("href=\"/inbox\""),
        "Getting started guidance should link to inbox"
    );
    assert!(
        body.contains("href=\"/review\""),
        "Getting started guidance should link to weekly review"
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

#[tokio::test]
async fn dashboard_shows_dynamic_next_actions_count() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "dashcount@example.com", "securepassword123").await;

    // Initially should show 0
    let response = client
        .get(format!("{}/dashboard", &app.address))
        .send()
        .await
        .expect("Failed to GET /dashboard");
    let body = response.text().await.unwrap();
    assert!(
        body.contains(">0 items</dd>"),
        "Dashboard should show 0 next actions initially"
    );

    // Add a next action
    let context_id = get_first_context_id(&client, &app.address).await;
    client
        .post(format!("{}/next-actions", &app.address))
        .form(&[
            ("title", "Dashboard count test"),
            ("context_id", &context_id),
        ])
        .send()
        .await
        .expect("Failed to add next action");

    // Dashboard should now show 1
    let response = client
        .get(format!("{}/dashboard", &app.address))
        .send()
        .await
        .expect("Failed to GET /dashboard");
    let body = response.text().await.unwrap();
    assert!(
        body.contains(">1 items</dd>"),
        "Dashboard should show 1 next action after adding one"
    );
}
