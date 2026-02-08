use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

#[derive(Template)]
#[template(path = "forgot_password.html")]
pub struct ForgotPasswordTemplate;

pub async fn get_forgot_password() -> Result<impl IntoResponse, ForgotPasswordError> {
    let template = ForgotPasswordTemplate;
    Ok(Html(template.render()?))
}

#[derive(Debug, thiserror::Error)]
pub enum ForgotPasswordError {
    #[error("Failed to render template")]
    Render(#[from] askama::Error),
}

impl IntoResponse for ForgotPasswordError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "Forgot password error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Something went wrong</h1>".to_string()),
        )
            .into_response()
    }
}
