use todo_list::configuration::get_configuration;
use todo_list::startup::Application;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let settings = get_configuration().expect("Failed to read configuration");
    let app = Application::build(&settings).await?;
    app.run().await?;

    Ok(())
}
