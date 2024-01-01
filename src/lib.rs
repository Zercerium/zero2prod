use std::net::TcpListener;

use axum::{routing::get, serve::Serve, Router};

async fn health_check() {}

pub fn run(listener: TcpListener) -> Result<Serve<Router, Router>, std::io::Error> {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health_check", get(health_check));

    listener.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(listener)?;

    let server = axum::serve(listener, app);
    Ok(server)
}
