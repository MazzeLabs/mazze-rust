

use crate::{
    message::RequestId,
    sync::{
        message::{Context, Handleable},
        Error,
    },
};
use mazze_types::H256;
use rlp_derive::{RlpDecodable, RlpEncodable};

#[derive(Debug, PartialEq, RlpDecodable, RlpEncodable)]
pub struct GetTerminalBlockHashesResponse {
    pub request_id: RequestId,
    pub hashes: Vec<H256>,
}

impl Handleable for GetTerminalBlockHashesResponse {
    fn handle(self, ctx: &Context) -> Result<(), Error> {
        debug!("on_terminal_block_hashes_response, msg=:{:?}", self);

        ctx.match_request(self.request_id)?;

        let missing_hash = self
            .hashes
            .iter()
            .filter(|x| !ctx.manager.graph.contains_block_header(&x))
            .cloned()
            .collect::<Vec<H256>>();
        ctx.manager.request_block_headers(
            ctx.io,
            Some(ctx.node_id.clone()),
            missing_hash,
            true, /* ignore_db */
        );

        Ok(())
    }
}
