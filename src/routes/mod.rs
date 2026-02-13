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
mod projects;
mod register;
mod someday_maybe;
mod todos;
mod waiting_for;

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
pub use gtd_placeholders::get_review;
pub use health_check::health_check;
pub use inbox::{get_inbox, post_clarify_inbox_item, post_delete_inbox_item, post_inbox};
pub use index::index;
pub use login::{get_login, post_login, post_logout};
pub use next_actions::{
    get_edit_next_action, get_next_action_item, get_next_actions, post_complete_next_action,
    post_delete_next_action, post_edit_next_action, post_next_action,
};
pub use projects::{
    get_edit_project, get_project_detail, get_project_item, get_projects, post_complete_project,
    post_delete_project, post_edit_project, post_project, post_project_next_action,
};
pub use register::{get_register, post_register};
pub use someday_maybe::{
    get_edit_someday_maybe, get_someday_maybe, get_someday_maybe_item, post_activate_someday_maybe,
    post_delete_someday_maybe, post_edit_someday_maybe, post_someday_maybe,
};
pub use todos::{
    get_edit_todo, get_todo_item, get_todos_page, post_delete_todo, post_edit_todo, post_todo,
    post_toggle_todo,
};
pub use waiting_for::{
    get_edit_waiting_for, get_waiting_for, get_waiting_for_item, post_complete_waiting_for,
    post_delete_waiting_for, post_edit_waiting_for, post_waiting_for,
};
