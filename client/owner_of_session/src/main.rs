#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;
use alloc::string::String;

use casper_contract::contract_api::{runtime, storage};
use casper_types::{runtime_args, ContractHash, Key, RuntimeArgs};

const ENTRY_POINT_OWNER_OF: &str = "owner_of";
const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_KEY_NAME: &str = "key_name";
const ARG_TOKEN_ID: &str = "token_id";
const ARG_TOKEN_HASH: &str = "token_hash";
const ARG_IS_HASH_IDENTIFIER_MODE: &str = "is_hash_identifier_mode";

#[no_mangle]
pub extern "C" fn call() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
        .into_hash()
        .map(|hash| ContractHash::new(hash))
        .unwrap();
    let key_name: String = runtime::get_named_arg(ARG_KEY_NAME);

    let owner = if runtime::get_named_arg(ARG_IS_HASH_IDENTIFIER_MODE) {
        let token_hash = runtime::get_named_arg::<String>(ARG_TOKEN_HASH);
        runtime::call_contract::<Key>(
            nft_contract_hash,
            ENTRY_POINT_OWNER_OF,
            runtime_args! {
            ARG_TOKEN_HASH => token_hash,
        },)
    } else {
        let token_id = runtime::get_named_arg::<u64>(ARG_TOKEN_ID);
        runtime::call_contract::<Key>(
            nft_contract_hash,
            ENTRY_POINT_OWNER_OF,
            runtime_args! {
            ARG_TOKEN_ID => token_id,
        },)
    };
    runtime::put_key(&key_name, storage::new_uref(owner).into());
}
