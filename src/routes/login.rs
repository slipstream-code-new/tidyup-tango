use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use sqlx::PgPool;
use tower_sessions::Session;

use crate::services::authentication::{authenticate_user, AuthenticationError};

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub error_message: Option<String>,
    pub form_email: String,
}

pub async fn get_login() -> Result<impl IntoResponse, LoginError> {
    let template = LoginTemplate {
        error_message: None,
        form_email: String::new(),
    };
    Ok(Html(template.render()?))
}

#[derive(serde::Deserialize)]
pub struct LoginFormData {
    pub email: String,
    pub password: String,
}

pub async fn post_login(
    State(pool): State<PgPool>,
    session: Session,
    Form(form): Form<LoginFormData>,
) -> Result<Response, LoginError> {
    match authenticate_user(&pool, form.email.clone(), form.password).await {
        Ok(user_id) => {
            session.cycle_id().await.map_err(|e| {
                LoginError::Session(anyhow::anyhow!("Failed to cycle session: {e}"))
            })?;
            session
                .insert("user_id", user_id.as_uuid().to_string())
                .await
                .map_err(|e| {
                    LoginError::Session(anyhow::anyhow!("Failed to store user_id in session: {e}"))
                })?;
            Ok(Redirect::to("/todos").into_response())
        }
        Err(AuthenticationError::InvalidCredentials) => {
            let template = LoginTemplate {
                error_message: Some("That email or password didn't work. Try again.".to_string()),
                form_email: form.email,
            };
            let body = template.render().map_err(LoginError::Render)?;
            Ok((StatusCode::UNPROCESSABLE_ENTITY, Html(body)).into_response())
        }
        Err(AuthenticationError::Unexpected(err)) => {
            tracing::error!(error = ?err, "Unexpected authentication error");
            let template = LoginTemplate {
                error_message: Some("Something went wrong. Please try again.".to_string()),
                form_email: form.email,
            };
            let body = template.render().map_err(LoginError::Render)?;
            Ok((StatusCode::INTERNAL_SERVER_ERROR, Html(body)).into_response())
        }
    }
}

pub async fn post_logout(session: Session) -> Result<Response, LoginError> {
    session
        .flush()
        .await
        .map_err(|e| LoginError::Session(anyhow::anyhow!("Failed to flush session: {e}")))?;
    tracing::info!("User logged out");
    Ok(Redirect::to("/login").into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("Failed to render template")]
    Render(#[from] askama::Error),
    #[error("Session error: {0}")]
    Session(anyhow::Error),
}

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "Login error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Something went wrong</h1>".to_string()),
        )
            .into_response()
    }
}
