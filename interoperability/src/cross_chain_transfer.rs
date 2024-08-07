
use crate::{BlockchainMessage, Bridge};
use secp256k1::{Secp256k1, SecretKey, PublicKey};
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransferError {
    #[error("Invalid transfer request")]
    InvalidRequest,
    #[error("Bridge processing error")]
    BridgeError(#[from] BridgeError),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransferRequest {
    pub from_chain: String,
    pub to_chain: String,
    pub amount: u64,
    pub sender: String,
    pub recipient: String,
}

impl TransferRequest {
    pub fn new(from_chain: &str, to_chain: &str, amount: u64, sender: &str, recipient: &str) -> Self {
        TransferRequest {
            from_chain: from_chain.to_string(),
            to_chain: to_chain.to_string(),
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

pub struct CrossChainTransfer {
    bridge: Bridge,
}

impl CrossChainTransfer {
    pub fn new(bridge: Bridge) -> Self {
        CrossChainTransfer { bridge }
    }

    pub fn initiate_transfer(&self, transfer_request: TransferRequest, secret_key: &SecretKey) -> Result<(), TransferError> {
        if transfer_request.amount == 0 {
            return Err(TransferError::InvalidRequest);
        }

        let message = transfer_request.to_message(secret_key);
        self.bridge.process_message(message)?;

        Ok(())
    }
}
