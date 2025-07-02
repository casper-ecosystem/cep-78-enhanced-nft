#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;
use alloc::string::String;

use casper_contract::{
    contract_api::{runtime, storage},
    ext_ffi::casper_get_named_arg_size,
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{api_error::result_from, runtime_args, AddressableEntityHash, ApiError, Key};

const ENTRY_POINT_GET_APPROVED: &str = "get_approved";
const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_KEY_NAME: &str = "key_name";
const ARG_TOKEN_ID: &str = "token_id";
const ARG_TOKEN_HASH: &str = "token_hash";

#[no_mangle]
pub extern "C" fn call() {
    let nft_contract_hash: AddressableEntityHash =
        runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
            .into_entity_hash()
            .unwrap_or_revert_with(ApiError::User(1201));
    let key_name: String = runtime::get_named_arg(ARG_KEY_NAME);

    let mut token_hash: String = String::new();
    if let Some(arg_size) = get_named_arg_size(ARG_TOKEN_HASH) {
        if arg_size > 0 {
            token_hash = runtime::get_named_arg::<String>(ARG_TOKEN_HASH);
        }
    }

    let maybe_approved_account = if !token_hash.is_empty() {
        let token_hash = runtime::get_named_arg::<String>(ARG_TOKEN_HASH);
        runtime::call_contract::<Option<Key>>(
            nft_contract_hash.into(),
            ENTRY_POINT_GET_APPROVED,
            runtime_args! {
                ARG_TOKEN_HASH => token_hash,
            },
        )
    } else {
        let token_id = runtime::get_named_arg::<u64>(ARG_TOKEN_ID);
        runtime::call_contract::<Option<Key>>(
            nft_contract_hash.into(),
            ENTRY_POINT_GET_APPROVED,
            runtime_args! {
                ARG_TOKEN_ID => token_id,
            },
        )
    };
    runtime::put_key(&key_name, storage::new_uref(maybe_approved_account).into());
}

fn get_named_arg_size(name: &str) -> Option<usize> {
    let mut arg_size: usize = 0;
    let ret = unsafe {
        casper_get_named_arg_size(
            name.as_bytes().as_ptr(),
            name.len(),
            &mut arg_size as *mut usize,
        )
    };
    match result_from(ret) {
        Ok(_) => Some(arg_size),
        Err(ApiError::MissingArgument) => None,
        Err(e) => runtime::revert(e),
    }
}
