mod auth;
mod dashboard;
mod forgot_password;
mod gtd_placeholders;
mod health_check;
mod index;
mod login;
mod register;
mod todos;

pub use dashboard::get_dashboard;
pub use forgot_password::get_forgot_password;
pub use gtd_placeholders::{
    get_inbox, get_next_actions, get_projects, get_review, get_someday_maybe, get_waiting_for,
};
pub use health_check::health_check;
pub use index::index;
pub use login::{get_login, post_login, post_logout};
pub use register::{get_register, post_register};
pub use todos::{
    get_edit_todo, get_todo_item, get_todos_page, post_delete_todo, post_edit_todo, post_todo,
    post_toggle_todo,
};
