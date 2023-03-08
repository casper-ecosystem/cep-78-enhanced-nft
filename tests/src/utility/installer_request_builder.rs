use std::collections::BTreeMap;

use contract::constants::{
    ARG_ADDITIONAL_REQUIRED_METADATA, ARG_ALLOW_MINTING, ARG_BURN_MODE, ARG_COLLECTION_NAME,
    ARG_COLLECTION_SYMBOL, ARG_CONTRACT_WHITELIST, ARG_EVENTS_MODE, ARG_HOLDER_MODE,
    ARG_IDENTIFIER_MODE, ARG_JSON_SCHEMA, ARG_METADATA_MUTABILITY, ARG_MINTING_MODE,
    ARG_NAMED_KEY_CONVENTION, ARG_NFT_KIND, ARG_NFT_METADATA_KIND, ARG_OPTIONAL_METADATA,
    ARG_OWNERSHIP_MODE, ARG_OWNER_LOOKUP_MODE, ARG_TOTAL_TOKEN_SUPPLY, ARG_WHITELIST_MODE,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use casper_engine_test_support::ExecuteRequestBuilder;
use casper_execution_engine::core::engine_state::ExecuteRequest;
use casper_types::{account::AccountHash, bytesrepr::Bytes, CLValue, ContractHash, RuntimeArgs};

// Modalities reexports.
pub use contract::modalities::{
    EventsMode, MintingMode, NFTHolderMode, NFTKind, OwnershipMode, TokenIdentifier, WhitelistMode,
};

use super::constants::{NFT_TEST_COLLECTION, NFT_TEST_SYMBOL};

pub(crate) static TEST_CUSTOM_METADATA_SCHEMA: Lazy<CustomMetadataSchema> = Lazy::new(|| {
    let mut properties = BTreeMap::new();
    properties.insert(
        "deity_name".to_string(),
        MetadataSchemaProperty {
            name: "deity_name".to_string(),
            description: "The name of deity from a particular pantheon.".to_string(),
            required: true,
        },
    );
    properties.insert(
        "mythology".to_string(),
        MetadataSchemaProperty {
            name: "mythology".to_string(),
            description: "The mythology the deity belongs to.".to_string(),
            required: true,
        },
    );
    CustomMetadataSchema { properties }
});

pub(crate) static TEST_CUSTOM_METADATA: Lazy<BTreeMap<String, String>> = Lazy::new(|| {
    let mut attributes = BTreeMap::new();
    attributes.insert("deity_name".to_string(), "Baldur".to_string());
    attributes.insert("mythology".to_string(), "Nordic".to_string());
    attributes
});
pub(crate) static TEST_CUSTOM_UPDATED_METADATA: Lazy<BTreeMap<String, String>> = Lazy::new(|| {
    let mut attributes = BTreeMap::new();
    attributes.insert("deity_name".to_string(), "Baldur".to_string());
    attributes.insert("mythology".to_string(), "Nordic".to_string());
    attributes.insert("enemy".to_string(), "Loki".to_string());
    attributes
});

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct MetadataSchemaProperty {
    name: String,
    description: String,
    required: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct CustomMetadataSchema {
    properties: BTreeMap<String, MetadataSchemaProperty>,
}

#[derive(Serialize, Deserialize)]
struct Metadata {
    name: String,
    symbol: String,
    token_uri: String,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum NFTMetadataKind {
    CEP78 = 0,
    NFT721 = 1,
    Raw = 2,
    CustomValidated = 3,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum NFTIdentifierMode {
    Ordinal = 0,
    Hash = 1,
}

#[repr(u8)]
pub enum MetadataMutability {
    Immutable = 0,
    Mutable = 1,
}

#[repr(u8)]
pub enum BurnMode {
    Burnable = 0,
    NonBurnable = 1,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Clone, Copy)]
pub enum OwnerReverseLookupMode {
    NoLookUp = 0,
    Complete = 1,
    TransfersOnly = 2,
}

#[repr(u8)]
pub enum NamedKeyConventionMode {
    DerivedFromCollectionName = 0,
    V1_0Standard = 1,
    V1_0Custom = 2,
}

#[derive(Debug)]
pub(crate) struct InstallerRequestBuilder {
    account_hash: AccountHash,
    session_file: String,
    collection_name: CLValue,
    collection_symbol: CLValue,
    total_token_supply: CLValue,
    allow_minting: CLValue,
    minting_mode: CLValue,
    ownership_mode: CLValue,
    nft_kind: CLValue,
    holder_mode: CLValue,
    whitelist_mode: CLValue,
    contract_whitelist: CLValue,
    json_schema: CLValue,
    nft_metadata_kind: CLValue,
    identifier_mode: CLValue,
    metadata_mutability: CLValue,
    burn_mode: CLValue,
    reporting_mode: CLValue,
    named_key_convention: CLValue,
    additional_required_metadata: CLValue,
    optional_metadata: CLValue,
    events_mode: CLValue,
}

impl InstallerRequestBuilder {
    pub(crate) fn new(account_hash: AccountHash, session_file: &str) -> Self {
        Self::default()
            .with_account_hash(account_hash)
            .with_session_file(session_file.to_string())
    }

    pub(crate) fn default() -> Self {
        InstallerRequestBuilder {
            account_hash: AccountHash::default(),
            session_file: String::default(),
            collection_name: CLValue::from_t(NFT_TEST_COLLECTION.to_string())
                .expect("name is legit CLValue"),
            collection_symbol: CLValue::from_t(NFT_TEST_SYMBOL)
                .expect("collection_symbol is legit CLValue"),
            total_token_supply: CLValue::from_t(1u64).expect("total_token_supply is legit CLValue"),
            allow_minting: CLValue::from_t(true).unwrap(),
            minting_mode: CLValue::from_t(MintingMode::Installer as u8).unwrap(),
            ownership_mode: CLValue::from_t(OwnershipMode::Minter as u8).unwrap(),
            nft_kind: CLValue::from_t(NFTKind::Physical as u8).unwrap(),
            holder_mode: CLValue::from_t(NFTHolderMode::Mixed as u8).unwrap(),
            whitelist_mode: CLValue::from_t(WhitelistMode::Unlocked as u8).unwrap(),
            contract_whitelist: CLValue::from_t(Vec::<ContractHash>::new()).unwrap(),
            json_schema: CLValue::from_t("test".to_string())
                .expect("test_metadata was created from a concrete value"),
            nft_metadata_kind: CLValue::from_t(NFTMetadataKind::NFT721 as u8).unwrap(),
            identifier_mode: CLValue::from_t(NFTIdentifierMode::Ordinal as u8).unwrap(),
            metadata_mutability: CLValue::from_t(MetadataMutability::Mutable as u8).unwrap(),
            burn_mode: CLValue::from_t(BurnMode::Burnable as u8).unwrap(),
            reporting_mode: CLValue::from_t(OwnerReverseLookupMode::Complete as u8).unwrap(),
            named_key_convention: CLValue::from_t(
                NamedKeyConventionMode::DerivedFromCollectionName as u8,
            )
            .unwrap(),
            additional_required_metadata: CLValue::from_t(Bytes::new()).unwrap(),
            optional_metadata: CLValue::from_t(Bytes::new()).unwrap(),
            events_mode: CLValue::from_t(EventsMode::CES as u8).unwrap(),
        }
    }

    pub(crate) fn with_account_hash(mut self, account_hash: AccountHash) -> Self {
        self.account_hash = account_hash;
        self
    }

    pub(crate) fn with_session_file(mut self, session_file: String) -> Self {
        self.session_file = session_file;
        self
    }

    pub(crate) fn with_collection_name(mut self, collection_name: String) -> Self {
        self.collection_name =
            CLValue::from_t(collection_name).expect("collection_name is legit CLValue");
        self
    }

    pub(crate) fn with_invalid_collection_name(mut self, collection_name: CLValue) -> Self {
        self.collection_name = collection_name;
        self
    }

    pub(crate) fn with_collection_symbol(mut self, collection_symbol: String) -> Self {
        self.collection_symbol =
            CLValue::from_t(collection_symbol).expect("collection_symbol is legit CLValue");
        self
    }

    pub(crate) fn with_invalid_collection_symbol(mut self, collection_symbol: CLValue) -> Self {
        self.collection_symbol = collection_symbol;
        self
    }

    pub(crate) fn with_total_token_supply(mut self, total_token_supply: u64) -> Self {
        self.total_token_supply =
            CLValue::from_t(total_token_supply).expect("total_token_supply is legit CLValue");
        self
    }

    pub(crate) fn with_invalid_total_token_supply(mut self, total_token_supply: CLValue) -> Self {
        self.total_token_supply = total_token_supply;
        self
    }

    // Why Option here? The None case should be taken care of when running default
    pub(crate) fn with_allowing_minting(mut self, allow_minting: bool) -> Self {
        self.allow_minting =
            CLValue::from_t(allow_minting).expect("allow minting is legit CLValue");
        self
    }

    pub(crate) fn with_minting_mode(mut self, minting_mode: MintingMode) -> Self {
        self.minting_mode =
            CLValue::from_t(minting_mode as u8).expect("public minting is legit CLValue");
        self
    }

    pub(crate) fn with_ownership_mode(mut self, ownership_mode: OwnershipMode) -> Self {
        self.ownership_mode = CLValue::from_t(ownership_mode as u8).unwrap();
        self
    }

    pub(crate) fn with_holder_mode(mut self, holder_mode: NFTHolderMode) -> Self {
        self.holder_mode = CLValue::from_t(holder_mode as u8).unwrap();
        self
    }

    pub(crate) fn with_whitelist_mode(mut self, whitelist_mode: WhitelistMode) -> Self {
        self.whitelist_mode = CLValue::from_t(whitelist_mode as u8).unwrap();
        self
    }

    pub(crate) fn with_contract_whitelist(mut self, contract_whitelist: Vec<ContractHash>) -> Self {
        self.contract_whitelist = CLValue::from_t(contract_whitelist).unwrap();
        self
    }

    pub(crate) fn with_nft_metadata_kind(mut self, nft_metadata_kind: NFTMetadataKind) -> Self {
        self.nft_metadata_kind = CLValue::from_t(nft_metadata_kind as u8).unwrap();
        self
    }

    pub(crate) fn with_additional_required_metadata(
        mut self,
        additional_required_metadata: Vec<u8>,
    ) -> Self {
        self.additional_required_metadata =
            CLValue::from_t(Bytes::from(additional_required_metadata)).unwrap();
        self
    }

    pub(crate) fn with_optional_metadata(mut self, optional_metadata: Vec<u8>) -> Self {
        self.optional_metadata = CLValue::from_t(Bytes::from(optional_metadata)).unwrap();
        self
    }

    pub(crate) fn with_json_schema(mut self, json_schema: String) -> Self {
        self.json_schema = CLValue::from_t(json_schema).expect("json_schema is legit CLValue");
        self
    }

    pub(crate) fn with_identifier_mode(mut self, identifier_mode: NFTIdentifierMode) -> Self {
        self.identifier_mode = CLValue::from_t(identifier_mode as u8).unwrap();
        self
    }

    pub(crate) fn with_metadata_mutability(
        mut self,
        metadata_mutability: MetadataMutability,
    ) -> Self {
        self.metadata_mutability = CLValue::from_t(metadata_mutability as u8).unwrap();
        self
    }

    pub(crate) fn with_burn_mode(mut self, burn_mode: BurnMode) -> Self {
        self.burn_mode = CLValue::from_t(burn_mode as u8).unwrap();
        self
    }

    pub(crate) fn with_reporting_mode(mut self, reporting_mode: OwnerReverseLookupMode) -> Self {
        self.reporting_mode = CLValue::from_t(reporting_mode as u8).unwrap();
        self
    }

    pub(crate) fn with_events_mode(mut self, events_mode: EventsMode) -> Self {
        self.events_mode = CLValue::from_t(events_mode as u8).unwrap();
        self
    }

    pub(crate) fn build(self) -> ExecuteRequest {
        let mut runtime_args = RuntimeArgs::new();
        runtime_args.insert_cl_value(ARG_COLLECTION_NAME, self.collection_name);
        runtime_args.insert_cl_value(ARG_COLLECTION_SYMBOL, self.collection_symbol);
        runtime_args.insert_cl_value(ARG_TOTAL_TOKEN_SUPPLY, self.total_token_supply);
        runtime_args.insert_cl_value(ARG_ALLOW_MINTING, self.allow_minting);
        runtime_args.insert_cl_value(ARG_MINTING_MODE, self.minting_mode.clone());
        runtime_args.insert_cl_value(ARG_OWNERSHIP_MODE, self.ownership_mode);
        runtime_args.insert_cl_value(ARG_NFT_KIND, self.nft_kind);
        runtime_args.insert_cl_value(ARG_HOLDER_MODE, self.holder_mode);
        runtime_args.insert_cl_value(ARG_WHITELIST_MODE, self.whitelist_mode);
        runtime_args.insert_cl_value(ARG_CONTRACT_WHITELIST, self.contract_whitelist);
        runtime_args.insert_cl_value(ARG_JSON_SCHEMA, self.json_schema);
        runtime_args.insert_cl_value(ARG_NFT_METADATA_KIND, self.nft_metadata_kind);
        runtime_args.insert_cl_value(ARG_IDENTIFIER_MODE, self.identifier_mode);
        runtime_args.insert_cl_value(ARG_METADATA_MUTABILITY, self.metadata_mutability);
        runtime_args.insert_cl_value(ARG_BURN_MODE, self.burn_mode);
        runtime_args.insert_cl_value(ARG_OWNER_LOOKUP_MODE, self.reporting_mode);
        runtime_args.insert_cl_value(ARG_NAMED_KEY_CONVENTION, self.named_key_convention);
        runtime_args.insert_cl_value(ARG_EVENTS_MODE, self.events_mode);
        runtime_args.insert_cl_value(
            ARG_ADDITIONAL_REQUIRED_METADATA,
            self.additional_required_metadata,
        );
        runtime_args.insert_cl_value(ARG_OPTIONAL_METADATA, self.optional_metadata);
        ExecuteRequestBuilder::standard(self.account_hash, &self.session_file, runtime_args).build()
    }
}
