use axum::{
    body::Body,
    http::Request,
    routing::{get, post},
    serve::Serve,
    Router,
};
use sea_orm::DatabaseConnection;
use std::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::routes::{health_check, subscribe};

#[derive(Clone)]
pub struct AppState {
    pub connection: DatabaseConnection,
}

pub fn run(
    listener: TcpListener,
    connection: DatabaseConnection,
) -> Result<Serve<Router, Router>, std::io::Error> {
    let state = AppState { connection };

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
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
