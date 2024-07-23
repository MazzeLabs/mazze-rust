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

// Copyright 2024 Mazze Labs. All rights reserved.
// Mazze is free software and distributed under GNU General Public License.
// See http://www.gnu.org/licenses/

// Importing necessary crates and modules
use mazzekey::Password;
use rpassword::read_password;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

// Constant error message for non-interactive terminal errors
const PASSWORD_STDIN_ERROR: &str =
    "Unable to ask for password on non-interactive terminal.";

/// Flush output buffer.
pub fn flush_stdout() {
    io::stdout().flush().expect("stdout is flushable; qed");
}

/// Prompts user asking for password and validates by repeating.
pub fn password_prompt() -> Result<Password, String> {
    println!("Please note that password is NOT RECOVERABLE.");
    print!("Type password: ");
    flush_stdout(); // Ensure the prompt is displayed immediately

    // Read password from user input
    let password = read_password()
        .map_err(|_| PASSWORD_STDIN_ERROR.to_owned())?
        .into();

    print!("Repeat password: ");
    flush_stdout(); // Ensure the prompt is displayed immediately

    // Read password again for confirmation
    let password_repeat = read_password()
        .map_err(|_| PASSWORD_STDIN_ERROR.to_owned())?
        .into();

    // Check if both passwords match
    if password != password_repeat {
        return Err("Passwords do not match!".into());
    }

    Ok(password)
}

/// Prompts user for a single password input without validation.
pub fn input_password() -> Result<Password, String> {
    print!("Type password: ");
    flush_stdout(); // Ensure the prompt is displayed immediately

    // Read password from user input
    let password = read_password()
        .map_err(|_| PASSWORD_STDIN_ERROR.to_owned())?
        .into();

    Ok(password)
}

/// Reads a password from a given file path.
pub fn password_from_file(path: String) -> Result<Password, String> {
    let passwords = passwords_from_files(&[path])?;
    // Use only the first password from the file
    passwords
        .get(0)
        .map(Password::clone)
        .ok_or_else(|| "Password file seems to be empty.".to_owned())
}

/// Reads passwords from multiple files. Each line in a file is treated as a separate password.
pub fn passwords_from_files(files: &[String]) -> Result<Vec<Password>, String> {
    let passwords = files.iter().map(|filename| {
        // Open the password file
        let file = File::open(filename).map_err(|_| format!("{} Unable to read password file. Ensure it exists and permissions are correct.", filename))?;
        let reader = BufReader::new(&file);
        // Read each line in the file and collect them as passwords
        let lines = reader.lines()
            .filter_map(|l| l.ok()) // Filter out any lines that failed to read
            .map(|pwd| pwd.trim().to_owned().into()) // Convert each line to a Password
            .collect::<Vec<Password>>();
        Ok(lines)
    }).collect::<Result<Vec<Vec<Password>>, String>>();
    Ok(passwords?.into_iter().flatten().collect()) // Flatten the result and collect into a single vector
}