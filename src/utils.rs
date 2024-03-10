use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub fn e500<T>(e: T) -> Response
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    tracing::error!("{:?}", e);
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}
