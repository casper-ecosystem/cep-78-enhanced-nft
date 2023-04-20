use core::panic;

use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{runtime_args, ContractHash, Key, RuntimeArgs};
use contract::constants::{
    ACL_WHITELIST, ARG_ACL_WHITELIST, ARG_CONTRACT_WHITELIST, ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER,
    ENTRY_POINT_MINT, ENTRY_POINT_SET_VARIABLES, TOKEN_OWNERS,
};

use crate::utility::{
    constants::{
        ARG_NFT_CONTRACT_HASH, ARG_REVERSE_LOOKUP, MINTING_CONTRACT_WASM, NFT_CONTRACT_WASM,
        TEST_PRETTY_721_META_DATA,
    },
    installer_request_builder::{
        InstallerRequestBuilder, MintingMode, NFTHolderMode, OwnerReverseLookupMode, OwnershipMode,
        WhitelistMode,
    },
    support::{
        self, assert_expected_error, get_dictionary_value_from_key, get_minting_contract_hash,
        get_nft_contract_hash,
    },
};

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
        .with_minting_mode(MintingMode::Acl)
        .with_contract_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let is_whitelisted_account = support::get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &minting_contract_hash.to_string(),
    );

    assert!(is_whitelisted_account, "acl whitelist is incorrectly set");
}

#[test]
fn should_not_install_with_minting_mode_not_acl_and_a_acl_list() {
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
        .with_minting_mode(MintingMode::Acl)
        .with_acl_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let is_whitelisted_account = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &minting_contract_hash.to_string(),
    );

    assert!(is_whitelisted_account, "acl whitelist is incorrectly set");

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
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
        Key::from(ContractHash::from([1u8; 32])),
        Key::from(ContractHash::from([2u8; 32])),
        Key::from(ContractHash::from([3u8; 32])),
    ];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::Acl)
        .with_acl_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
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
        .with_minting_mode(MintingMode::Acl)
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
        .with_minting_mode(MintingMode::Acl)
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
