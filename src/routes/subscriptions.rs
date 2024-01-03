use std::mem;

use axum::{extract::State, http::StatusCode, Form};
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use time::OffsetDateTime;
use tracing::Instrument;
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
        request_id = %Uuid::new_v4(),
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(State(state): State<AppState>, mut form: Form<FormData>) -> StatusCode {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber",
        %request_id,
        email = %form.email,
        name = %form.name
    );
    let _request_span_guard = request_span.enter();

    let subscription = subscriptions::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(mem::take(&mut form.email)),
        name: Set(mem::take(&mut form.name)),
        subscribed_at: Set(OffsetDateTime::now_utc()),
    };

    let query_span = tracing::info_span!("Saving new subscriber details in the database");

    match subscription
        .insert(&state.connection)
        .instrument(query_span)
        .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id {} - New subscriber details have been saved",
                request_id
            );
            StatusCode::OK
        }
        Err(err) => {
            tracing::error!(
                "request_id {} - Failed to execute query: {:?}",
                request_id,
                err
            );
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
