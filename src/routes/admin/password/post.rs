use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
    Extension, Form,
};
use axum_messages::Messages;
use secrecy::{ExposeSecret, Secret};

use crate::{
    authentication::{self, validate_credentials, AuthError, Credentials, UserId},
    routes::admin::dashboard::get_username,
    startup::AppState,
    utils::e500,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    State(state): State<AppState>,
    messages: Messages,
    user_id: Extension<UserId>,
    Form(form): Form<FormData>,
) -> Result<Response, Response> {
    let user_id = **user_id;
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        messages.error("You entered two different new passwords - the field values must match.");
        return Ok(Redirect::to("/admin/password").into_response());
    }
    let username = get_username(user_id, &state.connection)
        .await
        .map_err(e500)?;
    let credentials = Credentials {
        username,
        password: form.current_password,
    };
    if let Err(e) = validate_credentials(credentials, &state.connection).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                messages.error("The current password is incorrect.");
                Ok(Redirect::to("/admin/password").into_response())
            }
            AuthError::UnexpectedError(_) => Err(e500(e)),
        };
    }
    authentication::change_password(user_id, form.new_password, &state.connection)
        .await
        .map_err(e500)?;
    messages.success("Your password has been changed.");
    Ok(Redirect::to("/admin/password").into_response())
}
