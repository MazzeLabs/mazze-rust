

use crate::rpc::{
    error_codes::{
        call_execution_error, internal_error, invalid_params,
        request_rejected_in_catch_up_mode, unknown_block,
    },
    impls::RpcImplConfiguration,
    traits::eth_space::eth::Eth,
    types::{
        eth::{
            AccountPendingTransactions, Block as RpcBlock, BlockNumber,
            CallRequest, EthRpcLogFilter, Log, Receipt, SyncInfo, SyncStatus,
            Transaction,
        },
        Bytes, FeeHistory, Index, MAX_GAS_CALL_REQUEST, U64 as HexU64,
    },
};
use mazze_execute_helper::estimation::{
    decode_error, EstimateExt, EstimateRequest,
};
use mazze_executor::executive::{
    revert_reason_decode, ExecutionError, ExecutionOutcome, TxDropError,
};
use mazze_parameters::rpc::GAS_PRICE_DEFAULT_VALUE;
use mazze_statedb::StateDbExt;
use mazze_types::{
    Address, AddressSpaceUtil, BigEndianHash, Space, H160, H256, U256, U64,
};
use mazze_vm_types::Error as VmError;
use mazzecore::{
    consensus::PhantomBlock,
    rpc_errors::{
        invalid_params_check, Error as MazzeRpcError, Result as MazzeRpcResult,
    },
    ConsensusGraph, ConsensusGraphTrait, SharedConsensusGraph,
    SharedSynchronizationService, SharedTransactionPool,
};
use clap::crate_version;
use jsonrpc_core::{Error as RpcError, Result as RpcResult};
use primitives::{
    filter::LogFilter,
    receipt::EVM_SPACE_SUCCESS,
    transaction::{
        Eip1559Transaction, Eip155Transaction, Eip2930Transaction,
        EthereumTransaction::*, EIP1559_TYPE, EIP2930_TYPE, LEGACY_TX_TYPE,
    },
    Action, BlockHashOrEpochNumber, EpochNumber, SignedTransaction, StorageKey,
    StorageValue, TransactionStatus, TransactionWithSignature,
};
use rustc_hex::ToHex;
use std::{cmp::min, convert::TryInto};

pub struct EthHandler {
    config: RpcImplConfiguration,
    consensus: SharedConsensusGraph,
    sync: SharedSynchronizationService,
    tx_pool: SharedTransactionPool,
}

impl EthHandler {
    pub fn new(
        config: RpcImplConfiguration, consensus: SharedConsensusGraph,
        sync: SharedSynchronizationService, tx_pool: SharedTransactionPool,
    ) -> Self {
        EthHandler {
            config,
            consensus,
            sync,
            tx_pool,
        }
    }

    fn consensus_graph(&self) -> &ConsensusGraph {
        self.consensus
            .as_any()
            .downcast_ref::<ConsensusGraph>()
            .expect("downcast should succeed")
    }
}

pub fn sign_call(
    chain_id: u32, request: CallRequest,
) -> RpcResult<SignedTransaction> {
    let max_gas = U256::from(MAX_GAS_CALL_REQUEST);
    let gas = min(request.gas.unwrap_or(max_gas), max_gas);
    let nonce = request.nonce.unwrap_or_default();
    let action = request.to.map_or(Action::Create, |addr| Action::Call(addr));
    let value = request.value.unwrap_or_default();

    let default_type_id = if request.max_fee_per_gas.is_some()
        || request.max_priority_fee_per_gas.is_some()
    {
        EIP1559_TYPE
    } else if request.access_list.is_some() {
        EIP2930_TYPE
    } else {
        LEGACY_TX_TYPE
    };
    let transaction_type = request
        .transaction_type
        .unwrap_or(U64::from(default_type_id));

    let gas_price = request.gas_price.unwrap_or(1.into());
    let max_fee_per_gas = request
        .max_fee_per_gas
        .or(request.max_priority_fee_per_gas)
        .unwrap_or(gas_price);
    let max_priority_fee_per_gas =
        request.max_priority_fee_per_gas.unwrap_or(U256::zero());
    let access_list = request.access_list.unwrap_or(vec![]);
    let data = request.data.unwrap_or_default().into_vec();

    let transaction = match transaction_type.as_usize() as u8 {
        LEGACY_TX_TYPE => Eip155(Eip155Transaction {
            nonce,
            gas_price,
            gas,
            action,
            value,
            chain_id: Some(chain_id),
            data,
        }),
        EIP2930_TYPE => Eip2930(Eip2930Transaction {
            chain_id,
            nonce,
            gas_price,
            gas,
            action,
            value,
            data,
            access_list,
        }),
        EIP1559_TYPE => Eip1559(Eip1559Transaction {
            chain_id,
            nonce,
            max_priority_fee_per_gas,
            max_fee_per_gas,
            gas,
            action,
            value,
            data,
            access_list,
        }),
        x => {
            return Err(
                invalid_params("Unrecognized transaction type", x).into()
            );
        }
    };

    let from = request.from.unwrap_or(Address::zero());

    Ok(transaction.fake_sign_rpc(from.with_evm_space()))
}

fn block_tx_by_index(
    phantom_block: Option<PhantomBlock>, idx: usize,
) -> Option<Transaction> {
    match phantom_block {
        None => None,
        Some(pb) => match pb.transactions.get(idx) {
            None => None,
            Some(tx) => {
                let block_number = Some(pb.main_header.height().into());
                let receipt = pb.receipts.get(idx).unwrap();
                let status = receipt.outcome_status.in_space(Space::Ethereum);
                let contract_address = match status == EVM_SPACE_SUCCESS {
                    true => Transaction::deployed_contract_address(&tx),
                    false => None,
                };
                Some(Transaction::from_signed(
                    &tx,
                    (
                        Some(pb.main_header.hash()),
                        block_number,
                        Some(idx.into()),
                    ),
                    (Some(status.into()), contract_address),
                ))
            }
        },
    }
}

impl EthHandler {
    fn exec_transaction(
        &self, mut request: CallRequest,
        block_number_or_hash: Option<BlockNumber>,
    ) -> MazzeRpcResult<(ExecutionOutcome, EstimateExt)> {
        let consensus_graph = self.consensus_graph();

        let epoch = match block_number_or_hash.unwrap_or_default() {
            BlockNumber::Hash { hash, .. } => {
                match consensus_graph.get_block_epoch_number(&hash) {
                    Some(e) => {
                        // do not expose non-main blocks in eth RPC
                        let main = consensus_graph
                            .get_block_hashes_by_epoch(EpochNumber::Number(e))?
                            .last()
                            .cloned();

                        if Some(hash) != main {
                            bail!("Block {:?} not found", hash);
                        }

                        EpochNumber::Number(e)
                    }
                    None => bail!("Block {:?} not found", hash),
                }
            }
            epoch => epoch.try_into()?,
        };

        // if gas_price is zero, it is considered as not set
        request.unset_zero_gas_price();

        let estimate_request = EstimateRequest {
            has_sender: request.from.is_some(),
            has_gas_limit: request.gas.is_some(),
            has_gas_price: request.gas_price.is_some(),
            has_nonce: request.nonce.is_some(),
            has_storage_limit: false,
        };

        let chain_id = self.consensus.best_chain_id();
        let signed_tx = sign_call(chain_id.in_evm_space(), request)?;

        trace!("call tx {:?}, request {:?}", signed_tx, estimate_request);
        consensus_graph.call_virtual(&signed_tx, epoch, estimate_request)
    }

    fn send_transaction_with_signature(
        &self, tx: TransactionWithSignature,
    ) -> MazzeRpcResult<H256> {
        if self.sync.catch_up_mode() {
            warn!("Ignore send_transaction request {}. Cannot send transaction when the node is still in catch-up mode.", tx.hash());
            bail!(request_rejected_in_catch_up_mode(None));
        }
        let (signed_trans, failed_trans) =
            self.tx_pool.insert_new_transactions(vec![tx]);
        // FIXME: how is it possible?
        if signed_trans.len() + failed_trans.len() > 1 {
            // This should never happen
            error!("insert_new_transactions failed, invalid length of returned result vector {}", signed_trans.len() + failed_trans.len());
            Ok(H256::zero().into())
        } else if signed_trans.len() + failed_trans.len() == 0 {
            // For tx in transactions_pubkey_cache, we simply ignore them
            debug!("insert_new_transactions ignores inserted transactions");
            // FIXME: this is not invalid params
            bail!(invalid_params("tx", String::from("tx already exist")))
        } else if signed_trans.is_empty() {
            let tx_err = failed_trans.iter().next().expect("Not empty").1;
            // FIXME: this is not invalid params
            bail!(invalid_params("tx", tx_err))
        } else {
            let tx_hash = signed_trans[0].hash();
            self.sync.append_received_transactions(signed_trans);
            Ok(tx_hash.into())
        }
    }

    fn construct_rpc_receipt(
        &self, b: &PhantomBlock, idx: usize, prior_log_index: &mut usize,
    ) -> jsonrpc_core::Result<Receipt> {
        if b.transactions.len() != b.receipts.len() {
            return Err(internal_error(
                "Inconsistent state: transactions and receipts length mismatch",
            ));
        }

        if b.transactions.len() != b.errors.len() {
            return Err(internal_error(
                "Inconsistent state: transactions and errors length mismatch",
            ));
        }

        if idx >= b.transactions.len() {
            return Err(internal_error(
                "Inconsistent state: tx index out of bound",
            ));
        }

        let tx = &b.transactions[idx];
        let receipt = &b.receipts[idx];

        if receipt.logs.iter().any(|l| l.space != Space::Ethereum) {
            return Err(internal_error(
                "Inconsistent state: native tx in phantom block",
            ));
        }

        let contract_address = match receipt.outcome_status {
            TransactionStatus::Success => {
                Transaction::deployed_contract_address(tx)
            }
            _ => None,
        };

        let transaction_hash = tx.hash();
        let transaction_index: U256 = idx.into();
        let block_hash = b.main_header.hash();
        let block_height: U256 = b.main_header.height().into();

        let logs: Vec<_> = receipt
            .logs
            .iter()
            .cloned()
            .enumerate()
            .map(|(idx, log)| Log {
                address: log.address,
                topics: log.topics,
                data: Bytes(log.data),
                block_hash,
                block_number: block_height,
                transaction_hash,
                transaction_index,
                log_index: Some((*prior_log_index + idx).into()),
                transaction_log_index: Some(idx.into()),
                removed: false,
            })
            .collect();

        *prior_log_index += logs.len();

        let gas_used = match idx {
            0 => receipt.accumulated_gas_used,
            idx => {
                receipt.accumulated_gas_used
                    - b.receipts[idx - 1].accumulated_gas_used
            }
        };

        let tx_exec_error_msg = if b.errors[idx].is_empty() {
            None
        } else {
            Some(b.errors[idx].clone())
        };

        let effective_gas_price =
            if let Some(base_price) = b.main_header.base_price() {
                let base_price = base_price[tx.space()];
                if *tx.gas_price() < base_price {
                    *tx.gas_price()
                } else {
                    tx.effective_gas_price(&base_price)
                }
            } else {
                *tx.gas_price()
            };

        Ok(Receipt {
            transaction_hash,
            transaction_index,
            block_hash,
            from: tx.sender().address,
            to: match tx.action() {
                Action::Create => None,
                Action::Call(addr) => Some(*addr),
            },
            block_number: block_height,
            cumulative_gas_used: receipt.accumulated_gas_used,
            gas_used,
            contract_address,
            logs,
            logs_bloom: receipt.log_bloom,
            status_code: receipt
                .outcome_status
                .in_space(Space::Ethereum)
                .into(),
            effective_gas_price,
            tx_exec_error_msg,
            transaction_type: receipt
                .burnt_gas_fee
                .is_some()
                .then_some(U64::from(tx.type_id())),
            burnt_gas_fee: receipt.burnt_gas_fee,
        })
    }

    fn get_tx_from_txpool(&self, hash: H256) -> Option<Transaction> {
        let tx = self.tx_pool.get_transaction(&hash)?;

        if tx.space() == Space::Ethereum {
            Some(Transaction::from_signed(
                &tx,
                (None, None, None),
                (None, None),
            ))
        } else {
            None
        }
    }
}

impl Eth for EthHandler {
    fn client_version(&self) -> jsonrpc_core::Result<String> {
        info!("RPC Request: web3_clientVersion");
        Ok(parity_version::version(crate_version!()))
    }

    fn net_version(&self) -> jsonrpc_core::Result<String> {
        info!("RPC Request: net_version");
        Ok(format!("{}", self.consensus.best_chain_id().in_evm_space()))
    }

    fn protocol_version(&self) -> jsonrpc_core::Result<String> {
        info!("RPC Request: eth_protocolVersion");
        // 65 is a common ETH version now
        Ok(format!("{}", 65))
    }

    fn syncing(&self) -> jsonrpc_core::Result<SyncStatus> {
        info!("RPC Request: eth_syncing");
        if self.sync.catch_up_mode() {
            Ok(
                // Now pass some statistics of Mazze just to make the
                // interface happy
                SyncStatus::Info(SyncInfo {
                    starting_block: U256::from(self.consensus.block_count()),
                    current_block: U256::from(self.consensus.block_count()),
                    highest_block: U256::from(
                        self.sync.get_synchronization_graph().block_count(),
                    ),
                    warp_chunks_amount: None,
                    warp_chunks_processed: None,
                }),
            )
        } else {
            Ok(SyncStatus::None)
        }
    }

    fn hashrate(&self) -> jsonrpc_core::Result<U256> {
        info!("RPC Request: eth_hashrate");
        // We do not mine
        Ok(U256::zero())
    }

    fn author(&self) -> jsonrpc_core::Result<H160> {
        info!("RPC Request: eth_coinbase");
        // We do not care this, just return zero address
        Ok(H160::zero())
    }

    fn is_mining(&self) -> jsonrpc_core::Result<bool> {
        info!("RPC Request: eth_mining");
        // We do not mine from ETH perspective
        Ok(false)
    }

    fn chain_id(&self) -> jsonrpc_core::Result<Option<U64>> {
        info!("RPC Request: eth_chainId");
        return Ok(Some(self.consensus.best_chain_id().in_evm_space().into()));
    }

    fn gas_price(&self) -> jsonrpc_core::Result<U256> {
        info!("RPC Request: eth_gasPrice");
        let (_, maybe_base_price) =
            self.tx_pool.get_best_info_with_parent_base_price();
        if let Some(base_price) = maybe_base_price {
            return Ok(base_price[Space::Ethereum]);
        }

        let consensus_gas_price = self
            .consensus_graph()
            .gas_price(Space::Ethereum)
            .unwrap_or(GAS_PRICE_DEFAULT_VALUE.into());
        Ok(std::cmp::max(
            consensus_gas_price,
            self.tx_pool.config.min_eth_tx_price.into(),
        ))
    }

    fn max_priority_fee_per_gas(&self) -> jsonrpc_core::Result<U256> {
        info!("RPC Request: eth_maxPriorityFeePerGas");
        let evm_ratio =
            self.tx_pool.machine().params().evm_transaction_block_ratio
                as usize;

        let fee_history = self.fee_history(
            HexU64::from(300),
            BlockNumber::Latest,
            vec![50f64],
        )?;

        let total_reward: U256 = fee_history
            .reward()
            .iter()
            .map(|x| x.first().unwrap())
            .fold(U256::zero(), |x, y| x + *y);

        Ok(total_reward * evm_ratio / 300)
    }

    fn accounts(&self) -> jsonrpc_core::Result<Vec<H160>> {
        info!("RPC Request: eth_accounts");
        // Mazze eSpace does not manage accounts
        Ok(vec![])
    }

    fn block_number(&self) -> jsonrpc_core::Result<U256> {
        let consensus_graph = self.consensus_graph();
        let epoch_num = EpochNumber::LatestState;
        info!("RPC Request: eth_blockNumber()");
        match consensus_graph.get_height_from_epoch_number(epoch_num.into()) {
            Ok(height) => Ok(height.into()),
            Err(e) => Err(jsonrpc_core::Error::invalid_params(e)),
        }
    }

    fn balance(
        &self, address: H160, num: Option<BlockNumber>,
    ) -> jsonrpc_core::Result<U256> {
        let epoch_num = num.unwrap_or_default().try_into()?;

        info!(
            "RPC Request: eth_getBalance address={:?} epoch_num={:?}",
            address, epoch_num
        );

        let state_db = self
            .consensus
            .get_eth_state_db_by_epoch_number(epoch_num, "num")?;
        let acc = state_db
            .get_account(&address.with_evm_space())
            .map_err(|err| MazzeRpcError::from(err))?;

        Ok(acc.map_or(U256::zero(), |acc| acc.balance).into())
    }

    fn storage_at(
        &self, address: H160, position: U256, block_num: Option<BlockNumber>,
    ) -> jsonrpc_core::Result<H256> {
        let epoch_num = block_num.unwrap_or_default().try_into()?;

        info!(
            "RPC Request: eth_getStorageAt address={:?}, position={:?}, block_num={:?})",
            address, position, epoch_num
        );

        let state_db = self
            .consensus
            .get_eth_state_db_by_epoch_number(epoch_num, "epoch_number")?;

        let position: H256 = H256::from_uint(&position);

        let key = StorageKey::new_storage_key(&address, position.as_ref())
            .with_evm_space();

        Ok(
            match state_db
                .get::<StorageValue>(key)
                .map_err(|err| MazzeRpcError::from(err))?
            {
                Some(entry) => H256::from_uint(&entry.value).into(),
                None => H256::zero(),
            },
        )
    }

    fn block_by_hash(
        &self, hash: H256, include_txs: bool,
    ) -> jsonrpc_core::Result<Option<RpcBlock>> {
        info!(
            "RPC Request: eth_getBlockByHash hash={:?} include_txs={:?}",
            hash, include_txs
        );

        let phantom_block = {
            // keep read lock to ensure consistent view
            let _inner = self.consensus_graph().inner.read();

            self.consensus_graph()
                .get_phantom_block_by_hash(
                    &hash, false, /* include_traces */
                )
                .map_err(RpcError::invalid_params)?
        };

        match phantom_block {
            None => Ok(None),
            Some(pb) => Ok(Some(RpcBlock::from_phantom(&pb, include_txs))),
        }
    }

    fn block_by_number(
        &self, block_num: BlockNumber, include_txs: bool,
    ) -> jsonrpc_core::Result<Option<RpcBlock>> {
        info!("RPC Request: eth_getBlockByNumber block_number={:?} include_txs={:?}", block_num, include_txs);

        let phantom_block = {
            // keep read lock to ensure consistent view
            let _inner = self.consensus_graph().inner.read();

            self.consensus_graph()
                .get_phantom_block_by_number(
                    block_num.try_into()?,
                    None,
                    false, /* include_traces */
                )
                .map_err(RpcError::invalid_params)?
        };

        match phantom_block {
            None => Ok(None),
            Some(pb) => Ok(Some(RpcBlock::from_phantom(&pb, include_txs))),
        }
    }

    fn transaction_count(
        &self, address: H160, num: Option<BlockNumber>,
    ) -> jsonrpc_core::Result<U256> {
        info!(
            "RPC Request: eth_getTransactionCount address={:?} block_number={:?}",
            address, num
        );

        let nonce = match num {
            Some(BlockNumber::Pending) => {
                self.tx_pool.get_next_nonce(&address.with_evm_space())
            }
            _ => {
                let num = num.unwrap_or_default().try_into()?;

                self.consensus_graph().next_nonce(
                    address.with_evm_space(),
                    BlockHashOrEpochNumber::EpochNumber(num),
                    "num",
                )?
            }
        };

        Ok(nonce)
    }

    fn block_transaction_count_by_hash(
        &self, hash: H256,
    ) -> jsonrpc_core::Result<Option<U256>> {
        info!(
            "RPC Request: eth_getBlockTransactionCountByHash hash={:?}",
            hash,
        );

        let phantom_block = {
            // keep read lock to ensure consistent view
            let _inner = self.consensus_graph().inner.read();

            self.consensus_graph()
                .get_phantom_block_by_hash(
                    &hash, false, /* include_traces */
                )
                .map_err(RpcError::invalid_params)?
        };

        match phantom_block {
            None => Ok(None),
            Some(pb) => Ok(Some(pb.transactions.len().into())),
        }
    }

    fn block_transaction_count_by_number(
        &self, block_num: BlockNumber,
    ) -> jsonrpc_core::Result<Option<U256>> {
        info!(
            "RPC Request: eth_getBlockTransactionCountByNumber block_number={:?}",
            block_num
        );

        let phantom_block = {
            // keep read lock to ensure consistent view
            let _inner = self.consensus_graph().inner.read();

            self.consensus_graph()
                .get_phantom_block_by_number(
                    block_num.try_into()?,
                    None,
                    false, /* include_traces */
                )
                .map_err(RpcError::invalid_params)?
        };

        match phantom_block {
            None => Ok(None),
            Some(pb) => Ok(Some(pb.transactions.len().into())),
        }
    }

    fn block_uncles_count_by_hash(
        &self, hash: H256,
    ) -> jsonrpc_core::Result<Option<U256>> {
        info!("RPC Request: eth_getUncleCountByBlockHash hash={:?}", hash);

        let epoch_num = match self.consensus.get_block_epoch_number(&hash) {
            None => return Ok(None),
            Some(n) => n,
        };

        let maybe_main_hash = self
            .consensus
            .get_block_hashes_by_epoch(epoch_num.into())
            .ok()
            .and_then(|hs| hs.last().cloned());

        match maybe_main_hash {
            Some(h) if h == hash => Ok(Some(0.into())),
            _ => Ok(None),
        }
    }

    fn block_uncles_count_by_number(
        &self, block_num: BlockNumber,
    ) -> jsonrpc_core::Result<Option<U256>> {
        info!(
            "RPC Request: eth_getUncleCountByBlockNumber block_number={:?}",
            block_num
        );

        let maybe_epoch = self
            .consensus
            .get_block_hashes_by_epoch(block_num.try_into()?)
            .ok();

        Ok(maybe_epoch.map(|_| 0.into()))
    }

    fn code_at(
        &self, address: H160, epoch_num: Option<BlockNumber>,
    ) -> jsonrpc_core::Result<Bytes> {
        let epoch_num = epoch_num.unwrap_or_default().try_into()?;

        info!(
            "RPC Request: eth_getCode address={:?} epoch_num={:?}",
            address, epoch_num
        );

        let state_db = self
            .consensus
            .get_eth_state_db_by_epoch_number(epoch_num, "num")?;

        let address = address.with_evm_space();

        let code = match state_db
            .get_account(&address)
            .map_err(|err| MazzeRpcError::from(err))?
        {
            Some(acc) => match state_db
                .get_code(&address, &acc.code_hash)
                .map_err(|err| MazzeRpcError::from(err))?
            {
                Some(code) => (*code.code).clone(),
                _ => vec![],
            },
            None => vec![],
        };

        Ok(Bytes::new(code))
    }

    fn send_raw_transaction(&self, raw: Bytes) -> jsonrpc_core::Result<H256> {
        info!(
            "RPC Request: eth_sendRawTransaction / eth_submitTransaction raw={:?}",
            raw,
        );
        let tx: TransactionWithSignature = invalid_params_check(
            "raw",
            TransactionWithSignature::from_raw(&raw.into_vec()),
        )?;

        if tx.space() != Space::Ethereum {
            bail!(invalid_params("tx", "Incorrect transaction space"));
        }

        if tx.recover_public().is_err() {
            bail!(invalid_params(
                "tx",
                "Can not recover pubkey for Ethereum like tx. Mazze eSpace only supports EIP-155 rather than EIP-1559 or other format transactions."
            ));
        }

        let r = self.send_transaction_with_signature(tx)?;
        Ok(r)
    }

    fn submit_transaction(&self, raw: Bytes) -> jsonrpc_core::Result<H256> {
        self.send_raw_transaction(raw)
    }

    fn call(
        &self, request: CallRequest, block_number_or_hash: Option<BlockNumber>,
    ) -> jsonrpc_core::Result<Bytes> {
        info!(
            "RPC Request: eth_call request={:?}, block_num={:?}",
            request, block_number_or_hash
        );
        // TODO: EVM core: Check the EVM error message. To make the
        // assert_error_eq test case in solidity project compatible.
        let (execution_outcome, _estimation) =
            self.exec_transaction(request, block_number_or_hash)?;
        match execution_outcome {
            ExecutionOutcome::NotExecutedDrop(TxDropError::OldNonce(
                expected,
                got,
            )) => bail!(call_execution_error(
                "Transaction can not be executed".into(),
                format! {"nonce is too old expected {:?} got {:?}", expected, got}
            )),
            ExecutionOutcome::NotExecutedDrop(
                TxDropError::InvalidRecipientAddress(recipient),
            ) => bail!(call_execution_error(
                "Transaction can not be executed".into(),
                format! {"invalid recipient address {:?}", recipient}
            )),
            ExecutionOutcome::NotExecutedDrop(
                TxDropError::NotEnoughGasLimit { expected, got },
            ) => bail!(call_execution_error(
                "Can not estimate: transaction can not be executed".into(),
                format! {"not enough gas limit with respected to tx size: expected {:?} got {:?}", expected, got}
            )),
            ExecutionOutcome::NotExecutedToReconsiderPacking(e) => {
                bail!(call_execution_error(
                    "Transaction can not be executed".into(),
                    format! {"{:?}", e}
                ))
            }
            ExecutionOutcome::ExecutionErrorBumpNonce(
                ExecutionError::VmError(VmError::Reverted),
                executed,
            ) => bail!(call_execution_error(
                format!(
                    "execution reverted: {}",
                    revert_reason_decode(&executed.output)
                ),
                format!("0x{}", executed.output.to_hex::<String>())
            )),
            ExecutionOutcome::ExecutionErrorBumpNonce(e, _) => {
                bail!(call_execution_error(
                    "Transaction execution failed".into(),
                    format! {"{:?}", e}
                ))
            }
            ExecutionOutcome::Finished(executed) => Ok(executed.output.into()),
        }
    }

    fn estimate_gas(
        &self, request: CallRequest, block_number_or_hash: Option<BlockNumber>,
    ) -> jsonrpc_core::Result<U256> {
        info!(
            "RPC Request: eth_estimateGas request={:?}, block_num={:?}",
            request, block_number_or_hash
        );
        // TODO: EVM core: same as call
        let (execution_outcome, estimation) =
            self.exec_transaction(request, block_number_or_hash)?;
        match execution_outcome {
            ExecutionOutcome::NotExecutedDrop(TxDropError::OldNonce(
                expected,
                got,
            )) => bail!(call_execution_error(
                "Can not estimate: transaction can not be executed".into(),
                format! {"nonce is too old expected {:?} got {:?}", expected, got}
            )),
            ExecutionOutcome::NotExecutedDrop(
                TxDropError::InvalidRecipientAddress(recipient),
            ) => bail!(call_execution_error(
                "Can not estimate: transaction can not be executed".into(),
                format! {"invalid recipient address {:?}", recipient}
            )),
            ExecutionOutcome::NotExecutedDrop(
                TxDropError::NotEnoughGasLimit { expected, got },
            ) => bail!(call_execution_error(
                "Can not estimate: transaction can not be executed".into(),
                format! {"not enough gas limit with respected to tx size: expected {:?} got {:?}", expected, got}
            )),
            ExecutionOutcome::NotExecutedToReconsiderPacking(e) => {
                bail!(call_execution_error(
                    "Can not estimate: transaction can not be executed".into(),
                    format! {"{:?}", e}
                ))
            }
            ExecutionOutcome::ExecutionErrorBumpNonce(
                ExecutionError::VmError(VmError::Reverted),
                executed,
            ) => {
                let (revert_error, innermost_error, errors) =
                    decode_error(&executed, |addr| *addr);

                bail!(call_execution_error(
                    format!(
                        "execution reverted: {}{}",
                        revert_error, innermost_error
                    ),
                    errors.join("\n"),
                ))
            }
            ExecutionOutcome::ExecutionErrorBumpNonce(e, _) => {
                bail!(call_execution_error(
                    format! {"Can not estimate: transaction execution failed, \
                    all gas will be charged (execution error: {:?})", e}
                    .into(),
                    format! {"{:?}", e}
                ))
            }
            ExecutionOutcome::Finished(executed) => executed,
        };

        Ok(estimation.estimated_gas_limit)
    }

    fn fee_history(
        &self, block_count: HexU64, newest_block: BlockNumber,
        reward_percentiles: Vec<f64>,
    ) -> jsonrpc_core::Result<FeeHistory> {
        info!(
            "RPC Request: eth_feeHistory: block_count={}, newest_block={:?}, reward_percentiles={:?}",
            block_count, newest_block, reward_percentiles
        );

        if block_count.as_u64() == 0 {
            return Ok(FeeHistory::new());
        }

        // keep read lock to ensure consistent view
        let _consensus = self.consensus_graph().inner.read();

        let fetch_block = |height| {
            let maybe_block = self
                .consensus_graph()
                .get_phantom_block_main_by_number(
                    EpochNumber::Number(height),
                    None,
                    false,
                )
                .map_err(RpcError::invalid_params)?;
            if let Some(block) = maybe_block {
                // Internal error happens only if the fetch header has
                // inconsistent block height
                Ok(block)
            } else {
                Err(RpcError::invalid_params(
                    "Specified block header does not exist",
                ))
            }
        };

        let start_height: u64 = self
            .consensus_graph()
            .get_height_from_epoch_number(newest_block.try_into()?)
            .map_err(RpcError::invalid_params)?;

        let mut current_height = start_height;

        let mut fee_history = FeeHistory::new();
        while current_height
            >= start_height.saturating_sub(block_count.as_u64() - 1)
        {
            let block = fetch_block(current_height)?;

            // Internal error happens only if the fetch header has inconsistent
            // block height
            fee_history
                .push_front_block(
                    Space::Ethereum,
                    &reward_percentiles,
                    &block.main_header,
                    block.transactions.iter().map(|x| &**x),
                )
                .map_err(|_| RpcError::internal_error())?;

            if current_height == 0 {
                break;
            } else {
                current_height -= 1;
            }
        }

        let block = fetch_block(start_height + 1)?;
        let oldest_block = if current_height == 0 {
            0
        } else {
            current_height + 1
        };
        fee_history.finish(
            oldest_block,
            block.main_header.base_price().as_ref(),
            Space::Ethereum,
        );
        Ok(fee_history)
    }

    fn transaction_by_hash(
        &self, hash: H256,
    ) -> jsonrpc_core::Result<Option<Transaction>> {
        info!("RPC Request: eth_getTransactionByHash({:?})", hash);

        let tx_index = match self
            .consensus
            .get_data_manager()
            .transaction_index_by_hash(&hash, false /* update_cache */)
        {
            None => return Ok(self.get_tx_from_txpool(hash)),
            Some(tx_index) => tx_index,
        };

        let epoch_num =
            match self.consensus.get_block_epoch_number(&tx_index.block_hash) {
                None => return Ok(self.get_tx_from_txpool(hash)),
                Some(n) => n,
            };

        let maybe_block = self
            .consensus_graph()
            .get_phantom_block_by_number(
                EpochNumber::Number(epoch_num),
                None,
                false, /* include_traces */
            )
            .map_err(RpcError::invalid_params)?;

        let phantom_block = match maybe_block {
            None => return Ok(self.get_tx_from_txpool(hash)),
            Some(b) => b,
        };

        for (idx, tx) in phantom_block.transactions.iter().enumerate() {
            if tx.hash() == hash {
                let tx = block_tx_by_index(Some(phantom_block), idx);
                if let Some(tx_ref) = &tx {
                    if tx_ref.status
                        == Some(
                            TransactionStatus::Skipped
                                .in_space(Space::Ethereum)
                                .into(),
                        )
                    {
                        // A skipped transaction is not available to clients if
                        // accessed by its hash.
                        return Ok(None);
                    }
                }
                return Ok(tx);
            }
        }

        Ok(self.get_tx_from_txpool(hash))
    }

    fn transaction_by_block_hash_and_index(
        &self, hash: H256, idx: Index,
    ) -> jsonrpc_core::Result<Option<Transaction>> {
        info!("RPC Request: eth_getTransactionByBlockHashAndIndex hash={:?}, idx={:?}", hash, idx);

        let phantom_block = {
            // keep read lock to ensure consistent view
            let _inner = self.consensus_graph().inner.read();

            self.consensus_graph()
                .get_phantom_block_by_hash(
                    &hash, false, /* include_traces */
                )
                .map_err(RpcError::invalid_params)?
        };

        Ok(block_tx_by_index(phantom_block, idx.value()))
    }

    fn transaction_by_block_number_and_index(
        &self, block_num: BlockNumber, idx: Index,
    ) -> jsonrpc_core::Result<Option<Transaction>> {
        info!("RPC Request: eth_getTransactionByBlockNumberAndIndex block_num={:?}, idx={:?}", block_num, idx);

        let phantom_block = {
            // keep read lock to ensure consistent view
            let _inner = self.consensus_graph().inner.read();

            self.consensus_graph()
                .get_phantom_block_by_number(
                    block_num.try_into()?,
                    None,
                    false, /* include_traces */
                )
                .map_err(RpcError::invalid_params)?
        };

        Ok(block_tx_by_index(phantom_block, idx.value()))
    }

    fn transaction_receipt(
        &self, tx_hash: H256,
    ) -> jsonrpc_core::Result<Option<Receipt>> {
        info!(
            "RPC Request: eth_getTransactionReceipt tx_hash={:?}",
            tx_hash
        );

        let tx_index =
            match self.consensus.get_data_manager().transaction_index_by_hash(
                &tx_hash, false, /* update_cache */
            ) {
                None => return Ok(None),
                Some(tx_index) => tx_index,
            };

        let epoch_num =
            match self.consensus.get_block_epoch_number(&tx_index.block_hash) {
                None => return Ok(None),
                Some(n) => n,
            };

        if epoch_num > self.consensus_graph().best_executed_state_epoch_number()
        {
            // The receipt is only visible to optimistic execution.
            return Ok(None);
        }

        let maybe_block = self
            .consensus_graph()
            .get_phantom_block_by_number(
                EpochNumber::Number(epoch_num),
                None,
                false, /* include_traces */
            )
            .map_err(RpcError::invalid_params)?;

        let phantom_block = match maybe_block {
            None => return Ok(None),
            Some(b) => b,
        };

        let mut prior_log_index = 0;

        for (idx, tx) in phantom_block.transactions.iter().enumerate() {
            if tx.hash() == tx_hash {
                let receipt = self.construct_rpc_receipt(
                    &phantom_block,
                    idx,
                    &mut prior_log_index,
                )?;
                // A skipped transaction is not available to clients if accessed
                // by its hash.
                if receipt.status_code
                    == TransactionStatus::Skipped
                        .in_space(Space::Ethereum)
                        .into()
                {
                    return Ok(None);
                }

                return Ok(Some(receipt));
            }

            // if the if-branch was not entered, we do the bookeeping here
            prior_log_index += phantom_block.receipts[idx].logs.len();
        }

        Ok(None)
    }

    fn uncle_by_block_hash_and_index(
        &self, hash: H256, idx: Index,
    ) -> jsonrpc_core::Result<Option<RpcBlock>> {
        info!(
            "RPC Request: eth_getUncleByBlockHashAndIndex hash={:?}, idx={:?}",
            hash, idx
        );
        // We do not have uncle block
        Ok(None)
    }

    fn uncle_by_block_number_and_index(
        &self, block_num: BlockNumber, idx: Index,
    ) -> jsonrpc_core::Result<Option<RpcBlock>> {
        info!("RPC Request: eth_getUncleByBlockNumberAndIndex block_num={:?}, idx={:?}", block_num, idx);
        // We do not have uncle block
        Ok(None)
    }

    fn logs(&self, filter: EthRpcLogFilter) -> jsonrpc_core::Result<Vec<Log>> {
        info!("RPC Request: eth_getLogs({:?})", filter);

        let filter: LogFilter =
            filter.into_primitive(self.consensus.clone())?;

        let logs = self
            .consensus_graph()
            .logs(filter)
            .map_err(|err| MazzeRpcError::from(err))?;

        // If the results does not fit into `max_limit`, report an error
        if let Some(max_limit) = self.config.get_logs_filter_max_limit {
            if logs.len() > max_limit {
                bail!(invalid_params("filter", format!("This query results in too many logs, max limitation is {}, please use a smaller block range", max_limit)));
            }
        }

        Ok(logs
            .iter()
            .cloned()
            .map(|l| Log::try_from_localized(l, self.consensus.clone(), false))
            .collect::<Result<_, _>>()?)
    }

    fn submit_hashrate(&self, _: U256, _: H256) -> jsonrpc_core::Result<bool> {
        info!("RPC Request: eth_submitHashrate");
        // We do not care mining
        Ok(false)
    }

    fn block_receipts(
        &self, block_num: Option<BlockNumber>,
    ) -> jsonrpc_core::Result<Vec<Receipt>> {
        info!(
            "RPC Request: parity_getBlockReceipts block_number={:?}",
            block_num
        );

        let block_num = block_num.unwrap_or_default();

        let b = {
            // keep read lock to ensure consistent view
            let _inner = self.consensus_graph().inner.read();

            let phantom_block = match block_num {
                BlockNumber::Hash { hash, .. } => self
                    .consensus_graph()
                    .get_phantom_block_by_hash(
                        &hash, false, /* include_traces */
                    )
                    .map_err(RpcError::invalid_params)?,
                _ => self
                    .consensus_graph()
                    .get_phantom_block_by_number(
                        block_num.try_into()?,
                        None,
                        false, /* include_traces */
                    )
                    .map_err(RpcError::invalid_params)?,
            };

            match phantom_block {
                None => return Err(unknown_block()),
                Some(b) => b,
            }
        };

        let mut block_receipts = vec![];
        let mut prior_log_index = 0;

        for idx in 0..b.receipts.len() {
            block_receipts.push(self.construct_rpc_receipt(
                &b,
                idx,
                &mut prior_log_index,
            )?);
        }

        Ok(block_receipts)
    }

    fn account_pending_transactions(
        &self, address: H160, maybe_start_nonce: Option<U256>,
        maybe_limit: Option<U64>,
    ) -> jsonrpc_core::Result<AccountPendingTransactions> {
        info!("RPC Request: eth_getAccountPendingTransactions(addr={:?}, start_nonce={:?}, limit={:?})",
              address, maybe_start_nonce, maybe_limit);

        let (pending_txs, tx_status, pending_count) =
            self.tx_pool.get_account_pending_transactions(
                &Address::from(address).with_evm_space(),
                maybe_start_nonce,
                maybe_limit.map(|limit| limit.as_usize()),
                self.consensus.best_epoch_number(),
            );
        Ok(AccountPendingTransactions {
            pending_transactions: pending_txs
                .into_iter()
                .map(|tx| {
                    Transaction::from_signed(
                        &tx,
                        (None, None, None),
                        (None, None),
                    )
                })
                .collect(),
            first_tx_status: tx_status,
            pending_count: pending_count.into(),
        })
    }
}
