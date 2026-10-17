use std::{
    env::VarError,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use crypto_api::AppState;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let trace_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()?;

    tracing_subscriber::fmt()
        .with_env_filter(trace_filter)
        .init();

    let host: IpAddr = match std::env::var("HOST") {
        Ok(host_value) => host_value.parse()?,
        Err(VarError::NotPresent) => Ipv4Addr::LOCALHOST.into(),
        Err(error) => return Err(error.into()),
    };

    let port: u16 = match std::env::var("PORT") {
        Ok(port_value) => port_value.parse()?,
        Err(VarError::NotPresent) => 3_000,
        Err(error) => return Err(error.into()),
    };

    let addr = SocketAddr::new(host, port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(addr = %listener.local_addr()?, "listening");

    axum::serve(listener, crypto_api::app(AppState::with_base64_encryptor())).await?;

    Ok(())
}
