use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DerivePartialModel, EntityTrait, FromQueryResult, QueryFilter,
};

use entity::subscriptions::{self, Entity as Subscriptions};
use serde::Serialize;

use crate::{domain::SubscriberEmail, routes::AppJson, startup::AppState};

use super::error_chain_fmt;

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for PublishError {
    fn into_response(self) -> Response {
        // How we want errors responses to be serialized
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        tracing::error!(exception.details = ?self, exception.message = %self);

        let status = match &self {
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let message = self.to_string();

        (status, AppJson(ErrorResponse { message })).into_response()
    }
}

pub async fn publish_newsletter(
    State(state): State<AppState>,
    Json(body): Json<BodyData>,
) -> Result<StatusCode, PublishError> {
    let _subscribers = get_confirmed_subscribers(&state.connection).await?;
    let subscribers = get_confirmed_subscribers(&state.connection).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                state
                    .email_client
                    .send_email(
                        &subscriber.email,
                        &body.title,
                        &body.content.html,
                        &body.content.text,
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })?;
            }
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber. \
                    Their stored contact details are invalid",
                );
            }
        }
    }
    Ok(StatusCode::OK)
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(conn))]
async fn get_confirmed_subscribers(
    conn: &DatabaseConnection,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    #[derive(DerivePartialModel, FromQueryResult)]
    #[sea_orm(entity = "Subscriptions")]
    struct Row {
        email: String,
    }

    let rows = Subscriptions::find()
        .filter(subscriptions::Column::Status.contains("confirmed"))
        .into_partial_model::<Row>()
        .all(conn)
        .await?;
    let confirmed_subscribers = rows
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(error) => Err(anyhow::anyhow!(error)),
        })
        .collect();
    Ok(confirmed_subscribers)
}
