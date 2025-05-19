use std::str::FromStr;
use std::env;

use temporal_client::{Client, RetryClient};
use temporal_sdk::sdk_client_options;
use url::Url;

pub async fn get_client() -> Result<RetryClient<Client>, anyhow::Error> {
    // Read the Temporal server address from environment variable, with fallback
    let temporal_address = env::var("TEMPORAL_URL")
        .unwrap_or_else(|_| "http://localhost:7233".to_string());

    log::info!("🔌 Connecting to Temporal server at {}", temporal_address);

    // Parse the address as a URL
    let url = Url::from_str(&temporal_address)
        .map_err(|e| anyhow::anyhow!("Invalid TEMPORAL_URL: {}", e))?;

    let server_options = sdk_client_options(url).build()?;
    let client = server_options.connect("default", None).await?;
    log::info!("✅ Successfully connected to Temporal server at {}", temporal_address);
    Ok(client)
}