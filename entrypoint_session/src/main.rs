#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;
use alloc::string::String;
use casper_contract::contract_api::{runtime, storage};
use casper_types::{runtime_args, ContractHash, Key, RuntimeArgs, U256};

const ARG_ENTRY_POINT_NAME: &str = "entry_point_name";
const ENTRY_POINT_MINT: &str = "mint";
const ENTRY_POINT_BALANCE_OF: &str = "balance_of";
const ENTRY_POINT_OWNER_OF: &str = "owner_of";
const ENTRY_POINT_GET_APPROVED: &str = "get_approved";

const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_TOKEN_OWNER: &str = "token_owner";
const ARG_TOKEN_ID: &str = "token_id";
const ARG_TOKEN_META_DATA: &str = "token_meta_data";

const OWNED_TOKENS_DICTIONARY_KEY: &str = "owned_tokens_dictionary_key";

// This session code is used for entrypoints with a return value.
// This session code calls the entrypoint with the specified entrypoint_name and stores the return value
// under the current context so that it can later be queried. See also the function support::call_entry_point_with_ret
#[no_mangle]
pub extern "C" fn call() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg(ARG_NFT_CONTRACT_HASH);
    let entry_point_name = runtime::get_named_arg::<String>(ARG_ENTRY_POINT_NAME);
    let entry_point_name = entry_point_name.as_str();

    match entry_point_name {
        ENTRY_POINT_BALANCE_OF => {
            let token_owner: Key = runtime::get_named_arg(ARG_TOKEN_OWNER);
            let balance = runtime::call_contract::<U256>(
                nft_contract_hash,
                ENTRY_POINT_BALANCE_OF,
                runtime_args! {
                    ARG_TOKEN_OWNER => token_owner,
                },
            );

            runtime::put_key(ENTRY_POINT_BALANCE_OF, storage::new_uref(balance).into());
        }
        ENTRY_POINT_OWNER_OF => {
            let token_id: U256 = runtime::get_named_arg(ARG_TOKEN_ID);
            let owner = runtime::call_contract::<Key>(
                nft_contract_hash,
                ENTRY_POINT_OWNER_OF,
                runtime_args! {
                    ARG_TOKEN_ID => token_id,
                },
            );

            runtime::put_key(entry_point_name, storage::new_uref(owner).into());
        }
        ENTRY_POINT_MINT => {
            let token_owner = runtime::get_named_arg::<Key>(ARG_TOKEN_OWNER);
            let token_metadata: String = runtime::get_named_arg(ARG_TOKEN_META_DATA);

            let owned_tokens_dictionary_key = runtime::call_contract::<Key>(
                nft_contract_hash,
                ENTRY_POINT_MINT,
                runtime_args! {
                    ARG_TOKEN_OWNER => token_owner,
                    ARG_TOKEN_META_DATA => token_metadata
                },
            );
            runtime::put_key(OWNED_TOKENS_DICTIONARY_KEY, owned_tokens_dictionary_key);
        }
        ENTRY_POINT_GET_APPROVED => {
            let token_id = runtime::get_named_arg::<U256>(ARG_TOKEN_ID);
            let maybe_operator = runtime::call_contract::<Option<Key>>(
                nft_contract_hash,
                ENTRY_POINT_GET_APPROVED,
                runtime_args! {
                    ARG_TOKEN_ID => token_id,
                },
            );
            runtime::put_key(entry_point_name, storage::new_uref(maybe_operator).into());
        }
        _ => { //runtime::revert(NFTCoreError::InvalidEntryPoint)
        }
    }
}
