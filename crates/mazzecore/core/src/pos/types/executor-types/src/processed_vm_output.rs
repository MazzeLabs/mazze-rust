// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0



#![forbid(unsafe_code)]

use crate::{ExecutedTrees, StateComputeResult};
use diem_crypto::{hash::EventAccumulatorHasher, HashValue};
use diem_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    block_info::MainBlockDecision,
    contract_event::ContractEvent,
    epoch_state::EpochState,
    proof::accumulator::InMemoryAccumulator,
    term_state::PosState,
    transaction::{TransactionStatus, Version},
};
use std::{collections::HashMap, sync::Arc};

/// The entire set of data associated with a transaction. In addition to the
/// output generated by VM which includes the write set and events, this also
/// has the in-memory trees.
#[derive(Clone, Debug)]
pub struct TransactionData {
    /// Each entry in this map represents the new blob value of an account
    /// touched by this transaction. The blob is obtained by deserializing
    /// the previous blob into a BTreeMap, applying relevant portion of
    /// write set on the map and serializing the updated map into a
    /// new blob.
    account_blobs: HashMap<AccountAddress, AccountStateBlob>,

    /// The list of events emitted during this transaction.
    events: Vec<ContractEvent>,

    /// The execution status set by the VM.
    status: TransactionStatus,

    /// Root hash of the state tree.
    state_root_hash: HashValue,

    /// The in-memory Merkle Accumulator that has all events emitted by this
    /// transaction.
    event_tree: Arc<InMemoryAccumulator<EventAccumulatorHasher>>,

    /// The amount of gas used.
    gas_used: u64,

    /// The transaction info hash if the VM status output was keep, None
    /// otherwise
    txn_info_hash: Option<HashValue>,
}

impl TransactionData {
    pub fn new(
        account_blobs: HashMap<AccountAddress, AccountStateBlob>,
        events: Vec<ContractEvent>, status: TransactionStatus,
        state_root_hash: HashValue,
        event_tree: Arc<InMemoryAccumulator<EventAccumulatorHasher>>,
        gas_used: u64, txn_info_hash: Option<HashValue>,
    ) -> Self {
        TransactionData {
            account_blobs,
            events,
            status,
            state_root_hash,
            event_tree,
            gas_used,
            txn_info_hash,
        }
    }

    pub fn account_blobs(&self) -> &HashMap<AccountAddress, AccountStateBlob> {
        &self.account_blobs
    }

    pub fn events(&self) -> &[ContractEvent] { &self.events }

    pub fn status(&self) -> &TransactionStatus { &self.status }

    pub fn state_root_hash(&self) -> HashValue { self.state_root_hash }

    pub fn event_root_hash(&self) -> HashValue { self.event_tree.root_hash() }

    pub fn gas_used(&self) -> u64 { self.gas_used }

    pub fn txn_info_hash(&self) -> Option<HashValue> { self.txn_info_hash }
}

/// The output of Processing the vm output of a series of transactions to the
/// parent in-memory state merkle tree and accumulator.
#[derive(Debug, Clone)]
pub struct ProcessedVMOutput {
    /// The entire set of data associated with each transaction.
    transaction_data: Vec<TransactionData>,

    /// The in-memory Merkle Accumulator and state Sparse Merkle Tree after
    /// appending all the transactions in this set.
    executed_trees: ExecutedTrees,

    /// If set, this is the new epoch info that should be changed to if this
    /// block is committed.
    epoch_state: Option<EpochState>,

    /// If set, this is the selected main block in current transaction.
    main_block: Option<MainBlockDecision>,
}

impl ProcessedVMOutput {
    pub fn new(
        transaction_data: Vec<TransactionData>, executed_trees: ExecutedTrees,
        epoch_state: Option<EpochState>,
        main_block: Option<MainBlockDecision>,
    ) -> Self {
        ProcessedVMOutput {
            transaction_data,
            executed_trees,
            epoch_state,
            main_block,
        }
    }

    pub fn transaction_data(&self) -> &[TransactionData] {
        &self.transaction_data
    }

    pub fn executed_trees(&self) -> &ExecutedTrees { &self.executed_trees }

    pub fn accu_root(&self) -> HashValue { self.executed_trees().state_id() }

    pub fn version(&self) -> Option<Version> { self.executed_trees().version() }

    pub fn epoch_state(&self) -> &Option<EpochState> { &self.epoch_state }

    pub fn main_block(&self) -> &Option<MainBlockDecision> {
        &self.main_block
    }

    pub fn has_reconfiguration(&self) -> bool { self.epoch_state.is_some() }

    pub fn compute_result(
        &self, parent_frozen_subtree_roots: Vec<HashValue>,
        parent_num_leaves: u64,
    ) -> StateComputeResult {
        let txn_accu = self.executed_trees().txn_accumulator();
        // Now that we have the root hash and execution status we can send the
        // response to consensus.
        // TODO: The VM will support a special transaction to set the validators
        // for the next epoch that is part of a block execution.
        StateComputeResult::new(
            if parent_num_leaves == 0 {
                self.accu_root()
            } else {
                Default::default()
            },
            txn_accu.frozen_subtree_roots().clone(),
            txn_accu.num_leaves(),
            parent_frozen_subtree_roots,
            parent_num_leaves,
            self.epoch_state.clone(),
            self.transaction_data()
                .iter()
                .map(|txn_data| txn_data.status())
                .cloned()
                .collect(),
            self.transaction_data()
                .iter()
                .filter_map(|x| x.txn_info_hash())
                .collect(),
            self.main_block().clone(),
        )
    }

    pub fn replace_pos_state(&mut self, new_pos_state: PosState) {
        self.executed_trees.pos_state = new_pos_state;
    }

    pub fn set_pos_state_skipped(&mut self) {
        self.executed_trees.set_pos_state_skipped(true);
    }
}
