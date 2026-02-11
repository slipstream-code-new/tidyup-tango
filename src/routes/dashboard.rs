use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

use super::auth::AuthenticatedUser;

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    current_page: &'static str,
    inbox_count: i64,
}

pub async fn get_dashboard(
    AuthenticatedUser(_user_id): AuthenticatedUser,
) -> Result<impl IntoResponse, DashboardError> {
    let template = DashboardTemplate {
        current_page: "dashboard",
        inbox_count: 0, // Placeholder until inbox table exists
    };
    Ok(Html(template.render()?))
}

#[derive(Debug, thiserror::Error)]
pub enum DashboardError {
    #[error("Failed to render template")]
    Render(#[from] askama::Error),
}

impl IntoResponse for DashboardError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "Dashboard error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Something went wrong</h1>".to_string()),
        )
            .into_response()
    }
}
