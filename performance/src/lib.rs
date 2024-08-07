pub mod optimization;
pub mod parallel_processing;
pub mod caching;

pub use optimization::optimize_block_processing;
pub use parallel_processing::process_transactions_parallel;
pub use caching::{cache_block, get_cached_block, invalidate_cache};
