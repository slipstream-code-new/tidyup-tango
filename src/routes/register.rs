use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {
    pub general_error: Option<String>,
    pub form_email: String,
    pub email_error: Option<String>,
    pub password_error: Option<String>,
    pub password_confirmation_error: Option<String>,
}

pub async fn get_register() -> Result<impl IntoResponse, RegisterError> {
    let template = RegisterTemplate {
        general_error: None,
        form_email: String::new(),
        email_error: None,
        password_error: None,
        password_confirmation_error: None,
    };
    Ok(Html(template.render()?))
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterError {
    #[error("Failed to render template")]
    Render(#[from] askama::Error),
}

impl IntoResponse for RegisterError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "Registration template rendering failed");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Something went wrong</h1>".to_string()),
        )
            .into_response()
    }
}
