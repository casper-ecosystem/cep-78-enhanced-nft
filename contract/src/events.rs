// seed_uref + (token_idx, row_idx)
// named_key under well defined literal which will be same for every CEP-78 to store that seed_uref
// nft_status_changes -> token_idx => status
// nft_status_row_idx_tracker -> token_idx => rox_idx

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::convert::{TryFrom, TryInto};
use casper_contract::contract_api::runtime;

use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::bytesrepr::ToBytes;

use crate::{
    utils, NFTCoreError, NFTCoreError::MissingTokenEventId, TokenIdentifier, EVENTS,
    EVENT_ID_TRACKER,
};

#[repr(u8)]
#[derive(PartialEq)]
pub(crate) enum TokenEvent {
    Minted = 0,
    Transferred = 1,
    Burned = 2,
    Approved = 3,
}

impl TryFrom<u8> for TokenEvent {
    type Error = NFTCoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TokenEvent::Minted),
            1 => Ok(TokenEvent::Transferred),
            2 => Ok(TokenEvent::Burned),
            3 => Ok(TokenEvent::Approved),
            _ => Err(NFTCoreError::InvalidTokenEvent),
        }
    }
}

impl ToString for TokenEvent {
    fn to_string(&self) -> String {
        match self {
            TokenEvent::Minted => "Minted".to_string(),
            TokenEvent::Transferred => "Transferred".to_string(),
            TokenEvent::Burned => "Burned".to_string(),
            TokenEvent::Approved => "Approved".to_string(),
        }
    }
}

pub(crate) fn emit_minted_event(token_identifier: TokenIdentifier) -> Result<(), NFTCoreError> {
    // Since mint is the first event to be "emitted" there should be no value present
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
    utils::upsert_dictionary_value_from_key(
        EVENTS,
        &event_item_key,
        TokenEvent::Minted as u8,
    );
    Ok(())
}

pub(crate) fn emit_transfer_event(token_identifier: TokenIdentifier) -> Result<(), NFTCoreError> {
    emit_event(token_identifier, TokenEvent::Transferred)
}

pub(crate) fn emit_burn_event(token_identifier: TokenIdentifier) -> Result<(), NFTCoreError> {
    emit_event(token_identifier, TokenEvent::Burned)
}

pub(crate) fn emit_approve_event(token_identifier: TokenIdentifier) -> Result<(), NFTCoreError> {
    emit_event(token_identifier, TokenEvent::Approved)
}

fn emit_event(
    token_identifier: TokenIdentifier,
    token_event: TokenEvent,
) -> Result<(), NFTCoreError> {
    let current_event_id = utils::get_dictionary_value_from_key::<u64>(
        EVENT_ID_TRACKER,
        &token_identifier.get_dictionary_item_key(),
    )
    .unwrap_or_revert_with(NFTCoreError::MissingTokenEventId);
    // The Burn event represents the end of the token life cycle, so further
    // updates to the event must be invalid
    if is_last_event_burned(&token_identifier, current_event_id) {
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
        token_event as u8,
    );
    Ok(())
}

fn get_event_item_key(token_identifier: &TokenIdentifier, event_id: u64) -> String {
    let mut preimage = Vec::new();
    preimage.append(&mut token_identifier.get_dictionary_item_key().to_bytes().unwrap_or_revert());
    preimage.append(&mut event_id.to_bytes().unwrap_or_revert());
    base16::encode_lower(&runtime::blake2b(&preimage))
}

fn is_last_event_burned(token_identifier: &TokenIdentifier, event_id: u64) -> bool {
    let last_event: TokenEvent = utils::get_dictionary_value_from_key::<u8>(
        EVENTS,
        &get_event_item_key(token_identifier, event_id),
    )
    .unwrap_or_revert()
    .try_into()
    .unwrap_or_revert_with(NFTCoreError::InvalidTokenEvent);
    last_event == TokenEvent::Burned
}

pub(crate) fn get_events(
    token_identifier: TokenIdentifier,
    starting_index: u64,
    maybe_last_index: Option<u64>,
) -> Vec<String> {
    let mut events = Vec::<String>::new();
    let last_index = match maybe_last_index {
        None => utils::get_dictionary_value_from_key(
            EVENT_ID_TRACKER,
            &token_identifier.get_dictionary_item_key(),
        )
        .unwrap_or_revert_with(MissingTokenEventId),
        Some(last_event_index) => last_event_index,
    };
    for event_index in starting_index..=last_index {
        let event: TokenEvent = utils::get_dictionary_value_from_key::<u8>(
            EVENTS,
            &get_event_item_key(&token_identifier, event_index),
        )
        .unwrap_or_revert_with(NFTCoreError::MissingTokenEventId)
        .try_into()
        .unwrap_or_revert();
        events.push(event.to_string())
    }
    events
}
