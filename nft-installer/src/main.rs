#![no_main]
#![no_std]

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::{boxed::Box, string::String, string::ToString, vec};
use casper_contract::contract_api::{runtime, storage};

use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::contracts::NamedKeys;
use casper_types::{
    runtime_args, CLType, ContractHash, ContractVersion, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, Key, Parameter, PublicKey, RuntimeArgs, U256,
};

use nft_contract::*;

fn store() -> (ContractHash, ContractVersion) {
    let entry_points = {
        let mut entry_points = EntryPoints::new();

        let init_contract = EntryPoint::new(
            ENTRY_POINT_INIT,
            vec![
                Parameter::new(ARG_COLLECTION_NAME, CLType::String),
                Parameter::new(ARG_COLLECTION_SYMBOL, CLType::String),
                Parameter::new(ARG_TOTAL_TOKEN_SUPPLY, CLType::U256),
                Parameter::new(ARG_ALLOW_MINTING, CLType::Bool),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        // TODO: update once we've figured out what variables can be set.
        let set_variables = EntryPoint::new(
            ENTRY_POINT_SET_VARIABLES,
            vec![Parameter::new(ARG_ALLOW_MINTING, CLType::Bool)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        let mint = EntryPoint::new(
            ENTRY_POINT_MINT,
            vec![Parameter::new(ARG_TOKEN_OWNER, CLType::PublicKey)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        let burn = EntryPoint::new(
            ENTRY_POINT_BURN,
            vec![
                Parameter::new(ARG_TOKEN_OWNER, CLType::PublicKey),
                Parameter::new(ARG_TOKEN_ID, CLType::U256),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        // let collection_name = EntryPoint::new(
        //     ENTRY_POINT_COLLECTION_NAME,
        //     vec![],
        //     CLType::String,
        //     EntryPointAccess::Public,
        //     EntryPointType::Contract,
        // );

        // let mint = EntryPoint::new(
        //     ENTRY_POINT_MINT,
        //     vec![
        //         Parameter::new( ARG_TOKEN_OWNER, CLType::PublicKey),
        //         Parameter::new( ARG_TOKEN_ID, CLType::U256),
        //         Parameter::new( ARG_TOKEN_NAME, CLType::String),
        //         Parameter::new( ARG_TOKEN_META, CLType::String),
        //     ],
        //     CLType::Unit,
        //     EntryPointAccess::Public,
        //     EntryPointType::Contract,
        // );

        // let burn = EntryPoint::new(
        //      ENTRY_POINT_BURN,
        //     vec![
        //         Parameter::new( ARG_TOKEN_OWNER, CLType::PublicKey),
        //         Parameter::new( ARG_TOKEN_ID, CLType::U256),
        //     ],
        //     CLType::Unit,
        //     EntryPointAccess::Public,
        //     EntryPointType::Contract,
        // );

        // let transfer = EntryPoint::new(
        //      ENTRY_POINT_TRANSFER,
        //     vec![
        //         Parameter::new( ARG_TOKEN_OWNER, CLType::PublicKey),
        //         Parameter::new( ARG_TOKEN_RECEIVER, CLType::PublicKey),
        //         Parameter::new( ARG_TOKEN_ID, CLType::U256),
        //     ],
        //     CLType::Unit,
        //     EntryPointAccess::Public,
        //     EntryPointType::Contract,
        // );

        // let balance_of = EntryPoint::new(
        //      ENTRY_POINT_BALANCE_OF,
        //     vec![Parameter::new(
        //          ARG_TOKEN_OWNER,
        //         CLType::PublicKey,
        //     )],
        //     CLType::List(Box::new(CLType::U256)),
        //     EntryPointAccess::Public,
        //     EntryPointType::Contract,
        // );

        entry_points.add_entry_point(init_contract);
        entry_points.add_entry_point(set_variables);
        entry_points.add_entry_point(mint);
        entry_points.add_entry_point(burn);

        // entry_points.add_entry_point(transfer);
        // entry_points.add_entry_point(balance_of);

        entry_points
    };

    let named_keys = {
        let mut named_keys = NamedKeys::new();

        let _ = storage::new_dictionary(TOKEN_OWNERS).unwrap_or_revert();

        let last_token_id = U256::zero();
        let last_token_uref = storage::new_uref(last_token_id);

        // let token_owner: BTreeMap<U256, PublicKey> = BTreeMap::new();
        // let uref = storage::new_uref(token_owner);
        // named_keys.insert(TOKEN_OWNERS.to_string(), uref.into());

        named_keys.insert(NUMBER_OF_MINTED_TOKENS.to_string(), last_token_uref.into());
        named_keys.insert(INSTALLER.to_string(), runtime::get_caller().into());
        named_keys
    };

    storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(HASH_KEY_NAME.to_string()),
        Some(ACCESS_KEY_NAME.to_string()),
    )
}

#[no_mangle]
pub extern "C" fn call() {
    let collection_name: String = get_named_arg_with_user_errors(
        ARG_COLLECTION_NAME,
        NFTCoreError::MissingCollectionName,
        NFTCoreError::InvalidCollectionName,
    )
    .unwrap_or_revert();

    let collection_symbol: String = get_named_arg_with_user_errors(
        ARG_COLLECTION_SYMBOL,
        NFTCoreError::MissingCollectionSymbol,
        NFTCoreError::InvalidCollectionSymbol,
    )
    .unwrap_or_revert();

    let total_token_supply: U256 = get_named_arg_with_user_errors(
        ARG_TOTAL_TOKEN_SUPPLY,
        NFTCoreError::MissingTotalTokenSupply,
        NFTCoreError::InvalidTotalTokenSupply,
    )
    .unwrap_or_revert();

    let allow_minting: bool = get_optional_named_arg_with_user_errors(
        ARG_ALLOW_MINTING,
        NFTCoreError::MissingMintingStatus, // <-- Useless?
        NFTCoreError::InvalidMintingStatus,
    )
    .unwrap_or(true);

    let (contract_hash, contract_version) = store();

    // Store contract_hash and contract_version under the keys CONTRACT_NAME and CONTRACT_VERSION
    runtime::put_key(CONTRACT_NAME, contract_hash.into());
    runtime::put_key(CONTRACT_VERSION, storage::new_uref(contract_version).into());

    // Call contract to initialize it
    runtime::call_contract::<()>(
        contract_hash,
        ENTRY_POINT_INIT,
        runtime_args! {
             ARG_COLLECTION_NAME => collection_name,
             ARG_COLLECTION_SYMBOL => collection_symbol,
             ARG_TOTAL_TOKEN_SUPPLY => total_token_supply,
             ARG_ALLOW_MINTING => allow_minting,
        },
    );
}
