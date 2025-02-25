

pub struct SnapshotManager<SnapshotDbManager: SnapshotDbManagerTrait> {
    // FIXME: implement a new method and remove pub on fields.
    pub snapshot_db_manager: SnapshotDbManager,
}

impl<SnapshotDbManager: SnapshotDbManagerTrait> GetSnapshotDbManager
    for SnapshotManager<SnapshotDbManager>
{
    type SnapshotDb = SnapshotDbManager::SnapshotDb;
    type SnapshotDbManager = SnapshotDbManager;

    fn get_snapshot_db_manager(&self) -> &Self::SnapshotDbManager {
        &self.snapshot_db_manager
    }
}

impl<SnapshotDbManager: SnapshotDbManagerTrait> SnapshotManagerTrait
    for SnapshotManager<SnapshotDbManager>
{
    fn remove_old_main_snapshot(
        &self, snapshot_epoch_id: &EpochId,
    ) -> Result<()> {
        debug!("remove_old_main_snapshot {:?}", snapshot_epoch_id);
        self.get_snapshot_db_manager()
            .destroy_snapshot(snapshot_epoch_id)
    }

    fn remove_non_main_snapshot(
        &self, snapshot_epoch_id: &EpochId,
    ) -> Result<()> {
        debug!("remove_non_main_snapshot {:?}", snapshot_epoch_id);
        self.get_snapshot_db_manager()
            .destroy_snapshot(snapshot_epoch_id)
    }
}

use super::super::{
    super::{
        snapshot_manager::*,
        storage_db::snapshot_db_manager::SnapshotDbManagerTrait,
    },
    errors::*,
};
use primitives::EpochId;
