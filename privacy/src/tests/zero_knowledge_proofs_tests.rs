#[cfg(test)]
mod zero_knowledge_proofs_tests {
    use super::super::zero_knowledge_proofs::{generate_proof, verify_proof};
    use super::super::confidential_transactions::{create_confidential_transaction, verify_confidential_transaction};

    #[test]
    fn test_generate_and_verify_proof() {
        let statement = "I have more than 10 coins";
        let secret = "My actual balance is 15 coins";
        
        let proof = generate_proof(&statement, &secret).expect("Failed to generate proof");
        assert!(verify_proof(&statement, &proof).is_ok());
    }

    #[test]
    fn test_generate_proof_with_invalid_statement() {
        let statement = "I have more than 20 coins";
        let secret = "My actual balance is 15 coins";
        
        let proof = generate_proof(&statement, &secret).expect("Failed to generate proof");
        assert!(verify_proof(&statement, &proof).is_err());
    }

    #[test]
    fn test_confidential_transaction_creation_and_verification() {
        let sender_balance = 100;
        let receiver_balance = 50;
        let amount = 20;

        let (confidential_tx, sender_new_balance, receiver_new_balance) = create_confidential_transaction(
            sender_balance, receiver_balance, amount
        ).expect("Failed to create confidential transaction");

        assert_eq!(sender_new_balance, 80);
        assert_eq!(receiver_new_balance, 70);
        assert!(verify_confidential_transaction(&confidential_tx).is_ok());
    }

    #[test]
    fn test_confidential_transaction_invalid_amount() {
        let sender_balance = 30;
        let receiver_balance = 50;
        let amount = 40; // Invalid amount, exceeds sender's balance

        let result = create_confidential_transaction(sender_balance, receiver_balance, amount);
        assert!(result.is_err());
    }
}
