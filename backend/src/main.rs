use clap::Parser;

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

    // Server
    let listener =
        tokio::net::TcpListener::bind(format!("0.0.0.0:{}", env_vars.server_port)).await?;
    axum::serve(listener, routing::get_router(&env_vars)).await?;
    println!("Server listening on port {}", env_vars.server_port);

    Ok(())
}
