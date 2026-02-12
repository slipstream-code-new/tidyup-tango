use askama::Template;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use sqlx::PgPool;
use uuid::Uuid;

use super::auth::AuthenticatedUser;
use super::{htmx_response_with_announce, is_htmx_request};
use crate::domain::{Context, NextAction, Project, ProjectId, TodoTitleError};
use crate::services::context_service;
use crate::services::inbox_service;
use crate::services::project_service::{
    self, AddProjectError, AddProjectNextActionError, CompleteProjectError, DeleteProjectError,
    GetProjectError, UpdateProjectTitleError,
};

pub struct ProjectView {
    pub id: String,
    pub title: String,
    pub next_action_count: i64,
    pub is_stalled: bool,
}

pub struct ProjectNextActionView {
    pub id: String,
    pub title: String,
    pub context_name: String,
    pub is_completed: bool,
}

pub struct ContextOption {
    pub id: String,
    pub name: String,
}

#[derive(Template)]
#[template(path = "projects.html")]
struct ProjectsTemplate {
    current_page: &'static str,
    inbox_count: i64,
    projects: Vec<ProjectView>,
    error_message: Option<String>,
}

#[derive(Template)]
#[template(path = "project_item.html")]
struct ProjectItemTemplate {
    project: ProjectView,
}

#[derive(Template)]
#[template(path = "project_detail.html")]
struct ProjectDetailTemplate {
    current_page: &'static str,
    inbox_count: i64,
    project: ProjectView,
    active_actions: Vec<ProjectNextActionView>,
    completed_actions: Vec<ProjectNextActionView>,
    contexts: Vec<ContextOption>,
    error_message: Option<String>,
}

#[derive(Template)]
#[template(path = "project_edit.html")]
struct ProjectEditTemplate {
    project: ProjectView,
}

#[derive(Template)]
#[template(path = "project_next_action_item.html")]
struct ProjectNextActionItemTemplate {
    action: ProjectNextActionView,
}

fn build_project_view(project: &Project, next_action_count: i64) -> ProjectView {
    ProjectView {
        id: project.id().as_uuid().to_string(),
        title: project.title().as_str().to_string(),
        next_action_count,
        is_stalled: next_action_count == 0,
    }
}

fn build_action_view(action: &NextAction, contexts: &[Context]) -> ProjectNextActionView {
    let context_name = contexts
        .iter()
        .find(|c| c.id() == action.context_id())
        .map(|c| c.name().as_str().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    ProjectNextActionView {
        id: action.id().as_uuid().to_string(),
        title: action.title().as_str().to_string(),
        context_name,
        is_completed: action.is_completed(),
    }
}

pub async fn get_projects(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<Response, ProjectError> {
    let inbox_count = inbox_service::get_inbox_count(&pool, &user_id)
        .await
        .map_err(ProjectError::Unexpected)?;

    let projects = project_service::list_active_projects(&pool, &user_id)
        .await
        .map_err(ProjectError::Unexpected)?;

    let mut project_views = Vec::new();
    for project in &projects {
        let count = crate::infrastructure::project_repository::count_project_next_actions(
            &pool,
            project.id(),
        )
        .await
        .map_err(|e| ProjectError::Unexpected(anyhow::anyhow!("Database error: {e}")))?;
        project_views.push(build_project_view(project, count));
    }

    let template = ProjectsTemplate {
        current_page: "projects",
        inbox_count,
        projects: project_views,
        error_message: None,
    };
    Ok(Html(template.render()?).into_response())
}

#[derive(serde::Deserialize)]
pub struct AddProjectForm {
    pub title: String,
}

pub async fn post_project(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Form(form): Form<AddProjectForm>,
) -> Result<Response, ProjectError> {
    let htmx = is_htmx_request(&headers);

    if form.title.trim().is_empty() {
        if htmx {
            return Ok(StatusCode::NO_CONTENT.into_response());
        }
        return Ok(Redirect::to("/projects").into_response());
    }

    match project_service::add_project(&pool, user_id.clone(), form.title).await {
        Ok(project) => {
            if htmx {
                let template = ProjectItemTemplate {
                    project: build_project_view(&project, 0),
                };
                let body = template.render().map_err(ProjectError::Render)?;
                Ok(htmx_response_with_announce(Html(body), "Project added"))
            } else {
                Ok(Redirect::to("/projects").into_response())
            }
        }
        Err(AddProjectError::InvalidTitle(title_err)) => {
            let user_facing = match title_err {
                TodoTitleError::Empty => {
                    if htmx {
                        return Ok(StatusCode::NO_CONTENT.into_response());
                    }
                    return Ok(Redirect::to("/projects").into_response());
                }
                TodoTitleError::TooLong { max, .. } => {
                    format!("That title is too long (max {max} characters)")
                }
            };

            render_projects_with_error(&pool, &user_id, &user_facing).await
        }
        Err(AddProjectError::Unexpected(err)) => Err(ProjectError::Unexpected(err)),
    }
}

async fn render_projects_with_error(
    pool: &PgPool,
    user_id: &crate::domain::UserId,
    error_message: &str,
) -> Result<Response, ProjectError> {
    let inbox_count = inbox_service::get_inbox_count(pool, user_id)
        .await
        .map_err(ProjectError::Unexpected)?;

    let projects = project_service::list_active_projects(pool, user_id)
        .await
        .map_err(ProjectError::Unexpected)?;

    let mut project_views = Vec::new();
    for project in &projects {
        let count = crate::infrastructure::project_repository::count_project_next_actions(
            pool,
            project.id(),
        )
        .await
        .map_err(|e| ProjectError::Unexpected(anyhow::anyhow!("Database error: {e}")))?;
        project_views.push(build_project_view(project, count));
    }

    let template = ProjectsTemplate {
        current_page: "projects",
        inbox_count,
        projects: project_views,
        error_message: Some(error_message.to_string()),
    };
    let body = template.render().map_err(ProjectError::Render)?;
    Ok((StatusCode::UNPROCESSABLE_ENTITY, Html(body)).into_response())
}

pub async fn get_project_detail(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    Path(project_id): Path<Uuid>,
) -> Result<Response, ProjectError> {
    let pid = ProjectId::from_uuid(project_id);

    let detail = match project_service::get_project_detail(&pool, &pid, &user_id).await {
        Ok(d) => d,
        Err(GetProjectError::NotFound) => {
            return Ok((
                StatusCode::NOT_FOUND,
                Html("<h1>Project not found</h1>".to_string()),
            )
                .into_response())
        }
        Err(GetProjectError::Unauthorized) => {
            return Ok((
                StatusCode::FORBIDDEN,
                Html("<h1>Not authorized</h1>".to_string()),
            )
                .into_response())
        }
        Err(GetProjectError::Unexpected(err)) => return Err(ProjectError::Unexpected(err)),
    };

    let contexts = context_service::list_contexts(&pool, &user_id)
        .await
        .map_err(ProjectError::Unexpected)?;

    let inbox_count = inbox_service::get_inbox_count(&pool, &user_id)
        .await
        .map_err(ProjectError::Unexpected)?;

    let project_view = build_project_view(&detail.project, detail.active_next_action_count);

    let active_actions: Vec<ProjectNextActionView> = detail
        .next_actions
        .iter()
        .filter(|a| a.is_active())
        .map(|a| build_action_view(a, &contexts))
        .collect();

    let completed_actions: Vec<ProjectNextActionView> = detail
        .next_actions
        .iter()
        .filter(|a| a.is_completed())
        .map(|a| build_action_view(a, &contexts))
        .collect();

    let context_options: Vec<ContextOption> = contexts
        .iter()
        .map(|c| ContextOption {
            id: c.id().as_uuid().to_string(),
            name: c.name().as_str().to_string(),
        })
        .collect();

    let template = ProjectDetailTemplate {
        current_page: "projects",
        inbox_count,
        project: project_view,
        active_actions,
        completed_actions,
        contexts: context_options,
        error_message: None,
    };
    Ok(Html(template.render()?).into_response())
}

pub async fn post_complete_project(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Response, ProjectError> {
    let htmx = is_htmx_request(&headers);
    let pid = ProjectId::from_uuid(project_id);

    match project_service::complete_project(&pool, &pid, &user_id).await {
        Ok(()) => {
            if htmx {
                Ok(htmx_response_with_announce(
                    Html(String::new()),
                    "Project completed",
                ))
            } else {
                Ok(Redirect::to("/projects").into_response())
            }
        }
        Err(CompleteProjectError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Project not found</h1>".to_string()),
        )
            .into_response()),
        Err(CompleteProjectError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(CompleteProjectError::Unexpected(err)) => Err(ProjectError::Unexpected(err)),
    }
}

pub async fn post_delete_project(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Response, ProjectError> {
    let htmx = is_htmx_request(&headers);
    let pid = ProjectId::from_uuid(project_id);

    match project_service::delete_project(&pool, &pid, &user_id).await {
        Ok(()) => {
            if htmx {
                Ok(htmx_response_with_announce(
                    Html(String::new()),
                    "Project deleted",
                ))
            } else {
                Ok(Redirect::to("/projects").into_response())
            }
        }
        Err(DeleteProjectError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Project not found</h1>".to_string()),
        )
            .into_response()),
        Err(DeleteProjectError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(DeleteProjectError::Unexpected(err)) => Err(ProjectError::Unexpected(err)),
    }
}

pub async fn get_edit_project(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Response, ProjectError> {
    let pid = ProjectId::from_uuid(project_id);
    let project = crate::infrastructure::project_repository::find_project_by_id(&pool, &pid)
        .await
        .map_err(|e| ProjectError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or_else(|| ProjectError::Unexpected(anyhow::anyhow!("Project not found")))?;

    if project.user_id() != &user_id {
        return Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response());
    }

    let htmx = is_htmx_request(&headers);

    if htmx {
        let template = ProjectEditTemplate {
            project: build_project_view(&project, 0),
        };
        let body = template.render().map_err(ProjectError::Render)?;
        Ok(Html(body).into_response())
    } else {
        Ok(Redirect::to("/projects").into_response())
    }
}

#[derive(serde::Deserialize)]
pub struct EditProjectForm {
    pub title: String,
}

pub async fn post_edit_project(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Form(form): Form<EditProjectForm>,
) -> Result<Response, ProjectError> {
    let htmx = is_htmx_request(&headers);
    let pid = ProjectId::from_uuid(project_id);

    match project_service::update_project_title(&pool, &pid, &user_id, form.title).await {
        Ok(()) => {
            if htmx {
                let project =
                    crate::infrastructure::project_repository::find_project_by_id(&pool, &pid)
                        .await
                        .map_err(|e| {
                            ProjectError::Unexpected(anyhow::anyhow!("Database error: {e}"))
                        })?
                        .ok_or_else(|| {
                            ProjectError::Unexpected(anyhow::anyhow!("Project not found"))
                        })?;

                let count = crate::infrastructure::project_repository::count_project_next_actions(
                    &pool, &pid,
                )
                .await
                .map_err(|e| ProjectError::Unexpected(anyhow::anyhow!("Database error: {e}")))?;

                let template = ProjectItemTemplate {
                    project: build_project_view(&project, count),
                };
                let body = template.render().map_err(ProjectError::Render)?;
                Ok(htmx_response_with_announce(Html(body), "Project updated"))
            } else {
                Ok(Redirect::to("/projects").into_response())
            }
        }
        Err(UpdateProjectTitleError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Project not found</h1>".to_string()),
        )
            .into_response()),
        Err(UpdateProjectTitleError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(UpdateProjectTitleError::InvalidTitle(title_err)) => {
            let user_facing = match title_err {
                TodoTitleError::Empty => "Title cannot be empty".to_string(),
                TodoTitleError::TooLong { max, .. } => {
                    format!("That title is too long (max {max} characters)")
                }
            };
            render_projects_with_error(&pool, &user_id, &user_facing).await
        }
        Err(UpdateProjectTitleError::Unexpected(err)) => Err(ProjectError::Unexpected(err)),
    }
}

pub async fn get_project_item(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    Path(project_id): Path<Uuid>,
) -> Result<Response, ProjectError> {
    let pid = ProjectId::from_uuid(project_id);
    let project = crate::infrastructure::project_repository::find_project_by_id(&pool, &pid)
        .await
        .map_err(|e| ProjectError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or_else(|| ProjectError::Unexpected(anyhow::anyhow!("Project not found")))?;

    if project.user_id() != &user_id {
        return Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response());
    }

    let count = crate::infrastructure::project_repository::count_project_next_actions(&pool, &pid)
        .await
        .map_err(|e| ProjectError::Unexpected(anyhow::anyhow!("Database error: {e}")))?;

    let template = ProjectItemTemplate {
        project: build_project_view(&project, count),
    };
    let body = template.render().map_err(ProjectError::Render)?;
    Ok(Html(body).into_response())
}

#[derive(serde::Deserialize)]
pub struct AddProjectNextActionForm {
    pub title: String,
    pub context_id: Uuid,
}

pub async fn post_project_next_action(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Form(form): Form<AddProjectNextActionForm>,
) -> Result<Response, ProjectError> {
    let htmx = is_htmx_request(&headers);
    let pid = ProjectId::from_uuid(project_id);
    let context_id = crate::domain::ContextId::from_uuid(form.context_id);

    if form.title.trim().is_empty() {
        if htmx {
            return Ok(StatusCode::NO_CONTENT.into_response());
        }
        return Ok(Redirect::to(&format!("/projects/{project_id}")).into_response());
    }

    match project_service::add_next_action_to_project(&pool, &user_id, &pid, context_id, form.title)
        .await
    {
        Ok(action) => {
            if htmx {
                let contexts = context_service::list_contexts(&pool, &user_id)
                    .await
                    .map_err(ProjectError::Unexpected)?;

                let template = ProjectNextActionItemTemplate {
                    action: build_action_view(&action, &contexts),
                };
                let body = template.render().map_err(ProjectError::Render)?;
                Ok(htmx_response_with_announce(
                    Html(body),
                    "Next action added to project",
                ))
            } else {
                Ok(Redirect::to(&format!("/projects/{project_id}")).into_response())
            }
        }
        Err(AddProjectNextActionError::ProjectNotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Project not found</h1>".to_string()),
        )
            .into_response()),
        Err(AddProjectNextActionError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(AddProjectNextActionError::InvalidTitle(title_err)) => {
            let user_facing = match title_err {
                TodoTitleError::Empty => {
                    if htmx {
                        return Ok(StatusCode::NO_CONTENT.into_response());
                    }
                    return Ok(Redirect::to(&format!("/projects/{project_id}")).into_response());
                }
                TodoTitleError::TooLong { max, .. } => {
                    format!("That title is too long (max {max} characters)")
                }
            };
            // For now, redirect with error — full re-render would need project detail context
            let _ = user_facing;
            Ok(Redirect::to(&format!("/projects/{project_id}")).into_response())
        }
        Err(AddProjectNextActionError::Unexpected(err)) => Err(ProjectError::Unexpected(err)),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("Failed to render template")]
    Render(#[from] askama::Error),
    #[error(transparent)]
    Unexpected(anyhow::Error),
}

impl IntoResponse for ProjectError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "Project error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Something went wrong</h1>".to_string()),
        )
            .into_response()
    }
}
