use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{ContractPackageHash, Key, URef};

use crate::{
    constants::{CEP78_PREFIX, HASH_KEY_NAME_1_0_0},
    error::NFTCoreError,
    modalities::TokenIdentifier,
    utils::get_stored_value_with_user_errors,
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
    // ApproveAll{
    //     owner: Key,
    //     spender: Key,
    // },
    Transfer {
        sender: Key,
        recipient: Key,
        token_id: TokenIdentifier,
    },
    MetadataUpdate {
        token_id: TokenIdentifier,
    },
}

pub(crate) fn record_event(event: &CEP47Event) {
    let collection_name: String = get_stored_value_with_user_errors(
        crate::constants::COLLECTION_NAME,
        NFTCoreError::MissingCollectionName,
        NFTCoreError::InvalidCollectionName,
    );

    let package: ContractPackageHash =
        runtime::get_key(&format!("{}{}", CEP78_PREFIX, collection_name))
            .unwrap_or_revert()
            .into_hash()
            .map(ContractPackageHash::new)
            .unwrap();

    let event: BTreeMap<&str, String> = match event {
        CEP47Event::Mint {
            recipient,
            token_id,
        } => {
                let mut event = BTreeMap::new();
                event.insert(HASH_KEY_NAME_1_0_0, package.to_string());
                event.insert("event_type", "Mint".to_string());
                event.insert("recipient", recipient.to_string());
                event.insert("token_id", token_id.to_string());
                event
        }
        CEP47Event::Burn { owner, token_id } => {
                let mut event = BTreeMap::new();
                event.insert(HASH_KEY_NAME_1_0_0, package.to_string());
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
                event.insert(HASH_KEY_NAME_1_0_0, package.to_string());
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
                event.insert(HASH_KEY_NAME_1_0_0, package.to_string());
                event.insert("event_type", "Transfer".to_string());
                event.insert("sender", sender.to_string());
                event.insert("recipient", recipient.to_string());
                event.insert("token_id", token_id.to_string());
                event
        }
        CEP47Event::MetadataUpdate { token_id } => {
            let mut event = BTreeMap::new();
            event.insert(HASH_KEY_NAME_1_0_0, package.to_string());
            event.insert("event_type", "MetadataUpdate".to_string());
            event.insert("token_id", token_id.to_string());
            event
        }
        // CEP47Event::ApproveAll { owner, spender } => {
        //     let mut event = BTreeMap::new();
        //     event.insert(HASH_KEY_NAME_1_0_0, package.to_string());
        //     event.insert("event_type", "ApproveAll".to_string());
        //     event.insert("owner", owner.to_string());
        //     event.insert("spender", spender.to_string());
        //     event
        // },
    };
    let _: URef = storage::new_uref(event);
}
