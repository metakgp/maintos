//! Router, [`handlers`], [`middleware`], state, and response utils.

use std::sync::Arc;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use bollard::Docker;
use http::{HeaderValue, Method};
use serde::Serialize;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{self, TraceLayer},
};

use crate::env::EnvVars;

mod handlers;
mod middleware;

/// Returns the Axum router for maintos
pub fn get_router(env_vars: &EnvVars, docker: Arc<Docker>) -> axum::Router {
    let state = Arc::new(RouterState {
        env_vars: env_vars.clone(),
        docker,
    });

    axum::Router::new()
        .route("/profile", axum::routing::get(handlers::profile))
        .route("/deployments", axum::routing::get(handlers::deployments))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::verify_jwt_middleware,
        ))
        .route("/oauth", axum::routing::post(handlers::oauth))
        .route("/healthcheck", axum::routing::get(handlers::healthcheck))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        .layer(
            CorsLayer::new()
                .allow_headers(Any)
                .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
                .allow_origin(
                    env_vars
                        .cors_allowed_origins
                        .split(',')
                        .map(|origin| {
                            origin
                                .trim()
                                .parse::<HeaderValue>()
                                .expect("CORS Allowed Origins Invalid")
                        })
                        .collect::<Vec<HeaderValue>>(),
                ),
        )
}

#[derive(Clone)]
/// The state of the axum router, containing the environment variables and the database connection.
struct RouterState {
    pub env_vars: EnvVars,
    pub docker: Arc<Docker>,
}

#[derive(Clone, Copy)]
/// The status of a server response
enum Status {
    Success,
    Error,
}

impl From<Status> for String {
    fn from(value: Status) -> Self {
        match value {
            Status::Success => "success".into(),
            Status::Error => "error".into(),
        }
    }
}

impl Serialize for Status {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&String::from(*self))
    }
}

/// Standard backend response format (serialized as JSON)
#[derive(serde::Serialize)]
struct BackendResponse<T: Serialize> {
    /// Whether the operation succeeded or failed
    pub status: Status,
    /// A message describing the state of the operation (success/failure message)
    pub message: String,
    /// Any optional data sent (only sent if the operation was a success)
    pub data: Option<T>,
}

impl<T: serde::Serialize> BackendResponse<T> {
    /// Creates a new success backend response with the given message and data
    pub fn ok(message: String, data: T) -> (StatusCode, Self) {
        (
            StatusCode::OK,
            Self {
                status: Status::Success,
                message,
                data: Some(data),
            },
        )
    }

    /// Creates a new error backend response with the given message, data, and an HTTP status code
    pub fn error(message: String, status_code: StatusCode) -> (StatusCode, Self) {
        (
            status_code,
            Self {
                status: Status::Error,
                message,
                data: None,
            },
        )
    }
}

impl<T: Serialize> IntoResponse for BackendResponse<T> {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

/// A struct representing the error returned by a handler. This is automatically serialized into JSON and sent as an internal server error (500) backend response. The `?` operator can be used anywhere inside a handler to do so.
pub(super) struct AppError(anyhow::Error);
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("An error occured: {}", self.0);

        BackendResponse::<()>::error(
            "An internal server error occured. Please try again later.".into(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
