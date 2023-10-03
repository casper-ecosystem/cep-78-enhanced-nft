use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    PRODUCTION_RUN_GENESIS_REQUEST,
};
use casper_types::{runtime_args, ContractHash, Key, RuntimeArgs};
use contract::{
    constants::{
        ACL_PACKAGE_MODE, ALLOW_MINTING, ARG_ACL_PACKAGE_MODE, ARG_ALLOW_MINTING,
        ARG_OPERATOR_BURN_MODE, ARG_PACKAGE_OPERATOR_MODE, ENTRY_POINT_SET_VARIABLES,
        OPERATOR_BURN_MODE, PACKAGE_OPERATOR_MODE,
    },
    error::NFTCoreError,
    events::events_ces::VariablesSet,
};

use crate::utility::{
    constants::{
        ACCOUNT_USER_1, CONTRACT_NAME, NFT_CONTRACT_WASM, NFT_TEST_COLLECTION, NFT_TEST_SYMBOL,
    },
    installer_request_builder::{InstallerRequestBuilder, OwnerReverseLookupMode},
    support::{self, assert_expected_error, get_nft_contract_hash},
};

#[test]
fn only_installer_should_be_able_to_toggle_allow_minting() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let other_user_account =
        support::create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1));

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_allowing_minting(false)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    // Account other than installer account should not be able to change allow_minting
    // Red test
    let other_user_set_variables_request = ExecuteRequestBuilder::contract_call_by_hash(
        other_user_account,
        nft_contract_hash,
        ENTRY_POINT_SET_VARIABLES,
        runtime_args! { ARG_ALLOW_MINTING => true },
    )
    .build();

    // ACCOUNT_USER_1 account should NOT be able to change allow_minting
    // Red test
    builder
        .exec(other_user_set_variables_request)
        .expect_failure()
        .commit();

    let error = builder.get_error().expect("should have an error");
    assert_expected_error(
        error,
        NFTCoreError::InvalidAccount as u16,
        "Invalid Account to set variables",
    );

    let allow_minting: bool =
        support::query_stored_value(&builder, nft_contract_key, vec![ALLOW_MINTING.to_string()]);

    assert!(!allow_minting);

    // Installer account should be able to change allow_minting
    // Green test
    let installer_set_variables_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_VARIABLES,
        runtime_args! { ARG_ALLOW_MINTING => true },
    )
    .build();

    builder
        .exec(installer_set_variables_request)
        .expect_success()
        .commit();

    let allow_minting: bool =
        support::query_stored_value(&builder, nft_contract_key, vec![ALLOW_MINTING.to_string()]);

    assert!(allow_minting);

    // Expect VariablesSet event.
    let expected_event = VariablesSet::new();
    let actual_event: VariablesSet = support::get_event(&builder, &nft_contract_key, 0).unwrap();
    assert_eq!(actual_event, expected_event, "Expected VariablesSet event.");
}

#[test]
fn installer_should_be_able_to_toggle_acl_package_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .build();

    builder.exec(install_request).expect_success().commit();

    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key: Key = *account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let nft_contract_hash = Key::into_hash(nft_contract_key)
        .map(ContractHash::new)
        .expect("failed to find nft contract");

    let is_acl_packge_mode: bool = support::query_stored_value(
        &builder,
        nft_contract_key,
        vec![ARG_ACL_PACKAGE_MODE.to_string()],
    );

    assert!(!is_acl_packge_mode);

    // Installer account should be able to change ACL package mode
    let installer_set_variables_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_VARIABLES,
        runtime_args! { ARG_ACL_PACKAGE_MODE => true },
    )
    .build();

    builder
        .exec(installer_set_variables_request)
        .expect_success()
        .commit();

    let is_acl_packge_mode: bool = support::query_stored_value(
        &builder,
        nft_contract_key,
        vec![ACL_PACKAGE_MODE.to_string()],
    );

    assert!(is_acl_packge_mode);

    // Expect VariablesSet event.
    let expected_event = VariablesSet::new();
    let actual_event: VariablesSet = support::get_event(&builder, &nft_contract_key, 0).unwrap();
    assert_eq!(actual_event, expected_event, "Expected VariablesSet event.");
}

#[test]
fn installer_should_be_able_to_toggle_package_operator_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .build();

    builder.exec(install_request).expect_success().commit();

    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key: Key = *account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let nft_contract_hash = Key::into_hash(nft_contract_key)
        .map(ContractHash::new)
        .expect("failed to find nft contract");

    let is_package_operator_mode: bool = support::query_stored_value(
        &builder,
        nft_contract_key,
        vec![ARG_PACKAGE_OPERATOR_MODE.to_string()],
    );

    assert!(!is_package_operator_mode);

    // Installer account should be able to change package operator mode
    let installer_set_variables_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_VARIABLES,
        runtime_args! { ARG_PACKAGE_OPERATOR_MODE => true },
    )
    .build();

    builder
        .exec(installer_set_variables_request)
        .expect_success()
        .commit();

    let is_package_operator_mode: bool = support::query_stored_value(
        &builder,
        nft_contract_key,
        vec![PACKAGE_OPERATOR_MODE.to_string()],
    );

    assert!(is_package_operator_mode);

    // Expect VariablesSet event.
    let expected_event = VariablesSet::new();
    let actual_event: VariablesSet = support::get_event(&builder, &nft_contract_key, 0).unwrap();
    assert_eq!(actual_event, expected_event, "Expected VariablesSet event.");
}

#[test]
fn installer_should_be_able_to_toggle_operator_burn_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .build();

    builder.exec(install_request).expect_success().commit();

    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key: Key = *account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let nft_contract_hash = Key::into_hash(nft_contract_key)
        .map(ContractHash::new)
        .expect("failed to find nft contract");

    let is_package_operator_mode: bool = support::query_stored_value(
        &builder,
        nft_contract_key,
        vec![ARG_OPERATOR_BURN_MODE.to_string()],
    );

    assert!(!is_package_operator_mode);

    // Installer account should be able to change package operator mode
    let installer_set_variables_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_VARIABLES,
        runtime_args! { ARG_OPERATOR_BURN_MODE => true },
    )
    .build();

    builder
        .exec(installer_set_variables_request)
        .expect_success()
        .commit();

    let is_package_operator_mode: bool = support::query_stored_value(
        &builder,
        nft_contract_key,
        vec![OPERATOR_BURN_MODE.to_string()],
    );

    assert!(is_package_operator_mode);

    // Expect VariablesSet event.
    let expected_event = VariablesSet::new();
    let actual_event: VariablesSet = support::get_event(&builder, &nft_contract_key, 0).unwrap();
    assert_eq!(actual_event, expected_event, "Expected VariablesSet event.");
}
