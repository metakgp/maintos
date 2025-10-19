use anyhow::{Ok, anyhow};
use http::StatusCode;
use reqwest::Client;
use serde::Deserialize;

use crate::utils::Res;

#[derive(Deserialize)]
struct GithubAccessTokenResponse {
    access_token: String,
}
/// Fetches the access token generated from a Github OAuth request
pub async fn get_access_token(
    client: &Client,
    client_id: &str,
    client_secret: &str,
    code: &str,
) -> Res<String> {
    // get the access token for authenticating other endpoints
    let response = client
        .get(format!(
            "https://github.com/login/oauth/access_token?client_id={}&client_secret={}&code={}",
            client_id, client_secret, code
        ))
        .header("accept", "application/json")
        .send()
        .await?;

    if response.status() != StatusCode::OK {
        tracing::error!(
            "Github OAuth error getting access token: {}",
            response.text().await?
        );

        return Err(anyhow!("Github API response error."));
    }

    let access_token = response
        .json::<GithubAccessTokenResponse>()
        .await?
        .access_token;

    Ok(access_token)
}

#[derive(Deserialize)]
struct GithubUserResponse {
    login: String,
}

pub async fn get_username(client: &Client, access_token: &str) -> Res<String> {
    let response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {access_token}"))
        .header("User-Agent", "bruh") // Why is this required :ded:
        .send()
        .await?;

    if response.status() != StatusCode::OK {
        tracing::error!(
            "Github OAuth error getting username: {}",
            response.text().await?
        );

        return Err(anyhow!("Github API response error."));
    }

    let username = response.json::<GithubUserResponse>().await?.login;
    Ok(username)
}

/// Runs a Github API request authenticated with the admin access token
pub async fn admin_gh_request(
    client: &Client,
    admin_token: &str,
    path: String,
) -> Result<reqwest::Response, reqwest::Error> {
    client
        .get(format!("https://api.github.com/{path}",))
        .header("Authorization", format!("Bearer {}", admin_token))
        .header("User-Agent", "bruh why is this required")
        .send()
        .await
}

pub async fn check_membership(
    client: &Client,
    admin_token: &str,
    org: &str,
    username: &str,
) -> Res<bool> {
    let response = admin_gh_request(
        client,
        admin_token,
        format!("orgs/{}/members/{}", org, username),
    )
    .await?;

    // See API: https://docs.github.com/en/rest/orgs/members?apiVersion=2022-11-28#check-organization-membership-for-a-user
    match response.status().as_u16() {
        302 => Err(anyhow!(
            "Error: Github API token is from a non-organization member."
        )),
        404 => Ok(false),
        204 => Ok(true),
        code => {
            tracing::error!(
                "Error getting org membership data ({code}): {}",
                response.text().await?
            );
            Err(anyhow!("Github API response error."))
        }
    }
}

#[derive(Deserialize)]
struct GithubCollabResponse {
    role_name: String,
}

// See https://docs.github.com/en/rest/collaborators/collaborators?apiVersion=2022-11-28#get-repository-permissions-for-a-user
pub async fn get_collaborator_role(
    client: &Client,
    admin_token: &str,
    org: &str,
    repo: &str,
    username: &str,
) -> Res<Option<String>> {
    let response = admin_gh_request(
        client,
        admin_token,
        format!("repos/{org}/{repo}/collaborators/{username}/permission"),
    )
    .await?;

    match response.status() {
        StatusCode::OK => {
            let collab_role = response.json::<GithubCollabResponse>().await?.role_name;

            Ok(collab_role.into())
        }
        StatusCode::NOT_FOUND => Ok(None),
        _ => {
            tracing::error!(
                "Error fetching {username}'s collaborator role on {org}/{repo}: {}",
                response.text().await?
            );
            Err(anyhow!("Error fetching {username}'s collaborator role."))
        }
    }
}
