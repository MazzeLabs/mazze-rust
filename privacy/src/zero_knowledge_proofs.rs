use rand::Rng;
use blake2::{Blake2b, Digest};

pub struct ZkProof {
    pub value: u64,
    pub randomness: u64,
    pub hash: Vec<u8>,
}

impl ZkProof {
    pub fn new() -> Self {
        ZkProof {
            value: 0,
            randomness: 0,
            hash: vec![],
        }
    }

    pub fn generate_proof(&mut self, value: u64) -> Vec<u8> {
        self.value = value;
        self.randomness = rand::thread_rng().gen();
        let mut hasher = Blake2b::new();
        hasher.update(self.value.to_le_bytes());
        hasher.update(self.randomness.to_le_bytes());
        self.hash = hasher.finalize().to_vec();
        self.hash.clone()
    }

    pub fn verify_proof(&self, proof: &[u8]) -> bool {
        let mut hasher = Blake2b::new();
        hasher.update(self.value.to_le_bytes());
        hasher.update(self.randomness.to_le_bytes());
        hasher.finalize().to_vec() == proof
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_knowledge_proof() {
        let mut zk_proof = ZkProof::new();
        let value = 42;
        let proof = zk_proof.generate_proof(value);

        assert!(zk_proof.verify_proof(&proof));
    }
}
