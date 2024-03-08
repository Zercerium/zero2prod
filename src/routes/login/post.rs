use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Form,
};
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, Secret};

use crate::{
    authentication::{validate_credentials, AuthError, Credentials},
    routes::error_chain_fmt,
    startup::AppState,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[tracing::instrument(
    skip(state, form),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(State(state): State<AppState>, Form(form): Form<FormData>) -> Response {
    // ) -> Result<Response, LoginError> {
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    match validate_credentials(credentials, &state.connection).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::debug(&user_id));
            Response::builder()
                .status(StatusCode::SEE_OTHER)
                .header(header::LOCATION, "/")
                .body(Body::empty())
                .unwrap()
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };
            let query_string = format!("error={}", urlencoding::Encoded::new(e.to_string()));
            let hmac_tag = {
                let mut mac = Hmac::<sha2::Sha256>::new_from_slice(
                    state.hmac_secret.0.expose_secret().as_bytes(),
                )
                .unwrap();
                mac.update(query_string.as_bytes());
                mac.finalize().into_bytes()
            };
            let mut header = HeaderMap::new();
            header.insert(
                header::LOCATION,
                format!("/login?{query_string}&tag={hmac_tag:x}")
                    .parse()
                    .unwrap(),
            );

            (StatusCode::SEE_OTHER, header).into_response()
        }
    }
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
