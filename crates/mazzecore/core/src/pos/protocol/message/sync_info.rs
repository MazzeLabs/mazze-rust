
use crate::{
    pos::{
        consensus::network::ConsensusMsg,
        protocol::sync_protocol::{Context, Handleable},
    },
    sync::Error,
};
use consensus_types::sync_info::SyncInfo;
use diem_logger::prelude::diem_debug;
use std::mem::discriminant;

impl Handleable for SyncInfo {
    fn handle(self, ctx: &Context) -> Result<(), Error> {
        diem_debug!("on_sync_info, msg={:?}", &self);

        let peer_address = ctx.get_peer_account_address()?;

        let msg = ConsensusMsg::SyncInfo(Box::new(self));
        ctx.manager
            .consensus_network_task
            .consensus_messages_tx
            .push((peer_address, discriminant(&msg)), (peer_address, msg))?;
        Ok(())
    }
}
