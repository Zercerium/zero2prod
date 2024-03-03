use axum::{extract::Query, http::StatusCode};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(params))]
pub async fn confirm(Query(params): Query<Parameters>) -> StatusCode {
    StatusCode::OK
}
