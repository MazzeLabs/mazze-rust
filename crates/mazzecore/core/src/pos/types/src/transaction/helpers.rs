// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0



use crate::{
    account_address::AccountAddress,
    chain_id::ChainId,
    transaction::{RawTransaction, SignedTransaction, TransactionPayload},
    validator_config::{ConsensusPrivateKey, ConsensusPublicKey},
};
use anyhow::Result;
use chrono::Utc;
use diem_crypto::{test_utils::KeyPair, traits::SigningKey};

pub fn create_unsigned_txn(
    payload: TransactionPayload, sender_address: AccountAddress,
    txn_expiration_duration_secs: i64, /* for compatibility with UTC's
                                        * timestamp. */
    chain_id: ChainId,
) -> RawTransaction {
    RawTransaction::new(
        sender_address,
        payload,
        (Utc::now().timestamp() + txn_expiration_duration_secs) as u64,
        chain_id,
    )
}

pub trait TransactionSigner {
    fn sign_txn(&self, raw_txn: RawTransaction) -> Result<SignedTransaction>;
}

/// Craft a transaction request.
pub fn create_user_txn<T: TransactionSigner + ?Sized>(
    signer: &T, payload: TransactionPayload, sender_address: AccountAddress,
    txn_expiration_duration_secs: i64, /* for compatibility with UTC's
                                        * timestamp. */
    chain_id: ChainId,
) -> Result<SignedTransaction> {
    let raw_txn = create_unsigned_txn(
        payload,
        sender_address,
        txn_expiration_duration_secs,
        chain_id,
    );
    signer.sign_txn(raw_txn)
}

impl TransactionSigner for KeyPair<ConsensusPrivateKey, ConsensusPublicKey> {
    fn sign_txn(&self, raw_txn: RawTransaction) -> Result<SignedTransaction> {
        let signature = self.private_key.sign(&raw_txn);
        Ok(SignedTransaction::new(
            raw_txn,
            self.public_key.clone(),
            signature,
        ))
    }
}
