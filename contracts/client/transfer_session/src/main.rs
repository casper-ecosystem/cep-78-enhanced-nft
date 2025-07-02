#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

use casper_contract::{
    contract_api::runtime, ext_ffi::casper_get_named_arg_size, unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{api_error::result_from, runtime_args, AddressableEntityHash, ApiError, Key};

const ENTRY_POINT_TRANSFER: &str = "transfer";

const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_TOKEN_ID: &str = "token_id";
const ARG_TOKEN_HASH: &str = "token_hash";
const ARG_TARGET_KEY: &str = "target_key";
const ARG_SOURCE_KEY: &str = "source_key";

#[no_mangle]
pub extern "C" fn call() {
    let nft_contract_hash: AddressableEntityHash =
        runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
            .into_entity_hash()
            .unwrap_or_revert_with(ApiError::User(1601));

    let source_key: Key = runtime::get_named_arg(ARG_SOURCE_KEY);
    let target_key: Key = runtime::get_named_arg(ARG_TARGET_KEY);

    let mut token_hash: String = String::new();
    if let Some(arg_size) = get_named_arg_size(ARG_TOKEN_HASH) {
        if arg_size > 0 {
            token_hash = runtime::get_named_arg::<String>(ARG_TOKEN_HASH);
        }
    }

    let (receipt_name, owned_tokens_dictionary_key) = if token_hash.is_empty() {
        let token_id: u64 = runtime::get_named_arg(ARG_TOKEN_ID);
        runtime::call_contract::<(String, Key)>(
            nft_contract_hash.into(),
            ENTRY_POINT_TRANSFER,
            runtime_args! {
                ARG_TOKEN_ID => token_id,
                ARG_TARGET_KEY => target_key,
                ARG_SOURCE_KEY => source_key
            },
        )
    } else {
        let token_hash: String = runtime::get_named_arg(ARG_TOKEN_HASH);
        runtime::call_contract::<(String, Key)>(
            nft_contract_hash.into(),
            ENTRY_POINT_TRANSFER,
            runtime_args! {
                ARG_TOKEN_HASH => token_hash,
                ARG_TARGET_KEY => target_key,
                ARG_SOURCE_KEY => source_key
            },
        )
    };

    runtime::put_key(&receipt_name, owned_tokens_dictionary_key)
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
