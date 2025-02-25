

#[macro_use]
pub(super) mod merkle_patricia_trie;
pub(super) mod delta_mpt;
pub(super) mod node_merkle_proof;
pub(super) mod proof_merger;
pub(super) mod recording_storage;
pub(super) mod replicated_state;
pub(super) mod single_mpt_state;
pub(super) mod snapshot_sync;
pub(super) mod state;
pub(super) mod state_manager;
pub(super) mod state_proof;
pub(super) mod storage_db;
pub(super) mod storage_manager;

pub(super) use mazze_db_errors::storage as errors;

pub mod defaults {
    pub use super::delta_mpt::DEFAULT_NODE_MAP_SIZE;
    // By default do not check for data-integrity for snapshot mpt for
    // production runs.
    pub const DEFAULT_DEBUG_SNAPSHOT_CHECKER_THREADS: u16 = 0;
    pub const DEFAULT_DELTA_MPTS_CACHE_RECENT_LFU_FACTOR: f64 =
        DeltaMptsNodeMemoryManager::R_LFU_FACTOR;
    pub const DEFAULT_DELTA_MPTS_CACHE_SIZE: u32 =
        DeltaMptsNodeMemoryManager::MAX_CACHED_TRIE_NODES_DISK_HYBRID;
    pub const DEFAULT_DELTA_MPTS_CACHE_START_SIZE: u32 =
        DeltaMptsNodeMemoryManager::START_CAPACITY;
    pub const DEFAULT_DELTA_MPTS_SLAB_IDLE_SIZE: u32 =
        DeltaMptsNodeMemoryManager::MAX_DIRTY_AND_TEMPORARY_TRIE_NODES;
    pub const DEFAULT_EXECUTION_PREFETCH_THREADS: usize = 4;
    /// Limit the number of open snapshots to set an upper limit on open files
    /// in Storage subsystem.
    pub const DEFAULT_MAX_OPEN_SNAPSHOTS: u16 = 10;
    pub const MAX_CACHED_TRIE_NODES_R_LFU_COUNTER: u32 =
        DeltaMptsNodeMemoryManager::MAX_CACHED_TRIE_NODES_R_LFU_COUNTER;

    /// The max number of opened MPT databases at the same time.
    /// Accessing a state currently involves both the intermediate MPT and delta
    /// MPT, so setting this to 4 allows to access two states at the same
    /// time.
    pub const DEFAULT_MAX_OPEN_MPT: u32 = 4;

    use super::delta_mpt::node_memory_manager::DeltaMptsNodeMemoryManager;
}
