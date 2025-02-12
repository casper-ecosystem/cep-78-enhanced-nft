use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use events_cep47::emit_cep47;
use events_ces::{
    emit_ces, Approval, ApprovalForAll, ApprovalRevoked, Burn, MetadataUpdated, Migration, Mint,
    RevokedForAll, Transfer, VariablesSet,
};
use events_native::{emit_native_bytes, emit_native_string};
use serde::{Deserialize, Serialize};

use crate::{constants::EVENTS_MODE, error::NFTCoreError, modalities::EventsMode, utils};

pub mod events_ces;
pub mod events_native;

// A feature to allow the contract to be used
// as a library and a binary.
pub mod events_cep47;

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Event {
    Mint(Mint),
    Burn(Burn),
    Approval(Approval),
    ApprovalForAll(ApprovalForAll),
    ApprovalRevoked(ApprovalRevoked),
    Transfer(Transfer),
    MetadataUpdated(MetadataUpdated),
    Migration(Migration),
    RevokedForAll(RevokedForAll),
    VariablesSet(VariablesSet),
}

pub fn emit_event(event: Event) {
    let events_mode = EventsMode::try_from(utils::get_stored_value_with_user_errors::<u8>(
        EVENTS_MODE,
        NFTCoreError::MissingEventsMode,
        NFTCoreError::InvalidEventsMode,
    ))
    .unwrap_or_revert();

    match events_mode {
        EventsMode::NoEvents => {}
        EventsMode::CEP47 => emit_cep47(event),
        EventsMode::CES => emit_ces(event),
        EventsMode::Native => emit_native_string(event),
        EventsMode::NativeBytes => emit_native_bytes(event),
    }
}
