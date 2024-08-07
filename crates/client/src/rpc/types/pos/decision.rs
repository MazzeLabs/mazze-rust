

use mazze_types::{H256, U64};
use diem_types::block_info::MainBlockDecision;
use serde_derive::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Decision {
    pub block_hash: H256,
    pub height: U64,
}

impl From<&MainBlockDecision> for Decision {
    fn from(pd: &MainBlockDecision) -> Self {
        Decision {
            block_hash: pd.block_hash,
            height: U64::from(pd.height),
        }
    }
}
