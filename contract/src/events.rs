use alloc::{string::String, vec::Vec};
use casper_event_standard::Event;
use casper_types::Key;

use crate::modalities::TokenIdentifier;

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Mint {
    recipient: Key,
    token_id: TokenIdentifier,
    data: String,
}

impl Mint {
    pub fn new(recipient: Key, token_id: TokenIdentifier, data: String) -> Self {
        Self {
            recipient,
            token_id,
            data,
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Burn {
    owner: Key,
    token_id: TokenIdentifier,
}

impl Burn {
    pub fn new(owner: Key, token_id: TokenIdentifier) -> Self {
        Self { owner, token_id }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Approval {
    owner: Key,
    operator: Key,
    token_id: TokenIdentifier,
}

impl Approval {
    pub fn new(owner: Key, operator: Key, token_id: TokenIdentifier) -> Self {
        Self {
            owner,
            operator,
            token_id,
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct ApprovalForAll {
    owner: Key,
    operator: Option<Key>,
    token_ids: Vec<TokenIdentifier>,
}

impl ApprovalForAll {
    pub fn new(owner: Key, operator: Option<Key>, token_ids: Vec<TokenIdentifier>) -> Self {
        Self {
            owner,
            operator,
            token_ids,
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Transfer {
    owner: Key,
    operator: Option<Key>,
    recipient: Key,
    token_id: TokenIdentifier,
}

impl Transfer {
    pub fn new(
        owner: Key,
        operator: Option<Key>,
        recipient: Key,
        token_id: TokenIdentifier,
    ) -> Self {
        Self {
            owner,
            operator,
            recipient,
            token_id,
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct MetadataUpdated {
    token_id: TokenIdentifier,
    data: String,
}

impl MetadataUpdated {
    pub fn new(token_id: TokenIdentifier, data: String) -> Self {
        Self { token_id, data }
    }
}

#[derive(Event, Debug, PartialEq, Eq, Default)]
pub struct VariablesSet {}

impl VariablesSet {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Event, Debug, PartialEq, Eq, Default)]
pub struct Migration {}

impl Migration {
    pub fn new() -> Self {
        Self {}
    }
}
