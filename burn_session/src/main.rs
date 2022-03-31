#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use casper_contract::contract_api::runtime;
use casper_types::{runtime_args, ContractHash, PublicKey, RuntimeArgs, URef};

const ENTRY_POINT_MINT: &str = "mint";

const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_TOKEN_OWNER: &str = "token_owner";
const ARG_TOKEN_META_DATA: &str = "token_meta_data";

#[no_mangle]
pub extern "C" fn call() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg(ARG_NFT_CONTRACT_HASH);
    let token_owner: PublicKey = runtime::get_named_arg(ARG_TOKEN_OWNER);
    let token_metadata: String = runtime::get_named_arg(ARG_TOKEN_META_DATA);

    let owned_tokens_uref = runtime::call_contract::<URef>(
        nft_contract_hash,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_OWNER => token_owner,
            ARG_TOKEN_META_DATA => token_metadata
        },
    );

    let nft_named_key = format!("nft-contract-{}", nft_contract_hash);
    runtime::put_key(&nft_named_key, owned_tokens_uref.into())
}
