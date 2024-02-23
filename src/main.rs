use std::net::TcpListener;

use migration::{Migrator, MigratorTrait};
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "zero2prod=info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection = sea_orm::Database::connect(configuration.database.with_db())
        .await
        .expect("Failed to connect to the database.");
    Migrator::up(&connection, None).await?;

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    Ok(run(listener, connection)?.await?)
}
