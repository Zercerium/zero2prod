use std::{borrow::Borrow, mem};

use axum::{extract::State, http::StatusCode, Form};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set, Unchanged},
};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::startup::AppState;

use entity::subscriptions::{self, Entity as Subscription};

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(State(state): State<AppState>, mut form: Form<FormData>) -> StatusCode {
    let subscription = subscriptions::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(mem::take(&mut form.email)),
        name: Set(mem::take(&mut form.name)),
        subscribed_at: Set(OffsetDateTime::now_utc()),
        ..Default::default()
    };

    match subscription.insert(&state.connection).await {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            println!("Failed to execute query: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
