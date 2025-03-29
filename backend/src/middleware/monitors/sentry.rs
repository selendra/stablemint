use axum::{body::Body, http::Request, middleware::Next, response::Response};
use sentry::{ClientInitGuard, Hub};

use crate::{config::SentryConfig, errors::AppError};
use anyhow::Context;

pub fn init_sentry() -> Result<ClientInitGuard, AppError> {
    let config = SentryConfig::from_env().context("Failed to load sentry configuration")?;
    Ok(sentry::init((
        config.sentry_dsn,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    )))
}

pub async fn sentry_middleware(req: Request<Body>, next: Next) -> Result<Response, axum::Error> {
    let hub = Hub::new_from_top(Hub::current());

    let response = next.run(req).await;

    if response.status().is_server_error() {
        hub.capture_message("Internal server error", sentry::Level::Error);
    }

    Ok(response)
}
