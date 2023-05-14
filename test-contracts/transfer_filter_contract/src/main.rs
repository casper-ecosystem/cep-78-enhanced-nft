#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

use alloc::{string::ToString, vec};

use casper_contract::contract_api::{
    runtime::{self, ret},
    storage,
};
use casper_types::{
    contracts::NamedKeys, CLType, CLValue, ContractHash, ContractVersion, EntryPoint,
    EntryPointAccess, EntryPointType, EntryPoints, Key, Parameter,
};

const CONTRACT_NAME: &str = "transfer_filter_contract_hash";
const CONTRACT_VERSION: &str = "transfer_filter_contract_version";
const HASH_KEY_NAME: &str = "transfer_filter_contract_package_hash";
const ACCESS_KEY_NAME: &str = "transfer_filter_contract_access_uref";
const ARG_FILTER_CONTRACT_RETURN_VALUE: &str = "return_value";

fn install_filter_contract() -> (ContractHash, ContractVersion) {
    let can_transfer_entry_point = EntryPoint::new(
        "can_transfer",
        vec![
            Parameter::new("source_key", CLType::Key),
            Parameter::new("target_key", CLType::Key),
        ],
        CLType::U8,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let set_return_value = EntryPoint::new(
        "set_return_value",
        vec![Parameter::new(ARG_FILTER_CONTRACT_RETURN_VALUE, CLType::U8)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(can_transfer_entry_point);
    entry_points.add_entry_point(set_return_value);

    let mut named_keys = NamedKeys::new();
    named_keys.insert(
        ARG_FILTER_CONTRACT_RETURN_VALUE.to_string(),
        storage::new_uref(0u8).into(),
    );

    storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(HASH_KEY_NAME.to_string()),
        Some(ACCESS_KEY_NAME.to_string()),
    )
}

#[no_mangle]
pub extern "C" fn set_return_value() {
    let return_value: u8 = runtime::get_named_arg(ARG_FILTER_CONTRACT_RETURN_VALUE);
    let uref = runtime::get_key(ARG_FILTER_CONTRACT_RETURN_VALUE)
        .unwrap()
        .into_uref()
        .unwrap();

    storage::write(uref, return_value);

    runtime::put_key(ARG_FILTER_CONTRACT_RETURN_VALUE, Key::from(uref));
}

#[no_mangle]
pub extern "C" fn can_transfer() {
    let uref = runtime::get_key(ARG_FILTER_CONTRACT_RETURN_VALUE)
        .unwrap()
        .into_uref()
        .unwrap();

    let return_value = storage::read::<u8>(uref).unwrap().unwrap();

    ret(CLValue::from_t(return_value).unwrap());
}

#[no_mangle]
pub extern "C" fn call() {
    let (contract_hash, contract_version) = install_filter_contract();

    runtime::put_key(CONTRACT_NAME, contract_hash.into());
    runtime::put_key(CONTRACT_VERSION, storage::new_uref(contract_version).into());
}
