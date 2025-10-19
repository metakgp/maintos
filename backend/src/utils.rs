use std::str::FromStr;

use anyhow::anyhow;
use git2::Repository;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{env::EnvVars, github};

pub(crate) type Res<T> = Result<T, anyhow::Error>;

#[derive(Deserialize, Serialize)]
/// All the information for a repository
pub struct Deployment {
    name: String,
    repo_url: String,
    repo_owner: String,
    repo_name: String,
}

/// Get a list of deployments
pub async fn get_deployments(env_vars: &EnvVars, username: &str) -> Res<Vec<Deployment>> {
    let deployments_dir = &env_vars.deployments_dir;

    let mut deployments = Vec::new();

    // To be reused for collaborator permission checking requests
    let client = reqwest::Client::new();

    let mut dir_iter = fs::read_dir(deployments_dir).await?;
    while let Some(path) = dir_iter.next_entry().await? {
        if path.file_type().await?.is_dir()
            && let Ok(repo) = Repository::open(path.path())
        {
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

            let parsed_url = Url::from_str(&repo_url)?;
            let mut url_paths = parsed_url
                .path_segments()
                .ok_or(anyhow!("Error parsing repository remote URL."))?;

            let repo_owner = url_paths
                .next()
                .ok_or(anyhow!(
                    "Error parsing repository remote URL: Repo owner not found."
                ))?
                .to_string();
            let repo_name = url_paths
                .next()
                .ok_or(anyhow!(
                    "Error parsing repository remote URL: Repo name not found."
                ))?
                .to_string();

            // Only include repositories owned by the organization
            if repo_owner == env_vars.gh_org_name {
                let collab_role = github::get_collaborator_role(
                    &client,
                    &env_vars.gh_org_admin_token,
                    &repo_owner,
                    &repo_name,
                    username,
                )
                .await?;

                // `None` means the user is not a collaborator
                if let Some(role) = collab_role.as_deref()
                    && (role == "maintain" || role == "admin")
                {
                    deployments.push(Deployment {
                        name,
                        repo_url,
                        repo_owner,
                        repo_name,
                    });
                }
            }
        }
    }

    Ok(deployments)
}
