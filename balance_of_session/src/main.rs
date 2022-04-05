#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;
use alloc::string::String;
use casper_contract::contract_api::{runtime, storage};
use casper_types::{runtime_args, ContractHash, Key, PublicKey, RuntimeArgs, URef, U256};

const ENTRY_POINT_BALANCE_OF: &str = "balance_of";
const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_TOKEN_OWNER: &str = "token_owner";
const ARG_TOKEN_META_DATA: &str = "token_meta_data";

#[no_mangle]
pub extern "C" fn call() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg(ARG_NFT_CONTRACT_HASH);
    let token_owner: Key = runtime::get_named_arg(ARG_TOKEN_OWNER);
    let token_metadata: String = runtime::get_named_arg(ARG_TOKEN_META_DATA);

    let balance = runtime::call_contract::<U256>(
        nft_contract_hash,
        ENTRY_POINT_BALANCE_OF,
        runtime_args! {
            ARG_TOKEN_OWNER => token_owner,
            ARG_TOKEN_META_DATA => token_metadata
        },
    );

    runtime::put_key("balance_of", storage::new_uref(balance).into());
}
