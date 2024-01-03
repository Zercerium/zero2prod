use std::net::TcpListener;

use migration::{Migrator, MigratorTrait};
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber(
        "zero2prod".into(),
        "zero2prod=info,tower_http=debug,axum::rejection=trace".into(),
    );
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection_string = configuration.database.connection_string();
    let connection = sea_orm::Database::connect(connection_string).await?;
    Migrator::up(&connection, None).await?;

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    Ok(run(listener, connection)?.await?)
}
