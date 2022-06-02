#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

use alloc::{format, vec};
use alloc::string::{String, ToString};
use casper_contract::contract_api::{runtime, storage};
use casper_types::{CLType, ContractHash, ContractVersion, EntryPoint, EntryPointAccess, EntryPoints, EntryPointType, Key, Parameter, runtime_args, RuntimeArgs, U256};
use casper_types::contracts::NamedKeys;

const CONTRACT_NAME: &str = "minting_contract_hash";
const CONTRACT_VERSION: &str = "minting_contract_version";
const INSTALLER: &str = "installer";
const HASH_KEY_NAME: &str = "minting_contract_package_hash";
const ACCESS_KEY_NAME: &str = "minting_contract_access_uref";

const ENTRY_POINT_MINT: &str = "mint";
const ENTRY_POINT_TRANSFER: &str = "transfer";

const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_TOKEN_OWNER: &str = "token_owner";
const ARG_TOKEN_META_DATA: &str = "token_meta_data";
const ARG_TOKEN_URI: &str = "token_uri";
const ARG_TO_ACCOUNT_HASH: &str = "to_account_hash";
const ARG_FROM_ACCOUNT_HASH: &str = "from_account_hash";
const ARG_TOKEN_ID: &str = "token_id";


#[no_mangle]
pub extern "C" fn mint() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
        .into_hash()
        .map(|hash| ContractHash::new(hash))
        .unwrap();

    let token_owner = runtime::get_named_arg::<Key>(ARG_TOKEN_OWNER);
    let token_metadata: String = runtime::get_named_arg(ARG_TOKEN_META_DATA);
    let token_uri: String = runtime::get_named_arg(ARG_TOKEN_URI);

    let (owned_tokens_dictionary_key, collection_name) = runtime::call_contract::<(Key, String)>(
        nft_contract_hash,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_OWNER => token_owner,
            ARG_TOKEN_META_DATA => token_metadata,
            ARG_TOKEN_URI =>token_uri,
        },
    );

    let nft_contract_named_key = format!("{}_{}", nft_contract_hash.to_formatted_string(), collection_name);
    runtime::put_key(&nft_contract_named_key, owned_tokens_dictionary_key)
}

#[no_mangle]
pub extern "C" fn transfer() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
        .into_hash()
        .map(|hash| ContractHash::new(hash))
        .unwrap();

    let token_id = runtime::get_named_arg::<U256>(ARG_TOKEN_ID);
    let from_token_owner = runtime::get_named_arg::<Key>(ARG_FROM_ACCOUNT_HASH);
    let target_token_owner = runtime::get_named_arg::<Key>(ARG_TO_ACCOUNT_HASH);

    runtime::call_contract::<()>(
        nft_contract_hash,
        ENTRY_POINT_TRANSFER,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_FROM_ACCOUNT_HASH => from_token_owner,
            ARG_TO_ACCOUNT_HASH => target_token_owner
        }
    );
}



fn install_minting_contract() -> (ContractHash, ContractVersion) {
    let mint_entry_point = EntryPoint::new(
        ENTRY_POINT_MINT,
        vec![
            Parameter::new(ARG_TOKEN_META_DATA, CLType::Key),
            Parameter::new(ARG_TOKEN_OWNER, CLType::Key),
            Parameter::new(ARG_TOKEN_META_DATA, CLType::String),
            Parameter::new(ARG_TOKEN_URI, CLType::String)
        ],
        CLType::Unit,
            EntryPointAccess::Public,
        EntryPointType::Session,
    );

    let transfer_entry_point = EntryPoint::new(
        ENTRY_POINT_TRANSFER,
        vec![
            Parameter::new(ARG_TOKEN_ID, CLType::U256),
            Parameter::new(ARG_FROM_ACCOUNT_HASH, CLType::Key),
            Parameter::new(ARG_TO_ACCOUNT_HASH, CLType::Key),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(mint_entry_point);
    entry_points.add_entry_point(transfer_entry_point);

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

