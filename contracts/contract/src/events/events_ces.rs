use alloc::string::{String, ToString};

use casper_event_standard::Event;
use casper_types::Key;
use serde::{Deserialize, Serialize};

use crate::modalities::TokenIdentifier;

use super::Event;

#[derive(Serialize, Deserialize, Event, Debug, PartialEq, Eq)]
pub struct Mint {
    pub recipient: Key,
    pub token_id: String,
    pub data: String,
}

impl Mint {
    pub fn new(recipient: Key, token_id: &TokenIdentifier, data: String) -> Self {
        Self {
            recipient,
            token_id: token_id.to_string(),
            data,
        }
    }
}

#[derive(Serialize, Deserialize, Event, Debug, PartialEq, Eq)]
pub struct Burn {
    pub owner: Key,
    pub token_id: String,
    pub burner: Key,
}

impl Burn {
    pub fn new(owner: Key, token_id: &TokenIdentifier, burner: Key) -> Self {
        Self {
            owner,
            token_id: token_id.to_string(),
            burner,
        }
    }
}

#[derive(Serialize, Deserialize, Event, Debug, PartialEq, Eq)]
pub struct Approval {
    pub owner: Key,
    pub spender: Key,
    pub token_id: String,
}

impl Approval {
    pub fn new(owner: Key, spender: Key, token_id: &TokenIdentifier) -> Self {
        Self {
            owner,
            spender,
            token_id: token_id.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Event, Debug, PartialEq, Eq)]
pub struct ApprovalRevoked {
    pub owner: Key,
    pub token_id: String,
}

impl ApprovalRevoked {
    pub fn new(owner: Key, token_id: &TokenIdentifier) -> Self {
        Self {
            owner,
            token_id: token_id.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Event, Debug, PartialEq, Eq)]
pub struct ApprovalForAll {
    pub owner: Key,
    pub operator: Key,
}

impl ApprovalForAll {
    pub fn new(owner: Key, operator: Key) -> Self {
        Self { owner, operator }
    }
}

#[derive(Serialize, Deserialize, Event, Debug, PartialEq, Eq)]
pub struct RevokedForAll {
    pub owner: Key,
    pub operator: Key,
}

impl RevokedForAll {
    pub fn new(owner: Key, operator: Key) -> Self {
        Self { owner, operator }
    }
}

#[derive(Serialize, Deserialize, Event, Debug, PartialEq, Eq)]
pub struct Transfer {
    pub owner: Key,
    pub spender: Option<Key>,
    pub recipient: Key,
    pub token_id: String,
}

impl Transfer {
    pub fn new(
        owner: Key,
        spender: Option<Key>,
        recipient: Key,
        token_id: &TokenIdentifier,
    ) -> Self {
        Self {
            owner,
            spender,
            recipient,
            token_id: token_id.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Event, Debug, PartialEq, Eq)]
pub struct MetadataUpdated {
    pub token_id: String,
    pub data: String,
}

impl MetadataUpdated {
    pub fn new(token_id: &TokenIdentifier, data: String) -> Self {
        Self {
            token_id: token_id.to_string(),
            data,
        }
    }
}

#[derive(Serialize, Deserialize, Event, Debug, PartialEq, Eq, Default)]
pub struct VariablesSet {}

impl VariablesSet {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Serialize, Deserialize, Event, Debug, PartialEq, Eq, Default)]
pub struct Migration {}

impl Migration {
    pub fn new() -> Self {
        Self {}
    }
}

pub fn emit_ces(event: Event) {
    match event {
        Event::Mint(e) => casper_event_standard::emit(e),
        Event::Burn(e) => casper_event_standard::emit(e),
        Event::Approval(e) => casper_event_standard::emit(e),
        Event::ApprovalForAll(e) => casper_event_standard::emit(e),
        Event::ApprovalRevoked(e) => casper_event_standard::emit(e),
        Event::Transfer(e) => casper_event_standard::emit(e),
        Event::MetadataUpdated(e) => casper_event_standard::emit(e),
        Event::Migration(e) => casper_event_standard::emit(e),
        Event::RevokedForAll(e) => casper_event_standard::emit(e),
        Event::VariablesSet(e) => casper_event_standard::emit(e),
    }
}
