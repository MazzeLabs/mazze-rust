use crate::{BlockchainMessage, Bridge};
use secp256k1::{Secp256k1, SecretKey, PublicKey};
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SwapError {
    #[error("Invalid swap request")]
    InvalidRequest,
    #[error("Bridge processing error")]
    BridgeError(#[from] BridgeError),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SwapRequest {
    pub from_chain: String,
    pub to_chain: String,
    pub from_asset: String,
    pub to_asset: String,
    pub amount: u64,
    pub sender: String,
    pub recipient: String,
}

impl SwapRequest {
    pub fn new(
        from_chain: &str,
        to_chain: &str,
        from_asset: &str,
        to_asset: &str,
        amount: u64,
        sender: &str,
        recipient: &str,
    ) -> Self {
        SwapRequest {
            from_chain: from_chain.to_string(),
            to_chain: to_chain.to_string(),
            from_asset: from_asset.to_string(),
            to_asset: to_asset.to_string(),
            amount,
            sender: sender.to_string(),
            recipient: recipient.to_string(),
        }
    }

    pub fn to_message(&self, secret_key: &SecretKey) -> BlockchainMessage {
        let payload = serde_json::to_vec(self).unwrap();
        BlockchainMessage::new(&self.from_chain, &self.to_chain, payload, secret_key)
    }
}

pub struct CrossChainSwap {
    bridge: Bridge,
}

impl CrossChainSwap {
    pub fn new(bridge: Bridge) -> Self {
        CrossChainSwap { bridge }
    }

    pub fn initiate_swap(&self, swap_request: SwapRequest, secret_key: &SecretKey) -> Result<(), SwapError> {
        if swap_request.amount == 0 {
            return Err(SwapError::InvalidRequest);
        }

        let message = swap_request.to_message(secret_key);
        self.bridge.process_message(message)?;

        Ok(())
    }
}
