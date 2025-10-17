use anyhow::anyhow;
use git2::Repository;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::env::EnvVars;

pub(crate) type Res<T> = Result<T, anyhow::Error>;

#[derive(Deserialize, Serialize)]
/// All the information for a repository
pub struct Deployment {
    name: String,
    repo_url: String,
}

/// Get a list of deployments
pub async fn get_deployments(env_vars: &EnvVars) -> Res<Vec<Deployment>> {
    let deployments_dir = &env_vars.deployments_dir;

    let mut deployments = Vec::new();

    let mut dir_iter = fs::read_dir(deployments_dir).await?;
    while let Some(path) = dir_iter.next_entry().await? {
        if path.file_type().await?.is_dir() {
            println!("{:?}", path);

            if let Ok(repo) = Repository::open(path.path()) {
                let name = path
                    .file_name()
                    .into_string()
                    .map_err(|err| anyhow!("{}", err.display()))?;

                let repo_url = repo
                    .find_remote("origin")?
                    .url()
                    .ok_or(anyhow!(
                        "Error: Origin remote URL not found for repo {name}."
                    ))?
                    .to_string();

                deployments.push(Deployment { name, repo_url });
            }
        }
    }

    Ok(deployments)
}
