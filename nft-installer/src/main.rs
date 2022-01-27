#![no_main]
#![no_std]

extern crate alloc;

use alloc::{vec, string::ToString, string::String};
use casper_contract::contract_api::{runtime, storage};

use casper_types::{CLType, ContractHash, ContractVersion, EntryPoint, EntryPointAccess, EntryPoints, EntryPointType, Parameter, runtime_args, RuntimeArgs};
use casper_types::contracts::NamedKeys;


fn store() -> (ContractHash, ContractVersion) {
    let entry_points = {
        let mut entry_points = EntryPoints::new();

        let init_contract = EntryPoint::new(
            nft_contract::ENTRY_POINT_INIT,
            vec![],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        );

        let set_variables = EntryPoint::new(
            nft_contract::ENTRY_POINT_SET_VARIABLES,
            vec![Parameter::new(nft_contract::ARG_TOKEN_OWNER, CLType::String)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        );

        entry_points.add_entry_point(init_contract);
        entry_points.add_entry_point(set_variables);

        entry_points
    };

    let named_keys = {
        let mut named_keys = NamedKeys::new();
        named_keys.insert(nft_contract::INSTALLER.to_string(), runtime::get_caller().into());
        named_keys
    };

    storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(nft_contract::HASH_KEY_NAME.to_string()),
        Some(nft_contract::ACCESS_KEY_NAME.to_string()),
    )
}

#[no_mangle]
pub extern "C" fn call() {
    let token_owner: String = runtime::get_named_arg(nft_contract::ARG_TOKEN_OWNER);
    let (contract_hash, contract_version) = store();
    runtime::put_key(
        nft_contract::CONTRACT_VERSION,
        storage::new_uref(contract_version).into()
    );
    runtime::put_key(nft_contract::CONTRACT_NAME, contract_hash.into());

    let _: () = runtime::call_contract(
        contract_hash,
        nft_contract::ENTRY_POINT_INIT,
        RuntimeArgs::default()
    );
    runtime::call_contract::<()>(
        contract_hash,
        nft_contract::ENTRY_POINT_SET_VARIABLES,
        runtime_args! {
            nft_contract::ARG_TOKEN_OWNER => token_owner
        }
    );
}