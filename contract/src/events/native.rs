use core::convert::TryFrom;

use crate::modalities::TokenIdentifier;
use casper_types::{contract_messages::MessagePayload, Key};
use serde::Serialize;
pub const EVENTS_TOPIC: &str = "events";

#[derive(Serialize)]
pub enum CEP78Message {
    Mint {
        recipient: Key,
        token_id: TokenIdentifier,
    },
    Burn {
        owner: Key,
        token_id: TokenIdentifier,
        burner: Key,
    },
    ApprovalGranted {
        owner: Key,
        spender: Key,
        token_id: TokenIdentifier,
    },
    ApprovalRevoked {
        owner: Key,
        token_id: TokenIdentifier,
    },
    ApprovalForAll {
        owner: Key,
        operator: Key,
    },
    RevokedForAll {
        owner: Key,
        operator: Key,
    },
    Transfer {
        sender: Key,
        recipient: Key,
        token_id: TokenIdentifier,
    },
    MetadataUpdate {
        token_id: TokenIdentifier,
    },
    VariablesSet,
    Migrate,
}

impl TryFrom<CEP78Message> for MessagePayload {
    type Error = serde_json::Error;

    fn try_from(value: CEP78Message) -> Result<Self, Self::Error> {
        let json_value = serde_json::to_string(&value)?;
        Ok(json_value.into())
    }
}
