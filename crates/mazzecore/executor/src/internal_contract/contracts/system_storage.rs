

use super::preludes::*;
use mazze_parameters::internal_contract_addresses::SYSTEM_STORAGE_ADDRESS;
use mazze_types::U256;

make_solidity_contract! {
    pub struct SystemStorage(SYSTEM_STORAGE_ADDRESS, SolFnTable::default, initialize: |params: &CommonParams| params.transition_numbers.cip94n, is_active: |spec: &Spec| spec.cip94);
}

pub fn base_slot(contract: Address) -> U256 {
    let hash = keccak(H256::from(contract).as_ref());
    U256::from_big_endian(hash.as_ref())
}
