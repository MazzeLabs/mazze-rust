

use crate::epoch_execution_commitment::EpochExecutionCommitment;
use primitives::EpochId;

pub trait StateMaintenanceTrait {
    fn get_main_hash_from_epoch_number(
        &self, epoch_number: u64,
    ) -> Result<EpochId, String>;

    fn get_epoch_execution_commitment_with_db(
        &self, block_hash: &EpochId,
    ) -> Option<EpochExecutionCommitment>;

    fn remove_epoch_execution_commitment_from_db(&self, block_hash: &EpochId);
}
