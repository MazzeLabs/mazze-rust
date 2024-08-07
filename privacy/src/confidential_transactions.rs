use crate::zero_knowledge_proofs::ZkProof;

pub struct ConfidentialTx {
    pub inputs: Vec<u64>,
    pub outputs: Vec<u64>,
    pub proof: Vec<u8>,
}

impl ConfidentialTx {
    pub fn new() -> Self {
        ConfidentialTx {
            inputs: vec![],
            outputs: vec![],
            proof: vec![],
        }
    }

    pub fn create_transaction(&self, inputs: Vec<u64>, outputs: Vec<u64>) -> ConfidentialTx {
        let mut zk_proof = ZkProof::new();
        let input_sum: u64 = inputs.iter().sum();
        let output_sum: u64 = outputs.iter().sum();

        assert_eq!(input_sum, output_sum, "Inputs and outputs must balance");

        let proof = zk_proof.generate_proof(input_sum);

        ConfidentialTx {
            inputs,
            outputs,
            proof,
        }
    }

    pub fn verify(&self, transaction: &ConfidentialTx) -> bool {
        let input_sum: u64 = transaction.inputs.iter().sum();
        let output_sum: u64 = transaction.outputs.iter().sum();

        input_sum == output_sum && ZkProof::new().verify_proof(&transaction.proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidential_transaction() {
        let tx_manager = ConfidentialTx::new();
        let inputs = vec![30, 70];
        let outputs = vec![50, 50];
        let tx = tx_manager.create_transaction(inputs, outputs);

        assert!(tx_manager.verify(&tx));
    }

    #[test]
    #[should_panic(expected = "Inputs and outputs must balance")]
    fn test_unbalanced_transaction() {
        let tx_manager = ConfidentialTx::new();
        let inputs = vec![30, 70];
        let outputs = vec![60, 50];
        tx_manager.create_transaction(inputs, outputs);
    }
}
