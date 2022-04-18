#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;
use alloc::string::String;
use casper_contract::contract_api::{runtime, storage};
use casper_types::{runtime_args, ContractHash, Key, RuntimeArgs, U256};

const ENTRY_POINT_GET_APPROVED: &str = "get_approved";
const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_KEY_NAME: &str = "key_name";
const ARG_TOKEN_ID: &str = "token_id";

// This session code is used for entrypoints with a return value.
// This session code calls the entrypoint with the specified entrypoint_name and stores the return value
// under the current context so that it can later be queried. See also the function support::call_entry_point_with_ret
#[no_mangle]
pub extern "C" fn call() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg(ARG_NFT_CONTRACT_HASH);
    let key_name: String = runtime::get_named_arg(ARG_KEY_NAME);
    let token_id = runtime::get_named_arg::<U256>(ARG_TOKEN_ID);

    let maybe_operator = runtime::call_contract::<Option<Key>>(
        nft_contract_hash,
        ENTRY_POINT_GET_APPROVED,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
        },
    );
    runtime::put_key(&key_name, storage::new_uref(maybe_operator).into());
}
