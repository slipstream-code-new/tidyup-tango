use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use sqlx::PgPool;

use super::auth::AuthenticatedUser;
use crate::services::inbox_service;
use crate::services::next_action_service;

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    current_page: &'static str,
    inbox_count: i64,
    next_actions_count: i64,
}

pub async fn get_dashboard(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, DashboardError> {
    let inbox_count = inbox_service::get_inbox_count(&pool, &user_id)
        .await
        .map_err(DashboardError::Unexpected)?;

    let next_actions_count = next_action_service::count_active_next_actions(&pool, &user_id)
        .await
        .map_err(DashboardError::Unexpected)?;

    let template = DashboardTemplate {
        current_page: "dashboard",
        inbox_count,
        next_actions_count,
    };
    Ok(Html(template.render()?))
}

#[derive(Debug, thiserror::Error)]
pub enum DashboardError {
    #[error("Failed to render template")]
    Render(#[from] askama::Error),
    #[error(transparent)]
    Unexpected(anyhow::Error),
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
