use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

pub async fn index() -> Result<impl IntoResponse, IndexError> {
    let template = IndexTemplate;
    Ok(Html(template.render()?))
}

#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error("Failed to render template")]
    Render(#[from] askama::Error),
}

impl IntoResponse for IndexError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "Template rendering failed");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Something went wrong</h1>".to_string()),
        )
            .into_response()
    }
}
