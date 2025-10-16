//! ### Environment Variables
//!
//!  Each field in the struct `EnvVars` corresponds to an environment variable. The environment variable name will be in all capitals. The default values are set using the `arg()` macro of the `clap` crate. Check the source code for the defaults.

use std::path::PathBuf;

use clap::Parser;
use hmac::{Hmac, Mac, digest::InvalidLength};
use sha2::Sha256;

#[derive(Parser, Clone)]
pub struct EnvVars {
    // Auth
    #[arg(env)]
    /// OAuth app client id (public token)
    pub gh_client_id: String,
    #[arg(env)]
    /// An org admin's Github token (with the `read:org` permission)
    pub gh_org_admin_token: String,
    #[arg(env)]
    /// JWT encryption secret (make it a long, randomized string)
    jwt_secret: String,
    #[arg(env)]
    /// OAuth app client secret
    pub gh_client_secret: String,
    #[arg(env, default_value = "")]
    /// Github organization name
    pub gh_org_name: String,

    // Config
    #[arg(env, default_value = "/deployments")]
    /// Directory in which all the project deployments are stored
    pub deployments_dir: PathBuf,

    // Server
    #[arg(env, default_value = "8080")]
    /// The port the server listens on
    pub server_port: i32,

    // CORS
    #[arg(env, default_value = "https://maint.metakgp.org,http://localhost:5173")]
    /// List of origins allowed (as a list of values separated by commas `origin1, origin2`)
    pub cors_allowed_origins: String,
}

impl EnvVars {
    /// Returns the JWT signing key
    pub fn get_jwt_key(&self) -> Result<Hmac<Sha256>, InvalidLength> {
        Hmac::new_from_slice(self.jwt_secret.as_bytes())
    }
}
