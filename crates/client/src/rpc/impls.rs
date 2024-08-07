

#[derive(Clone, Default)]
pub struct RpcImplConfiguration {
    pub get_logs_filter_max_limit: Option<usize>,
    /// If it's `true`, `DEFERRED_STATE_EPOCH_COUNT` blocks are generated after
    /// receiving a new tx through RPC calling to pack and execute this
    /// transaction.
    pub dev_pack_tx_immediately: bool,

    // maximum response payload size allowed
    // note: currently we only handle this for `mazze_getEpochReceipts`,
    // other APIs will disconnect on oversized response
    pub max_payload_bytes: usize,

    pub enable_metrics: bool,

    pub poll_lifetime_in_seconds: Option<u32>,
}

pub mod mazze;
pub mod eth;
pub mod pos;
pub mod trace;

pub use mazze::{mazze_filter, common, light, pool, pubsub};
pub use eth::{debug, eth_filter, eth_handler::EthHandler, eth_pubsub};
