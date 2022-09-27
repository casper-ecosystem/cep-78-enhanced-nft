use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_ACCOUNT_PUBLIC_KEY, DEFAULT_RUN_GENESIS_REQUEST, MINIMUM_ACCOUNT_CREATION_BALANCE,
};
use casper_types::{
    account::AccountHash, runtime_args, system::mint, ContractHash, Key, PublicKey, RuntimeArgs,
    SecretKey, U512,
};

use crate::utility::installer_request_builder;
use crate::utility::installer_request_builder::{InstallerRequestBuilder, NFTIdentifierMode, NFTMetadataKind, OwnershipMode};
use crate::utility::constants::{MINT_SESSION_WASM, NFT_CONTRACT_WASM, NFT_TEST_COLLECTION, NFT_TEST_SYMBOL, ARG_NFT_CONTRACT_HASH, ARG_TOKEN_OWNER, ARG_TOKEN_META_DATA};
use crate::utility::support;

#[test]
fn mint_cost_should_remain_stable() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(
        *DEFAULT_ACCOUNT_ADDR,
            NFT_CONTRACT_WASM
    )
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder
        .exec(install_request)
        .expect_success()
        .commit();

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let first_mint_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => "",
        }
    ).build();

    builder.exec(first_mint_request)
        .expect_success()
        .commit();

    let first_mint_gas_costs = builder.last_exec_gas_cost();

    let second_mint_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => "",
        }
    ).build();

    builder.exec(second_mint_request)
        .expect_success()
        .commit();

    let second_mint_gas_costs = builder.last_exec_gas_cost();

    // assert_eq!(first_mint_gas_costs, second_mint_gas_costs);

    let third_mint_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => "",
        }
    ).build();

    builder.exec(third_mint_request)
        .expect_success()
        .commit();

    let third_mint_gas_costs = builder.last_exec_gas_cost();

    assert_eq!(second_mint_gas_costs, third_mint_gas_costs);
}