use std::mem;

use axum::{extract::State, http::StatusCode, Form};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DbErr};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    startup::AppState,
};

use entity::subscriptions::{self};

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;
    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
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
    let new_subscriber: NewSubscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => {
            return StatusCode::BAD_REQUEST;
        }
    };
    if insert_subscribe(&state, new_subscriber.clone())
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    let email_client = state.email_client;
    if send_confirmation_email(&email_client, new_subscriber, &state.base_url)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(state, new_subscriber)
)]
pub async fn insert_subscribe(
    state: &AppState,
    new_subscriber: NewSubscriber,
) -> Result<(), DbErr> {
    let subscription = subscriptions::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(mem::take(&mut new_subscriber.email.into())),
        name: Set(mem::take(&mut new_subscriber.name.into())),
        subscribed_at: Set(OffsetDateTime::now_utc()),
        status: Set("pending_confirmation".to_string()),
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

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token=mytoken",
        base_url
    );
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
    Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(new_subscriber.email, "Welcome!", &html_body, &plain_body)
        .await
}
