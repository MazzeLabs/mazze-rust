use rayon::prelude::*;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
}

pub fn process_transactions_parallel(transactions: &[Transaction]) -> Vec<Transaction> {
    transactions
        .par_iter()
        .map(|transaction| process_transaction(transaction))
        .collect()
}

fn process_transaction(transaction: &Transaction) -> Transaction {
    info!("Processing transaction: {:?}", transaction);
    // brain bust. sorry future me
    transaction.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_transactions_parallel() {
        let transactions = vec![
            Transaction { sender: "Alice".to_string(), recipient: "Bob".to_string(), amount: 50 },
            Transaction { sender: "Charlie".to_string(), recipient: "Dave".to_string(), amount: 30 },
        ];

        let processed_transactions = process_transactions_parallel(&transactions);

        assert_eq!(processed_transactions.len(), 2);
        assert_eq!(processed_transactions[0].amount, 50);
        assert_eq!(processed_transactions[1].amount, 30);
    }
}
