

use mazze_types::{H256, U64};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlameInfo {
    pub blame: Option<U64>,
    pub deferred_state_root: Option<H256>,
    pub deferred_receipts_root: Option<H256>,
    pub deferred_logs_bloom_hash: Option<H256>,
}
