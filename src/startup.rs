use axum::Router;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::PostgresStore;

use crate::configuration::Settings;
use crate::routes::{
    get_contexts, get_dashboard, get_edit_context, get_edit_next_action, get_edit_project,
    get_edit_todo, get_forgot_password, get_inbox, get_login, get_next_action_item,
    get_next_actions, get_project_detail, get_project_item, get_projects, get_register, get_review,
    get_someday_maybe, get_todo_item, get_todos_page, get_waiting_for, health_check, index,
    post_clarify_inbox_item, post_complete_next_action, post_complete_project, post_context,
    post_delete_context, post_delete_inbox_item, post_delete_next_action, post_delete_project,
    post_delete_todo, post_edit_context, post_edit_next_action, post_edit_project, post_edit_todo,
    post_inbox, post_login, post_logout, post_next_action, post_project, post_project_next_action,
    post_register, post_todo, post_toggle_todo,
};

pub struct Application {
    listener: TcpListener,
    pool: PgPool,
}

impl Application {
    pub async fn build(settings: &Settings) -> Result<Self, anyhow::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(settings.database.connect_options())
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        let session_store = PostgresStore::new(pool.clone());
        session_store.migrate().await?;

        let address = format!(
            "{}:{}",
            settings.application.host, settings.application.port
        );
        let listener = TcpListener::bind(&address).await?;

        tracing::info!("Listening on {}", listener.local_addr()?);

        Ok(Self { listener, pool })
    }

    pub fn port(&self) -> u16 {
        self.listener.local_addr().unwrap().port()
    }

    pub fn router(pool: PgPool) -> Router {
        let x_request_id = axum::http::HeaderName::from_static("x-request-id");

        let session_store = PostgresStore::new(pool.clone());
        let session_layer = SessionManagerLayer::new(session_store);

        Router::new()
            .route("/", axum::routing::get(index))
            .route("/dashboard", axum::routing::get(get_dashboard))
            .route("/inbox", axum::routing::get(get_inbox).post(post_inbox))
            .route(
                "/inbox/{id}/delete",
                axum::routing::post(post_delete_inbox_item),
            )
            .route(
                "/inbox/{id}/clarify",
                axum::routing::post(post_clarify_inbox_item),
            )
            .route(
                "/contexts",
                axum::routing::get(get_contexts).post(post_context),
            )
            .route(
                "/contexts/{id}/edit",
                axum::routing::get(get_edit_context).post(post_edit_context),
            )
            .route(
                "/contexts/{id}/delete",
                axum::routing::post(post_delete_context),
            )
            .route(
                "/next-actions",
                axum::routing::get(get_next_actions).post(post_next_action),
            )
            .route(
                "/next-actions/{id}",
                axum::routing::get(get_next_action_item),
            )
            .route(
                "/next-actions/{id}/complete",
                axum::routing::post(post_complete_next_action),
            )
            .route(
                "/next-actions/{id}/delete",
                axum::routing::post(post_delete_next_action),
            )
            .route(
                "/next-actions/{id}/edit",
                axum::routing::get(get_edit_next_action).post(post_edit_next_action),
            )
            .route(
                "/projects",
                axum::routing::get(get_projects).post(post_project),
            )
            .route("/projects/{id}", axum::routing::get(get_project_detail))
            .route("/projects/{id}/item", axum::routing::get(get_project_item))
            .route(
                "/projects/{id}/complete",
                axum::routing::post(post_complete_project),
            )
            .route(
                "/projects/{id}/delete",
                axum::routing::post(post_delete_project),
            )
            .route(
                "/projects/{id}/edit",
                axum::routing::get(get_edit_project).post(post_edit_project),
            )
            .route(
                "/projects/{id}/next-actions",
                axum::routing::post(post_project_next_action),
            )
            .route("/waiting-for", axum::routing::get(get_waiting_for))
            .route("/someday-maybe", axum::routing::get(get_someday_maybe))
            .route("/review", axum::routing::get(get_review))
            .route(
                "/register",
                axum::routing::get(get_register).post(post_register),
            )
            .route("/login", axum::routing::get(get_login).post(post_login))
            .route("/forgot-password", axum::routing::get(get_forgot_password))
            .route("/logout", axum::routing::post(post_logout))
            .route("/todos", axum::routing::get(get_todos_page).post(post_todo))
            .route("/todos/{id}", axum::routing::get(get_todo_item))
            .route("/todos/{id}/toggle", axum::routing::post(post_toggle_todo))
            .route("/todos/{id}/delete", axum::routing::post(post_delete_todo))
            .route(
                "/todos/{id}/edit",
                axum::routing::get(get_edit_todo).post(post_edit_todo),
            )
            .route("/health_check", axum::routing::get(health_check))
            .nest_service("/static", ServeDir::new("static"))
            .layer(session_layer)
            .layer(TraceLayer::new_for_http())
            .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
            .layer(SetRequestIdLayer::new(x_request_id, MakeRequestUuid))
            .with_state(pool)
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let router = Self::router(self.pool);
        axum::serve(self.listener, router).await
    }
}
