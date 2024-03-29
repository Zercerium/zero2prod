use axum::response::{Html, IntoResponse, Response};
use axum_messages::Messages;
use handlebars::Handlebars;
use serde_json::json;
use std::fmt::Write;

pub async fn change_password_form(messages: Messages) -> Result<Response, Response> {
    let mut msg_html = String::new();
    for m in messages.into_iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.message).unwrap();
    }

    let reg = Handlebars::new();
    let html = reg
        .render_template(
            include_str!("./get.html"),
            &json!({
                "messages": msg_html,
            }),
        )
        .expect("Failed to render password page.");

    Ok(Html::from(html).into_response())
}
