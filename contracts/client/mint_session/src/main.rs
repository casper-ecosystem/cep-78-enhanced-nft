#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

use alloc::string::String;
use casper_contract::{contract_api::runtime, ext_ffi, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{api_error, runtime_args, AddressableEntityHash, ApiError, Key, URef};

const ENTRY_POINT_MINT: &str = "mint";
const ENTRY_POINT_REGISTER_OWNER: &str = "register_owner";

const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_TOKEN_OWNER: &str = "token_owner";
const ARG_TOKEN_META_DATA: &str = "token_meta_data";
const ARG_TOKEN_HASH: &str = "token_hash";

#[no_mangle]
pub extern "C" fn call() {
    let nft_contract_hash: AddressableEntityHash =
        runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
            .into_entity_hash()
            .unwrap_or_revert_with(ApiError::User(1401));

    let token_owner = runtime::get_named_arg::<Key>(ARG_TOKEN_OWNER);
    let token_metadata: String = runtime::get_named_arg(ARG_TOKEN_META_DATA);
    let mut token_hash: String = String::new();
    if let Some(arg_size) = get_named_arg_size(ARG_TOKEN_HASH) {
        if arg_size > 0 {
            token_hash = runtime::get_named_arg::<String>(ARG_TOKEN_HASH);
        }
    }

    let (register_name, package_uref) = runtime::call_contract::<(String, URef)>(
        nft_contract_hash.into(),
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_owner
        },
    );
    runtime::put_key(&register_name, package_uref.into());

    let (receipt_name, owned_tokens_dictionary_key, _token_id_string) =
        runtime::call_contract::<(String, Key, String)>(
            nft_contract_hash.into(),
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_HASH => token_hash,
                ARG_TOKEN_OWNER => token_owner,
                ARG_TOKEN_META_DATA => token_metadata,
            },
        );

    runtime::put_key(&receipt_name, owned_tokens_dictionary_key);
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
