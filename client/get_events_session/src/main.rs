#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use casper_contract::contract_api::{runtime, storage};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::{ContractHash, Key, runtime_args, RuntimeArgs};

const ENTRY_POINT_GET_TOKEN_EVENTS: &str = "get_token_events";

const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const ARG_IS_HASH_IDENTIFIER_MODE: &str = "is_hash_identifier_mode";
const ARG_TOKEN_ID: &str = "token_id";
const ARG_TOKEN_HASH: &str = "token_hash";
const ARG_STARTING_EVENT_ID: &str = "starting_event_id";
const ARG_LAST_EVENT_ID: &str = "last_event_id";
const ARG_ALL_EVENTS: &str = "all_events";

fn get_token_identifier_runtime_args() -> RuntimeArgs {
    if runtime::get_named_arg::<bool>(ARG_IS_HASH_IDENTIFIER_MODE) {
        let token_hash: String = runtime::get_named_arg(ARG_TOKEN_HASH);
        runtime_args! {
            ARG_TOKEN_HASH => token_hash
        }
    } else {
        let token_id: u64 = runtime::get_named_arg(ARG_TOKEN_ID);
        runtime_args! {
            ARG_TOKEN_ID => token_id,
        }
    }
}


#[no_mangle]
pub extern "C" fn call() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_HASH)
        .into_hash()
        .map(|hash| ContractHash::new(hash))
        .unwrap();

    let all_events = runtime::get_named_arg::<bool>(ARG_ALL_EVENTS);
    let starting_event_index = runtime::get_named_arg::<u64>(ARG_STARTING_EVENT_ID);

    let mut get_token_events_args = get_token_identifier_runtime_args();
    get_token_events_args.insert(ARG_STARTING_EVENT_ID.to_string(), starting_event_index)
        .unwrap_or_revert();
    if !all_events {
        let last_event_index = runtime::get_named_arg::<u64>(ARG_LAST_EVENT_ID);
        get_token_events_args.insert(ARG_LAST_EVENT_ID.to_string(), last_event_index)
            .unwrap_or_revert();
    }

    let (receipt_name,events): (String,Vec<String>) = runtime::call_contract(nft_contract_hash, ENTRY_POINT_GET_TOKEN_EVENTS, get_token_events_args);
    let events_uref = storage::new_uref(events);
    runtime::put_key(
        &receipt_name,
        events_uref.into()
    );
}