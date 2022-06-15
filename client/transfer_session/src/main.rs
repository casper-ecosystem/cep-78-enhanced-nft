#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

use casper_contract::contract_api::runtime;
use casper_types::{ContractHash, Key, runtime_args, RuntimeArgs};

const ENTRY_POINT_TRANSFER: &str = "transfer";

const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_IS_HASH_IDENTIFIER_MODE: &str = "is_hash_identifier_mode";
const ARG_TOKEN_ID: &str = "token_id";
const ARG_TOKEN_HASH: &str = "token_hash";
const ARG_TARGET_KEY: &str = "target_key";
const ARG_SOURCE_KEY: &str = "source_key";

#[no_mangle]
pub extern "C" fn call() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
        .into_hash()
        .map(|hash| ContractHash::new(hash))
        .unwrap();

    let source_key: Key = runtime::get_named_arg(ARG_SOURCE_KEY);
    let target_key: Key = runtime::get_named_arg(ARG_TARGET_KEY);


    let (receipt_name, owned_tokens_dictionary_key, ) = if !runtime::get_named_arg::<bool>(ARG_IS_HASH_IDENTIFIER_MODE) {
        let token_id: u64 = runtime::get_named_arg(ARG_TOKEN_ID);
        runtime::call_contract::<(String, Key)>(
            nft_contract_hash,
            ENTRY_POINT_TRANSFER,
            runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_TARGET_KEY => target_key,
            ARG_SOURCE_KEY => source_key
        })
    } else {
        let token_hash: String = runtime::get_named_arg(ARG_TOKEN_HASH);
        runtime::call_contract::<(String, Key)>(
            nft_contract_hash,
            ENTRY_POINT_TRANSFER,
            runtime_args! {
            ARG_TOKEN_HASH => token_hash,
            ARG_TARGET_KEY => target_key,
            ARG_SOURCE_KEY => source_key
        })
    };

    runtime::put_key(&receipt_name, owned_tokens_dictionary_key)
}