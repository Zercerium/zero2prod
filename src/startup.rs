use axum::{
    body::Body,
    http::Request,
    routing::{get, post},
    serve::Serve,
    Router,
};
use axum_messages::MessagesManagerLayer;
use migration::{Migrator, MigratorTrait};
use sea_orm::DatabaseConnection;
use secrecy::Secret;
use std::net::TcpListener;
use std::sync::Arc;
use time::Duration;
use tower_http::trace::TraceLayer;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::{
    fred::{
        self,
        clients::RedisPool,
        interfaces::ClientLike,
        types::{RedisConfig, ServerConfig},
    },
    RedisStore,
};

use crate::{
    configuration::{RedisSettings, Settings},
    email_client::EmailClient,
    routes::{
        admin_dashboard, change_password, change_password_form, confirm, health_check, home,
        log_out, login, login_form, publish_newsletter, subscribe,
    },
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
            configuration.application.hmac_secret,
            configuration.redis,
        )
        .await?;

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

async fn run(
    listener: TcpListener,
    connection: DatabaseConnection,
    email_client: EmailClient,
    base_url: String,
    _hmac_secret: Secret<String>,
    redis: RedisSettings,
) -> Result<Serve<Router, Router>, anyhow::Error> {
    let email_client = Arc::new(email_client);
    let state = AppState {
        connection,
        email_client,
        base_url,
    };

    let redis_config = RedisConfig {
        server: ServerConfig::Centralized {
            server: fred::types::Server::new(redis.host, redis.port),
        },
        ..Default::default()
    };
    let redis_pool = RedisPool::new(redis_config, None, None, None, 6)?;
    let _redis_conn = redis_pool.connect();
    redis_pool.wait_for_connect().await?;

    let session_store = RedisStore::new(redis_pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(10)));

    let app = Router::new()
        .route("/", get(home))
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions/confirm", get(confirm))
        .route("/newsletters", post(publish_newsletter))
        .route("/login", get(login_form).post(login))
        .route("/admin/dashboard", get(admin_dashboard))
        .route(
            "/admin/password",
            get(change_password_form).post(change_password),
        )
        .route("/admin/logout", post(log_out))
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
        .layer(MessagesManagerLayer)
        // `actix_session` needs a key which will be used for signing the session cookies
        // https://docs.rs/actix-session/latest/actix_session/struct.SessionMiddleware.html#method.new
        // this is not the case for `tower_sessions` see https://github.com/maxcountryman/tower-sessions/discussions/100
        // > tower-sessions doesn't provide signing because no data is stored in the cookie.
        // > In other words, the cookie value is a pointer to the data stored server side.
        .layer(session_layer)
        .with_state(state);

    listener.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(listener)?;

    let server = axum::serve(listener, app);
    Ok(server)
}

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);
