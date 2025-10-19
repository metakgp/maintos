//! Utils for Github OAuth integration and JWT authentication
//!
//! Currently this is only used in the admin dashboard and uses Github OAuth for authentication

use anyhow::anyhow;
use jwt::{Claims, RegisteredClaims, SignWithKey, VerifyWithKey};
use std::collections::BTreeMap;

use crate::{env::EnvVars, github, utils::Res};

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

    let claims = claims.map_err(|_| anyhow!("Claims not found on the JWT."))?;
    let username = claims
        .private
        .get("username")
        .ok_or("Username not in the claims.")
        .map_err(|_| anyhow!("Username not in the claims."))?;
    let username = username
        .as_str()
        .ok_or("Username is not a string.")
        .map_err(|_| anyhow!("Username is not a string."))?;

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
        .ok_or("Error checking JWT expiration date")
        .map_err(|_| anyhow!("Error setting JWT expiry date."))?
        .timestamp()
        .unsigned_abs(); // 7 Days expiration

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

/// Takes a Github OAuth code and creates a JWT authentication token for the user
/// 1. Uses the OAuth code to get an access token.
/// 2. Uses the access token to get the user's username.
/// 3. Uses the username and an admin's access token to verify whether the user is a member of the admins github team, or the admin themselves.
///
/// Returns the JWT if the user is authenticated, `None` otherwise.
pub async fn authenticate_user(code: &str, env_vars: &EnvVars) -> Res<Option<String>> {
    let client = reqwest::Client::new();

    // Get the access token for authenticating other endpoints
    let access_token = github::get_access_token(
        &client,
        &env_vars.gh_client_id,
        &env_vars.gh_client_secret,
        code,
    )
    .await?;

    // Get the username of the user who made the request
    let username = github::get_username(&client, &access_token).await?;

    // Check the user's membership in the github org
    let client = reqwest::Client::new();

    let is_member = github::check_membership(
        &client,
        &env_vars.gh_org_admin_token,
        &env_vars.gh_org_name,
        &username,
    )
    .await?;

    if is_member {
        Ok(Some(generate_token(&username, env_vars).await?))
    } else {
        Ok(None)
    }
}
