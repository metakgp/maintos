//! Utils for Github OAuth integration and JWT authentication
//!
//! Currently this is only used in the admin dashboard and uses Github OAuth for authentication

use std::collections::BTreeMap;

use http::StatusCode;
use jwt::{Claims, RegisteredClaims, SignWithKey, VerifyWithKey};
use serde::Deserialize;

use crate::{env::EnvVars, utils::Res};

#[derive(Clone)]
/// Struct containing the auth information of a user
pub struct Auth {
    pub jwt: String,
    pub username: String,
}

/// Verifies whether a JWT is valid and signed with the secret key
///
/// Returns the username and jwt in a struct
pub async fn verify_token(token: &str, env_vars: &EnvVars) -> Res<Auth> {
    let jwt_key = env_vars.get_jwt_key()?;
    let claims: Result<Claims, _> = token.verify_with_key(&jwt_key);

    let claims = claims.map_err(|_| "Claims not found on the JWT.")?;
    let username = claims
        .private
        .get("username")
        .ok_or("Username not in the claims.")?;
    let username = username.as_str().ok_or("Username is not a string.")?;

    Ok(Auth {
        jwt: token.to_owned(),
        username: username.to_owned(),
    })
}

/// Generates a JWT with the username (for claims) and secret key
async fn generate_token(username: &str, env_vars: &EnvVars) -> Res<String> {
    let jwt_key = env_vars.get_jwt_key()?;

    let expiration = chrono::Utc::now()
        .checked_add_days(chrono::naive::Days::new(7))
        .ok_or("Error checking JWT expiration date")? // 7 Days expiration
        .timestamp()
        .unsigned_abs();

    let mut private_claims = BTreeMap::new();
    private_claims.insert(
        "username".into(),
        serde_json::Value::String(username.into()),
    );

    let claims = Claims {
        registered: RegisteredClaims {
            audience: None,
            issued_at: None,
            issuer: None,
            subject: None,
            not_before: None,
            json_web_token_id: None,
            expiration: Some(expiration),
        },
        private: private_claims,
    };

    Ok(claims.sign_with_key(&jwt_key)?)
}

#[derive(Deserialize)]
struct GithubAccessTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GithubUserResponse {
    login: String,
}

#[derive(Deserialize)]
struct GithubMembershipResponse {
    state: String,
}

/// Takes a Github OAuth code and creates a JWT authentication token for the user
/// 1. Uses the OAuth code to get an access token.
/// 2. Uses the access token to get the user's username.
/// 3. Uses the username and an admin's access token to verify whether the user is a member of the admins github team, or the admin themselves.
///
/// Returns the JWT if the user is authenticated, `None` otherwise.
pub async fn authenticate_user(code: &String, env_vars: &EnvVars) -> Res<Option<String>> {
    let client = reqwest::Client::new();

    // Get the access token for authenticating other endpoints
    let response = client
        .get(format!(
            "https://github.com/login/oauth/access_token?client_id={}&client_secret={}&code={}",
            env_vars.gh_client_id, env_vars.gh_client_secret, code
        ))
        .header("Accept", "application/json")
        .send()
        .await?;

    if response.status() != StatusCode::OK {
        return Err("Github API response error.".into());
    }

    let access_token =
        serde_json::from_slice::<GithubAccessTokenResponse>(&response.bytes().await?)?.access_token;

    // Get the username of the user who made the request
    let response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "bruh") // Why is this required :ded:
        .send()
        .await?;

    if response.status() != StatusCode::OK {
        tracing::error!(
            "Github OAuth error getting username: {}",
            response.text().await?
        );

        return Err("Github API response error.".into());
    }

    let username = serde_json::from_slice::<GithubUserResponse>(&response.bytes().await?)?.login;

    // Check the user's membership in the github org
    let response = client
        .get(format!(
            "https://api.github.com/orgs/{}/members/{}",
            env_vars.gh_org_name, username
        ))
        .header(
            "Authorization",
            format!("Bearer {}", env_vars.gh_org_admin_token),
        )
        .header("User-Agent", "bruh why is this required")
        .send()
        .await?;

    // See API: https://docs.github.com/en/rest/orgs/members?apiVersion=2022-11-28#check-organization-membership-for-a-user
    match response.status().as_u16() {
        302 => Err("Error: Github API token is from a non-organization member.".into()),
        404 => Ok(None),
        204 => Ok(Some(generate_token(&username, env_vars).await?)),
        code => {
            tracing::error!(
                "Error getting org membership data ({code}): {}",
                response.text().await?
            );
            Err("Github API response error.".into())
        }
    }
}
