mod auth;
mod health_check;
mod index;
mod login;
mod register;
mod todos;

pub use health_check::health_check;
pub use index::index;
pub use login::{get_login, post_login, post_logout};
pub use register::{get_register, post_register};
pub use todos::{get_todos_page, post_todo};
