
use crate::BlockchainMessage;
use secp256k1::{Secp256k1, SecretKey, PublicKey};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Unsupported chain")]
    UnsupportedChain,
    #[error("Failed to process message")]
    ProcessingError,
}

pub struct Bridge {
    supported_chains: Vec<String>,
    public_keys: HashMap<String, PublicKey>,
}

impl Bridge {
    pub fn new(supported_chains: Vec<String>, public_keys: HashMap<String, PublicKey>) -> Self {
        Bridge {
            supported_chains,
            public_keys,
        }
    }

    pub fn process_message(&self, message: BlockchainMessage) -> Result<(), BridgeError> {
        if !self.supported_chains.contains(&message.to_chain) {
            return Err(BridgeError::UnsupportedChain);
        }

        let public_key = self
            .public_keys
            .get(&message.from_chain)
            .ok_or(BridgeError::UnsupportedChain)?;

        if !message.verify(public_key) {
            return Err(BridgeError::InvalidSignature);
        }

        // Process the message payload 
        self.forward_message(message)?;

        Ok(())
    }

    fn forward_message(&self, message: BlockchainMessage) -> Result<(), BridgeError> {
        // To do: Implement the logic to forward the message to the destination chain
        // ??? Sending the message to a specific node or API endpoint
        Ok(())
    }
}
