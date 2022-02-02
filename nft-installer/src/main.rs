#![no_main]
#![no_std]

extern crate alloc;
use alloc::{boxed::Box, string::String, string::ToString, vec};
use casper_contract::contract_api::{runtime, storage};

use casper_types::contracts::NamedKeys;
use casper_types::{
    runtime_args, CLType, ContractHash, ContractVersion, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, Parameter, RuntimeArgs,
};

fn store() -> (ContractHash, ContractVersion) {
    let entry_points = {
        let mut entry_points = EntryPoints::new();

        let init_contract = EntryPoint::new(
            nft_contract::ENTRY_POINT_INIT,
            vec![Parameter::new(
                nft_contract::ARG_COLLECTION_NAME,
                CLType::String,
            )],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        // TODO: update once we've figured out what variables can be set.
        let set_variables = EntryPoint::new(
            nft_contract::ENTRY_POINT_SET_VARIABLES,
            vec![Parameter::new(
                nft_contract::ARG_COLLECTION_NAME,
                CLType::String,
            )],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        let mint = EntryPoint::new(
            nft_contract::ENTRY_POINT_MINT,
            vec![
                Parameter::new(nft_contract::ARG_TOKEN_OWNER, CLType::PublicKey),
                Parameter::new(nft_contract::ARG_TOKEN_ID, CLType::U256),
                Parameter::new(nft_contract::ARG_TOKEN_NAME, CLType::String),
                Parameter::new(nft_contract::ARG_TOKEN_META, CLType::String),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        let burn = EntryPoint::new(
            nft_contract::ENTRY_POINT_BURN,
            vec![
                Parameter::new(nft_contract::ARG_TOKEN_OWNER, CLType::PublicKey),
                Parameter::new(nft_contract::ARG_TOKEN_ID, CLType::U256),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        let transfer = EntryPoint::new(
            nft_contract::ENTRY_POINT_TRANSFER,
            vec![
                Parameter::new(nft_contract::ARG_TOKEN_OWNER, CLType::PublicKey),
                Parameter::new(nft_contract::ARG_TOKEN_RECEIVER, CLType::PublicKey),
                Parameter::new(nft_contract::ARG_TOKEN_ID, CLType::U256),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        let balance_of = EntryPoint::new(
            nft_contract::ENTRY_POINT_BALANCE_OF,
            vec![Parameter::new(
                nft_contract::ARG_TOKEN_OWNER,
                CLType::PublicKey,
            )],
            CLType::List(Box::new(CLType::U256)),
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        entry_points.add_entry_point(init_contract);
        entry_points.add_entry_point(set_variables);
        entry_points.add_entry_point(mint);
        entry_points.add_entry_point(burn);
        entry_points.add_entry_point(transfer);
        entry_points.add_entry_point(balance_of);

        entry_points
    };

    let named_keys = {
        let mut named_keys = NamedKeys::new();
        named_keys.insert(
            nft_contract::INSTALLER.to_string(),
            runtime::get_caller().into(),
        );
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
    let collection_name: String = runtime::get_named_arg(nft_contract::ARG_COLLECTION_NAME);
    let (contract_hash, contract_version) = store();

    // Store contract_hash and contract_version under the keys CONTRACT_NAME and CONTRACT_VERSION
    runtime::put_key(nft_contract::CONTRACT_NAME, contract_hash.into());
    runtime::put_key(
        nft_contract::CONTRACT_VERSION,
        storage::new_uref(contract_version).into(),
    );

    // Call contract to initialize it
    runtime::call_contract::<()>(
        contract_hash,
        nft_contract::ENTRY_POINT_INIT,
        runtime_args! { nft_contract::ARG_COLLECTION_NAME => collection_name},
    );
}
