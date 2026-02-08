use sqlx::{Connection, Executor, PgConnection, PgPool};
use todo_list::configuration::{get_configuration, DatabaseSettings};
use todo_list::startup::Application;
use uuid::Uuid;

#[allow(dead_code)]
pub struct TestApp {
    pub address: String,
    pub pool: PgPool,
    pub db_name: String,
}

pub async fn spawn_app() -> TestApp {
    let mut settings = get_configuration().expect("Failed to read configuration");

    // Use a random database name for test isolation
    settings.database.database_name = Uuid::new_v4().to_string();

    // Use port 0 to get a random available port
    settings.application.port = 0;

    // Create the test database
    configure_database(&settings.database).await;

    let app = Application::build(&settings)
        .await
        .expect("Failed to build application");

    let address = format!("http://127.0.0.1:{}", app.port());
    let pool = PgPool::connect_with(settings.database.connect_options())
        .await
        .expect("Failed to connect to database");

    tokio::spawn(app.run());

    TestApp {
        address,
        pool,
        db_name: settings.database.database_name,
    }
}

async fn configure_database(settings: &DatabaseSettings) {
    // Connect to the default postgres database to create our test database
    let mut connection = PgConnection::connect_with(&settings.connect_options_without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, settings.database_name).as_str())
        .await
        .expect("Failed to create database");
}
