#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

use alloc::format;
use alloc::string::String;

use casper_contract::contract_api::{runtime};
use casper_types::{runtime_args, ContractHash, Key, RuntimeArgs, URef};

const ENTRY_POINT_MINT: &str = "mint";
const ENTRY_POINT_REGISTER_OWNER: &str = "register_owner";

const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_TOKEN_OWNER: &str = "token_owner";
const ARG_TOKEN_META_DATA: &str = "token_meta_data";
const ARG_COLLECTION_NAME: &str = "collection_name";


#[no_mangle]
pub extern "C" fn call() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
        .into_hash()
        .map(|hash| ContractHash::new(hash))
        .unwrap();

    let token_owner = runtime::get_named_arg::<Key>(ARG_TOKEN_OWNER);
    let token_metadata: String = runtime::get_named_arg(ARG_TOKEN_META_DATA);
    let collection_name: String = runtime::get_named_arg(ARG_COLLECTION_NAME);

    if runtime::get_key(&format!("cep78_{}", collection_name)).is_none() {
        let (register_name, package_uref) = runtime::call_contract::<(String, URef)>(nft_contract_hash, ENTRY_POINT_REGISTER_OWNER, runtime_args! {
        ARG_TOKEN_OWNER => token_owner,
         });
        runtime::put_key(&register_name, package_uref.into())
    }

    let (receipt_name, owned_tokens_dictionary_key, _token_id_string) = runtime::call_contract::<(String, Key, String)>(
        nft_contract_hash,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_OWNER => token_owner,
            ARG_TOKEN_META_DATA => token_metadata,
        },
    );

    runtime::put_key(&receipt_name, owned_tokens_dictionary_key)
}
