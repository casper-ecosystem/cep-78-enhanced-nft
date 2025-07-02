use alloc::string::String;
use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{bytesrepr::FromBytes, contract_messages::MessagePayload};

use crate::{constants::EVENTS, error::NFTCoreError};

use super::Event;

pub fn emit_native_string(event: Event) {
    let payload_data = event.to_json();
    runtime::emit_message(EVENTS, &payload_data.into()).unwrap_or_revert()
}

pub fn emit_native_bytes(event: Event) {
    let payload_data = event.to_json();
    let (payload, _) = MessagePayload::from_bytes(payload_data.as_bytes()).unwrap_or_revert();
    runtime::emit_message(EVENTS, &payload).unwrap_or_revert()
}

impl Event {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self)
            .map_err(|_| NFTCoreError::FailedToConvertEventToJson)
            .unwrap_or_revert()
    }
}
