use axum::Router;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::net::TcpListener;

use crate::configuration::Settings;
use crate::routes::health_check;

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
        Router::new()
            .route("/health_check", axum::routing::get(health_check))
            .with_state(pool)
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let router = Self::router(self.pool);
        axum::serve(self.listener, router).await
    }
}
