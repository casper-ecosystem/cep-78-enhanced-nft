#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

use casper_contract::contract_api::runtime;
use casper_types::{ContractHash, Key, runtime_args, RuntimeArgs};

const ENTRY_POINT_TRANSFER: &str = "transfer";

const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_TOKEN_ID: &str = "token_id";
const ARG_TARGET_KEY: &str = "target_key";
const ARG_SOURCE_KEY: &str = "source_key";

#[no_mangle]
pub extern "C" fn call() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
        .into_hash()
        .map(|hash| ContractHash::new(hash))
        .unwrap();

    let token_id: String = runtime::get_named_arg(ARG_TOKEN_ID);
    let source_key: Key = runtime::get_named_arg(ARG_SOURCE_KEY);
    let target_key: Key = runtime::get_named_arg(ARG_TARGET_KEY);

    let (collection_name, owned_tokens_dictionary_key, ) = runtime::call_contract::<(String, Key)>(
        nft_contract_hash,
        ENTRY_POINT_TRANSFER,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_TARGET_KEY => target_key,
            ARG_SOURCE_KEY => source_key
        },
    );

    runtime::put_key(&collection_name, owned_tokens_dictionary_key)
}