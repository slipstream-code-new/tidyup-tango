use askama::Template;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use sqlx::PgPool;
use uuid::Uuid;

use super::auth::AuthenticatedUser;
use super::{htmx_response_with_announce, is_htmx_request};
use crate::domain::{TodoItem, TodoItemId};
use crate::services::todo_service::{
    add_todo, delete_todo, get_todos, toggle_todo_completion, update_todo_title, AddTodoError,
    DeleteTodoError, ToggleTodoError, UpdateTitleError,
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

#[derive(Template)]
#[template(path = "todo_item.html")]
pub struct TodoItemTemplate {
    pub todo: TodoItemView,
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
    headers: HeaderMap,
    Form(form): Form<AddTodoForm>,
) -> Result<Response, TodosError> {
    let htmx = is_htmx_request(&headers);

    // Silently ignore empty/whitespace-only submissions per US-5
    if form.title.trim().is_empty() {
        if htmx {
            return Ok(StatusCode::NO_CONTENT.into_response());
        }
        return Ok(Redirect::to("/todos").into_response());
    }

    match add_todo(&pool, user_id.clone(), form.title).await {
        Ok(item) => {
            if htmx {
                let template = TodoItemTemplate {
                    todo: TodoItemView::from(&item),
                };
                let body = template.render().map_err(TodosError::Render)?;
                Ok(htmx_response_with_announce(Html(body), "Todo added"))
            } else {
                Ok(Redirect::to("/todos").into_response())
            }
        }
        Err(AddTodoError::InvalidTitle(title_err)) => {
            let user_facing = match title_err {
                crate::domain::TodoTitleError::Empty => {
                    // Should not reach here due to empty check above, but handle gracefully
                    if htmx {
                        return Ok(StatusCode::NO_CONTENT.into_response());
                    }
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
    headers: HeaderMap,
    Path(todo_id): Path<Uuid>,
) -> Result<Response, TodosError> {
    let htmx = is_htmx_request(&headers);
    let todo_item_id = TodoItemId::from_uuid(todo_id);

    match toggle_todo_completion(&pool, &todo_item_id, &user_id).await {
        Ok(toggled) => {
            if htmx {
                let announce = if toggled.is_completed() {
                    "Todo completed"
                } else {
                    "Todo marked pending"
                };
                let template = TodoItemTemplate {
                    todo: TodoItemView::from(&toggled),
                };
                let body = template.render().map_err(TodosError::Render)?;
                Ok(htmx_response_with_announce(Html(body), announce))
            } else {
                Ok(Redirect::to("/todos").into_response())
            }
        }
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

pub async fn post_delete_todo(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(todo_id): Path<Uuid>,
) -> Result<Response, TodosError> {
    let htmx = is_htmx_request(&headers);
    let todo_item_id = TodoItemId::from_uuid(todo_id);

    match delete_todo(&pool, &todo_item_id, &user_id).await {
        Ok(()) => {
            if htmx {
                Ok(htmx_response_with_announce(
                    Html(String::new()),
                    "Todo deleted",
                ))
            } else {
                Ok(Redirect::to("/todos").into_response())
            }
        }
        Err(DeleteTodoError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Todo not found</h1>".to_string()),
        )
            .into_response()),
        Err(DeleteTodoError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(DeleteTodoError::Unexpected(err)) => Err(TodosError::Unexpected(err)),
    }
}

#[derive(Template)]
#[template(path = "edit_todo.html")]
pub struct EditTodoTemplate {
    pub todo_id: String,
    pub current_title: String,
    pub error_message: Option<String>,
}

#[derive(Template)]
#[template(path = "todo_item_edit.html")]
pub struct TodoItemEditTemplate {
    pub todo_id: String,
    pub current_title: String,
}

pub async fn get_edit_todo(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(todo_id): Path<Uuid>,
) -> Result<Response, TodosError> {
    let htmx = is_htmx_request(&headers);
    let todo_item_id = TodoItemId::from_uuid(todo_id);

    let item = crate::infrastructure::todo_repository::find_todo_by_id(&pool, &todo_item_id)
        .await
        .map_err(|e| TodosError::Unexpected(anyhow::anyhow!("Database error: {e}")))?;

    match item {
        Some(todo) if todo.user_id() == &user_id => {
            if htmx {
                let template = TodoItemEditTemplate {
                    todo_id: todo_id.to_string(),
                    current_title: todo.title().as_str().to_string(),
                };
                Ok(Html(template.render()?).into_response())
            } else {
                let template = EditTodoTemplate {
                    todo_id: todo_id.to_string(),
                    current_title: todo.title().as_str().to_string(),
                    error_message: None,
                };
                Ok(Html(template.render()?).into_response())
            }
        }
        Some(_) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        None => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Todo not found</h1>".to_string()),
        )
            .into_response()),
    }
}

#[derive(serde::Deserialize)]
pub struct EditTodoForm {
    pub title: String,
}

pub async fn post_edit_todo(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(todo_id): Path<Uuid>,
    Form(form): Form<EditTodoForm>,
) -> Result<Response, TodosError> {
    let htmx = is_htmx_request(&headers);
    let todo_item_id = TodoItemId::from_uuid(todo_id);

    match update_todo_title(&pool, &todo_item_id, &user_id, form.title.clone()).await {
        Ok(()) => {
            if htmx {
                // Return the updated item fragment
                let item =
                    crate::infrastructure::todo_repository::find_todo_by_id(&pool, &todo_item_id)
                        .await
                        .map_err(|e| {
                            TodosError::Unexpected(anyhow::anyhow!("Database error: {e}"))
                        })?;
                match item {
                    Some(todo) => {
                        let template = TodoItemTemplate {
                            todo: TodoItemView::from(&todo),
                        };
                        let body = template.render().map_err(TodosError::Render)?;
                        Ok(htmx_response_with_announce(Html(body), "Todo updated"))
                    }
                    None => Ok(Redirect::to("/todos").into_response()),
                }
            } else {
                Ok(Redirect::to("/todos").into_response())
            }
        }
        Err(UpdateTitleError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Todo not found</h1>".to_string()),
        )
            .into_response()),
        Err(UpdateTitleError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(UpdateTitleError::InvalidTitle(title_err)) => {
            let user_facing = match title_err {
                crate::domain::TodoTitleError::Empty => "Title cannot be empty".to_string(),
                crate::domain::TodoTitleError::TooLong { max, .. } => {
                    format!("That title is too long (max {max} characters)")
                }
            };
            if htmx {
                // Return the inline edit form with error
                let template = TodoItemEditTemplate {
                    todo_id: todo_id.to_string(),
                    current_title: form.title,
                };
                let body = template.render().map_err(TodosError::Render)?;
                Ok((StatusCode::UNPROCESSABLE_ENTITY, Html(body)).into_response())
            } else {
                let template = EditTodoTemplate {
                    todo_id: todo_id.to_string(),
                    current_title: form.title,
                    error_message: Some(user_facing),
                };
                let body = template.render().map_err(TodosError::Render)?;
                Ok((StatusCode::UNPROCESSABLE_ENTITY, Html(body)).into_response())
            }
        }
        Err(UpdateTitleError::Unexpected(err)) => Err(TodosError::Unexpected(err)),
    }
}

/// Return a single todo item as an HTML fragment (for HTMX cancel-edit).
pub async fn get_todo_item(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    Path(todo_id): Path<Uuid>,
) -> Result<Response, TodosError> {
    let todo_item_id = TodoItemId::from_uuid(todo_id);

    let item = crate::infrastructure::todo_repository::find_todo_by_id(&pool, &todo_item_id)
        .await
        .map_err(|e| TodosError::Unexpected(anyhow::anyhow!("Database error: {e}")))?;

    match item {
        Some(todo) if todo.user_id() == &user_id => {
            let template = TodoItemTemplate {
                todo: TodoItemView::from(&todo),
            };
            Ok(Html(template.render()?).into_response())
        }
        Some(_) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        None => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Todo not found</h1>".to_string()),
        )
            .into_response()),
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
