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
    get_login, get_register, get_todos_page, health_check, index, post_login, post_logout,
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
            .route(
                "/register",
                axum::routing::get(get_register).post(post_register),
            )
            .route("/login", axum::routing::get(get_login).post(post_login))
            .route("/logout", axum::routing::post(post_logout))
            .route("/todos", axum::routing::get(get_todos_page).post(post_todo))
            .route("/todos/{id}/toggle", axum::routing::post(post_toggle_todo))
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
