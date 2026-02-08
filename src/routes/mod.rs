mod auth;
mod forgot_password;
mod health_check;
mod index;
mod login;
mod register;
mod todos;

pub use forgot_password::get_forgot_password;
pub use health_check::health_check;
pub use index::index;
pub use login::{get_login, post_login, post_logout};
pub use register::{get_register, post_register};
pub use todos::{
    get_edit_todo, get_todo_item, get_todos_page, post_delete_todo, post_edit_todo, post_todo,
    post_toggle_todo,
};
