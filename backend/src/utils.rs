//! Utility functions for the backend

use anyhow::anyhow;
use tokio::fs;

use crate::{deployments::Deployment, env::EnvVars};

pub(crate) type Res<T> = Result<T, anyhow::Error>;

/// Get a list of deployments
pub async fn get_deployments(env_vars: &EnvVars) -> Res<Vec<Deployment>> {
    let deployments_dir = &env_vars.deployments_dir;

    let mut deployments = Vec::new();

    let mut dir_iter = fs::read_dir(deployments_dir).await?;
    while let Some(path) = dir_iter.next_entry().await? {
        if path.file_type().await?.is_dir() {
            let deployment_dir = path
                .file_name()
                .into_string()
                .map_err(|_| anyhow!("Invalid project name"))?;

            if let Ok(deployment) = Deployment::from_deployment_dir(env_vars, &deployment_dir).await
            {
                deployments.push(deployment);
            }
        }
    }

    Ok(deployments)
}
