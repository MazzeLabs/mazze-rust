
pub mod bridge;
pub mod cross_chain_transfer;
pub mod cross_chain_swap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use secp256k1::{Secp256k1, SecretKey, PublicKey};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockchainMessage {
    pub from_chain: String,
    pub to_chain: String,
    pub payload: Vec<u8>,
    pub signature: Vec<u8>,
}

impl BlockchainMessage {
    pub fn new(from_chain: &str, to_chain: &str, payload: Vec<u8>, secret_key: &SecretKey) -> Self {
        let secp = Secp256k1::new();
        let message_hash = Sha256::digest(&payload);
        let signature = secp.sign(&message_hash.into(), secret_key);
        BlockchainMessage {
            from_chain: from_chain.to_string(),
            to_chain: to_chain.to_string(),
            payload,
            signature: signature.serialize_compact().to_vec(),
        }
    }

    pub fn verify(&self, public_key: &PublicKey) -> bool {
        let secp = Secp256k1::new();
        let message_hash = Sha256::digest(&self.payload);
        let signature = secp256k1::Signature::from_compact(&self.signature).unwrap();
        secp.verify(&message_hash.into(), &signature, public_key).is_ok()
    }
}
