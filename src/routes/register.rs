use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use sqlx::PgPool;

use crate::domain::{EmailValidationError, PasswordError};
use crate::services::registration::{register_user, RegistrationError};

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

#[derive(serde::Deserialize)]
pub struct RegisterFormData {
    pub email: String,
    pub password: String,
    pub password_confirmation: String,
}

pub async fn post_register(
    State(pool): State<PgPool>,
    Form(form): Form<RegisterFormData>,
) -> Result<Response, RegisterError> {
    match register_user(
        &pool,
        form.email.clone(),
        form.password,
        form.password_confirmation,
    )
    .await
    {
        Ok(_user_id) => Ok(Redirect::to("/dashboard").into_response()),
        Err(e) => {
            let mut template = RegisterTemplate {
                general_error: None,
                form_email: form.email,
                email_error: None,
                password_error: None,
                password_confirmation_error: None,
            };

            match &e {
                RegistrationError::InvalidEmail(email_err) => {
                    template.email_error = Some(user_facing_email_error(email_err));
                }
                RegistrationError::InvalidPassword(pwd_err) => {
                    template.password_error = Some(user_facing_password_error(pwd_err));
                }
                RegistrationError::PasswordMismatch => {
                    template.password_confirmation_error =
                        Some("Passwords do not match".to_string());
                }
                RegistrationError::DuplicateEmail => {
                    template.general_error = Some(
                        "Unable to create account. If you already have an account, try <a href=\"/login\">signing in</a>.".to_string()
                    );
                }
                RegistrationError::Unexpected(err) => {
                    tracing::error!(error = ?err, "Unexpected registration error");
                    template.general_error =
                        Some("Something went wrong. Please try again.".to_string());
                }
            }

            let body = template.render().map_err(RegisterError::Render)?;
            Ok((StatusCode::UNPROCESSABLE_ENTITY, Html(body)).into_response())
        }
    }
}

fn user_facing_email_error(err: &EmailValidationError) -> String {
    match err {
        EmailValidationError::Empty => "Enter your email address".to_string(),
        EmailValidationError::MissingAtSymbol => {
            "Enter a valid email address, like name@example.com".to_string()
        }
        EmailValidationError::TooLong => "That email address is too long".to_string(),
    }
}

fn user_facing_password_error(err: &PasswordError) -> String {
    match err {
        PasswordError::Empty => "Enter a password".to_string(),
        PasswordError::TooShort => "Your password needs at least 8 characters".to_string(),
        PasswordError::TooLong => "That password is too long".to_string(),
    }
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
