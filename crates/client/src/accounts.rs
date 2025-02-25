

use app_dirs::{get_app_root, AppDataType, AppInfo};
use mazzecore_accounts::{AccountProvider, AccountProviderSettings};
use mazzestore::{accounts_dir::RootDiskDirectory, MazzeStore};
use dir::helpers::replace_home;
use std::{path::PathBuf, time::Duration};

pub fn account_provider(
    dir: Option<String>, sstore_iterations: Option<u32>,
    refresh_time: Option<Duration>,
) -> Result<AccountProvider, String> {
    let dir = match dir {
        Some(dir) => dir,
        None => keys_path(),
    };

    let dir = Box::new(keys_dir(dir)?);
    let secret_store = Box::new(secret_store(dir, sstore_iterations)?);

    if let Some(t) = refresh_time {
        secret_store.set_refresh_time(t);
    }

    Ok(AccountProvider::new(
        secret_store,
        AccountProviderSettings::default(),
    ))
}

pub fn keys_dir(path: String) -> Result<RootDiskDirectory, String> {
    let path = PathBuf::from(&path);
    RootDiskDirectory::create(path)
        .map_err(|e| format!("Could not open keys directory: {}", e))
}

fn secret_store(
    dir: Box<RootDiskDirectory>, iterations: Option<u32>,
) -> Result<MazzeStore, String> {
    match iterations {
        Some(i) => MazzeStore::open_with_iterations(dir, i),
        None => MazzeStore::open(dir),
    }
    .map_err(|e| format!("Could not open keys store: {}", e))
}

/// Default data path
fn default_data_path() -> String {
    let app_info = AppInfo {
        name: "Mazze",
        author: "Mazze",
    };
    get_app_root(AppDataType::UserData, &app_info)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "$HOME/.mazze".to_owned())
}

pub fn keys_path() -> String {
    let data_path = default_data_path();
    replace_home(&data_path, "$BASE/keys")
}
