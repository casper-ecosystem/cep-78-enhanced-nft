#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

use alloc::{vec, string::{String, ToString}, format};

use casper_contract::contract_api::{runtime, storage};
use casper_types::{CLType, ContractHash, ContractVersion, EntryPoint, EntryPointAccess, EntryPoints, EntryPointType, Key, Parameter, runtime_args, RuntimeArgs};
use casper_types::contracts::NamedKeys;

const CONTRACT_NAME: &str = "minting_contract_hash";
const CONTRACT_VERSION: &str = "minting_contract_version";
const INSTALLER: &str = "installer";
const HASH_KEY_NAME: &str = "minting_contract_package_hash";
const ACCESS_KEY_NAME: &str = "minting_contract_access_uref";

const ENTRY_POINT_MINT: &str = "mint";
const ENTRY_POINT_TRANSFER: &str = "transfer";
const ENTRY_POINT_BURN: &str = "burn";
const ENTRY_POINT_METADATA: &str = "metadata";

const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_TOKEN_OWNER: &str = "token_owner";
const ARG_TOKEN_META_DATA: &str = "token_meta_data";
const ARG_TARGET_KEY: &str = "target_key";
const ARG_SOURCE_KEY: &str = "source_key";
const ARG_TOKEN_ID: &str = "token_id";


#[no_mangle]
pub extern "C" fn mint() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
        .into_hash()
        .map(|hash| ContractHash::new(hash))
        .unwrap();

    let token_owner = runtime::get_named_arg::<Key>(ARG_TOKEN_OWNER);
    let token_metadata: String = runtime::get_named_arg(ARG_TOKEN_META_DATA);

    let (collection_name, owned_tokens_dictionary_key,_token_id_string ) = runtime::call_contract::<(String, Key, String)>(
        nft_contract_hash,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_OWNER => token_owner,
            ARG_TOKEN_META_DATA => token_metadata,
        },
    );

    runtime::put_key(&collection_name, owned_tokens_dictionary_key)
}

#[no_mangle]
pub extern "C" fn transfer() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
        .into_hash()
        .map(|hash| ContractHash::new(hash))
        .unwrap();

    let token_id = runtime::get_named_arg::<u64>(ARG_TOKEN_ID);
    let from_token_owner = runtime::get_named_arg::<Key>(ARG_SOURCE_KEY);
    let target_token_owner = runtime::get_named_arg::<Key>(ARG_TARGET_KEY);

    let (collection_name, owned_tokens_dictionary_key) = runtime::call_contract::<(String, Key)>(
        nft_contract_hash,
        ENTRY_POINT_TRANSFER,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_SOURCE_KEY => from_token_owner,
            ARG_TARGET_KEY => target_token_owner
        }
    );

    runtime::put_key(&collection_name, owned_tokens_dictionary_key)
}

#[no_mangle]
pub extern "C" fn burn() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
        .into_hash()
        .map(|hash| ContractHash::new(hash))
        .unwrap();

    let token_id = runtime::get_named_arg::<u64>(ARG_TOKEN_ID);

    runtime::call_contract::<()>(
        nft_contract_hash,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => token_id
        }
    )
}


#[no_mangle]
pub extern "C" fn metadata() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
        .into_hash()
        .map(|hash| ContractHash::new(hash))
        .unwrap();

    let token_id = runtime::get_named_arg::<u64>(ARG_TOKEN_ID);

    let metadata = runtime::call_contract::<String>(
        nft_contract_hash,
        ENTRY_POINT_METADATA,
        runtime_args! {
            ARG_TOKEN_ID => token_id
        }
    );

    runtime::put_key("metadata", storage::new_uref(metadata).into());
}


fn install_minting_contract() -> (ContractHash, ContractVersion) {
    let mint_entry_point = EntryPoint::new(
        ENTRY_POINT_MINT,
        vec![
            Parameter::new(ARG_TOKEN_META_DATA, CLType::Key),
            Parameter::new(ARG_TOKEN_OWNER, CLType::Key),
            Parameter::new(ARG_TOKEN_META_DATA, CLType::String),
        ],
        CLType::Unit,
            EntryPointAccess::Public,
        EntryPointType::Session,
    );

    let transfer_entry_point = EntryPoint::new(
        ENTRY_POINT_TRANSFER,
        vec![
            Parameter::new(ARG_TOKEN_ID, CLType::U64),
            Parameter::new(ARG_SOURCE_KEY, CLType::Key),
            Parameter::new(ARG_TARGET_KEY, CLType::Key),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let burn_entry_point = EntryPoint::new(
        ENTRY_POINT_BURN,
        vec![Parameter::new(ARG_TOKEN_ID, CLType::U64)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let metadata_entry_point = EntryPoint::new(
        ENTRY_POINT_METADATA,
        vec![Parameter::new(ARG_TOKEN_ID, CLType::U64)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(mint_entry_point);
    entry_points.add_entry_point(transfer_entry_point);
    entry_points.add_entry_point(burn_entry_point);
    entry_points.add_entry_point(metadata_entry_point);

    let named_keys = {
        let mut named_keys = NamedKeys::new();
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
    let (contract_hash, contract_version) = install_minting_contract();

    runtime::put_key(CONTRACT_NAME, contract_hash.into());
    runtime::put_key(CONTRACT_VERSION, storage::new_uref(contract_version).into());
}

