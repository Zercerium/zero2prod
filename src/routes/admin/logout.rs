use axum::response::{IntoResponse, Redirect, Response};
use axum_messages::Messages;

use crate::{session_state::TypedSession, utils::e500};

pub async fn log_out(session: TypedSession, messages: Messages) -> Result<Response, Response> {
    if session.get_user_id().await.map_err(e500)?.is_none() {
        Ok(Redirect::to("/login").into_response())
    } else {
        session.log_out().await.map_err(e500)?;
        messages.info("You have successfully logged out.");
        Ok(Redirect::to("/login").into_response())
    }
}
