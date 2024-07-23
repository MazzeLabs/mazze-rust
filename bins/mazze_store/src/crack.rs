// Copyright 2024 Mazze Labs. All rights reserved.
// Mazze is free software and distributed under GNU General Public License.
// See http://www.gnu.org/licenses/

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



use parking_lot::Mutex; // Importing Mutex from the parking_lot crate for mutual exclusion.
use std::{cmp, collections::VecDeque, sync::Arc, thread}; // Importing necessary modules from std.

use mazzestore::{mazzekey::Password, Error, PresaleWallet}; // Importing types and traits from mazzestore.
use num_cpus; // Importing num_cpus crate to get the number of CPUs.

/// Runs the password cracking attempt on a presale wallet.
/// 
/// # Arguments
/// 
/// * `passwords` - A deque containing passwords to be tested.
/// * `wallet_path` - A path to the presale wallet file.
pub fn run(passwords: VecDeque<Password>, wallet_path: &str) -> Result<(), Error> {
    // Wraps the passwords in an Arc and Mutex to share between threads safely.
    let passwords = Arc::new(Mutex::new(passwords));

    let mut handles = Vec::new(); // Vector to hold the thread handles.

    // Spawning threads equal to the number of CPUs available.
    for _ in 0..num_cpus::get() {
        let passwords = passwords.clone(); // Clone the Arc for thread-safe sharing.
        let wallet = PresaleWallet::open(&wallet_path)?; // Open the presale wallet.
        
        // Spawn a new thread to look for the password.
        handles.push(thread::spawn(move || {
            look_for_password(passwords, wallet);
        }));
    }

    // Join all threads and handle any potential errors.
    for handle in handles {
        handle.join().map_err(|err| {
            Error::Custom(format!("Error finishing thread: {:?}", err))
        })?;
    }

    Ok(())
}

/// Attempts to find the correct password for the wallet by decrypting it.
///
/// # Arguments
///
/// * `passwords` - Shared deque of passwords to be tested.
/// * `wallet` - The presale wallet to be decrypted.
fn look_for_password(passwords: Arc<Mutex<VecDeque<Password>>>, wallet: PresaleWallet) {
    let mut counter = 0; // Counter to keep track of the number of attempts.
    
    // Loop until there are no more passwords to test.
    while !passwords.lock().is_empty() {
        // Split off a chunk of up to 32 passwords for processing.
        let package = {
            let mut passwords = passwords.lock();
            let len = passwords.len();
            passwords.split_off(cmp::min(len, 32))
        };

        // Iterate through the passwords in the current package.
        for pass in package {
            counter += 1;
            match wallet.decrypt(&pass) {
                // If decryption is successful, print the found password and clear the password list.
                Ok(_) => {
                    println!("Found password: {}", pass.as_str());
                    passwords.lock().clear();
                    return;
                }
                // Print a dot every 100 attempts to indicate progress.
                _ if counter % 100 == 0 => print!("."),
                // Do nothing if decryption fails.
                _ => {}
            }
        }
    }
}