use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use sqlx::PgPool;
use uuid::Uuid;

use super::auth::AuthenticatedUser;
use crate::domain::{TodoItem, TodoItemId};
use crate::services::todo_service::{
    add_todo, get_todos, toggle_todo_completion, AddTodoError, ToggleTodoError,
};

/// View model for a single todo item in the template.
pub struct TodoItemView {
    pub id: String,
    pub title: String,
    pub is_completed: bool,
}

impl From<&TodoItem> for TodoItemView {
    fn from(item: &TodoItem) -> Self {
        Self {
            id: item.id().as_uuid().to_string(),
            title: item.title().as_str().to_string(),
            is_completed: item.is_completed(),
        }
    }
}

#[derive(Template)]
#[template(path = "todos.html")]
pub struct TodosTemplate {
    pub todos: Vec<TodoItemView>,
    pub error_message: Option<String>,
}

pub async fn get_todos_page(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, TodosError> {
    let todos = get_todos(&pool, &user_id)
        .await
        .map_err(TodosError::Unexpected)?;

    let todo_views: Vec<TodoItemView> = todos.iter().map(TodoItemView::from).collect();

    let template = TodosTemplate {
        todos: todo_views,
        error_message: None,
    };
    Ok(Html(template.render()?))
}

#[derive(serde::Deserialize)]
pub struct AddTodoForm {
    pub title: String,
}

pub async fn post_todo(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    Form(form): Form<AddTodoForm>,
) -> Result<Response, TodosError> {
    // Silently ignore empty/whitespace-only submissions per US-5
    if form.title.trim().is_empty() {
        return Ok(Redirect::to("/todos").into_response());
    }

    match add_todo(&pool, user_id.clone(), form.title).await {
        Ok(_) => Ok(Redirect::to("/todos").into_response()),
        Err(AddTodoError::InvalidTitle(title_err)) => {
            let user_facing = match title_err {
                crate::domain::TodoTitleError::Empty => {
                    // Should not reach here due to empty check above, but handle gracefully
                    return Ok(Redirect::to("/todos").into_response());
                }
                crate::domain::TodoTitleError::TooLong { max, .. } => {
                    format!("That title is too long (max {max} characters)")
                }
            };

            let todos = get_todos(&pool, &user_id)
                .await
                .map_err(TodosError::Unexpected)?;
            let todo_views: Vec<TodoItemView> = todos.iter().map(TodoItemView::from).collect();

            let template = TodosTemplate {
                todos: todo_views,
                error_message: Some(user_facing),
            };
            let body = template.render().map_err(TodosError::Render)?;
            Ok((StatusCode::UNPROCESSABLE_ENTITY, Html(body)).into_response())
        }
        Err(AddTodoError::Unexpected(err)) => Err(TodosError::Unexpected(err)),
    }
}

pub async fn post_toggle_todo(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    Path(todo_id): Path<Uuid>,
) -> Result<Response, TodosError> {
    let todo_item_id = TodoItemId::from_uuid(todo_id);

    match toggle_todo_completion(&pool, &todo_item_id, &user_id).await {
        Ok(_) => Ok(Redirect::to("/todos").into_response()),
        Err(ToggleTodoError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Todo not found</h1>".to_string()),
        )
            .into_response()),
        Err(ToggleTodoError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(ToggleTodoError::Unexpected(err)) => Err(TodosError::Unexpected(err)),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TodosError {
    #[error("Failed to render template")]
    Render(#[from] askama::Error),
    #[error(transparent)]
    Unexpected(anyhow::Error),
}

impl IntoResponse for TodosError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "Todos error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Something went wrong</h1>".to_string()),
        )
            .into_response()
    }
}
