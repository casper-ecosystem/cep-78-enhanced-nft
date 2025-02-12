#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec,
};
use casper_contract::{
    contract_api::{runtime, storage},
    ext_ffi,
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    api_error,
    contracts::{ContractHash, ContractVersion},
    runtime_args, AddressableEntityHash, ApiError, CLType, EntryPoint, EntryPointAccess,
    EntryPointPayment, EntryPointType, EntryPoints, Key, NamedKeys, PackageHash, Parameter, URef,
};

const CONTRACT_NAME: &str = "minting_contract_hash";
const CONTRACT_VERSION: &str = "minting_contract_version";
const INSTALLER: &str = "installer";
const PACKAGE_HASH_KEY_NAME: &str = "minting_contract_package_hash";
const ACCESS_KEY_NAME: &str = "minting_contract_access_uref";

const ENTRY_POINT_MINT: &str = "mint";
const ENTRY_POINT_TRANSFER: &str = "transfer";
const ENTRY_POINT_BURN: &str = "burn";
const ENTRY_POINT_METADATA: &str = "metadata";
const ENTRY_POINT_REGISTER_OWNER: &str = "register_owner";
const ENTRY_POINT_APPROVE: &str = "approve";
const ENTRY_POINT_REVOKE: &str = "revoke";

const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_TOKEN_OWNER: &str = "token_owner";
const ARG_TOKEN_META_DATA: &str = "token_meta_data";
const ARG_TARGET_KEY: &str = "target_key";
const ARG_SOURCE_KEY: &str = "source_key";
const ARG_SPENDER: &str = "spender";
const ARG_TOKEN_ID: &str = "token_id";
const ARG_TOKEN_HASH: &str = "token_hash";
const ARG_REVERSE_LOOKUP: &str = "reverse_lookup";
const ARG_IS_HASH_IDENTIFIER_MODE: &str = "is_hash_identifier_mode";

#[no_mangle]
pub extern "C" fn mint() {
    let nft_contract_hash: AddressableEntityHash =
        runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
            .into_entity_hash()
            .unwrap_or_revert_with(ApiError::User(1001));

    let token_owner = runtime::get_named_arg::<Key>(ARG_TOKEN_OWNER);
    let token_metadata: String = runtime::get_named_arg(ARG_TOKEN_META_DATA);
    let reverse_lookup_enabled: bool = runtime::get_named_arg(ARG_REVERSE_LOOKUP);
    let mut token_hash: String = String::new();
    if let Some(arg_size) = get_named_arg_size(ARG_TOKEN_HASH) {
        if arg_size > 0 {
            token_hash = runtime::get_named_arg::<String>(ARG_TOKEN_HASH);
        }
    }
    if reverse_lookup_enabled {
        runtime::call_contract::<(String, URef)>(
            nft_contract_hash.into(),
            ENTRY_POINT_REGISTER_OWNER,
            runtime_args! {
                ARG_TOKEN_OWNER => token_owner,
            },
        );

        let (collection_name, owned_tokens_dictionary_key, _token_id_string) =
            runtime::call_contract::<(String, Key, String)>(
                nft_contract_hash.into(),
                ENTRY_POINT_MINT,
                runtime_args! {
                    ARG_TOKEN_HASH => token_hash,
                    ARG_TOKEN_OWNER => token_owner,
                    ARG_TOKEN_META_DATA => token_metadata,
                },
            );

        runtime::put_key(&collection_name, owned_tokens_dictionary_key)
    } else {
        runtime::call_contract::<()>(
            nft_contract_hash.into(),
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_HASH => token_hash,
                ARG_TOKEN_OWNER => token_owner,
                ARG_TOKEN_META_DATA => token_metadata,
            },
        );
    }
}

#[no_mangle]
pub extern "C" fn approve() {
    let nft_contract_hash: AddressableEntityHash =
        runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
            .into_entity_hash()
            .unwrap_or_revert_with(ApiError::User(1002));

    let token_id = runtime::get_named_arg::<u64>(ARG_TOKEN_ID);
    let spender_key = runtime::get_named_arg::<Key>(ARG_SPENDER);

    runtime::call_contract::<()>(
        nft_contract_hash.into(),
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_SPENDER => spender_key
        },
    )
}

#[no_mangle]
pub extern "C" fn revoke() {
    let nft_contract_hash: AddressableEntityHash =
        runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
            .into_entity_hash()
            .unwrap_or_revert_with(ApiError::User(1003));

    let token_id = runtime::get_named_arg::<u64>(ARG_TOKEN_ID);

    runtime::call_contract::<()>(
        nft_contract_hash.into(),
        ENTRY_POINT_REVOKE,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
        },
    )
}

#[no_mangle]
pub extern "C" fn transfer() {
    let nft_contract_hash: AddressableEntityHash =
        runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
            .into_entity_hash()
            .unwrap_or_revert_with(ApiError::User(1004));

    let token_id = runtime::get_named_arg::<u64>(ARG_TOKEN_ID);
    let from_token_owner = runtime::get_named_arg::<Key>(ARG_SOURCE_KEY);
    let target_token_owner = runtime::get_named_arg::<Key>(ARG_TARGET_KEY);

    let (collection_name, owned_tokens_dictionary_key) = runtime::call_contract::<(String, Key)>(
        nft_contract_hash.into(),
        ENTRY_POINT_TRANSFER,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_SOURCE_KEY => from_token_owner,
            ARG_TARGET_KEY => target_token_owner
        },
    );

    runtime::put_key(&collection_name, owned_tokens_dictionary_key)
}

#[no_mangle]
pub extern "C" fn burn() {
    let nft_contract_hash: AddressableEntityHash =
        runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
            .into_entity_hash()
            .unwrap_or_revert_with(ApiError::User(1005));

    let token_id = runtime::get_named_arg::<u64>(ARG_TOKEN_ID);

    runtime::call_contract::<()>(
        nft_contract_hash.into(),
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => token_id
        },
    )
}

#[no_mangle]
pub extern "C" fn metadata() {
    let nft_contract_hash: AddressableEntityHash =
        runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
            .into_entity_hash()
            .unwrap_or_revert_with(ApiError::User(1006));

    let metadata: String = if runtime::get_named_arg::<bool>(ARG_IS_HASH_IDENTIFIER_MODE) {
        let token_hash = runtime::get_named_arg::<String>(ARG_TOKEN_HASH);
        runtime::call_contract::<String>(
            nft_contract_hash.into(),
            ENTRY_POINT_METADATA,
            runtime_args! {
                ARG_TOKEN_HASH => token_hash
            },
        )
    } else {
        let token_id = runtime::get_named_arg::<u64>(ARG_TOKEN_ID);
        runtime::call_contract::<String>(
            nft_contract_hash.into(),
            ENTRY_POINT_METADATA,
            runtime_args! {
                ARG_TOKEN_ID => token_id
            },
        )
    };
    runtime::put_key("metadata", storage::new_uref(metadata).into());
}

#[no_mangle]
pub extern "C" fn register_contract() {
    let nft_contract_hash: AddressableEntityHash =
        runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
            .into_entity_hash()
            .unwrap_or_revert_with(ApiError::User(1007));

    let token_owner = runtime::get_named_arg::<Key>(ARG_TOKEN_OWNER);

    runtime::call_contract::<(String, URef)>(
        nft_contract_hash.into(),
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_owner
        },
    );
}

fn install_minting_contract() -> (ContractHash, ContractVersion) {
    let entry_points = get_entry_points();
    let named_keys = named_keys();
    storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(PACKAGE_HASH_KEY_NAME.to_string()),
        Some(ACCESS_KEY_NAME.to_string()),
        None,
    )
}

fn upgrade_minting_contract(name: &str) -> (ContractHash, ContractVersion) {
    let contract_package_hash: PackageHash = runtime::get_key(name)
        .unwrap_or_revert_with(ApiError::User(1008))
        .into_package_hash()
        .unwrap_or_revert_with(ApiError::User(1009));
    let named_keys = named_keys();
    let entry_points = get_entry_points();
    storage::add_contract_version(
        contract_package_hash.into(),
        entry_points,
        named_keys,
        BTreeMap::new(),
    )
}

fn named_keys() -> NamedKeys {
    let mut named_keys = NamedKeys::new();
    named_keys.insert(INSTALLER.to_string(), runtime::get_caller().into());
    named_keys
}

fn get_entry_points() -> EntryPoints {
    let mint_entry_point = EntryPoint::new(
        ENTRY_POINT_MINT,
        vec![
            Parameter::new(ARG_TOKEN_META_DATA, CLType::Key),
            Parameter::new(ARG_TOKEN_OWNER, CLType::Key),
            Parameter::new(ARG_TOKEN_META_DATA, CLType::String),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Called,
        EntryPointPayment::Caller,
    );

    let approve_entry_point = EntryPoint::new(
        ENTRY_POINT_APPROVE,
        vec![Parameter::new(ARG_SPENDER, CLType::Key)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Called,
        EntryPointPayment::Caller,
    );

    let revoke_entry_point = EntryPoint::new(
        ENTRY_POINT_REVOKE,
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Called,
        EntryPointPayment::Caller,
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
        EntryPointType::Called,
        EntryPointPayment::Caller,
    );

    let burn_entry_point = EntryPoint::new(
        ENTRY_POINT_BURN,
        vec![Parameter::new(ARG_TOKEN_ID, CLType::U64)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Called,
        EntryPointPayment::Caller,
    );

    let metadata_entry_point = EntryPoint::new(
        ENTRY_POINT_METADATA,
        vec![Parameter::new(ARG_TOKEN_ID, CLType::U64)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Called,
        EntryPointPayment::Caller,
    );
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(mint_entry_point);
    entry_points.add_entry_point(transfer_entry_point);
    entry_points.add_entry_point(approve_entry_point);
    entry_points.add_entry_point(revoke_entry_point);
    entry_points.add_entry_point(burn_entry_point);
    entry_points.add_entry_point(metadata_entry_point);
    entry_points
}

#[no_mangle]
pub extern "C" fn call() {
    let contract_name = PACKAGE_HASH_KEY_NAME;
    let (contract_hash, contract_version) = if runtime::get_key(contract_name).is_some() {
        upgrade_minting_contract(contract_name)
    } else {
        install_minting_contract()
    };

    runtime::put_key(
        CONTRACT_NAME,
        Key::contract_entity_key(contract_hash.into()),
    );
    runtime::put_key(CONTRACT_VERSION, storage::new_uref(contract_version).into());
}

fn get_named_arg_size(name: &str) -> Option<usize> {
    let mut arg_size: usize = 0;
    let ret = unsafe {
        ext_ffi::casper_get_named_arg_size(
            name.as_bytes().as_ptr(),
            name.len(),
            &mut arg_size as *mut usize,
        )
    };
    match api_error::result_from(ret) {
        Ok(_) => Some(arg_size),
        Err(ApiError::MissingArgument) => None,
        Err(e) => runtime::revert(e),
    }
}
