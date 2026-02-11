use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

use super::auth::AuthenticatedUser;

#[derive(Template)]
#[template(path = "gtd_placeholder.html")]
struct GtdPlaceholderTemplate {
    current_page: &'static str,
    page_title: &'static str,
    heading: &'static str,
    description: &'static str,
    inbox_count: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum GtdPlaceholderError {
    #[error("Failed to render template")]
    Render(#[from] askama::Error),
}

impl IntoResponse for GtdPlaceholderError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "GTD placeholder error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Something went wrong</h1>".to_string()),
        )
            .into_response()
    }
}

pub async fn get_inbox(
    AuthenticatedUser(_user_id): AuthenticatedUser,
) -> Result<impl IntoResponse, GtdPlaceholderError> {
    let template = GtdPlaceholderTemplate {
        current_page: "inbox",
        page_title: "Inbox -- Todo List",
        heading: "Inbox",
        description: "Capture anything on your mind. Process it later.",
        inbox_count: 0,
    };
    Ok(Html(template.render()?))
}

pub async fn get_next_actions(
    AuthenticatedUser(_user_id): AuthenticatedUser,
) -> Result<impl IntoResponse, GtdPlaceholderError> {
    let template = GtdPlaceholderTemplate {
        current_page: "next_actions",
        page_title: "Next Actions -- Todo List",
        heading: "Next Actions",
        description: "Concrete actions you can do right now, organized by context.",
        inbox_count: 0,
    };
    Ok(Html(template.render()?))
}

pub async fn get_projects(
    AuthenticatedUser(_user_id): AuthenticatedUser,
) -> Result<impl IntoResponse, GtdPlaceholderError> {
    let template = GtdPlaceholderTemplate {
        current_page: "projects",
        page_title: "Projects -- Todo List",
        heading: "Projects",
        description: "Outcomes that need more than one action step.",
        inbox_count: 0,
    };
    Ok(Html(template.render()?))
}

pub async fn get_waiting_for(
    AuthenticatedUser(_user_id): AuthenticatedUser,
) -> Result<impl IntoResponse, GtdPlaceholderError> {
    let template = GtdPlaceholderTemplate {
        current_page: "waiting_for",
        page_title: "Waiting For -- Todo List",
        heading: "Waiting For",
        description: "Items you're waiting on from others.",
        inbox_count: 0,
    };
    Ok(Html(template.render()?))
}

pub async fn get_someday_maybe(
    AuthenticatedUser(_user_id): AuthenticatedUser,
) -> Result<impl IntoResponse, GtdPlaceholderError> {
    let template = GtdPlaceholderTemplate {
        current_page: "someday_maybe",
        page_title: "Someday/Maybe -- Todo List",
        heading: "Someday/Maybe",
        description: "Ideas and possibilities for the future.",
        inbox_count: 0,
    };
    Ok(Html(template.render()?))
}

pub async fn get_review(
    AuthenticatedUser(_user_id): AuthenticatedUser,
) -> Result<impl IntoResponse, GtdPlaceholderError> {
    let template = GtdPlaceholderTemplate {
        current_page: "review",
        page_title: "Weekly Review -- Todo List",
        heading: "Weekly Review",
        description: "Keep your system current and trustworthy.",
        inbox_count: 0,
    };
    Ok(Html(template.render()?))
}
