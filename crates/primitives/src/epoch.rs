

use mazze_types::H256;
use keccak_hash::KECCAK_EMPTY;
use serde_derive::{Deserialize, Serialize};

pub type EpochId = H256;
pub const NULL_EPOCH: EpochId = KECCAK_EMPTY;

/// Uniquely identifies epoch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EpochNumber {
    /// Epoch number within canon blockchain.
    Number(u64),
    /// Earliest block (checkpoint).
    Earliest,
    /// The latest checkpoint (cur_era_genesis)
    LatestCheckpoint,
    /// The latest finalized (confirmed by PoS) block
    LatestFinalized,
    /// The latest confirmed block (based on the estimation of the confirmation
    /// meter)
    LatestConfirmed,
    /// Latest block with state.
    LatestState,
    /// Latest mined block.
    LatestMined,
}

impl Into<EpochNumber> for u64 {
    fn into(self) -> EpochNumber { EpochNumber::Number(self) }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BlockHashOrEpochNumber {
    BlockHashWithOption {
        hash: H256,
        require_main: Option<bool>,
    },
    EpochNumber(EpochNumber),
}
