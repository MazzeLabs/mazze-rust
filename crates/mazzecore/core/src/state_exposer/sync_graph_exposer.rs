

use mazze_types::{H256, U256};
use std::mem;

pub struct SyncGraphBlockState {
    pub block_hash: H256,
    pub parent: H256,
    pub referees: Vec<H256>,
    pub nonce: U256,
    pub timestamp: u64,
    pub adaptive: bool,
}

#[derive(Default)]
/// This struct maintains some inner state of synchronization graph.
pub struct SyncGraphStates {
    pub ready_block_vec: Vec<SyncGraphBlockState>,
}

impl SyncGraphStates {
    pub fn retrieve(&mut self) -> Self {
        let mut ready_block_vec = Vec::new();
        mem::swap(&mut ready_block_vec, &mut self.ready_block_vec);
        Self { ready_block_vec }
    }
}
