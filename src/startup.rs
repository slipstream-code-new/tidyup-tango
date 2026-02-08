use axum::Router;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::configuration::Settings;
use crate::routes::{health_check, index};

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

        Router::new()
            .route("/", axum::routing::get(index))
            .route("/health_check", axum::routing::get(health_check))
            .nest_service("/static", ServeDir::new("static"))
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
