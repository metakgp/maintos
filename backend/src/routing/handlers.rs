//! All endpoint handlers and their response types.
//!
//! All endpoints accept JSON or URL query parameters as the request. The response of each handler is a [`BackendResponse`] serialized as JSON and the return type of the handler function determines the schema of the data sent in the response (if successful)
//!
//! The request format is described

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use anyhow::anyhow;
use axum::extract::State;
use axum::{Extension, extract::Json, http::StatusCode};
use envfile::EnvFile;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use tokio::fs;

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

/// Updates the given key-value pairs in the .env file
/// Returns the new env vars
pub async fn update_env(
    Extension(deployment): Extension<Deployment>,
    Json(updates): Json<HashMap<String, String>>,
) -> HandlerReturn<BTreeMap<String, String>> {
    let env_path = deployment.settings.env_path;

    let env_path = if let Some(env_path) = env_path {
        env_path
    } else {
        return Ok(BackendResponse::error(
            "Error: No .env file found.".into(),
            StatusCode::BAD_REQUEST,
        ));
    };

    let env_path = env_path.as_path();
    let env_dir = env_path.parent().ok_or(anyhow!(
        "Error: Error getting parent directory of the env file."
    ))?;

    // Take a backup of the env file
    let backup_env_path = env_dir.join(".env.bak");
    let copy_result = fs::copy(env_path, backup_env_path.as_path()).await;
    if copy_result.is_err() {
        return Ok(BackendResponse::error(
            "Error copying .env file to the backup file `.env.bak`".into(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    // Read the existing env file
    let mut env_file = EnvFile::new(env_path)?;

    // Update the values (in memory, not on disk)
    for (key, value) in updates {
        env_file.update(&key, &value);
    }

    // Attempt to write the updated env vars to disk
    if let Err(error) = env_file.write() {
        // Restore backup if failed
        let copy_result = fs::copy(backup_env_path.as_path(), env_path).await;

        return if let Err(cp_error) = copy_result {
            Ok(BackendResponse::error(
                format!(
                    "Failed to write updated env file to the disk and FAILED to restore the backup file.\nWrite error: {error}.\nBackup restore error: {cp_error}."
                ),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        } else {
            Ok(BackendResponse::error(
                format!(
                    "Failed to write updated env file to the disk and restored the backup file.\nError: {error}."
                ),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        };
    }

    Ok(BackendResponse::ok(
        "Successfully updated the env file.".into(),
        env_file.store,
    ))
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

/// Stops all containers in a deployment if the user has access to it
pub async fn stop(Extension(deployment): Extension<Deployment>) -> HandlerReturn<Value> {
    deployment.down().await?;

    Ok(BackendResponse::ok(
        "Successfully stopped containers.".into(),
        Value::Null,
    ))
}

/// Starts all containers in a deployment if the user has access to it
pub async fn start(Extension(deployment): Extension<Deployment>) -> HandlerReturn<Value> {
    deployment.up().await?;

    Ok(BackendResponse::ok(
        "Successfully started containers.".into(),
        Value::Null,
    ))
}
