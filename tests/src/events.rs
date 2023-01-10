use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{
    account::AccountHash, runtime_args, system::mint, ContractHash, Key, RuntimeArgs,
};

use crate::utility::{
    constants::{
        ACCOUNT_USER_1, ARG_APPROVE_ALL, ARG_COLLECTION_NAME, ARG_NFT_CONTRACT_HASH, ARG_OPERATOR,
        ARG_TOKEN_HASH, ARG_TOKEN_ID, ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, BALANCES, BURNT_TOKENS,
        CONTRACT_NAME, ENTRY_POINT_BURN, ENTRY_POINT_MINT, ENTRY_POINT_SET_APPROVE_FOR_ALL,
        MINTING_CONTRACT_WASM, MINT_SESSION_WASM, NFT_CONTRACT_WASM, NFT_TEST_COLLECTION,
        TEST_PRETTY_721_META_DATA, TOKEN_COUNTS,
    },
    installer_request_builder::{
        BurnMode, InstallerRequestBuilder, MetadataMutability, MintingMode, NFTHolderMode,
        NFTIdentifierMode, OwnerReverseLookupMode, OwnershipMode, WhitelistMode,
    },
    support::{
        self, get_dictionary_value_from_key, get_minting_contract_hash, get_nft_contract_hash,
    },
};

#[test]
fn should_record_cep47_style_mint_event() {}