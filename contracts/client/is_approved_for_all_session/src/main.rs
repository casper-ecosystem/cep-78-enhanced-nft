#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;
use alloc::string::String;

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{runtime_args, AddressableEntityHash, ApiError, Key};

const ENTRY_POINT_IS_APPROVED_FOR_ALL: &str = "is_approved_for_all";
const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_KEY_NAME: &str = "key_name";
const ARG_TOKEN_OWNER: &str = "token_owner";
const ARG_OPERATOR: &str = "operator";

#[no_mangle]
pub extern "C" fn call() {
    let nft_contract_hash: AddressableEntityHash =
        runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
            .into_entity_hash()
            .unwrap_or_revert_with(ApiError::User(1301));

    let key_name: String = runtime::get_named_arg(ARG_KEY_NAME);
    let owner = runtime::get_named_arg::<Key>(ARG_TOKEN_OWNER);
    let operator = runtime::get_named_arg::<Key>(ARG_OPERATOR);

    let maybe_operator = runtime::call_contract::<bool>(
        nft_contract_hash.into(),
        ENTRY_POINT_IS_APPROVED_FOR_ALL,
        runtime_args! {
            ARG_TOKEN_OWNER => owner,
            ARG_OPERATOR => operator,
        },
    );
    runtime::put_key(&key_name, storage::new_uref(maybe_operator).into());
}
