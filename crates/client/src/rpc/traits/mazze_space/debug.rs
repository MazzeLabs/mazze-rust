

use crate::rpc::types::{
    BlockHashOrEpochNumber, Bytes as RpcBytes, ConsensusGraphStates,
    EpochNumber, Receipt as RpcReceipt, RpcAddress, SendTxRequest,
    StatOnGasLoad, SyncGraphStates, Transaction as RpcTransaction,
    WrapTransaction,
};
use mazze_types::{H256, H520, U128, U64};
use mazzecore::verification::EpochReceiptProof;
use jsonrpc_core::{BoxFuture, Result as JsonRpcResult};
use jsonrpc_derive::rpc;
use network::{
    node_table::{Node, NodeId},
    throttling, SessionDetails, UpdateNodeOperation,
};
use std::collections::BTreeMap;

#[rpc(server)]
pub trait LocalRpc {
    #[rpc(name = "txpool_inspect")]
    fn txpool_inspect(
        &self, address: Option<RpcAddress>,
    ) -> JsonRpcResult<
        BTreeMap<String, BTreeMap<String, BTreeMap<usize, Vec<String>>>>,
    >;

    // return all txpool transactions grouped by hex address
    #[rpc(name = "txpool_content")]
    fn txpool_content(
        &self, address: Option<RpcAddress>,
    ) -> JsonRpcResult<
        BTreeMap<
            String,
            BTreeMap<String, BTreeMap<usize, Vec<RpcTransaction>>>,
        >,
    >;

    // return account ready + deferred transactions
    #[rpc(name = "txpool_accountTransactions")]
    fn txpool_get_account_transactions(
        &self, address: RpcAddress,
    ) -> JsonRpcResult<Vec<RpcTransaction>>;

    #[rpc(name = "txpool_clear")]
    fn txpool_clear(&self) -> JsonRpcResult<()>;

    #[rpc(name = "net_throttling")]
    fn net_throttling(&self) -> JsonRpcResult<throttling::Service>;

    #[rpc(name = "net_node")]
    fn net_node(
        &self, node_id: NodeId,
    ) -> JsonRpcResult<Option<(String, Node)>>;

    #[rpc(name = "net_disconnect_node")]
    fn net_disconnect_node(
        &self, id: NodeId, op: Option<UpdateNodeOperation>,
    ) -> JsonRpcResult<bool>;

    #[rpc(name = "net_sessions")]
    fn net_sessions(
        &self, node_id: Option<NodeId>,
    ) -> JsonRpcResult<Vec<SessionDetails>>;

    #[rpc(name = "current_sync_phase")]
    fn current_sync_phase(&self) -> JsonRpcResult<String>;

    #[rpc(name = "consensus_graph_state")]
    fn consensus_graph_state(&self) -> JsonRpcResult<ConsensusGraphStates>;

    #[rpc(name = "sync_graph_state")]
    fn sync_graph_state(&self) -> JsonRpcResult<SyncGraphStates>;

    #[rpc(name = "mazze_sendTransaction")]
    fn send_transaction(
        &self, tx: SendTxRequest, password: Option<String>,
    ) -> BoxFuture<H256>;

    /// Returns accounts list.
    #[rpc(name = "accounts")]
    fn accounts(&self) -> JsonRpcResult<Vec<RpcAddress>>;

    /// Create a new account
    #[rpc(name = "new_account")]
    fn new_account(&self, password: String) -> JsonRpcResult<RpcAddress>;

    /// Unlock an account
    #[rpc(name = "unlock_account")]
    fn unlock_account(
        &self, address: RpcAddress, password: String, duration: Option<U128>,
    ) -> JsonRpcResult<bool>;

    /// Lock an account
    #[rpc(name = "lock_account")]
    fn lock_account(&self, address: RpcAddress) -> JsonRpcResult<bool>;

    #[rpc(name = "sign")]
    fn sign(
        &self, data: RpcBytes, address: RpcAddress, password: Option<String>,
    ) -> JsonRpcResult<H520>;

    #[rpc(name = "mazze_signTransaction")]
    fn sign_transaction(
        &self, tx: SendTxRequest, password: Option<String>,
    ) -> JsonRpcResult<String>;

    #[rpc(name = "mazze_getEpochReceipts")]
    fn epoch_receipts(
        &self, epoch: BlockHashOrEpochNumber,
        include_eth_recepits: Option<bool>,
    ) -> JsonRpcResult<Option<Vec<Vec<RpcReceipt>>>>;

    #[rpc(name = "debug_statOnGasLoad")]
    fn stat_on_gas_load(
        &self, last_epoch: EpochNumber, time_window: U64,
    ) -> JsonRpcResult<Option<StatOnGasLoad>>;

    #[rpc(name = "debug_getEpochReceiptProofByTransaction")]
    fn epoch_receipt_proof_by_transaction(
        &self, tx_hash: H256,
    ) -> JsonRpcResult<Option<EpochReceiptProof>>;

    #[rpc(name = "debug_getTransactionsByEpoch")]
    fn transactions_by_epoch(
        &self, epoch_number: U64,
    ) -> JsonRpcResult<Vec<WrapTransaction>>;

    #[rpc(name = "debug_getTransactionsByBlock")]
    fn transactions_by_block(
        &self, block_hash: H256,
    ) -> JsonRpcResult<Vec<WrapTransaction>>;
}
