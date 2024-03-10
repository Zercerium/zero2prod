use axum::{http::StatusCode, response::Html};
use axum_messages::Messages;
use handlebars::Handlebars;
use serde_json::json;
use std::fmt::Write;

pub async fn login_form(flashes: Messages) -> (StatusCode, Html<String>) {
    let mut error_html = String::new();
    for message in flashes.into_iter() {
        writeln!(error_html, "<p><i>{}</i></p>", message.message).unwrap();
    }

    let login_form = include_str!("./login.html");
    let reg = Handlebars::new();
    let login_form = reg
        .render_template(login_form, &json!({"error_html": error_html}))
        .expect("Failed to render login form.");

    (StatusCode::OK, Html::from(login_form))
}
