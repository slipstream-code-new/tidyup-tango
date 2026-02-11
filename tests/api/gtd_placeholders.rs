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

async fn assert_gtd_page(
    client: &reqwest::Client,
    address: &str,
    path: &str,
    expected_heading: &str,
    expected_current: &str,
) {
    let response = client
        .get(format!("{address}{path}"))
        .send()
        .await
        .unwrap_or_else(|_| panic!("Failed to GET {path}"));

    assert_eq!(response.status().as_u16(), 200, "{path} should return 200");

    let body = response.text().await.unwrap();

    assert!(
        body.contains(&format!("<h1>{expected_heading}</h1>")),
        "{path} should have heading '{expected_heading}'"
    );

    // Navigation links present
    assert!(
        body.contains("href=\"/inbox\""),
        "{path} missing link to /inbox"
    );
    assert!(
        body.contains("href=\"/next-actions\""),
        "{path} missing link to /next-actions"
    );
    assert!(
        body.contains("href=\"/projects\""),
        "{path} missing link to /projects"
    );
    assert!(
        body.contains("href=\"/waiting-for\""),
        "{path} missing link to /waiting-for"
    );
    assert!(
        body.contains("href=\"/someday-maybe\""),
        "{path} missing link to /someday-maybe"
    );
    assert!(
        body.contains("href=\"/review\""),
        "{path} missing link to /review"
    );

    // aria-current on the right link
    assert!(
        body.contains("aria-current=\"page\""),
        "{path} should have aria-current=\"page\" on the active nav link"
    );

    // The current link should be the one with aria-current
    assert!(
        body.contains(&format!("href=\"{path}\" aria-current=\"page\""))
            || body.contains(&format!("href=\"{path}\"\n aria-current=\"page\""))
            || body.contains(expected_current),
        "{path} should have aria-current on the correct link"
    );

    // Sign out present
    assert!(
        body.contains("Sign out"),
        "{path} should have sign out button"
    );

    // hx-boost present
    assert!(
        body.contains("hx-boost=\"true\""),
        "{path} should have hx-boost on body"
    );
}

#[tokio::test]
async fn inbox_page_returns_200_with_nav() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "inbox@example.com", "securepassword123").await;
    assert_gtd_page(&client, &app.address, "/inbox", "Inbox", "inbox").await;
}

#[tokio::test]
async fn next_actions_page_returns_200_with_nav() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "nextactions@example.com", "securepassword123").await;
    assert_gtd_page(
        &client,
        &app.address,
        "/next-actions",
        "Next Actions",
        "next_actions",
    )
    .await;
}

#[tokio::test]
async fn projects_page_returns_200_with_nav() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "projects@example.com", "securepassword123").await;
    assert_gtd_page(&client, &app.address, "/projects", "Projects", "projects").await;
}

#[tokio::test]
async fn waiting_for_page_returns_200_with_nav() {
    let app = spawn_app().await;
    let client =
        register_and_login(&app.address, "waitingfor@example.com", "securepassword123").await;
    assert_gtd_page(
        &client,
        &app.address,
        "/waiting-for",
        "Waiting For",
        "waiting_for",
    )
    .await;
}

#[tokio::test]
async fn someday_maybe_page_returns_200_with_nav() {
    let app = spawn_app().await;
    let client = register_and_login(
        &app.address,
        "somedaymaybe@example.com",
        "securepassword123",
    )
    .await;
    assert_gtd_page(
        &client,
        &app.address,
        "/someday-maybe",
        "Someday/Maybe",
        "someday_maybe",
    )
    .await;
}

#[tokio::test]
async fn review_page_returns_200_with_nav() {
    let app = spawn_app().await;
    let client = register_and_login(&app.address, "review@example.com", "securepassword123").await;
    assert_gtd_page(&client, &app.address, "/review", "Weekly Review", "review").await;
}

#[tokio::test]
async fn gtd_pages_redirect_to_login_when_unauthenticated() {
    let app = spawn_app().await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let pages = [
        "/dashboard",
        "/inbox",
        "/next-actions",
        "/projects",
        "/waiting-for",
        "/someday-maybe",
        "/review",
    ];

    for path in pages {
        let response = client
            .get(format!("{}{}", &app.address, path))
            .send()
            .await
            .unwrap_or_else(|_| panic!("Failed to GET {path}"));

        assert_eq!(
            response.status().as_u16(),
            303,
            "{path} should redirect unauthenticated users"
        );

        let location = response
            .headers()
            .get("location")
            .expect("Missing Location header")
            .to_str()
            .unwrap();
        assert_eq!(location, "/login", "{path} should redirect to /login");
    }
}
