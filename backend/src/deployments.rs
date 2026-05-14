//! Structs and functions for interacting with deployments

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::anyhow;
use bollard::{Docker, container::LogOutput, query_parameters::{ListContainersOptionsBuilder, LogsOptions}, secret::ContainerSummary};
use compose_rs::{Compose, ComposeCommand};
use futures_util::StreamExt;
use git2::Repository;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tokio::fs;
use toml::Table;

use crate::{auth::Auth, env::EnvVars, github, utils::Res};

#[derive(Deserialize, Serialize, Clone)]
/// All the information for a repository
pub struct Deployment {
    #[serde(skip)]
    /// The parsed `.maint` file
    pub settings: DeploymentSettings,

    pub deployment_dir: String,
    pub repo_url: String,
    pub repo_owner: String,
    pub repo_name: String,
}

impl Deployment {
    pub async fn from_deployment_dir(env_vars: &EnvVars, deployment_dir: &str) -> Res<Self> {
        let deployments_dir = &env_vars.deployments_dir;
        let git_path = deployments_dir.join(deployment_dir).join(".git");
        if !git_path.exists() {
            return Err(anyhow!(
                "Directory {} is not a git repository.",
                deployment_dir
            ));
        }

        let deployment_path = deployments_dir.join(deployment_dir).canonicalize()?;
        let repo = Repository::open(&deployment_path)?;

        let repo_url = repo
            .find_remote("origin")?
            .url()
            .ok_or(anyhow!(
                "Error: Origin remote URL not found for repo {deployment_dir}."
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
            .to_string()
            .replace(".git", "");

        if repo_owner.is_empty() || repo_name.is_empty() || repo_owner != env_vars.gh_org_name {
            return Err(anyhow!(
                "Repository remote URL does not match expected format or organization: {repo_url}"
            ));
        }

        // Parse the `.maint` file
        let settings = DeploymentSettings::from_deployment_path(&deployment_path).await?;

        Ok(Self {
            settings,
            deployment_dir: deployment_dir.to_owned(),
            repo_url,
            repo_owner,
            repo_name,
        })
    }

    pub async fn has_access(&self, auth: &Auth) -> Res<bool> {
        let client = reqwest::Client::new();

        let collab_role = github::get_collaborator_role(
            &client,
            &auth.gh_access_token,
            &self.repo_owner,
            &self.repo_name,
            &auth.username,
        )
        .await?;

        // `None` means the user is not a collaborator
        if let Some(role) = collab_role.as_deref()
            && (role == "maintain" || role == "admin")
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get the environment variables for a project
    pub async fn get_env(&self) -> Res<Option<Map<String, Value>>> {
        self.settings
            .env_file
            .as_ref()
            .map(|env_path| {
                // Ideally the env_path should exist as it is checked while parsing. If it doesn't the dotenv parse function should catch that error.
                let parsed_env = dotenvy::from_path_iter(env_path)?
                    .collect::<Result<Vec<(String, String)>, dotenvy::Error>>()?;

                Ok(Map::from_iter(
                    parsed_env
                        .into_iter()
                        .map(|(key, value)| (key, Value::String(value))),
                ))
            })
            .transpose()
    }

    /// Get a list of all containers in the deployment
    pub async fn get_containers(&self, docker: &Docker) -> Res<Vec<ContainerSummary>> {
        let compose_file_path = self.settings.compose_file.as_path();

        let mut filter = HashMap::new();
        filter.insert(
            "label".to_string(),
            vec![format!(
                "com.docker.compose.project.config_files={}",
                compose_file_path.to_str().unwrap()
            )],
        );

        let containers = docker
            .list_containers(Some(
                ListContainersOptionsBuilder::default()
                    .all(true)
                    .filters(&filter)
                    .build(),
            ))
            .await?;

        Ok(containers)
    }

    /// Get the status of all containers in a deployment
    pub async fn get_containers_status(&self, docker: &Docker) -> Res<Value> {
        let containers = self.get_containers(docker).await?;

        Ok(json!(
            containers
                .iter()
                .map(|container| {
                    let service = container
                        .labels
                        .as_ref()
                        .and_then(|labels| labels.get("com.docker.compose.service"))
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());

                    let state = container
                        .state
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "unknown".to_string());

                    let status = container
                        .status
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "unknown".to_string());

                    json!({
                        "container": service,
                        "state": state,
                        "status": status,
                    })
                })
                .collect::<Vec<Value>>()
        ))
    }

    /// Stop all containers in a deployment
    pub async fn down(&self) -> Res<()> {
        let compose_file = self.settings.compose_file.as_path();
        let compose = Compose::builder()
            .path(compose_file.to_str().unwrap())
            .build()
            .map_err(|e| anyhow!("Error parsing compose file: {}", e))?;

        compose.down().exec()?;

        Ok(())
    }

    /// Start all containers in a deployment
    pub async fn up(&self) -> Res<()> {
        let compose_file = self.settings.compose_file.as_path();
        let compose = Compose::builder()
            .path(compose_file.to_str().unwrap())
            .build()
            .map_err(|e| anyhow!("Error parsing compose file: {}", e))?;

        compose.up().exec()?;

        Ok(())
    }

    /// Get logs of a container
    pub async fn get_container_logs(&self, docker: &Docker, container_id: &str) -> Res<Vec<String>> {
        let options = LogsOptions {
            follow: false,
            stdout: true,
            stderr: true,
            tail: "50".to_string(),
            ..Default::default()
        };

        let mut logs = docker.logs(container_id, Some(options));
        let mut output : Vec<String> = Vec::new();

        while let Some(log) = logs.next().await {
            match log? {
                LogOutput::StdOut { message } => {
                    output.push(String::from_utf8_lossy(&message).to_string());
                }
                LogOutput::StdErr { message } => {
                    output.push(String::from_utf8_lossy(&message).to_string());
                }
                _ => {}
            }
        }

        Ok(output)
    }

    /// Get logs of a container by its compose service name
    pub async fn get_container_logs_by_service(&self, docker: &Docker, service: &str,) -> Res<Vec<String>> {
        let containers = self.get_containers(docker).await?;

        let container_id = containers
            .iter()
            .find(|container| {
                container
                    .labels
                    .as_ref()
                    .and_then(|labels| labels.get("com.docker.compose.service"))
                    .is_some_and(|label_service| label_service == service)
            })
            .and_then(|container| container.id.as_deref())
            .ok_or_else(|| anyhow!("Container not found for service: {}", service))?;

        self.get_container_logs(docker, container_id).await
    }
}

#[derive(Clone, Default)]
/// Settings for a deployment
pub struct DeploymentSettings {
    /// Path to the compose file
    pub compose_file: PathBuf,
    /// Path to the environment variables file
    pub env_file: Option<PathBuf>,
}

impl DeploymentSettings {
    /// Get the project settings (stored in .maint on the top level of the project directory)
    pub async fn from_deployment_path(deployment_path: &Path) -> Res<Self> {
        let maint_file_path = deployment_path.join(".maint");
        let raw_settings = match fs::read_to_string(maint_file_path).await {
            Ok(contents) => contents.parse::<Table>(),
            Err(_) => Ok(Table::new()),
        }?;

        let deploy_dir = Self::resolve_deploy_dir(deployment_path, &raw_settings)?;
        let compose_file = Self::resolve_compose_file(&deploy_dir, &raw_settings)?;
        let compose_file = compose_file.canonicalize()?;
        let env_file = Self::resolve_env_file(&deploy_dir, &raw_settings);
        let env_file = env_file.map(|path| path.canonicalize()).transpose()?;

        Ok(Self {
            compose_file,
            env_file,
        })
    }

    /// Resolve the deployment directory
    fn resolve_deploy_dir(deployment_path: &Path, settings: &Table) -> Res<PathBuf> {
        let deploy_dir = settings
            .get("deploy_dir")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let deploy_dir = deployment_path.join(deploy_dir);
        if deploy_dir.exists() {
            Ok(deploy_dir)
        } else {
            Err(anyhow!(
                "Deploy directory does not exist: {}",
                deploy_dir.display()
            ))
        }
    }

    /// Resolve compose file path
    fn resolve_compose_file(deploy_dir: &Path, settings: &Table) -> Res<PathBuf> {
        if let Some(compose_file) = settings.get("compose_file").and_then(|v| v.as_str()) {
            let compose_file = deploy_dir.join(compose_file);

            return if compose_file.exists() {
                Ok(compose_file)
            } else {
                Err(anyhow!(
                    "Configured compose file does not exist: {}",
                    compose_file.display()
                ))
            };
        }

        for filename in &["docker-compose.yaml", "docker-compose.yml"] {
            let path = deploy_dir.join(filename);
            if path.exists() {
                return Ok(path);
            }
        }

        Err(anyhow!(
            "No compose file found in {}.",
            deploy_dir.display()
        ))
    }

    /// Resolve environment file path
    fn resolve_env_file(deploy_dir: &Path, settings: &Table) -> Option<PathBuf> {
        if let Some(env_file) = settings.get("env_file").and_then(|v| v.as_str()) {
            let env_file = deploy_dir.join(env_file);
            if env_file.exists() {
                return Some(env_file);
            }
        }

        let default_env = deploy_dir.join(".env");
        if default_env.exists() {
            Some(default_env)
        } else {
            None
        }
    }
}
