use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
};
use handlebars::Handlebars;
use hmac::{Hmac, Mac};
use secrecy::ExposeSecret;
use serde_json::json;

use crate::startup::{AppState, HmacSecret};

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: String,
    tag: String,
}

impl QueryParams {
    fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag)?;
        let query_string = format!("error={}", urlencoding::Encoded::new(&self.error));

        let mut mac =
            Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)?;

        Ok(self.error)
    }
}

pub async fn login_form(
    State(state): State<AppState>,
    query: Option<Query<QueryParams>>,
) -> (StatusCode, Html<String>) {
    let error_html = match query {
        None => "".into(),
        Some(query) => match query.0.verify(&state.hmac_secret) {
            Ok(error) => format!("<p><i>{}</i></p>", htmlescape::encode_minimal(&error)),
            Err(e) => {
                tracing::warn!(
                    error.message = %e,
                    error.cause_chain = ?e,
                    "Failed to verify query parameters using HMAC tag"
                );
                "".into()
            }
        },
    };

    let login_form = include_str!("./login.html");
    let reg = Handlebars::new();
    let login_form = reg
        .render_template(login_form, &json!({"error_html": error_html}))
        .expect("Failed to render login form.");

    (StatusCode::OK, Html::from(login_form))
}
