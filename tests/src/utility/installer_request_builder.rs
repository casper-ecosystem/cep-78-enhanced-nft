//use std::convert::TryFrom;

use casper_engine_test_support::ExecuteRequestBuilder;
use casper_execution_engine::core::engine_state::ExecuteRequest;
use casper_types::{account::AccountHash, CLValue, RuntimeArgs, U256};

use super::constants::{
    ARG_ALLOW_MINTING, ARG_COLLECTION_NAME, ARG_COLLECTION_SYMBOL, ARG_JSON_SCHEMA,
    ARG_NFT_ASSET_TYPE, ARG_OWNERSHIP_MODE, ARG_PUBLIC_MINTING, ARG_TOTAL_TOKEN_SUPPLY,
};

#[repr(u8)]
#[derive(Debug)]
pub enum OwnershipMode {
    Minter = 0,                // The minter owns it and can never transfer it.
    Assigned = 1,              // The minter assigns it to an address and can never be transferred.
    TransferableUnchecked = 2, // The NFT can be transferred even to an recipient that does not exist.
    TransferableChecked = 3, // The NFT can be transferred but only to a recipient that does exist.
                             // Maybe Shared(u8) // Shares of the NFT can be transferred and ownership is determined by the share.
}

#[repr(u8)]
#[derive(Debug)]
pub enum NFTAssetType {
    PhysicalAsset = 0,
    DigitalAsset = 1, // The minter assigns it to an address and can never be transferred.
    VirtualAsset = 2, // The NFT can be transferred even to an recipient that does not exist
}

#[derive(Debug)]
pub(crate) struct InstallerRequestBuilder {
    account_hash: AccountHash,
    session_file: String,
    collection_name: CLValue,
    collection_symbol: CLValue,
    total_token_supply: CLValue,
    allow_minting: CLValue,
    public_minting: CLValue,
    ownership_mode: CLValue,
    nft_asset_type: CLValue,
    json_schema: CLValue,
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
            collection_name: CLValue::from_t("name".to_string()).expect("name is legit CLValue"),
            collection_symbol: CLValue::from_t("SYM").expect("collection_symbol is legit CLValue"),
            total_token_supply: CLValue::from_t(U256::one())
                .expect("total_token_supply is legit CLValue"),
            allow_minting: CLValue::from_t(Some(true)).unwrap(),
            public_minting: CLValue::from_t(Some(false)).unwrap(),
            ownership_mode: CLValue::from_t(OwnershipMode::Minter as u8).unwrap(),
            nft_asset_type: CLValue::from_t(NFTAssetType::PhysicalAsset as u8).unwrap(),
            json_schema: CLValue::from_t("my_json_schema".to_string())
                .expect("my_json_schema is legit CLValue"),
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

    pub(crate) fn with_total_token_supply(mut self, total_token_supply: U256) -> Self {
        self.total_token_supply =
            CLValue::from_t(total_token_supply).expect("total_token_supply is legit CLValue");
        self
    }

    pub(crate) fn with_invalid_total_token_supply(mut self, total_token_supply: CLValue) -> Self {
        self.total_token_supply = total_token_supply;
        self
    }

    // Why Option here? The None case should be taken care of when running default
    pub(crate) fn with_allowing_minting(mut self, allow_minting: Option<bool>) -> Self {
        self.allow_minting =
            CLValue::from_t(allow_minting).expect("allow minting is legit CLValue");
        self
    }

    pub(crate) fn with_public_minting(mut self, public_minting: Option<bool>) -> Self {
        self.public_minting =
            CLValue::from_t(public_minting).expect("public minting is legit CLValue");
        self
    }

    pub(crate) fn with_ownership_mode(mut self, ownership_mode: OwnershipMode) -> Self {
        self.ownership_mode = CLValue::from_t(ownership_mode as u8).unwrap();
        self
    }

    pub(crate) fn _with_json_schema(mut self, json_schema: &str) -> Self {
        self.json_schema = CLValue::from_t(json_schema).expect("json_schema is legit CLValue");
        self
    }

    pub(crate) fn build(self) -> ExecuteRequest {
        let mut runtime_args = RuntimeArgs::new();
        runtime_args.insert_cl_value(ARG_COLLECTION_NAME, self.collection_name);
        runtime_args.insert_cl_value(ARG_COLLECTION_SYMBOL, self.collection_symbol);
        runtime_args.insert_cl_value(ARG_TOTAL_TOKEN_SUPPLY, self.total_token_supply);
        runtime_args.insert_cl_value(ARG_ALLOW_MINTING, self.allow_minting);
        runtime_args.insert_cl_value(ARG_PUBLIC_MINTING, self.public_minting);
        runtime_args.insert_cl_value(ARG_OWNERSHIP_MODE, self.ownership_mode);
        runtime_args.insert_cl_value(ARG_NFT_ASSET_TYPE, self.nft_asset_type);
        runtime_args.insert_cl_value(ARG_JSON_SCHEMA, self.json_schema);
        ExecuteRequestBuilder::standard(self.account_hash, &self.session_file, runtime_args).build()
    }
}
