use alloc::string::{String, ToString};

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
    burner: Key,
}

impl Burn {
    pub fn new(owner: Key, token_id: TokenIdentifier, burner: Key) -> Self {
        Self {
            owner,
            token_id: token_id.to_string(),
            burner,
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Approval {
    owner: Key,
    spender: Key,
    token_id: String,
}

impl Approval {
    pub fn new(owner: Key, spender: Key, token_id: TokenIdentifier) -> Self {
        Self {
            owner,
            spender,
            token_id: token_id.to_string(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct ApprovalRevoked {
    owner: Key,
    token_id: String,
}

impl ApprovalRevoked {
    pub fn new(owner: Key, token_id: TokenIdentifier) -> Self {
        Self {
            owner,
            token_id: token_id.to_string(),
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct ApprovalForAll {
    owner: Key,
    operator: Key,
}

impl ApprovalForAll {
    pub fn new(owner: Key, operator: Key) -> Self {
        Self { owner, operator }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct RevokedForAll {
    owner: Key,
    operator: Key,
}

impl RevokedForAll {
    pub fn new(owner: Key, operator: Key) -> Self {
        Self { owner, operator }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Transfer {
    owner: Key,
    spender: Option<Key>,
    recipient: Key,
    token_id: String,
}

impl Transfer {
    pub fn new(
        owner: Key,
        spender: Option<Key>,
        recipient: Key,
        token_id: TokenIdentifier,
    ) -> Self {
        Self {
            owner,
            spender,
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
            data,
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
