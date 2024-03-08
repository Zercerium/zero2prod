use axum::{http::StatusCode, response::Html};
use axum_extra::extract::CookieJar;
use handlebars::Handlebars;
use serde_json::json;

pub async fn login_form(jar: CookieJar) -> (StatusCode, CookieJar, Html<String>) {
    let error_html = match jar.get("_flash") {
        None => "".into(),
        Some(cookie) => {
            format!("<p><i>{}</i></p>", cookie.value())
        }
    };

    let login_form = include_str!("./login.html");
    let reg = Handlebars::new();
    let login_form = reg
        .render_template(login_form, &json!({"error_html": error_html}))
        .expect("Failed to render login form.");

    let jar = jar.remove("_flash");

    (StatusCode::OK, jar, Html::from(login_form))
}
