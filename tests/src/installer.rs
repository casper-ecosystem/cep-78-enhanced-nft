use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_event_standard::Schemas;
use casper_types::{runtime_args, CLValue, ContractHash, RuntimeArgs};
use contract::{
    constants::{
        ARG_ALLOW_MINTING, ARG_COLLECTION_NAME, ARG_COLLECTION_SYMBOL, ARG_CONTRACT_WHITELIST,
        ARG_HOLDER_MODE, ARG_MINTING_MODE, ARG_TOTAL_TOKEN_SUPPLY, ARG_WHITELIST_MODE,
        ENTRY_POINT_INIT, NUMBER_OF_MINTED_TOKENS,
    },
    events::events_ces::{
        Approval, ApprovalForAll, ApprovalRevoked, Burn, MetadataUpdated, Migration, Mint,
        Transfer, VariablesSet,
    },
};

use crate::utility::{
    constants::{CONTRACT_NAME, NFT_CONTRACT_WASM, NFT_TEST_COLLECTION, NFT_TEST_SYMBOL},
    installer_request_builder::{
        InstallerRequestBuilder, MintingMode, NFTHolderMode, NFTIdentifierMode, NFTMetadataKind,
        OwnerReverseLookupMode, OwnershipMode, WhitelistMode,
    },
    support,
};

#[test]
fn should_install_contract() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_total_token_supply(1u64)
        .build();

    builder.exec(install_request).expect_success().commit();

    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let query_result: String = support::query_stored_value(
        &builder,
        *nft_contract_key,
        vec![ARG_COLLECTION_NAME.to_string()],
    );

    assert_eq!(
        query_result,
        NFT_TEST_COLLECTION.to_string(),
        "collection_name initialized at installation should exist"
    );

    let query_result: String = support::query_stored_value(
        &builder,
        *nft_contract_key,
        vec![ARG_COLLECTION_SYMBOL.to_string()],
    );

    assert_eq!(
        query_result,
        NFT_TEST_SYMBOL.to_string(),
        "collection_symbol initialized at installation should exist"
    );

    let query_result: u64 = support::query_stored_value(
        &builder,
        *nft_contract_key,
        vec![ARG_TOTAL_TOKEN_SUPPLY.to_string()],
    );

    assert_eq!(
        query_result, 1u64,
        "total_token_supply initialized at installation should exist"
    );

    let query_result: bool = support::query_stored_value(
        &builder,
        *nft_contract_key,
        vec![ARG_ALLOW_MINTING.to_string()],
    );

    assert!(query_result, "Allow minting should default to true");

    let query_result: u8 = support::query_stored_value(
        &builder,
        *nft_contract_key,
        vec![ARG_MINTING_MODE.to_string()],
    );

    assert_eq!(
        query_result, 0u8,
        "minting mode should default to installer"
    );

    let query_result: u64 = support::query_stored_value(
        &builder,
        *nft_contract_key,
        vec![NUMBER_OF_MINTED_TOKENS.to_string()],
    );

    assert_eq!(
        query_result, 0u64,
        "number_of_minted_tokens initialized at installation should exist"
    );

    // Expects Schemas to be registerd.
    let expected_schemas = Schemas::new()
        .with::<Mint>()
        .with::<Burn>()
        .with::<Approval>()
        .with::<ApprovalRevoked>()
        .with::<ApprovalForAll>()
        .with::<Transfer>()
        .with::<MetadataUpdated>()
        .with::<VariablesSet>()
        .with::<Migration>();
    let actual_schemas: Schemas = support::query_stored_value(
        &builder,
        *nft_contract_key,
        vec![casper_event_standard::EVENTS_SCHEMA.to_string()],
    );
    assert_eq!(actual_schemas, expected_schemas, "Schemas mismatch.");
}

#[test]
fn should_only_allow_init_during_installation_session() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64)
            .with_reporting_mode(OwnerReverseLookupMode::NoLookUp);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let init_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_INIT,
        runtime_args! {
            ARG_COLLECTION_NAME => "collection_name".to_string(),
            ARG_COLLECTION_SYMBOL => "collection_symbol".to_string(),
            ARG_TOTAL_TOKEN_SUPPLY => "total_token_supply".to_string(),
            ARG_ALLOW_MINTING => true,
            ARG_MINTING_MODE => false,
        },
    )
    .build();
    builder.exec(init_request).expect_failure();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(
        error,
        58u16,
        "should not allow calls to init() after installation",
    );
}

#[test]
fn should_install_with_allow_minting_set_to_false() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .build();

    builder.exec(install_request).expect_success().commit();
}

#[test]
fn should_reject_invalid_collection_name() {
    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_invalid_collection_name(CLValue::from_t::<u64>(0u64).expect("expected CLValue"));

    support::assert_expected_invalid_installer_request(
        install_request_builder,
        18,
        "should reject installation when given an invalid collection name",
    );
}

#[test]
fn should_reject_invalid_collection_symbol() {
    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_invalid_collection_symbol(
                CLValue::from_t::<u64>(0u64).expect("expected CLValue"),
            );

    support::assert_expected_invalid_installer_request(
        install_request_builder,
        24,
        "should reject installation when given an invalid collection symbol",
    );
}

#[test]
fn should_reject_non_numerical_total_token_supply_value() {
    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_invalid_total_token_supply(
                CLValue::from_t::<String>("".to_string()).expect("expected CLValue"),
            );
    support::assert_expected_invalid_installer_request(
        install_request_builder,
        26,
        "should reject installation when given an invalid total supply value",
    );
}

#[test]
fn should_install_with_contract_holder_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Unlocked)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_contract_whitelist(vec![ContractHash::default()]);

    builder
        .exec(install_request.build())
        .expect_success()
        .commit();

    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let actual_holder_mode: u8 = support::query_stored_value(
        &builder,
        *nft_contract_key,
        vec![ARG_HOLDER_MODE.to_string()],
    );

    assert_eq!(
        actual_holder_mode,
        NFTHolderMode::Contracts as u8,
        "holder mode is not set to contracts"
    );

    let actual_whitelist_mode: u8 = support::query_stored_value(
        &builder,
        *nft_contract_key,
        vec![ARG_WHITELIST_MODE.to_string()],
    );

    assert_eq!(
        actual_whitelist_mode,
        WhitelistMode::Unlocked as u8,
        "whitelist mode is not set to unlocked"
    );

    let actual_contract_whitelist: Vec<ContractHash> = support::query_stored_value(
        &builder,
        *nft_contract_key,
        vec![ARG_CONTRACT_WHITELIST.to_string()],
    );

    assert_eq!(
        actual_contract_whitelist,
        vec![ContractHash::default()],
        "contract whitelist is incorrectly set"
    );
}

#[test]
fn should_disallow_installation_of_contract_with_empty_locked_whitelist() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_holder_mode(NFTHolderMode::Contracts)
            .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
            .with_whitelist_mode(WhitelistMode::Locked);

    support::assert_expected_invalid_installer_request(
        install_request_builder,
        83,
        "should fail execution since whitelist mode is locked and the provided whitelist is empty",
    );
}

#[test]
fn should_disallow_installation_with_zero_issuance() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(0u64)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_failure().commit();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(error, 123u16, "cannot install when issuance is 0");
}

#[test]
fn should_disallow_installation_with_supply_exceeding_hard_cap() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1_000_001u64)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_failure().commit();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(
        error,
        133u16,
        "cannot install when issuance is more than 1_000_000",
    );
}

#[test]
fn should_prevent_installation_with_ownership_and_minting_modality_conflict() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1_000u64)
        .with_minting_mode(MintingMode::Installer)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::Complete)
        .build();

    builder.exec(install_request).expect_failure().commit();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(
        error,
        130u16,
        "cannot install when Ownership::Minter and MintingMode::Installer",
    );
}

#[test]
fn should_prevent_installation_with_ownership_minter_and_owner_reverse_lookup_mode_transfer_only() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1_000u64)
        .with_minting_mode(MintingMode::Installer)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::TransfersOnly)
        .build();

    builder.exec(install_request).expect_failure().commit();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(
        error,
        140u16,
        "cannot install when Ownership::Minter and OwnerReverseLookupMode::TransfersOnly",
    );
}

#[test]
fn should_prevent_installation_with_ownership_assigned_and_owner_reverse_lookup_mode_transfer_only()
{
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1_000u64)
        .with_minting_mode(MintingMode::Installer)
        .with_ownership_mode(OwnershipMode::Assigned)
        .with_reporting_mode(OwnerReverseLookupMode::TransfersOnly)
        .build();

    builder.exec(install_request).expect_failure().commit();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(
        error,
        140u16,
        "cannot install when Ownership::Assigned and OwnerReverseLookupMode::TransfersOnly",
    );
}

#[test]
fn should_allow_installation_with_ownership_transferable_and_owner_reverse_lookup_mode_transfer_only(
) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1_000u64)
        .with_minting_mode(MintingMode::Installer)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_reporting_mode(OwnerReverseLookupMode::TransfersOnly)
        .build();

    builder.exec(install_request).expect_success().commit();
}
