use std::mem;

use axum::{extract::State, http::StatusCode, Form};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DbErr};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::startup::AppState;

use entity::subscriptions::{self};

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(state, form),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(State(state): State<AppState>, form: Form<FormData>) -> StatusCode {
    match insert_subscribe(&state, form.0).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(state, form)
)]
pub async fn insert_subscribe(state: &AppState, mut form: FormData) -> Result<(), DbErr> {
    let subscription = subscriptions::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(mem::take(&mut form.email)),
        name: Set(mem::take(&mut form.name)),
        subscribed_at: Set(OffsetDateTime::now_utc()),
    };
    subscription
        .insert(&state.connection)
        .await
        .map_err(|err| {
            tracing::error!("Failed to execute query: {:?}", err);
            err
        })?;
    Ok(())
}
