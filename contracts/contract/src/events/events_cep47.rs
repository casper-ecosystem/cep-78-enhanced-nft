use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::ApiError;

use crate::{
    constants::{
        BURNER, EVENTS, EVENT_TYPE, OPERATOR, OWNER, PREFIX_CEP78, PREFIX_HASH_KEY_NAME, RECIPIENT,
        SENDER, SPENDER, TOKEN_ID,
    },
    error::NFTCoreError,
    utils,
};

use super::Event;

pub fn emit_cep47(event: Event) {
    let collection_name: String = utils::get_stored_value_with_user_errors(
        crate::constants::COLLECTION_NAME,
        NFTCoreError::MissingCollectionName,
        NFTCoreError::InvalidCollectionName,
    );

    let package = utils::get_stored_value_with_user_errors::<String>(
        &format!("{PREFIX_CEP78}_{collection_name}"),
        NFTCoreError::MissingCep78PackageHash,
        NFTCoreError::InvalidCep78InvalidHash,
    );

    let event: BTreeMap<&str, String> = match event {
        Event::Mint(event_in) => {
            let mut event = BTreeMap::new();
            event.insert(PREFIX_HASH_KEY_NAME, package);
            event.insert(EVENT_TYPE, "Mint".to_string());
            event.insert(RECIPIENT, event_in.recipient.to_string());
            event.insert(TOKEN_ID, event_in.token_id);
            event
        }
        Event::Burn(event_in) => {
            let mut event = BTreeMap::new();
            event.insert(PREFIX_HASH_KEY_NAME, package);
            event.insert(EVENT_TYPE, "Burn".to_string());
            event.insert(OWNER, event_in.owner.to_string());
            event.insert(TOKEN_ID, event_in.token_id);
            event.insert(BURNER, event_in.burner.to_string());
            event
        }
        Event::Approval(event_in) => {
            let mut event = BTreeMap::new();
            event.insert(PREFIX_HASH_KEY_NAME, package);
            event.insert(EVENT_TYPE, "Approve".to_string());
            event.insert(OWNER, event_in.owner.to_string());
            event.insert(SPENDER, event_in.spender.to_string());
            event.insert(TOKEN_ID, event_in.token_id);
            event
        }
        Event::ApprovalRevoked(event_in) => {
            let mut event = BTreeMap::new();
            event.insert(PREFIX_HASH_KEY_NAME, package);
            event.insert(EVENT_TYPE, "ApprovalRevoked".to_string());
            event.insert(OWNER, event_in.owner.to_string());
            event.insert(TOKEN_ID, event_in.token_id);
            event
        }
        Event::ApprovalForAll(event_in) => {
            let mut event = BTreeMap::new();
            event.insert(PREFIX_HASH_KEY_NAME, package);
            event.insert(EVENT_TYPE, "ApprovalForAll".to_string());
            event.insert(OWNER, event_in.owner.to_string());
            event.insert(OPERATOR, event_in.operator.to_string());
            event
        }
        Event::RevokedForAll(event_in) => {
            let mut event = BTreeMap::new();
            event.insert(PREFIX_HASH_KEY_NAME, package);
            event.insert(EVENT_TYPE, "RevokedForAll".to_string());
            event.insert(OWNER, event_in.owner.to_string());
            event.insert(OPERATOR, event_in.operator.to_string());
            event
        }
        Event::Transfer(event_in) => {
            let mut event = BTreeMap::new();
            event.insert(PREFIX_HASH_KEY_NAME, package);
            event.insert(EVENT_TYPE, "Transfer".to_string());
            event.insert(
                SENDER,
                match event_in.spender {
                    Some(spender) => spender.to_string(),
                    None => event_in.owner.to_string(),
                },
            );
            event.insert(RECIPIENT, event_in.recipient.to_string());
            event.insert(TOKEN_ID, event_in.token_id);
            event
        }
        Event::MetadataUpdated(event_in) => {
            let mut event = BTreeMap::new();
            event.insert(PREFIX_HASH_KEY_NAME, package);
            event.insert(EVENT_TYPE, "MetadataUpdate".to_string());
            event.insert(TOKEN_ID, event_in.token_id);
            event
        }
        Event::Migration(_) => {
            let mut event = BTreeMap::new();
            event.insert(PREFIX_HASH_KEY_NAME, package);
            event.insert(EVENT_TYPE, "Migration".to_string());
            event
        }
        Event::VariablesSet(_) => {
            let mut event = BTreeMap::new();
            event.insert(PREFIX_HASH_KEY_NAME, package);
            event.insert(EVENT_TYPE, "VariablesSet".to_string());
            event
        }
    };
    let dictionary_uref = match runtime::get_key(EVENTS) {
        Some(dict_uref) => dict_uref
            .into_uref()
            .unwrap_or_revert_with(ApiError::User(334)),
        None => storage::new_dictionary(EVENTS).unwrap_or_revert_with(ApiError::User(335)),
    };
    let len = storage::dictionary_get(dictionary_uref, "len")
        .unwrap_or_revert_with(ApiError::User(336))
        .unwrap_or(0_u64);
    storage::dictionary_put(dictionary_uref, &len.to_string(), event);
    storage::dictionary_put(dictionary_uref, "len", len + 1);
}
