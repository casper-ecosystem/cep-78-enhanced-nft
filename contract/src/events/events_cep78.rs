// seed_uref + (token_idx, row_idx)
// named_key under well defined literal which will be same for every CEP-78 to store that seed_uref
// nft_status_changes -> token_idx => status
// nft_status_row_idx_tracker -> token_idx => rox_idx

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use casper_contract::contract_api::runtime;
use core::convert::{TryFrom, TryInto};

use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::bytesrepr::ToBytes;

use crate::{utils, NFTCoreError, TokenIdentifier, EVENTS, EVENT_ID_TRACKER};

#[repr(u8)]
#[derive(PartialEq, Eq)]
pub(crate) enum CEP78Event {
    Mint = 0,
    Transfer = 1,
    Burn = 2,
    Approve = 3,
    MetadataUpdate = 4,
}

impl TryFrom<u8> for CEP78Event {
    type Error = NFTCoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CEP78Event::Mint),
            1 => Ok(CEP78Event::Transfer),
            2 => Ok(CEP78Event::Burn),
            3 => Ok(CEP78Event::Approve),
            4 => Ok(CEP78Event::MetadataUpdate),
            _ => Err(NFTCoreError::InvalidTokenEvent),
        }
    }
}

impl ToString for CEP78Event {
    fn to_string(&self) -> String {
        match self {
            CEP78Event::Mint => "Mint".to_string(),
            CEP78Event::Transfer => "Transfer".to_string(),
            CEP78Event::Burn => "Burn".to_string(),
            CEP78Event::Approve => "Approve".to_string(),
            CEP78Event::MetadataUpdate => "MetadataUpdate".to_string(),
        }
    }
}

fn record_mint_event(token_identifier: TokenIdentifier) -> Result<(), NFTCoreError> {
    // Since mint is the first event to be "recorded" there should be no value present
    // for the event id tracker.
    if utils::get_dictionary_value_from_key::<u64>(
        EVENT_ID_TRACKER,
        &token_identifier.get_dictionary_item_key(),
    )
    .is_some()
    {
        return Err(NFTCoreError::InvalidTokenEventId);
    }
    // Start the event tracking with the first event ID set to 0.
    utils::upsert_dictionary_value_from_key(
        EVENT_ID_TRACKER,
        &token_identifier.get_dictionary_item_key(),
        0u64,
    );
    let event_item_key = get_event_item_key(&token_identifier, 0u64);
    utils::upsert_dictionary_value_from_key(EVENTS, &event_item_key, CEP78Event::Mint as u8);
    Ok(())
}

fn record_transfer_event(token_identifier: TokenIdentifier) -> Result<(), NFTCoreError> {
    store_event(token_identifier, CEP78Event::Transfer)
}

fn record_burn_event(token_identifier: TokenIdentifier) -> Result<(), NFTCoreError> {
    store_event(token_identifier, CEP78Event::Burn)
}

fn record_approve_event(token_identifier: TokenIdentifier) -> Result<(), NFTCoreError> {
    store_event(token_identifier, CEP78Event::Approve)
}

fn store_event(
    token_identifier: TokenIdentifier,
    cep_78_event: CEP78Event,
) -> Result<(), NFTCoreError> {
    let current_event_id = utils::get_dictionary_value_from_key::<u64>(
        EVENT_ID_TRACKER,
        &token_identifier.get_dictionary_item_key(),
    )
    .unwrap_or_revert_with(NFTCoreError::MissingTokenEventId);
    // The Burn event represents the end of the token life cycle, so further
    // updates to the event must be invalid
    if is_last_event_burn(&token_identifier, current_event_id) {
        return Err(NFTCoreError::InvalidTokenEventOrder);
    }
    utils::upsert_dictionary_value_from_key(
        EVENT_ID_TRACKER,
        &token_identifier.get_dictionary_item_key(),
        current_event_id + 1,
    );
    utils::upsert_dictionary_value_from_key(
        EVENTS,
        &get_event_item_key(&token_identifier, current_event_id + 1),
        cep_78_event as u8,
    );
    Ok(())
}

fn get_event_item_key(token_identifier: &TokenIdentifier, event_id: u64) -> String {
    let mut preimage = Vec::new();
    preimage.append(
        &mut token_identifier
            .get_dictionary_item_key()
            .to_bytes()
            .unwrap_or_revert(),
    );
    preimage.append(&mut event_id.to_bytes().unwrap_or_revert());
    base16::encode_lower(&runtime::blake2b(&preimage))
}

fn is_last_event_burn(token_identifier: &TokenIdentifier, event_id: u64) -> bool {
    let last_event: CEP78Event = utils::get_dictionary_value_from_key::<u8>(
        EVENTS,
        &get_event_item_key(token_identifier, event_id),
    )
    .unwrap_or_revert()
    .try_into()
    .unwrap_or_revert_with(NFTCoreError::InvalidTokenEvent);
    last_event == CEP78Event::Burn
}

pub(crate) fn record_event(token_identifier: TokenIdentifier, event: CEP78Event) {
    match event {
        CEP78Event::Mint => record_mint_event(token_identifier)
            .unwrap_or_revert_with(NFTCoreError::FailedToRecordMintEvent),
        CEP78Event::Transfer => record_transfer_event(token_identifier)
            .unwrap_or_revert_with(NFTCoreError::FailedToRecordTransferEvent),
        CEP78Event::Burn => record_burn_event(token_identifier)
            .unwrap_or_revert_with(NFTCoreError::FailedToRecordBurnedEvent),
        CEP78Event::Approve => record_approve_event(token_identifier)
            .unwrap_or_revert_with(NFTCoreError::FailedToRecordApproveEvent),
        CEP78Event::MetadataUpdate => todo!(),
    }
}
