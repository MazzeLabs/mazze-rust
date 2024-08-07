

/// Archive nodes and full nodes react differently for snapshot management.
pub trait SnapshotManagerTrait: GetSnapshotDbManager {
    // FIXME: add check_make_register_snapshot_background into trait

    fn get_snapshot_by_epoch_id(
        &self, epoch_id: &EpochId, try_open: bool, open_mpt_snapshot: bool,
    ) -> Result<Option<Self::SnapshotDb>> {
        self.get_snapshot_db_manager().get_snapshot_by_epoch_id(
            epoch_id,
            try_open,
            open_mpt_snapshot,
        )
    }

    fn remove_old_main_snapshot(
        &self, snapshot_epoch_id: &EpochId,
    ) -> Result<()>;

    fn remove_non_main_snapshot(
        &self, snapshot_epoch_id: &EpochId,
    ) -> Result<()>;
}

pub trait GetSnapshotDbManager {
    type SnapshotDb: SnapshotDbTrait<ValueType = Box<[u8]>>;
    type SnapshotDbManager: SnapshotDbManagerTrait<
        SnapshotDb = Self::SnapshotDb,
    >;

    fn get_snapshot_db_manager(&self) -> &Self::SnapshotDbManager;
}

use super::{
    impls::errors::*,
    storage_db::{snapshot_db::*, snapshot_db_manager::*},
};
use primitives::EpochId;
