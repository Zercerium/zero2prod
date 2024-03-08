mod health_check;
mod home;
mod login;
mod newsletters;
mod subscriptions;
mod subscriptions_confirm;

use axum::response::{IntoResponse, Response};
use axum_macros::FromRequest;
pub use health_check::*;
pub use home::*;
pub use login::*;
pub use newsletters::*;
pub use subscriptions::*;
pub use subscriptions_confirm::*;

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(SubscribeError))]
struct AppJson<T>(T);

impl<T> IntoResponse for AppJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
