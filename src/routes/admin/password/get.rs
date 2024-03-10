use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_messages::Messages;
use handlebars::Handlebars;
use serde_json::json;
use std::fmt::Write;

use crate::{session_state::TypedSession, utils::e500};

pub async fn change_password_form(
    session: TypedSession,
    messages: Messages,
) -> Result<Response, Response> {
    if session.get_user_id().await.map_err(e500)?.is_none() {
        return Err(Redirect::to("/login").into_response());
    }

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
