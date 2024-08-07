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



// Importing the necessary crates and modules
extern crate mazzecore_accounts;

use super::helpers::{password_from_file, password_prompt};
use mazzestore::{
    accounts_dir::RootDiskDirectory, import_account, import_accounts,
};
use clap;
use client::accounts::{account_provider, keys_dir, keys_path};
use std::path::PathBuf;

// Enum defining the different account commands
#[derive(Debug, PartialEq)]
pub enum AccountCmd {
    New(NewAccount),          // Command to create a new account
    List(ListAccounts),       // Command to list existing accounts
    Import(ImportAccounts),   // Command to import accounts from external sources
}

// Struct for the ListAccounts command
#[derive(Debug, PartialEq)]
pub struct ListAccounts {
    pub path: Option<String>, // Optional path to list accounts from
}

impl ListAccounts {
    // Constructor for ListAccounts
    pub fn new(_matches: &clap::ArgMatches) -> Self { 
        Self { path: None } 
    }
}

// Struct for the NewAccount command
#[derive(Debug, PartialEq)]
pub struct NewAccount {
    pub iterations: u32,            // Number of iterations for key generation
    pub path: Option<String>,       // Optional path for account storage
    pub password_file: Option<String>, // Optional path to a password file
}

impl NewAccount {
    // Constructor for NewAccount
    pub fn new(matches: &clap::ArgMatches) -> Self {
        let iterations: u32 = matches
            .value_of("key-iterations")
            .unwrap_or("0")
            .parse()
            .unwrap();
        let password_file = matches.value_of("password").map(|x| x.to_string());
        Self {
            iterations,
            path: None,
            password_file,
        }
    }
}

// Struct for the ImportAccounts command
#[derive(Debug, PartialEq)]
pub struct ImportAccounts {
    pub from: Vec<String>, // List of paths to import accounts from
    pub to: String,        // Destination path to import accounts to
}

impl ImportAccounts {
    // Constructor for ImportAccounts
    pub fn new(matches: &clap::ArgMatches) -> Self {
        let from: Vec<_> = matches
            .values_of("import-path")
            .expect("CLI argument is required; qed")
            .map(|s| s.to_string())
            .collect();
        Self {
            from,
            to: keys_path(),
        }
    }
}

// Function to execute the appropriate account command
pub fn execute(cmd: AccountCmd) -> Result<String, String> {
    match cmd {
        AccountCmd::New(new_cmd) => new(new_cmd),       // Execute the New command
        AccountCmd::List(list_cmd) => list(list_cmd),   // Execute the List command
        AccountCmd::Import(import_cmd) => import(import_cmd), // Execute the Import command
    }
}

// Function to create a new account
fn new(new_cmd: NewAccount) -> Result<String, String> {
    let password = match new_cmd.password_file {
        Some(file) => password_from_file(file)?, // Get password from file if provided
        None => password_prompt()?,              // Otherwise, prompt the user for a password
    };

    let acc_provider = account_provider(
        new_cmd.path,
        Some(new_cmd.iterations), // Number of key iterations
        None,                     // Refresh time (optional, not used)
    )?;

    let new_account = acc_provider
        .new_account(&password)
        .map_err(|e| format!("Could not create new account: {}", e))?;
    Ok(format!("0x{:x}", new_account))
}

// Function to list existing accounts
fn list(list_cmd: ListAccounts) -> Result<String, String> {
    let acc_provider = account_provider(
        list_cmd.path,
        None, // Number of key iterations (optional, not used)
        None, // Refresh time (optional, not used)
    )?;

    let accounts = acc_provider.accounts().map_err(|e| format!("{}", e))?;
    let result = accounts
        .into_iter()
        .map(|a| format!("0x{:x}", a))
        .collect::<Vec<String>>()
        .join("\n");

    Ok(result)
}

// Function to import accounts from external sources
fn import(import_cmd: ImportAccounts) -> Result<String, String> {
    let to = keys_dir(import_cmd.to)?; // Destination directory for imported accounts
    let mut imported = 0;

    for path in &import_cmd.from {
        let path = PathBuf::from(path);
        if path.is_dir() {
            let from = RootDiskDirectory::at(&path);
            imported += import_accounts(&from, &to)
                .map_err(|e| {
                    format!("Importing accounts from {:?} failed: {}", path, e)
                })?
                .len();
        } else if path.is_file() {
            import_account(&path, &to).map_err(|e| {
                format!("Importing account from {:?} failed: {}", path, e)
            })?;
            imported += 1;
        }
    }

    Ok(format!("{} account(s) imported", imported))
}