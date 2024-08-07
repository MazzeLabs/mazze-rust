

use super::Decision;
use mazze_types::{H256, U64};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    ///
    pub hash: H256,
    ///
    pub height: U64,
    ///
    pub epoch: U64,
    ///
    pub round: U64,
    ///
    pub last_tx_number: U64,
    ///
    pub miner: Option<H256>,
    ///
    pub parent_hash: H256,
    ///
    pub timestamp: U64,
    ///
    pub main_decision: Option<Decision>,
    ///
    pub signatures: Vec<Signature>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    ///
    pub account: H256,
    ///
    // pub signature: String,
    ///
    pub votes: U64,
}
