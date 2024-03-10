use std::ops::Deref;

use anyhow::Context;
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Extension, Form,
};
use entity::subscriptions::{self, Entity as Subscriptions};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DerivePartialModel, EntityTrait, FromQueryResult, QueryFilter,
};
use serde::Serialize;

use crate::{
    authentication::UserId,
    domain::SubscriberEmail,
    routes::{error_chain_fmt, AppJson},
    startup::AppState,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    content_html: String,
    content_txt: String,
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
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
            Self::AuthError(_) => StatusCode::UNAUTHORIZED,
        };

        let message = self.to_string();
        let mut headers = HeaderMap::new();
        let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();
        headers.insert(header::WWW_AUTHENTICATE, header_value);
        (status, headers, AppJson(ErrorResponse { message })).into_response()
    }
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(state, body),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    State(state): State<AppState>,
    user_id: Extension<UserId>,
    Form(body): Form<FormData>,
) -> Result<StatusCode, PublishError> {
    tracing::info!("Publishing a newsletter issue: {}", user_id.deref());
    let subscribers = get_confirmed_subscribers(&state.connection).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                state
                    .email_client
                    .send_email(
                        &subscriber.email,
                        &body.title,
                        &body.content_html,
                        &body.content_txt,
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
    #[derive(DerivePartialModel, FromQueryResult, Debug)]
    #[sea_orm(entity = "Subscriptions")]
    struct Row {
        email: String,
    }

    Ok(Subscriptions::find()
        .filter(subscriptions::Column::Status.contains("confirmed"))
        .into_partial_model::<Row>()
        .all(conn)
        .await?
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(error) => Err(anyhow::anyhow!(error)),
        })
        .collect())
}
