//! Utils for Github OAuth integration and JWT authentication

use jsonwebtoken::{self, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::{env::EnvVars, github, utils::Res};

#[derive(Clone)]
/// Struct containing the auth information of a user
pub struct Auth {
    pub jwt: String,
    pub username: String,
    pub gh_access_token: String,
}

/// Struct containing the JWT claims
#[derive(Clone, Serialize, Deserialize)]
struct JWTClaims {
    /// The Github username of the user
    username: String,
    /// Github access token for the user
    /// WARNING: This token is stored in the claims UNENCRYPTED. For this SPECIFIC USECASE, the token is only used to check org membership and repository access.
    /// The tokens are FINE GRAINED and NO OTHER permissions are granted. DO NOT USE this in case the tokens are sensitive.
    gh_access_token: String,
    /// Issued at time
    iat: usize,
    /// Expiry
    exp: usize,
}

/// Verifies whether a JWT is valid and signed with the secret key
///
/// Returns the username and jwt in a struct
pub async fn verify_token(token: &str, env_vars: &EnvVars) -> Res<Auth> {
    let (_, jwt_key) = env_vars.get_jwt_key();
    let validation = Validation::default();

    let claims = jsonwebtoken::decode::<JWTClaims>(token, &jwt_key, &validation)?.claims;

    Ok(Auth {
        jwt: token.to_owned(),
        username: claims.username,
        gh_access_token: claims.gh_access_token,
    })
}

/// Generates a JWT with the username (for claims) and secret key
async fn generate_token(username: &str, gh_access_token: &str, env_vars: &EnvVars) -> Res<String> {
    let (jwt_key, _) = env_vars.get_jwt_key();

    let now = chrono::Utc::now();
    let expiry = (now + chrono::Duration::hours(4)).timestamp(); // Github access tokens expire in 8 hours
    let issued_at = now.timestamp();

    let claims = JWTClaims {
        iat: issued_at as usize,
        exp: expiry as usize,
        username: username.to_owned(),
        gh_access_token: gh_access_token.to_owned(),
    };

    let header = Header::default();
    Ok(jsonwebtoken::encode(&header, &claims, &jwt_key)?)
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

    let is_member =
        github::check_membership(&client, &access_token, &env_vars.gh_org_name, &username).await?;

    if is_member {
        // Generate JWT
        Ok(Some(
            generate_token(&username, &access_token, env_vars).await?,
        ))
    } else {
        Ok(None)
    }
}
