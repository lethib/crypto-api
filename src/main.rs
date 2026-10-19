use std::net::SocketAddr;
mod config;

use crypto_api::{AppState, encryption::Base64Encryptor, signing::hmac_sha256::HmacSha256Signer};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let trace_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()?;

    tracing_subscriber::fmt()
        .with_env_filter(trace_filter)
        .init();

    let config = Config::from_env()?;

    let addr = SocketAddr::new(config.host, config.port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(addr = %listener.local_addr()?, "listening");

    let app_state = AppState::new(Base64Encryptor, HmacSha256Signer::new(config.hmac_secret));

    axum::serve(listener, crypto_api::app(app_state)).await?;

    Ok(())
}
