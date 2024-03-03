use axum::{
    body::Body,
    http::Request,
    routing::{get, post},
    serve::Serve,
    Router,
};
use migration::{Migrator, MigratorTrait};
use sea_orm::DatabaseConnection;
use std::net::TcpListener;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::{
    configuration::Settings,
    email_client::EmailClient,
    routes::{confirm, health_check, subscribe},
};

#[derive(Clone)]
pub struct AppState {
    pub connection: DatabaseConnection,
    pub email_client: Arc<EmailClient>,
    pub base_url: String,
}

pub struct Application {
    port: u16,
    server: Serve<Router, Router>,
}

impl Application {
    pub async fn build(configuration: Settings) -> anyhow::Result<Self> {
        let connection = sea_orm::Database::connect(configuration.database.with_db())
            .await
            .expect("Failed to connect to the database.");
        Migrator::up(&connection, None).await?;

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection,
            email_client,
            configuration.application.base_url,
        )?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub struct ApplicationBaseUrl(pub String);

pub fn run(
    listener: TcpListener,
    connection: DatabaseConnection,
    email_client: EmailClient,
    base_url: String,
) -> Result<Serve<Router, Router>, std::io::Error> {
    let email_client = Arc::new(email_client);
    let state = AppState {
        connection,
        email_client,
        base_url,
    };

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions/confirm", get(confirm))
        .layer(
            // thanks to https://github.com/tokio-rs/axum/discussions/2273
            tower::ServiceBuilder::new().layer(TraceLayer::new_for_http().make_span_with(
                |request: &Request<Body>| {
                    let request_id = uuid::Uuid::new_v4();
                    tracing::span!(
                        tracing::Level::INFO,
                        "request",
                        method = tracing::field::display(request.method()),
                        uri = tracing::field::display(request.uri()),
                        version = tracing::field::debug(request.version()),
                        request_id = tracing::field::display(request_id)
                    )
                },
            )),
        )
        .with_state(state);

    listener.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(listener)?;

    let server = axum::serve(listener, app);
    Ok(server)
}
