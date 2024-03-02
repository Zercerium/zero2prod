use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use strum::{Display, EnumString};

use crate::domain::SubscriberEmail;

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub ssl_mode: PostgresSslMode,
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: Secret<String>,
    pub timeout_milliseconds: u64,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");
    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;
    settings.try_deserialize::<Settings>()
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. \
                Use either `local` or `production`.",
                other
            )),
        }
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> DatabaseConnection {
    let connection = sea_orm::Database::connect(config.without_db())
        .await
        .expect("Failed to connect to Postgres.");

    connection
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!(r#"CREATE DATABASE "{}";"#, config.database_name),
        ))
        .await
        .expect("Failed to create database.");

    let connection = sea_orm::Database::connect(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    Migrator::up(&connection, None)
        .await
        .expect("Failed to migrate the database");
    connection
}

impl DatabaseSettings {
    pub fn without_db(&self) -> ConnectOptions {
        ConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(self.ssl_mode)
    }

    pub fn with_db(&self) -> ConnectOptions {
        self.without_db().database(&self.database_name)
    }
}

pub struct ConnectOptions {
    host: Option<String>,
    username: Option<String>,
    password: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    ssl_mode: Option<PostgresSslMode>,
}

impl ConnectOptions {
    pub fn new() -> Self {
        Self {
            host: None,
            username: None,
            password: None,
            port: None,
            database: None,
            ssl_mode: None,
        }
    }
    pub fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }
    pub fn username(mut self, username: &str) -> Self {
        self.username = Some(username.to_string());
        self
    }
    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
    pub fn database(mut self, database: &str) -> Self {
        self.database = Some(database.to_string());
        self
    }
    pub fn ssl_mode(mut self, ssl_mode: PostgresSslMode) -> Self {
        self.ssl_mode = Some(ssl_mode);
        self
    }
}

impl Default for ConnectOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ConnectOptions> for sea_orm::ConnectOptions {
    fn from(val: ConnectOptions) -> Self {
        let username = val.username.expect("username not set");
        let password = val.password.expect("password not set");
        let host = val.host.expect("host not set");
        let port = val.port.expect("port not set");
        let ssl_mode = val.ssl_mode.unwrap_or_default();
        let url = format!("postgres://{}:{}@{}:{}", username, password, host, port);
        let url = match val.database {
            Some(database) => format!("{}/{}", url, database).to_string(),
            None => url,
        };
        let url = format!("{}?sslmode={}", url, ssl_mode);
        sea_orm::ConnectOptions::new(url)
    }
}

#[derive(EnumString, Display, Deserialize, Default, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
pub enum PostgresSslMode {
    #[strum(ascii_case_insensitive)]
    Disable,
    #[strum(ascii_case_insensitive)]
    Allow,
    #[default]
    #[strum(ascii_case_insensitive)]
    Prefer,
    #[strum(ascii_case_insensitive)]
    Require,
    #[strum(ascii_case_insensitive)]
    VerifyCa,
    #[strum(ascii_case_insensitive)]
    VerifyFull,
}
