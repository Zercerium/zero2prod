use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new(
            "configuration.yaml",
            config::FileFormat::Yaml,
        ))
        .build()?;
    settings.try_deserialize::<Settings>()
}

pub async fn configure_database(config: &DatabaseSettings) -> DatabaseConnection {
    let connection =
        sea_orm::Database::connect(config.connection_string_without_db().expose_secret())
            .await
            .expect("Failed to connect to Postgres.");

    connection
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!(r#"CREATE DATABASE "{}";"#, config.database_name),
        ))
        .await
        .expect("Failed to create database.");

    let connection = sea_orm::Database::connect(config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");
    Migrator::up(&connection, None)
        .await
        .expect("Failed to migrate the database");
    connection
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }

    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}
