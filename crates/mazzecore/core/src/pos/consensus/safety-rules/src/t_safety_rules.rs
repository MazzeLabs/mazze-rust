// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0



use crate::{ConsensusState, Error};
use consensus_types::{
    block::Block, block_data::BlockData, timeout::Timeout, vote::Vote,
    vote_proposal::MaybeSignedVoteProposal,
};
use diem_types::{
    epoch_change::EpochChangeProof, validator_config::ConsensusSignature,
};

/// Interface for SafetyRules
pub trait TSafetyRules {
    /// Provides the internal state of SafetyRules for monitoring / debugging
    /// purposes. This does not include sensitive data like private keys.
    fn consensus_state(&mut self) -> Result<ConsensusState, Error>;

    /// Initialize SafetyRules using an Epoch ending LedgerInfo, this should map
    /// to what was provided in consensus_state. It will be used to
    /// initialize the ValidatorSet. This uses a EpochChangeProof because
    /// there's a possibility that consensus migrated to a new epoch but
    /// SafetyRules did not.
    fn initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error>;

    /// Attempts to vote for a given proposal following the voting rules.
    fn construct_and_sign_vote(
        &mut self, vote_proposal: &MaybeSignedVoteProposal,
    ) -> Result<Vote, Error>;

    /// As the holder of the private key, SafetyRules also signs proposals or
    /// blocks. A Block is a signed BlockData along with some additional
    /// metadata.
    fn sign_proposal(&mut self, block_data: BlockData) -> Result<Block, Error>;

    /// As the holder of the private key, SafetyRules also signs what is
    /// effectively a timeout message. This returns the signature for that
    /// timeout message.
    fn sign_timeout(
        &mut self, timeout: &Timeout,
    ) -> Result<ConsensusSignature, Error>;

    /// Allow the safety rule to start voting with saved secure data from
    /// another node.
    fn start_voting(&mut self, _initialize: bool) -> Result<(), Error> {
        Err(Error::SecureStorageUnexpectedError(
            "unsupported safety rule type".to_string(),
        ))
    }

    /// Stop the safety rule from voting and save secure data.
    fn stop_voting(&mut self) -> Result<(), Error> {
        Err(Error::SecureStorageUnexpectedError(
            "unsupported safety rule type".to_string(),
        ))
    }
}
