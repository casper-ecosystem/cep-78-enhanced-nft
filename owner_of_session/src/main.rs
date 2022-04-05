#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;
use alloc::string::String;
use casper_contract::contract_api::{runtime, storage};
use casper_types::{runtime_args, ContractHash, Key, PublicKey, RuntimeArgs, URef, U256};

const ENTRY_POINT_OWNER_OF: &str = "owner_of";
const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_TOKEN_ID: &str = "token_id";

#[no_mangle]
pub extern "C" fn call() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg(ARG_NFT_CONTRACT_HASH);
    let token_id: U256 = runtime::get_named_arg(ARG_TOKEN_ID);

    let owner = runtime::call_contract::<Key>(
        nft_contract_hash,
        ENTRY_POINT_OWNER_OF,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
        },
    );

    runtime::put_key("owner_of", storage::new_uref(owner).into());
}
