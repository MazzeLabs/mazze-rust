// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Ethereum key-management.

#![warn(missing_docs)]

extern crate mazze_types;
extern crate mazzekey as _mazzekey;
extern crate dir;
extern crate libc;
extern crate parity_crypto as crypto;
extern crate parity_wordlist;
extern crate parking_lot;
extern crate rand;
extern crate rustc_hex;
extern crate serde;
extern crate serde_json;
extern crate smallvec;
extern crate tempdir;
extern crate time;
extern crate tiny_keccak;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
#[macro_use]
extern crate matches;

mod account;
pub mod accounts_dir;
pub mod mazzekey;
mod mazzestore;
mod error;
mod import;
mod json;
mod presale;
mod random;
mod secret_store;

pub use self::{
    account::{Crypto, SafeAccount},
    mazzestore::{MazzeMultiStore, MazzeStore},
    error::Error,
    import::{import_account, import_accounts, read_geth_accounts},
    json::OpaqueKeyFile as KeyFile,
    parity_wordlist::random_phrase,
    presale::PresaleWallet,
    random::random_string,
    secret_store::{
        Derivation, IndexDerivation, SecretStore, SecretVaultRef,
        SimpleSecretStore, StoreAccountRef,
    },
};

/// An opaque wrapper for secret.
pub struct OpaqueSecret(::mazzekey::Secret);
