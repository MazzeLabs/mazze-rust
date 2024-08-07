use crate::zero_knowledge_proofs::ZkProof;
use crate::confidential_transactions::ConfidentialTx;

pub struct PrivacyManager {
    pub zk_proof: ZkProof,
    pub confidential_tx: ConfidentialTx,
}

impl PrivacyManager {
    pub fn new() -> Self {
        PrivacyManager {
            zk_proof: ZkProof::new(),
            confidential_tx: ConfidentialTx::new(),
        }
    }

    pub fn verify_transaction(&self, transaction: &ConfidentialTx) -> bool {
        self.zk_proof.verify_proof(&transaction.proof) && self.confidential_tx.verify(&transaction)
    }

    pub fn generate_confidential_transaction(&self, inputs: Vec<u64>, outputs: Vec<u64>) -> ConfidentialTx {
        self.confidential_tx.create_transaction(inputs, outputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_manager() {
        let manager = PrivacyManager::new();
        let inputs = vec![10, 20];
        let outputs = vec![15, 15];
        let tx = manager.generate_confidential_transaction(inputs, outputs);

        assert!(manager.verify_transaction(&tx));
    }
}
