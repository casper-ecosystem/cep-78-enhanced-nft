#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;
use alloc::string::String;

use casper_contract::contract_api::{runtime, storage};
use casper_types::{runtime_args, ContractHash, Key, RuntimeArgs};

const ENTRY_POINT_BALANCE_OF: &str = "balance_of";
const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_TOKEN_OWNER: &str = "token_owner";
const ARG_KEY_NAME: &str = "key_name";

#[no_mangle]
pub extern "C" fn call() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
        .into_hash()
        .map(|hash| ContractHash::new(hash))
        .unwrap();
    let key_name: String = runtime::get_named_arg(ARG_KEY_NAME);
    let token_owner: Key = runtime::get_named_arg(ARG_TOKEN_OWNER);

    let balance = runtime::call_contract::<u64>(
        nft_contract_hash,
        ENTRY_POINT_BALANCE_OF,
        runtime_args! {
            ARG_TOKEN_OWNER => token_owner,
        },
    );
    runtime::put_key(&key_name, storage::new_uref(balance).into());
}
