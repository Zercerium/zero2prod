use std::net::TcpListener;

use axum::{
    routing::{get, post},
    serve::Serve,
    Form, Router,
};

async fn health_check() {}

#[derive(serde::Deserialize, Debug)]
struct FormData {
    email: String,
    name: String,
}

async fn subscribe(form: Form<FormData>) {
    dbg!(form);
}

pub fn run(listener: TcpListener) -> Result<Serve<Router, Router>, std::io::Error> {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe));

    listener.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(listener)?;

    let server = axum::serve(listener, app);
    Ok(server)
}
