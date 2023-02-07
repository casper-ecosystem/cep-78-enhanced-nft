use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
};
use casper_contract::{
    contract_api::{
        runtime,
        storage::{dictionary_get, dictionary_put, new_dictionary},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::Key;

use crate::{
    constants::{CEP78_PREFIX, HASH_KEY_NAME_PREFIX},
    error::NFTCoreError,
    modalities::TokenIdentifier,
    utils::{self, get_stored_value_with_user_errors},
};

pub(crate) enum CEP47Event {
    Mint {
        recipient: Key,
        token_id: TokenIdentifier,
    },
    Burn {
        owner: Key,
        token_id: TokenIdentifier,
    },
    Approve {
        owner: Key,
        spender: Key,
        token_id: TokenIdentifier,
    },
    Transfer {
        sender: Key,
        recipient: Key,
        token_id: TokenIdentifier,
    },
    MetadataUpdate {
        token_id: TokenIdentifier,
    },
}

pub(crate) fn record_event_dictionary(event: &CEP47Event) {
    let collection_name: String = get_stored_value_with_user_errors(
        crate::constants::COLLECTION_NAME,
        NFTCoreError::MissingCollectionName,
        NFTCoreError::InvalidCollectionName,
    );

    let package = utils::get_stored_value_with_user_errors::<String>(
        &format!("{CEP78_PREFIX}{collection_name}"),
        NFTCoreError::MissingCep78PackageHash,
        NFTCoreError::InvalidCep78InvalidHash,
    );

    let event: BTreeMap<&str, String> = match event {
        CEP47Event::Mint {
            recipient,
            token_id,
        } => {
            let mut event = BTreeMap::new();
            event.insert(HASH_KEY_NAME_PREFIX, package);
            event.insert("event_type", "Mint".to_string());
            event.insert("recipient", recipient.to_string());
            event.insert("token_id", token_id.to_string());
            event
        }
        CEP47Event::Burn { owner, token_id } => {
            let mut event = BTreeMap::new();
            event.insert(HASH_KEY_NAME_PREFIX, package);
            event.insert("event_type", "Burn".to_string());
            event.insert("owner", owner.to_string());
            event.insert("token_id", token_id.to_string());
            event
        }
        CEP47Event::Approve {
            owner,
            spender,
            token_id,
        } => {
            let mut event = BTreeMap::new();
            event.insert(HASH_KEY_NAME_PREFIX, package);
            event.insert("event_type", "Approve".to_string());
            event.insert("owner", owner.to_string());
            event.insert("spender", spender.to_string());
            event.insert("token_id", token_id.to_string());
            event
        }
        CEP47Event::Transfer {
            sender,
            recipient,
            token_id,
        } => {
            let mut event = BTreeMap::new();
            event.insert(HASH_KEY_NAME_PREFIX, package);
            event.insert("event_type", "Transfer".to_string());
            event.insert("sender", sender.to_string());
            event.insert("recipient", recipient.to_string());
            event.insert("token_id", token_id.to_string());
            event
        }
        CEP47Event::MetadataUpdate { token_id } => {
            let mut event = BTreeMap::new();
            event.insert(HASH_KEY_NAME_PREFIX, package);
            event.insert("event_type", "MetadataUpdate".to_string());
            event.insert("token_id", token_id.to_string());
            event
        }
    };
    let dictionary_uref = match runtime::get_key("events") {
        Some(dict_uref) => dict_uref.into_uref().unwrap_or_revert(),
        None => new_dictionary("events").unwrap_or_revert(),
    };
    let len = dictionary_get(dictionary_uref, "len")
        .unwrap_or_revert()
        .unwrap_or(0_u64);
    dictionary_put(dictionary_uref, &len.to_string(), event);
    dictionary_put(dictionary_uref, "len", len + 1);
}
