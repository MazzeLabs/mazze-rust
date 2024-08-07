use super::super::confidential_transactions::{ConfidentialTransaction, generate_transaction, verify_transaction};
use rstest::rstest;

#[rstest]
fn test_generate_transaction() {
    // Test if a confidential transaction can be generated successfully
    let sender = "sender_address";
    let receiver = "receiver_address";
    let amount = 100;
    let tx = generate_transaction(sender, receiver, amount);

    assert_eq!(tx.sender, sender);
    assert_eq!(tx.receiver, receiver);
    assert_eq!(tx.amount, amount);
    assert!(tx.proof.is_some());
}

#[rstest]
fn test_verify_transaction_valid() {
    // Test if a valid transaction can be verified successfully
    let sender = "sender_address";
    let receiver = "receiver_address";
    let amount = 100;
    let tx = generate_transaction(sender, receiver, amount);

    let result = verify_transaction(&tx);
    assert!(result.is_ok());
}

#[rstest]
fn test_verify_transaction_invalid() {
    // Test if an invalid transaction is rejected
    let tx = ConfidentialTransaction {
        sender: "sender_address".to_string(),
        receiver: "receiver_address".to_string(),
        amount: 100,
        proof: None,  // Invalid because proof is missing
    };

    let result = verify_transaction(&tx);
    assert!(result.is_err());
}
