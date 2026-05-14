//! All endpoint handlers and their response types.
//!
//! All endpoints accept JSON or URL query parameters as the request. The response of each handler is a [`BackendResponse`] serialized as JSON and the return type of the handler function determines the schema of the data sent in the response (if successful)
//!
//! The request format is described

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::{Extension, extract::Json, http::StatusCode};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::auth::{self, Auth};
use crate::{deployments::Deployment, utils::get_deployments};

use super::{AppError, BackendResponse, RouterState};

/// The return type of a handler function. T is the data type returned if the operation was a success
type HandlerReturn<T> = Result<(StatusCode, BackendResponse<T>), AppError>;

/// Type of the State in the handler arguments
type HandlerState = State<Arc<RouterState>>;

/// Healthcheck route. Returns a `Hello World.` message if healthy.
pub async fn healthcheck() -> HandlerReturn<()> {
    Ok(BackendResponse::ok("Hello, World.".into(), ()))
}

#[derive(Deserialize)]
/// The request format for the OAuth endpoint
pub struct OAuthReq {
    code: String,
}

#[derive(Serialize)]
/// The response format for the OAuth endpoint
pub struct OAuthRes {
    token: String,
}

/// Takes a Github OAuth code and returns a JWT auth token to log in a user if authorized
///
/// Request format - [`OAuthReq`]
pub async fn oauth(
    State(state): HandlerState,
    Json(body): Json<OAuthReq>,
) -> HandlerReturn<OAuthRes> {
    if let Some(token) = auth::authenticate_user(&body.code, &state.env_vars).await? {
        Ok(BackendResponse::ok(
            "Successfully authorized the user.".into(),
            OAuthRes { token },
        ))
    } else {
        Ok(BackendResponse::error(
            "Error: User unauthorized.".into(),
            StatusCode::UNAUTHORIZED,
        ))
    }
}

#[derive(Serialize)]
/// The response format for the user profile endpoint
pub struct ProfileRes {
    token: String,
    username: String,
}

/// Returns a user's profile (the JWT and username) if authorized and the token is valid. Can be used to check if the user is logged in.
pub async fn profile(Extension(auth): Extension<Auth>) -> HandlerReturn<ProfileRes> {
    Ok(BackendResponse::ok(
        "Successfully authorized the user.".into(),
        ProfileRes {
            token: auth.jwt,
            username: auth.username,
        },
    ))
}

/// Returns a list of all deployments
pub async fn deployments(State(state): HandlerState) -> HandlerReturn<Vec<Deployment>> {
    Ok(BackendResponse::ok(
        "Successfully fetched deployments".into(),
        get_deployments(&state.env_vars).await?,
    ))
}

/// Gets the environment variables for a project if the user has access to it
pub async fn get_env_vars(Extension(deployment): Extension<Deployment>) -> HandlerReturn<Value> {
    let env_vars = deployment.get_env().await?;

    if let Some(vars) = env_vars {
        Ok(BackendResponse::ok(
            "Successfully fetched environment variables.".into(),
            vars.into(),
        ))
    } else {
        Ok(BackendResponse::error(
            "`.env` does not exist.".into(),
            StatusCode::NOT_FOUND,
        ))
    }
}

/// Gets the status of all containers in a deployment if the user has access to it
pub async fn get_status(
    State(state): HandlerState,
    Extension(deployment): Extension<Deployment>,
) -> HandlerReturn<Value> {
    let container_status = deployment.get_containers_status(&state.docker).await?;

    Ok(BackendResponse::ok(
        "Successfully fetched container status.".into(),
        container_status,
    ))
}

pub async fn stop(
    State(_state): HandlerState,
    Extension(deployment): Extension<Deployment>,
) -> HandlerReturn<Value> {
    deployment.down().await?;

    Ok(BackendResponse::ok(
        "Successfully stopped containers.".into(),
        Value::Null,
    ))
}

pub async fn start(
    State(_state): HandlerState,
    Extension(deployment): Extension<Deployment>,
) -> HandlerReturn<Value> {
    deployment.up().await?;

    Ok(BackendResponse::ok(
        "Successfully started containers.".into(),
        Value::Null,
    ))
}


pub async fn get_logs(
    State(state): HandlerState,
    Path((_project_name, container)): Path<(String, String)>,
    Extension(deployment): Extension<Deployment>,
) -> HandlerReturn<Vec<String>> {
    let logs = deployment
        .get_container_logs_by_service(&state.docker, &container)
        .await?;

    Ok(BackendResponse::ok(
        "Successfully got logs for container".into(),
        logs,
    ))
}