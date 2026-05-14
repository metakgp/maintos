//! Middleware for the axum router

use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use http::{HeaderMap, StatusCode};
use serde::Deserialize;

use crate::{
    auth::{self, Auth},
    deployments::Deployment,
};

use super::{AppError, BackendResponse, RouterState};

/// Verifies the JWT and authenticates a user. If the JWT is invalid, the user is sent an unauthorized status code. If the JWT is valid, the authentication is added to the state.
pub async fn verify_jwt(
    State(state): State<Arc<RouterState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    if let Some(auth_header) = headers.get("Authorization") {
        if let Some(jwt) = auth_header.to_str()?.strip_prefix("Bearer ") {
            let auth = auth::verify_token(jwt, &state.env_vars).await;

            if let Ok(auth) = auth {
                // If auth is fine, add it to the request extensions
                request.extensions_mut().insert(auth);
                Ok(next.run(request).await)
            } else {
                Ok(BackendResponse::<()>::error(
                    "Authorization token invalid.".into(),
                    StatusCode::UNAUTHORIZED,
                )
                .into_response())
            }
        } else {
            Ok(BackendResponse::<()>::error(
                "Authorization header format invalid.".into(),
                StatusCode::UNAUTHORIZED,
            )
            .into_response())
        }
    } else {
        Ok(BackendResponse::<()>::error(
            "Authorization header missing.".into(),
            StatusCode::UNAUTHORIZED,
        )
        .into_response())
    }
}

// temporary fix for now (?)
#[derive(Deserialize)]
pub struct DeploymentPath {
    project_name: String,
    #[allow(unused)]
    container: Option<String>,
}

/// Checks if a user has maintainer access to a given deployment (given the user is already _authenticated_)
/// If the user has access, passes the parsed `Deployment` to the next handler.
pub async fn parse_deployment(
    State(state): State<Arc<RouterState>>,
    Extension(auth): Extension<Auth>,
    Path(path): Path<DeploymentPath>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let project_name = path.project_name;
    
    let deployment = Deployment::from_deployment_dir(&state.env_vars, &project_name).await?;
    let access = deployment.has_access(&auth).await?;

    if access {
        // If the user has access, add the deployment to the request extensions
        request.extensions_mut().insert(deployment);
        Ok(next.run(request).await)
    } else {
        Ok(BackendResponse::<()>::error(
            "Access to this project denied.".into(),
            StatusCode::UNAUTHORIZED,
        )
        .into_response())
    }
}
