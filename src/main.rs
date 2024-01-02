use std::net::TcpListener;

use migration::{Migrator, MigratorTrait};

use zero2prod::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL")?;
    let connection = sea_orm::Database::connect(&database_url).await?;
    Migrator::up(&connection, None).await?;

    let listener = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind random port");
    Ok(run(listener)?.await?)
}
