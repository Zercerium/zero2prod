use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Form,
};
use axum_flash::Flash;
use secrecy::Secret;
use tower_sessions::Session;

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
    skip(state, flash, session, form),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    State(state): State<AppState>,
    flash: Flash,
    session: Session,
    Form(form): Form<FormData>,
) -> Result<Response, Response> {
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    match validate_credentials(credentials, &state.connection).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::debug(&user_id));

            let redirect = move |e: tower_sessions::session::Error| {
                login_redirect(flash.clone(), LoginError::UnexpectedError(e.into()))
            };
            session.cycle_id().await.map_err(&redirect)?;
            session
                .insert("user_id", user_id)
                .await
                .map_err(&redirect)?;

            Ok((StatusCode::SEE_OTHER, Redirect::to("/admin/dashboard")).into_response())
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };

            let flash = flash.error(e.to_string());

            Ok((flash, Redirect::to("/login")).into_response())
        }
    }
}

// Redirect to the login page with an error message.
fn login_redirect(flash: Flash, e: LoginError) -> Response {
    flash.error(e.to_string());
    (Redirect::to("/login")).into_response()
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
