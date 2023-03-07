use alloc::{string::{String, ToString}, vec::Vec};

use casper_event_standard::Event;
use casper_types::Key;

use crate::modalities::TokenIdentifier;

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Mint {
    recipient: Key,
    token_id: String,
    data: String,
}

impl Mint {
    pub fn new(recipient: Key, token_id: TokenIdentifier, data: String) -> Self {
        Self {
            recipient,
            token_id: token_id.to_string(),
            data,
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Burn {
    owner: Key,
    token_id: String,
}

impl Burn {
    pub fn new(owner: Key, token_id: TokenIdentifier) -> Self {
        Self { 
            owner, 
            token_id: token_id.to_string(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Approval {
    owner: Key,
    operator: Key,
    token_id: String,
}

impl Approval {
    pub fn new(owner: Key, operator: Key, token_id: TokenIdentifier) -> Self {
        Self {
            owner,
            operator,
            token_id: token_id.to_string(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct ApprovalForAll {
    owner: Key,
    operator: Option<Key>,
    token_ids: Vec<String>,
}

impl ApprovalForAll {
    pub fn new(owner: Key, operator: Option<Key>, token_ids: Vec<TokenIdentifier>) -> Self {
        Self {
            owner,
            operator,
            token_ids: token_ids.iter().map(|token_id| token_id.to_string()).collect(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Transfer {
    owner: Key,
    operator: Option<Key>,
    recipient: Key,
    token_id: String,
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
            token_id: token_id.to_string(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct MetadataUpdated {
    token_id: String,
    data: String,
}

impl MetadataUpdated {
    pub fn new(token_id: TokenIdentifier, data: String) -> Self {
        Self { 
            token_id: token_id.to_string(),
            data 
        }
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
