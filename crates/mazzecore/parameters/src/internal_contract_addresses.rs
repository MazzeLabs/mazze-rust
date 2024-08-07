

use mazze_types::{Address, H160};
use hex_literal::hex;

pub const ADMIN_CONTROL_CONTRACT_ADDRESS: Address =
    H160(hex!("0888000000000000000000000000000000000000"));
pub const SPONSOR_WHITELIST_CONTROL_CONTRACT_ADDRESS: Address =
    H160(hex!("0888000000000000000000000000000000000001"));
pub const STORAGE_INTEREST_STAKING_CONTRACT_ADDRESS: Address =
    H160(hex!("0888000000000000000000000000000000000002"));

pub const CONTEXT_CONTRACT_ADDRESS: Address =
    H160(hex!("0888000000000000000000000000000000000004"));
pub const POS_REGISTER_CONTRACT_ADDRESS: Address =
    H160(hex!("0888000000000000000000000000000000000005"));
pub const CROSS_SPACE_CONTRACT_ADDRESS: Address =
    H160(hex!("0888000000000000000000000000000000000006"));
pub const PARAMS_CONTROL_CONTRACT_ADDRESS: Address =
    H160(hex!("0888000000000000000000000000000000000007"));
pub const SYSTEM_STORAGE_ADDRESS: Address =
    H160(hex!("088800000000000000000000000000000000000a"));

// We reserve more addresses so we don't need to change the genesis hash
// in test mode each time adding new internal contracts.
pub const RESERVED3: Address =
    H160(hex!("0888000000000000000000000000000000000003"));
pub const RESERVED8: Address =
    H160(hex!("0888000000000000000000000000000000000008"));
pub const RESERVED9: Address =
    H160(hex!("0888000000000000000000000000000000000009"));
pub const RESERVED11: Address =
    H160(hex!("088800000000000000000000000000000000000b"));
