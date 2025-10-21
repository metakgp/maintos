use std::{collections::HashMap, path::PathBuf, str::FromStr};

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

/// Check if a user has permission to access a project
pub async fn check_access(env_vars: &EnvVars, username: &str, project_name: &str) -> Res<Deployment> {
    let deployments_dir = &env_vars.deployments_dir;

    let client = reqwest::Client::new();
    let repo_path = format!("{}/{}", deployments_dir.display(), project_name);
    let repo = Repository::open(repo_path)?;
    let repo_url = repo
        .find_remote("origin")?
        .url()
        .ok_or(anyhow!(
            "Error: Origin remote URL not found for repo {project_name}."
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
            return Ok(Deployment {
                name: project_name.to_string(),
                repo_url,
                repo_owner,
                repo_name,
            });
        }
    }
    Err(anyhow!("User does not have permission to access this project."))
}

/// Get a list of deployments
pub async fn get_deployments(env_vars: &EnvVars, username: &str) -> Res<Vec<Deployment>> {
    let deployments_dir = &env_vars.deployments_dir;

    let mut deployments = Vec::new();

    let mut dir_iter = fs::read_dir(deployments_dir).await?;
    while let Some(path) = dir_iter.next_entry().await? {
        if path.file_type().await?.is_dir()
        {
            let project_name = path.file_name().into_string().map_err(|_| anyhow!("Invalid project name"))?;
            if let Some(deployment) = check_access(env_vars, username, &project_name).await.ok() {
                deployments.push(deployment);
            }
        }
    }

    Ok(deployments)
}

#[derive(Deserialize, Serialize)]
/// Settings for a project
pub struct ProjectSettings {
    /// Subdirectory which is deployed (relative to the project root)
    pub deploy_dir: String,
}

/// Get the project settings (stored in .maint on the top level of the project directory)
pub async fn get_project_settings(env_vars: &EnvVars, project_name: &str) -> Res<ProjectSettings> {
    let maint_file_path = format!(
        "{}/{}/.maint",
        env_vars.deployments_dir.display(),
        project_name
    );

    if let Ok(maint_file_contents) = fs::read_to_string(maint_file_path).await {
        Ok(ProjectSettings { deploy_dir: maint_file_contents.trim().into() })
    } else {
        Ok(ProjectSettings { deploy_dir: ".".into() } )
    }
}

/// Get the environment variables for a project
pub async fn get_env(env_vars: &EnvVars, username: &str, project_name: &str) -> Res<HashMap<String, String>> {
    check_access(env_vars, username, project_name).await?;

    let project_settings = get_project_settings(env_vars, project_name).await?;

    let env_file_path = PathBuf::from(&env_vars.deployments_dir)
        .join(project_name)
        .join(&project_settings.deploy_dir)
        .join(".env");

    let map = dotenvy::from_path_iter(&env_file_path)?
        .collect::<Result<_, _>>()?;

    Ok(map)
}