use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};

mod auth;
mod contexts;
mod dashboard;
mod forgot_password;
mod gtd_placeholders;
mod health_check;
mod inbox;
mod index;
mod login;
mod next_actions;
mod register;
mod todos;

fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.contains_key("hx-request")
}

fn htmx_response_with_announce(body: Html<String>, message: &str) -> Response {
    let trigger_json = format!(r#"{{"announce":"{}"}}"#, message);
    (
        [(
            axum::http::header::HeaderName::from_static("hx-trigger"),
            axum::http::HeaderValue::from_str(&trigger_json).unwrap(),
        )],
        body,
    )
        .into_response()
}

pub use contexts::{
    get_contexts, get_edit_context, post_context, post_delete_context, post_edit_context,
};
pub use dashboard::get_dashboard;
pub use forgot_password::get_forgot_password;
pub use gtd_placeholders::{get_projects, get_review, get_someday_maybe, get_waiting_for};
pub use health_check::health_check;
pub use inbox::{get_inbox, post_clarify_inbox_item, post_delete_inbox_item, post_inbox};
pub use index::index;
pub use login::{get_login, post_login, post_logout};
pub use next_actions::{
    get_edit_next_action, get_next_action_item, get_next_actions, post_complete_next_action,
    post_delete_next_action, post_edit_next_action, post_next_action,
};
pub use register::{get_register, post_register};
pub use todos::{
    get_edit_todo, get_todo_item, get_todos_page, post_delete_todo, post_edit_todo, post_todo,
    post_toggle_todo,
};
