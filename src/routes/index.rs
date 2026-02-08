use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use tower_sessions::Session;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

pub async fn index(session: Session) -> Result<Response, IndexError> {
    // Check if the user is already authenticated
    let user_id: Option<String> = session
        .get("user_id")
        .await
        .map_err(|e| IndexError::Session(anyhow::anyhow!("Failed to read session: {e}")))?;

    if let Some(ref id_str) = user_id {
        if Uuid::parse_str(id_str).is_ok() {
            return Ok(Redirect::to("/todos").into_response());
        }
    }

    let template = IndexTemplate;
    Ok(Html(template.render()?).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error("Failed to render template")]
    Render(#[from] askama::Error),
    #[error("Session error: {0}")]
    Session(anyhow::Error),
}

impl IntoResponse for IndexError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "Index error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Something went wrong</h1>".to_string()),
        )
            .into_response()
    }
}
