use crate::{
    acl::support::get_minting_contract_hash_key,
    utility::{
        constants::{
            ACCOUNT_1_ADDR, ACCOUNT_1_KEY, ARG_NFT_CONTRACT_HASH, ARG_REVERSE_LOOKUP,
            DEFAULT_ACCOUNT_KEY, MINTING_CONTRACT_VERSION, MINTING_CONTRACT_WASM,
            NFT_CONTRACT_WASM, TEST_PRETTY_721_META_DATA,
        },
        installer_request_builder::{
            InstallerRequestBuilder, MintingMode, NFTHolderMode, OwnerReverseLookupMode,
            OwnershipMode, WhitelistMode,
        },
        support::{
            self, assert_expected_error, genesis, get_dictionary_value_from_key,
            get_minting_contract_hash, get_minting_contract_package_hash, get_nft_contract_hash,
            get_nft_contract_hash_key,
        },
    },
};
use casper_engine_test_support::{ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR};
use casper_types::{
    contracts::{ContractHash, ContractPackageHash},
    runtime_args, Key,
};
use cep78::constants::{
    ACL_WHITELIST, ARG_ACL_WHITELIST, ARG_CONTRACT_WHITELIST, ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER,
    ENTRY_POINT_MINT, ENTRY_POINT_SET_VARIABLES, TOKEN_OWNERS,
};

// Install

#[test]
fn should_install_with_acl_whitelist() {
    let mut builder = genesis();

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

    // TODO check
    let minting_contract_hash: ContractHash = get_minting_contract_hash(&builder).into();
    let contract_whitelist = vec![Key::from(minting_contract_hash)];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::Acl)
        .with_acl_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

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
    let mut builder = genesis();

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

    // TODO check
    let minting_contract_hash: ContractHash = get_minting_contract_hash(&builder).into();
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

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

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
    let mut builder = genesis();

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
    let mut builder = genesis();

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
    let mut builder = genesis();

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
    let mut builder = genesis();
    let account_user_1 = ACCOUNT_1_ADDR.to_owned();
    let account_whitelist = vec![*ACCOUNT_1_KEY];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Accounts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::Acl)
        .with_acl_whitelist(account_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let is_whitelisted_account = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &ACCOUNT_1_ADDR.to_string(),
    );

    assert!(is_whitelisted_account, "acl whitelist is incorrectly set");

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER =>  *ACCOUNT_1_KEY,
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

    let minting_contract_key: Key = *ACCOUNT_1_KEY;

    assert_eq!(actual_token_owner, minting_contract_key)
}

#[test]
fn should_disallow_unlisted_account_from_minting() {
    let mut builder = genesis();

    let account_whitelist = vec![Key::from(*DEFAULT_ACCOUNT_ADDR)];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Accounts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::Acl)
        .with_acl_whitelist(account_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let is_whitelisted_account = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &DEFAULT_ACCOUNT_ADDR.to_string(),
    );

    assert!(is_whitelisted_account, "acl whitelist is incorrectly set");

    let account_user_1 = ACCOUNT_1_ADDR.to_owned();

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
    let mut builder = genesis();

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

    // TODO check
    let minting_contract_hash: ContractHash = get_minting_contract_hash(&builder).into();
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

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

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
        minting_contract_hash.into(),
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

    let minting_contract_key: Key = Key::Hash(minting_contract_hash.value());

    assert_eq!(actual_token_owner, minting_contract_key)
}

#[test]
fn should_disallow_unlisted_contract_from_minting() {
    let mut builder = genesis();

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
        .with_minting_mode(MintingMode::Acl)
        .with_acl_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let owner = get_minting_contract_hash_key(&builder);

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => owner,
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
    let mut builder = genesis();

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

    let account_user_1 = ACCOUNT_1_ADDR.to_owned();
    let account_user_1_key = ACCOUNT_1_KEY.to_owned();
    let mixed_whitelist = vec![Key::Hash(minting_contract_hash.value()), account_user_1_key];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Mixed)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::Acl)
        .with_acl_whitelist(mixed_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    // Contract
    let is_whitelisted_contract = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &minting_contract_hash.to_string(),
    );

    assert!(is_whitelisted_contract, "acl whitelist is incorrectly set");

    let owner = get_minting_contract_hash_key(&builder);

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => owner,
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

    let minting_contract_key: Key = Key::Hash(minting_contract_hash.value());

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
        ARG_TOKEN_OWNER =>  account_user_1_key,
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

    let minting_contract_key: Key = account_user_1_key;

    assert_eq!(actual_token_owner, minting_contract_key)
}

#[test]
fn should_disallow_unlisted_contract_from_minting_with_mixed_account_contract() {
    let mut builder = genesis();

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
    let account_user_1 = ACCOUNT_1_ADDR.to_owned();
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
        .with_minting_mode(MintingMode::Acl)
        .with_acl_whitelist(mixed_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);
    let owner = get_minting_contract_hash_key(&builder);

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => owner,
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
    let mut builder = genesis();

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

    // TODO check
    let minting_contract_hash: ContractHash = get_minting_contract_hash(&builder).into();
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
        .with_minting_mode(MintingMode::Acl)
        .with_acl_whitelist(mixed_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash: ContractHash = get_nft_contract_hash(&builder).into();
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let is_whitelisted_account = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &DEFAULT_ACCOUNT_ADDR.to_string(),
    );

    assert!(is_whitelisted_account, "acl whitelist is incorrectly set");

    let account_user_1 = ACCOUNT_1_ADDR.to_owned();

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER =>  Key::Account(account_user_1),
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        ARG_REVERSE_LOOKUP => false
    };

    let mint_session_call = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        nft_contract_hash.into(),
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
    let mut builder = genesis();

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

    // TODO check
    let minting_contract_hash: ContractHash = get_minting_contract_hash(&builder).into();
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
        .with_minting_mode(MintingMode::Acl)
        .with_acl_whitelist(mixed_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash: ContractHash = get_nft_contract_hash(&builder).into();
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let is_whitelisted_account = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &DEFAULT_ACCOUNT_ADDR.to_string(),
    );

    assert!(is_whitelisted_account, "acl whitelist is incorrectly set");

    let account_user_1 = ACCOUNT_1_ADDR.to_owned();

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER =>  Key::Account(account_user_1),
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        ARG_REVERSE_LOOKUP => false
    };

    let mint_session_call = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        nft_contract_hash.into(),
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder.exec(mint_session_call).expect_failure();

    let error = builder.get_error().expect("should have an error");
    assert_expected_error(error, 76, "InvalidHolderMode(76) must have been raised");
}

#[test]
fn should_disallow_contract_from_whitelisted_package_to_mint_without_acl_package_mode() {
    let mut builder = genesis();

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
    let minting_contract_package_hash = get_minting_contract_package_hash(&builder);

    let contract_whitelist = vec![Key::from(minting_contract_package_hash)];

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

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let is_whitelisted_contract_package = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &minting_contract_package_hash.to_string(),
    );

    assert!(
        is_whitelisted_contract_package,
        "acl whitelist is incorrectly set"
    );

    let owner = get_minting_contract_hash_key(&builder);

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => owner,
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
        "Unlisted contract hash from whitelisted ContractPackageHash can not mint without ACL package mode",
    );
}

#[test]
fn should_allow_contract_from_whitelisted_package_to_mint_with_acl_package_mode() {
    let mut builder = genesis();

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
    let minting_contract_package_hash = get_minting_contract_package_hash(&builder);

    let contract_whitelist = vec![Key::Hash(minting_contract_package_hash.value())];
    let acl_package_mode = true;

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::Acl)
        .with_acl_whitelist(contract_whitelist)
        .with_acl_package_mode(acl_package_mode)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let is_whitelisted_contract_package = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &ContractPackageHash::new(minting_contract_package_hash.value()).to_string(),
    );

    assert!(
        is_whitelisted_contract_package,
        "acl whitelist is incorrectly set"
    );

    let owner = get_minting_contract_hash_key(&builder);

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => owner,
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

    let minting_contract_key: Key = Key::Hash(minting_contract_hash.value());

    assert_eq!(actual_token_owner, minting_contract_key)
}

#[test]
fn should_allow_contract_from_whitelisted_package_to_mint_with_acl_package_mode_after_contract_upgrade(
) {
    let mut builder = genesis();

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
    let minting_contract_package_hash = get_minting_contract_package_hash(&builder);

    let contract_whitelist = vec![Key::Hash(minting_contract_package_hash.value())];
    let acl_package_mode = true;

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::Acl)
        .with_acl_whitelist(contract_whitelist)
        .with_acl_package_mode(acl_package_mode)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let is_whitelisted_contract_package = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &ContractPackageHash::new(minting_contract_package_hash.value()).to_string(),
    );

    assert!(
        is_whitelisted_contract_package,
        "acl whitelist is incorrectly set"
    );

    let version_minting_contract = support::query_stored_value::<u32>(
        &builder,
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        MINTING_CONTRACT_VERSION,
    );

    assert_eq!(version_minting_contract, 1u32);

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let version_minting_contract = support::query_stored_value::<u32>(
        &builder,
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        MINTING_CONTRACT_VERSION,
    );

    assert_eq!(version_minting_contract, 2u32);

    let minting_upgraded_contract_hash = get_minting_contract_hash(&builder);

    assert_ne!(minting_contract_hash, minting_upgraded_contract_hash);

    let owner = get_minting_contract_hash_key(&builder);

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => owner,
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        ARG_REVERSE_LOOKUP => false
    };

    let mint_via_contract_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_upgraded_contract_hash,
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

    let minting_contract_key: Key = Key::Hash(minting_upgraded_contract_hash.value());

    assert_eq!(actual_token_owner, minting_contract_key)
}

// Update

#[test]
fn should_be_able_to_update_whitelist_for_minting_with_deprecated_arg_contract_whitelist() {
    let mut builder = genesis();

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

    let nft_contract_hash: ContractHash = get_nft_contract_hash(&builder).into();
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let seed_uref = *builder
        .get_entity_with_named_keys_by_entity_hash(nft_contract_hash.into())
        .expect("must have named keys")
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
        ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
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
        nft_contract_hash.into(),
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
    let mut builder = genesis();

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

    let nft_contract_hash: ContractHash = get_nft_contract_hash(&builder).into();
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let seed_uref = *builder
        .get_entity_with_named_keys_by_entity_hash(nft_contract_hash.into())
        .expect("must have named keys")
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
        ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
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

    let minting_contract_hash_key = get_minting_contract_hash_key(&builder);

    let update_whitelist_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash.into(),
        ENTRY_POINT_SET_VARIABLES,
        runtime_args! {
            ARG_ACL_WHITELIST => vec![minting_contract_hash_key]
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
