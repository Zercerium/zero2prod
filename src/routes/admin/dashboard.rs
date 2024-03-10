use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use entity::prelude::Users;
use handlebars::Handlebars;
use sea_orm::{DatabaseConnection, DerivePartialModel, EntityTrait, FromQueryResult};
use uuid::Uuid;

use crate::{session_state::TypedSession, startup::AppState, utils::e500};

pub async fn admin_dashboard(
    State(state): State<AppState>,
    session: TypedSession,
) -> Result<Response, Response> {
    let username = if let Some(user_id) = session.get_user_id().await.map_err(e500)? {
        get_username(user_id, &state.connection)
            .await
            .map_err(e500)?
    } else {
        return Ok(Redirect::to("/login").into_response());
    };

    let reg = Handlebars::new();
    let html = reg
        .render_template(
            include_str!("./dashboard.html"),
            &serde_json::json!({"username": username}),
        )
        .map_err(e500)?;

    Ok((StatusCode::OK, Html::from(html)).into_response())
}

#[tracing::instrument(name = "Get username", skip(conn))]
pub async fn get_username(
    user_id: Uuid,
    conn: &DatabaseConnection,
) -> Result<String, anyhow::Error> {
    #[derive(DerivePartialModel, FromQueryResult)]
    #[sea_orm(entity = "Users")]
    struct Row {
        username: String,
    }

    let row = Users::find_by_id(user_id)
        .into_partial_model::<Row>()
        .one(conn)
        .await
        .context("Failed to perform a query to retrieve a username.")?
        .ok_or_else(|| anyhow::anyhow!("User not found."))?;

    Ok(row.username)
}
