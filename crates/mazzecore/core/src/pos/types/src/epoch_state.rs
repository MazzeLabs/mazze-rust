// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0



use crate::{
    epoch_change::Verifier,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    on_chain_config::OnChainConfig,
    validator_verifier::ValidatorVerifier,
};
use anyhow::ensure;
use once_cell::sync::OnceCell;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};

pub static HARDCODED_COMMITTEE_FOR_EPOCH: OnceCell<
    BTreeMap<u64, ValidatorVerifier>,
> = OnceCell::new();

/// EpochState represents a trusted validator set to validate messages from the
/// specific epoch, it could be updated with EpochChangeProof.
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct EpochState {
    pub epoch: u64,

    verifier: ValidatorVerifier,

    pub vrf_seed: Vec<u8>,
}

impl EpochState {
    pub fn empty() -> Self {
        Self {
            epoch: 0,
            verifier: ValidatorVerifier::new(BTreeMap::new()),
            vrf_seed: vec![],
        }
    }

    pub fn new(
        epoch: u64, verifier: ValidatorVerifier, vrf_seed: Vec<u8>,
    ) -> Self {
        Self {
            epoch,
            verifier,
            vrf_seed,
        }
    }

    pub fn verifier(&self) -> &ValidatorVerifier {
        if let Some(verifier) = HARDCODED_COMMITTEE_FOR_EPOCH
            .get()
            .and_then(|m| m.get(&self.epoch))
        {
            verifier
        } else {
            &self.verifier
        }
    }
}

impl OnChainConfig for EpochState {
    const IDENTIFIER: &'static str = "DiemSystem";
}

impl Verifier for EpochState {
    fn verify(
        &self, ledger_info: &LedgerInfoWithSignatures,
    ) -> anyhow::Result<()> {
        ensure!(
            self.epoch == ledger_info.ledger_info().epoch(),
            "LedgerInfo has unexpected epoch {}, expected {}",
            ledger_info.ledger_info().epoch(),
            self.epoch
        );
        ledger_info.verify_signatures(&self.verifier())?;
        Ok(())
    }

    fn epoch_change_verification_required(&self, epoch: u64) -> bool {
        self.epoch < epoch
    }

    fn is_ledger_info_stale(&self, ledger_info: &LedgerInfo) -> bool {
        ledger_info.epoch() < self.epoch
    }
}

// this is required by structured log
impl fmt::Debug for EpochState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for EpochState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "EpochState [epoch: {}, validator: {}]",
            self.epoch,
            self.verifier()
        )
    }
}
