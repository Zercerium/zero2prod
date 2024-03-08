use axum::{http::StatusCode, response::Html};
use axum_flash::{IncomingFlashes, Level};
use handlebars::Handlebars;
use serde_json::json;
use std::fmt::Write;

pub async fn login_form(flashes: IncomingFlashes) -> (StatusCode, IncomingFlashes, Html<String>) {
    let mut error_html = String::new();
    for (_, text) in flashes.iter().filter(|(level, _)| *level == Level::Error) {
        writeln!(error_html, "<p><i>{}</i></p>", text).unwrap();
    }

    let login_form = include_str!("./login.html");
    let reg = Handlebars::new();
    let login_form = reg
        .render_template(login_form, &json!({"error_html": error_html}))
        .expect("Failed to render login form.");

    (StatusCode::OK, flashes, Html::from(login_form))
}
