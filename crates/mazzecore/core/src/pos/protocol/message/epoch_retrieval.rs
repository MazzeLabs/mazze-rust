

use crate::{
    pos::{
        consensus::network::ConsensusMsg,
        protocol::sync_protocol::{Context, Handleable},
    },
    sync::Error,
};
use consensus_types::epoch_retrieval::EpochRetrievalRequest;
use diem_logger::prelude::diem_debug;
use std::mem::discriminant;

impl Handleable for EpochRetrievalRequest {
    fn handle(self, ctx: &Context) -> Result<(), Error> {
        let peer_address = ctx.get_peer_account_address()?;
        diem_debug!(
            "Received epoch retrieval from peer {}, start epoch {}, end epoch {}",
            peer_address, self.start_epoch, self.end_epoch
        );
        let msg = ConsensusMsg::EpochRetrievalRequest(Box::new(self));
        ctx.manager
            .consensus_network_task
            .consensus_messages_tx
            .push((peer_address, discriminant(&msg)), (peer_address, msg))?;
        Ok(())
    }
}
