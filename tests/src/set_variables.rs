use casper_engine_test_support::{ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR};
use casper_types::{runtime_args, Key};
use cep78::{
    constants::{
        ACL_PACKAGE_MODE, ALLOW_MINTING, ARG_ACL_PACKAGE_MODE, ARG_ALLOW_MINTING,
        ARG_OPERATOR_BURN_MODE, ARG_PACKAGE_OPERATOR_MODE, ENTRY_POINT_SET_VARIABLES,
        OPERATOR_BURN_MODE, PACKAGE_OPERATOR_MODE,
    },
    error::NFTCoreError,
    events::events_ces::VariablesSet,
};

use crate::utility::{
    constants::{ACCOUNT_1_ADDR, NFT_CONTRACT_WASM, NFT_TEST_COLLECTION, NFT_TEST_SYMBOL},
    installer_request_builder::{InstallerRequestBuilder, OwnerReverseLookupMode},
    support::{
        self, assert_expected_error, genesis, get_nft_contract_hash, get_nft_contract_hash_key,
    },
};

#[test]
fn only_installer_should_be_able_to_toggle_allow_minting() {
    let mut builder = genesis();

    let other_user_account = ACCOUNT_1_ADDR.to_owned();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_allowing_minting(false)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

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
        support::query_stored_value(&builder, nft_contract_key, ALLOW_MINTING);

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
        support::query_stored_value(&builder, nft_contract_key, ALLOW_MINTING);

    assert!(allow_minting);

    // Expect VariablesSet event.
    let expected_event = VariablesSet::new();
    let actual_event: VariablesSet = support::get_event(&builder, &nft_contract_key, 0).unwrap();
    assert_eq!(actual_event, expected_event, "Expected VariablesSet event.");
}

#[test]
fn installer_should_be_able_to_toggle_acl_package_mode() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key = get_nft_contract_hash_key(&builder);

    let is_acl_packge_mode: bool =
        support::query_stored_value(&builder, nft_contract_key, ARG_ACL_PACKAGE_MODE);

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

    let is_acl_packge_mode: bool =
        support::query_stored_value(&builder, nft_contract_key, ACL_PACKAGE_MODE);

    assert!(is_acl_packge_mode);

    // Expect VariablesSet event.
    let expected_event = VariablesSet::new();
    let actual_event: VariablesSet = support::get_event(&builder, &nft_contract_key, 0).unwrap();
    assert_eq!(actual_event, expected_event, "Expected VariablesSet event.");
}

#[test]
fn installer_should_be_able_to_toggle_package_operator_mode() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key = get_nft_contract_hash_key(&builder);

    let is_package_operator_mode: bool =
        support::query_stored_value(&builder, nft_contract_key, ARG_PACKAGE_OPERATOR_MODE);

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

    let is_package_operator_mode: bool =
        support::query_stored_value(&builder, nft_contract_key, PACKAGE_OPERATOR_MODE);

    assert!(is_package_operator_mode);

    // Expect VariablesSet event.
    let expected_event = VariablesSet::new();
    let actual_event: VariablesSet = support::get_event(&builder, &nft_contract_key, 0).unwrap();
    assert_eq!(actual_event, expected_event, "Expected VariablesSet event.");
}

#[test]
fn installer_should_be_able_to_toggle_operator_burn_mode() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key = get_nft_contract_hash_key(&builder);

    let is_package_operator_mode: bool =
        support::query_stored_value(&builder, nft_contract_key, ARG_OPERATOR_BURN_MODE);

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

    let is_package_operator_mode: bool =
        support::query_stored_value(&builder, nft_contract_key, OPERATOR_BURN_MODE);

    assert!(is_package_operator_mode);

    // Expect VariablesSet event.
    let expected_event = VariablesSet::new();
    let actual_event: VariablesSet = support::get_event(&builder, &nft_contract_key, 0).unwrap();
    assert_eq!(actual_event, expected_event, "Expected VariablesSet event.");
}
