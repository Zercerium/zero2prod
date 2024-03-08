use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

pub async fn home() -> (StatusCode, Html<&'static str>) {
    (StatusCode::OK, Html::from(include_str!("./home/home.html")))
}
