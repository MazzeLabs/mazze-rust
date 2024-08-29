
use log::{error, info};
use mazze_types::H256;
use std::str::FromStr;
use tokio;

mod stratum_client;
use stratum_client::StratumClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    info!("Starting Stratum client");

    let addr = "0.0.0.0:32525"; // Replace with your actual server address
    let stratum_secret = "test";
    let mut client = match StratumClient::connect(addr, stratum_secret).await {
        Ok(client) => {
            info!("Connected to server successfully");
            client
        }
        Err(e) => {
            error!("Failed to connect to server: {:?}", e);
            return Err(e);
        }
    };

    if let Err(e) = client.run().await {
        error!("Error during client execution: {:?}", e);
        return Err(e);
    }

    Ok(())
}