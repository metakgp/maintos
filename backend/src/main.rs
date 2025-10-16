use std::sync::Arc;

use bollard::Docker;
use clap::Parser;
use tracing_subscriber::prelude::*;

use crate::utils::Res;

mod auth;
mod env;
mod routing;
mod utils;

#[tokio::main]
async fn main() -> Res<()> {
    // Read .env file if it exists
    if dotenvy::dotenv().is_ok() {
        println!("Loaded .env file.");
    }

    // Parse environment variables
    let env_vars = env::EnvVars::parse();

    // Tracing (log) config
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout));
    tracing::subscriber::set_global_default(subscriber)?;

    // Docker API connection
    let docker = Docker::connect_with_local_defaults()?;

    // Server
    let listener =
        tokio::net::TcpListener::bind(format!("0.0.0.0:{}", env_vars.server_port)).await?;
    tracing::info!("Starting server on port {}", env_vars.server_port);
    axum::serve(listener, routing::get_router(&env_vars, Arc::new(docker))).await?;

    Ok(())
}
