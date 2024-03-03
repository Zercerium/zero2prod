use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, IntoActiveModel, QueryFilter, Set,
};
use uuid::Uuid;

use entity::subscription_tokens::{self, Entity as SubscriptionToken};
use entity::subscriptions::Entity as Subscription;

use crate::startup::AppState;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(state, params))]
pub async fn confirm(
    State(state): State<AppState>,
    Query(params): Query<Parameters>,
) -> StatusCode {
    let id = match get_subscriber_id_from_token(&state.connection, &params.subscription_token).await
    {
        Ok(id) => id,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    match id {
        None => StatusCode::UNAUTHORIZED,
        Some(subscriber_id) => {
            if confirm_subscriber(&state.connection, subscriber_id)
                .await
                .is_err()
            {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        }
    }
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(connection, subscriber_id))]
async fn confirm_subscriber(connection: &DbConn, subscriber_id: Uuid) -> Result<(), DbErr> {
    let subscriber = Subscription::find_by_id(subscriber_id)
        .one(connection)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?
        .ok_or_else(|| {
            tracing::error!("Subscriber with id {} not found", subscriber_id);
            DbErr::RecordNotFound(format!("Subscriber with id {} not found", subscriber_id))
        })?;
    let mut subscriber = subscriber.into_active_model();
    subscriber.status = Set("confirmed".to_string());
    subscriber.update(connection).await?;
    Ok(())
}

#[tracing::instrument(
    name = "Get subscriber_id from token",
    skip(connection, subscription_token)
)]
async fn get_subscriber_id_from_token(
    connection: &DbConn,
    subscription_token: &str,
) -> Result<Option<Uuid>, DbErr> {
    let result = SubscriptionToken::find()
        .filter(subscription_tokens::Column::SubscriptionToken.contains(subscription_token))
        .one(connection)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    Ok(result.map(|r| r.subscriber_id))
}
