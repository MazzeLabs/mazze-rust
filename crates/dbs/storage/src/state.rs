

/// A block defines a list of transactions that it sees and the sequence of
/// the transactions (ledger). At the view of a block, after all
/// transactions being executed, the data associated with all addresses is
/// a State after the epoch defined by the block.
///
/// A writable state is copy-on-write reference to the base state in the
/// state manager. State is supposed to be owned by single user.
pub use super::impls::state::State;
use mazze_types::AddressWithSpace;

pub type WithProof = primitives::static_bool::Yes;
pub type NoProof = primitives::static_bool::No;

// The trait is created to separate the implementation to another file, and the
// concrete struct is put into inner mod, because the implementation is
// anticipated to be too complex to present in the same file of the API.
pub trait StateTrait: Sync + Send {
    // Actions.
    fn get(&self, access_key: StorageKeyWithSpace)
        -> Result<Option<Box<[u8]>>>;
    fn set(
        &mut self, access_key: StorageKeyWithSpace, value: Box<[u8]>,
    ) -> Result<()>;
    fn delete(&mut self, access_key: StorageKeyWithSpace) -> Result<()>;
    fn delete_test_only(
        &mut self, access_key: StorageKeyWithSpace,
    ) -> Result<Option<Box<[u8]>>>;
    // Delete everything prefixed by access_key and return deleted key value
    // pairs.
    fn delete_all(
        &mut self, access_key_prefix: StorageKeyWithSpace,
    ) -> Result<Option<Vec<MptKeyValue>>>;
    // TODO: Remove this mut.
    fn read_all(
        &mut self, access_key_prefix: StorageKeyWithSpace,
    ) -> Result<Option<Vec<MptKeyValue>>>;

    // Finalize
    /// It's costly to compute state root however it's only necessary to compute
    /// state root once before committing.
    fn compute_state_root(&mut self) -> Result<StateRootWithAuxInfo>;
    fn get_state_root(&self) -> Result<StateRootWithAuxInfo>;
    fn commit(&mut self, epoch: EpochId) -> Result<StateRootWithAuxInfo>;
}

pub trait StateTraitExt {
    fn get_with_proof(
        &self, access_key: StorageKeyWithSpace,
    ) -> Result<(Option<Box<[u8]>>, StateProof)>;

    /// Compute the merkle of the node under `access_key` in all tries.
    /// Node merkle is computed on the value and children hashes, ignoring the
    /// compressed path.
    fn get_node_merkle_all_versions<WithProof: StaticBool>(
        &self, access_key: StorageKeyWithSpace,
    ) -> Result<(NodeMerkleTriplet, NodeMerkleProof)>;
}

// We skip the accessed_entries for getting original value.
pub trait StateDbGetOriginalMethods {
    fn get_original_raw_with_proof(
        &self, key: StorageKeyWithSpace,
    ) -> Result<(Option<Box<[u8]>>, StateProof)>;

    fn get_original_storage_root(
        &self, address: &AddressWithSpace,
    ) -> Result<StorageRoot>;

    fn get_original_storage_root_with_proof(
        &self, address: &AddressWithSpace,
    ) -> Result<(StorageRoot, StorageRootProof)>;
}

use super::{
    impls::{
        errors::*, node_merkle_proof::NodeMerkleProof, state_proof::StateProof,
    },
    MptKeyValue, StateRootWithAuxInfo,
};
use crate::StorageRootProof;
use primitives::{
    EpochId, NodeMerkleTriplet, StaticBool, StorageKeyWithSpace, StorageRoot,
};
