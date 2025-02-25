

use crate::rpc::types::pos::NodeLockStatus;
use mazze_types::{H256, U64};
use serde_derive::Serialize;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    ///
    pub address: H256,
    ///
    pub block_number: U64,
    ///
    pub status: NodeLockStatus,
}
