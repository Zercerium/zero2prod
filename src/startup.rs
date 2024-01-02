use std::net::TcpListener;

use axum::{
    routing::{get, post},
    serve::Serve,
    Router,
};
use sea_orm::DatabaseConnection;

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
        .with_state(state);

    listener.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(listener)?;

    let server = axum::serve(listener, app);
    Ok(server)
}
