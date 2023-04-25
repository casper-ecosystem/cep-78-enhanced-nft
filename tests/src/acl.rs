use core::panic;

use crate::utility::{
    constants::{
        ACCOUNT_USER_1, ARG_NFT_CONTRACT_HASH, ARG_REVERSE_LOOKUP, CONTRACT_1_0_0_WASM,
        MINTING_CONTRACT_WASM, NFT_CONTRACT_WASM, NFT_TEST_COLLECTION, NFT_TEST_SYMBOL,
        TEST_PRETTY_721_META_DATA,
    },
    installer_request_builder::{
        InstallerRequestBuilder, MintingMode, NFTHolderMode, NFTMetadataKind,
        OwnerReverseLookupMode, OwnershipMode, WhitelistMode,
    },
    support::{
        self, assert_expected_error, get_dictionary_value_from_key, get_minting_contract_hash,
        get_nft_contract_hash,
    },
};
use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{runtime_args, ContractHash, Key, RuntimeArgs};
use contract::{
    constants::{
        ACL_WHITELIST, ARG_ACL_WHITELIST, ARG_COLLECTION_NAME, ARG_CONTRACT_WHITELIST,
        ARG_MINTING_MODE, ARG_NAMED_KEY_CONVENTION, ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER,
        ENTRY_POINT_MINT, ENTRY_POINT_SET_VARIABLES, TOKEN_OWNERS,
    },
    modalities::NamedKeyConventionMode,
};

// Install

#[test]
fn should_install_with_acl_whitelist() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let minting_contract_hash = get_minting_contract_hash(&builder);

    let contract_whitelist = vec![Key::from(minting_contract_hash)];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::ACL)
        .with_acl_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let is_whitelisted_contract = support::get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &minting_contract_hash.to_string(),
    );

    assert!(is_whitelisted_contract, "acl whitelist is incorrectly set");
}

#[test]
fn should_install_with_deprecated_contract_whitelist() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let minting_contract_hash = get_minting_contract_hash(&builder);

    let contract_whitelist = vec![minting_contract_hash];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::ACL)
        .with_contract_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let is_whitelisted_contract = support::get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &minting_contract_hash.to_string(),
    );

    assert!(is_whitelisted_contract, "acl whitelist is incorrectly set");
}

#[test]
fn should_not_install_with_minting_mode_not_acl_if_acl_whitelist_provided() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let contract_whitelist = vec![ContractHash::default()];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::Installer) // Not the right minting mode for acl
        .with_contract_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_failure();

    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(
        actual_error,
        38u16,
        "should disallow installing without acl minting mode if non empty acl list",
    );
}

fn should_disallow_installation_of_contract_with_empty_locked_whitelist_in_public_mode_with_holder_mode(
    nft_holder_mode: NFTHolderMode,
) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_holder_mode(nft_holder_mode)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_minting_mode(MintingMode::Public)
        .build();

    builder.exec(install_request).expect_success().commit();
}

#[test]
fn should_allow_installation_of_contract_with_empty_locked_whitelist_in_public_mode() {
    should_disallow_installation_of_contract_with_empty_locked_whitelist_in_public_mode_with_holder_mode(
        NFTHolderMode::Accounts,
    );
    should_disallow_installation_of_contract_with_empty_locked_whitelist_in_public_mode_with_holder_mode(
        NFTHolderMode::Contracts,
    );
    should_disallow_installation_of_contract_with_empty_locked_whitelist_in_public_mode_with_holder_mode(
        NFTHolderMode::Mixed,
    );
}

#[test]
fn should_disallow_installation_with_contract_holder_mode_and_installer_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let contract_whitelist = vec![
        Key::Hash([1u8; 32]),
        Key::Hash([2u8; 32]),
        Key::Hash([3u8; 32]),
    ];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::Installer)
        .with_acl_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_failure();

    let error = builder.get_error().expect("should have an error");
    assert_expected_error(error, 38, "Invalid MintingMode (not ACL) and NFTHolderMode");
}

// Mint

#[test]
fn should_allow_whitelisted_account_to_mint() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let account_user_1 = support::create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1));
    let account_whitelist = vec![Key::from(account_user_1)];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Accounts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::ACL)
        .with_acl_whitelist(account_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let is_whitelisted_account = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &account_user_1.to_string(),
    );

    assert!(is_whitelisted_account, "acl whitelist is incorrectly set");

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER =>  Key::Account(account_user_1),
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        ARG_REVERSE_LOOKUP => false
    };

    let mint_session_call = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_id = 0u64;

    let actual_token_owner: Key = get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    );

    let minting_contract_key: Key = account_user_1.into();

    assert_eq!(actual_token_owner, minting_contract_key)
}

#[test]
fn should_disallow_unlisted_account_from_minting() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let account_whitelist = vec![Key::from(*DEFAULT_ACCOUNT_ADDR)];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Accounts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::ACL)
        .with_acl_whitelist(account_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let is_whitelisted_account = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &DEFAULT_ACCOUNT_ADDR.to_string(),
    );

    assert!(is_whitelisted_account, "acl whitelist is incorrectly set");

    let account_user_1 = support::create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1));

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER =>  Key::Account(account_user_1),
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        ARG_REVERSE_LOOKUP => false
    };

    let mint_session_call = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder.exec(mint_session_call).expect_failure();

    let error = builder.get_error().expect("should have an error");
    assert_expected_error(
        error,
        36,
        "Unlisted account hash should not be permitted to mint",
    );
}

#[test]
fn should_allow_whitelisted_contract_to_mint() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let minting_contract_hash = get_minting_contract_hash(&builder);

    let contract_whitelist = vec![Key::from(minting_contract_hash)];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::ACL)
        .with_acl_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let is_whitelisted_contract = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &minting_contract_hash.to_string(),
    );

    assert!(is_whitelisted_contract, "acl whitelist is incorrectly set");

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => Key::from(minting_contract_hash),
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        ARG_REVERSE_LOOKUP => false
    };

    let mint_via_contract_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder
        .exec(mint_via_contract_call)
        .expect_success()
        .commit();

    let token_id = 0u64;

    let actual_token_owner: Key = get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    );

    let minting_contract_key: Key = minting_contract_hash.into();

    assert_eq!(actual_token_owner, minting_contract_key)
}

#[test]
fn should_disallow_unlisted_contract_from_minting() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let minting_contract_hash = get_minting_contract_hash(&builder);
    let contract_whitelist = vec![
        Key::Hash([1u8; 32]),
        Key::Hash([2u8; 32]),
        Key::Hash([3u8; 32]),
    ];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::ACL)
        .with_acl_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => Key::from(minting_contract_hash),
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        ARG_REVERSE_LOOKUP => false
    };

    let mint_via_contract_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder.exec(mint_via_contract_call).expect_failure();

    let error = builder.get_error().expect("should have an error");
    assert_expected_error(
        error,
        81,
        "Unlisted contract hash should not be permitted to mint",
    );
}

#[test]
fn should_allow_mixed_account_contract_to_mint() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let minting_contract_hash = get_minting_contract_hash(&builder);
    let account_user_1 = support::create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1));
    let mixed_whitelist = vec![Key::from(minting_contract_hash), Key::from(account_user_1)];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Mixed)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::ACL)
        .with_acl_whitelist(mixed_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    // Contract
    let is_whitelisted_contract = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &minting_contract_hash.to_string(),
    );

    assert!(is_whitelisted_contract, "acl whitelist is incorrectly set");

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => Key::from(minting_contract_hash),
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        ARG_REVERSE_LOOKUP => false
    };

    let mint_via_contract_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder
        .exec(mint_via_contract_call)
        .expect_success()
        .commit();

    let token_id = 0u64;

    let actual_token_owner: Key = get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    );

    let minting_contract_key: Key = minting_contract_hash.into();

    assert_eq!(actual_token_owner, minting_contract_key);

    // User
    let is_whitelisted_account = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &account_user_1.to_string(),
    );

    assert!(is_whitelisted_account, "acl whitelist is incorrectly set");

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER =>  Key::Account(account_user_1),
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        ARG_REVERSE_LOOKUP => false
    };

    let mint_session_call = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_id = 1u64;

    let actual_token_owner: Key = get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    );

    let minting_contract_key: Key = account_user_1.into();

    assert_eq!(actual_token_owner, minting_contract_key)
}

#[test]
fn should_disallow_unlisted_contract_from_minting_with_mixed_account_contract() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let minting_contract_hash = get_minting_contract_hash(&builder);
    let account_user_1 = support::create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1));
    let mixed_whitelist = vec![
        Key::from(ContractHash::from([1u8; 32])),
        Key::from(account_user_1),
    ];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Mixed)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::ACL)
        .with_acl_whitelist(mixed_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => Key::from(minting_contract_hash),
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        ARG_REVERSE_LOOKUP => false
    };

    let mint_via_contract_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder.exec(mint_via_contract_call).expect_failure();

    let error = builder.get_error().expect("should have an error");
    assert_expected_error(
        error,
        81,
        "Unlisted contract hash should not be permitted to mint",
    );
}

#[test]
fn should_disallow_unlisted_account_from_minting_with_mixed_account_contract() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let minting_contract_hash = get_minting_contract_hash(&builder);
    let mixed_whitelist = vec![
        Key::from(minting_contract_hash),
        Key::from(*DEFAULT_ACCOUNT_ADDR),
    ];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Mixed)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::ACL)
        .with_acl_whitelist(mixed_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let is_whitelisted_account = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &DEFAULT_ACCOUNT_ADDR.to_string(),
    );

    assert!(is_whitelisted_account, "acl whitelist is incorrectly set");

    let account_user_1 = support::create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1));

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER =>  Key::Account(account_user_1),
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        ARG_REVERSE_LOOKUP => false
    };

    let mint_session_call = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder.exec(mint_session_call).expect_failure();

    let error = builder.get_error().expect("should have an error");
    assert_expected_error(
        error,
        36,
        "Unlisted account hash should not be permitted to mint",
    );
}

#[test]
fn should_disallow_listed_account_from_minting_with_nftholder_contract() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let minting_contract_hash = get_minting_contract_hash(&builder);
    let mixed_whitelist = vec![
        Key::from(minting_contract_hash),
        Key::from(*DEFAULT_ACCOUNT_ADDR),
    ];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::ACL)
        .with_acl_whitelist(mixed_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let is_whitelisted_account = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &DEFAULT_ACCOUNT_ADDR.to_string(),
    );

    assert!(is_whitelisted_account, "acl whitelist is incorrectly set");

    let account_user_1 = support::create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1));

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER =>  Key::Account(account_user_1),
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        ARG_REVERSE_LOOKUP => false
    };

    let mint_session_call = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder.exec(mint_session_call).expect_failure();

    let error = builder.get_error().expect("should have an error");
    assert_expected_error(error, 76, "InvalidHolderMode(76) must have been raised");
}

// Update

#[test]
fn should_be_able_to_update_whitelist_for_minting_with_deprecated_arg_contract_whitelist() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let minting_contract_hash = get_minting_contract_hash(&builder);

    let contract_whitelist = vec![];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Unlocked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::ACL)
        .with_acl_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let seed_uref = *builder
        .query(None, nft_contract_key, &[])
        .expect("must have nft contract")
        .as_contract()
        .expect("must convert contract")
        .named_keys()
        .get(ACL_WHITELIST)
        .expect("must have key")
        .as_uref()
        .expect("must convert to seed uref");

    let is_whitelisted_account =
        builder.query_dictionary_item(None, seed_uref, &minting_contract_hash.to_string());

    assert!(
        is_whitelisted_account.is_err(),
        "acl whitelist is incorrectly set"
    );

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        ARG_REVERSE_LOOKUP => false,
    };

    let mint_via_contract_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args.clone(),
    )
    .build();

    builder.exec(mint_via_contract_call).expect_failure();

    let error = builder.get_error().expect("should have an error");
    assert_expected_error(
        error,
        81,
        "Unlisted contract hash should not be permitted to mint",
    );

    let update_whitelist_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_VARIABLES,
        runtime_args! {
          ARG_CONTRACT_WHITELIST => vec![minting_contract_hash]
        },
    )
    .build();

    builder
        .exec(update_whitelist_request)
        .expect_success()
        .commit();

    let is_updated_acl_whitelist = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &minting_contract_hash.to_string(),
    );

    assert!(is_updated_acl_whitelist, "acl whitelist is incorrectly set");

    let mint_via_contract_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder
        .exec(mint_via_contract_call)
        .expect_success()
        .commit();
}

#[test]
fn should_be_able_to_update_whitelist_for_minting() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let minting_contract_hash = get_minting_contract_hash(&builder);

    let contract_whitelist = vec![];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Unlocked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::ACL)
        .with_acl_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let seed_uref = *builder
        .query(None, nft_contract_key, &[])
        .expect("must have nft contract")
        .as_contract()
        .expect("must convert contract")
        .named_keys()
        .get(ACL_WHITELIST)
        .expect("must have key")
        .as_uref()
        .expect("must convert to seed uref");

    let is_whitelisted_account =
        builder.query_dictionary_item(None, seed_uref, &minting_contract_hash.to_string());

    assert!(
        is_whitelisted_account.is_err(),
        "acl whitelist is incorrectly set"
    );

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        ARG_REVERSE_LOOKUP => false,
    };

    let mint_via_contract_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args.clone(),
    )
    .build();

    builder.exec(mint_via_contract_call).expect_failure();

    let error = builder.get_error().expect("should have an error");
    assert_expected_error(
        error,
        81,
        "Unlisted contract hash should not be permitted to mint",
    );

    let update_whitelist_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_VARIABLES,
        runtime_args! {
            ARG_ACL_WHITELIST => vec![Key::from(minting_contract_hash)]
        },
    )
    .build();

    builder
        .exec(update_whitelist_request)
        .expect_success()
        .commit();

    let is_updated_acl_whitelist = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &minting_contract_hash.to_string(),
    );

    assert!(is_updated_acl_whitelist, "acl whitelist is incorrectly set");

    let mint_via_contract_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder
        .exec(mint_via_contract_call)
        .expect_success()
        .commit();
}

// Upgrade

#[test]
fn should_upgrade_from_named_keys_to_dict_and_acl_minting_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let minting_contract_hash = get_minting_contract_hash(&builder);
    let contract_whitelist = vec![minting_contract_hash];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, CONTRACT_1_0_0_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1000u64)
        .with_minting_mode(MintingMode::Installer)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_contract_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash_1_0_0 = support::get_nft_contract_hash_1_0_0(&builder);
    let nft_contract_key_1_0_0: Key = nft_contract_hash_1_0_0.into();

    let minting_mode = support::query_stored_value::<u8>(
        &builder,
        nft_contract_key_1_0_0,
        vec![ARG_MINTING_MODE.to_string()],
    );

    assert_eq!(
        minting_mode,
        MintingMode::Installer as u8,
        "minting mode should be set to public"
    );

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => support::get_nft_contract_package_hash(&builder),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let is_updated_acl_whitelist = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &minting_contract_hash.to_string(),
    );

    assert!(is_updated_acl_whitelist, "acl whitelist is incorrectly set");

    let minting_mode = support::query_stored_value::<u8>(
        &builder,
        nft_contract_key,
        vec![ARG_MINTING_MODE.to_string()],
    );

    assert_eq!(
        minting_mode,
        MintingMode::ACL as u8,
        "minting mode should be set to acl"
    );
}
