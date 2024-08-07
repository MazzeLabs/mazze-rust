use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub nonce: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
}

pub fn optimize_block_processing(block: &Block) -> Block {
    let mut optimized_block = block.clone();
    optimized_block.transactions = optimize_transactions(&block.transactions);
    optimized_block
}

fn optimize_transactions(transactions: &[Transaction]) -> Vec<Transaction> {
    let mut optimized_transactions = transactions.to_vec();
    optimized_transactions.sort_by(|a, b| a.amount.cmp(&b.amount));
    optimized_transactions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimize_block_processing() {
        let transactions = vec![
            Transaction { sender: "Alice".to_string(), recipient: "Bob".to_string(), amount: 50 },
            Transaction { sender: "Charlie".to_string(), recipient: "Dave".to_string(), amount: 30 },
        ];
        let block = Block {
            transactions,
            previous_hash: "abcd".to_string(),
            nonce: 1234,
        };

        let optimized_block = optimize_block_processing(&block);

        assert_eq!(optimized_block.transactions[0].amount, 30);
        assert_eq!(optimized_block.transactions[1].amount, 50);
    }
}