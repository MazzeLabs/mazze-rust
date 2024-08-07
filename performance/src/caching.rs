use cached::proc_macro::cached;
use cached::TimedCache;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use log::info;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Block {
    pub hash: String,
    pub transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
}

lazy_static::lazy_static! {
    static ref BLOCK_CACHE: Mutex<TimedCache<String, Block>> = Mutex::new(TimedCache::with_lifespan(600));
}

pub fn cache_block(block: Block) {
    let mut cache = BLOCK_CACHE.lock().unwrap();
    cache.cache_set(block.hash.clone(), block);
    info!("Block cached successfully");
}

pub fn get_cached_block(hash: &str) -> Option<Block> {
    let cache = BLOCK_CACHE.lock().unwrap();
    cache.cache_get(hash).cloned()
}

pub fn invalidate_cache(hash: &str) {
    let mut cache = BLOCK_CACHE.lock().unwrap();
    cache.cache_remove(hash);
    info!("Cache invalidated for block hash: {}", hash);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_block() {
        let transactions = vec![
            Transaction { sender: "Alice".to_string(), recipient: "Bob".to_string(), amount: 50 },
            Transaction { sender: "Charlie".to_string(), recipient: "Dave".to_string(), amount: 30 },
        ];
        let block = Block {
            hash: "abcd".to_string(),
            transactions,
        };

        cache_block(block.clone());
        let cached_block = get_cached_block(&"abcd").unwrap();

        assert_eq!(block, cached_block);
    }

    #[test]
    fn test_invalidate_cache() {
        let transactions = vec![
            Transaction { sender: "Alice".to_string(), recipient: "Bob".to_string(), amount: 50 },
            Transaction { sender: "Charlie".to_string(), recipient: "Dave".to_string(), amount: 30 },
        ];
        let block = Block {
            hash: "abcd".to_string(),
            transactions,
        };

        cache_block(block.clone());
        invalidate_cache(&"abcd");
        let cached_block = get_cached_block(&"abcd");

        assert!(cached_block.is_none());
    }
}