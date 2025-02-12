use crate::utility::{
    constants::{
        ACCOUNT_1_ADDR, ACCOUNT_1_KEY, ARG_NFT_CONTRACT_HASH, ARG_REVERSE_LOOKUP, CONTRACT_NAME,
        DEFAULT_ACCOUNT_KEY, MINTING_CONTRACT_WASM, MINT_SESSION_WASM, NFT_CONTRACT_WASM,
        NFT_TEST_COLLECTION, TEST_PRETTY_721_META_DATA,
    },
    installer_request_builder::{
        BurnMode, InstallerRequestBuilder, MetadataMutability, MintingMode, NFTHolderMode,
        NFTIdentifierMode, OwnerReverseLookupMode, OwnershipMode, WhitelistMode,
    },
    support::{
        self, genesis, get_dictionary_value_from_key, get_minting_contract_hash,
        get_minting_contract_package_hash, get_nft_contract_hash, get_nft_contract_hash_key,
    },
};
use casper_engine_test_support::{ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR};
use casper_types::{
    contracts::{ContractHash, ContractPackageHash},
    runtime_args, Key,
};
use cep78::{
    constants::{
        ARG_APPROVE_ALL, ARG_COLLECTION_NAME, ARG_OPERATOR, ARG_TOKEN_HASH, ARG_TOKEN_ID,
        ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, BURNT_TOKENS, BURN_MODE, ENTRY_POINT_BURN,
        ENTRY_POINT_MINT, ENTRY_POINT_SET_APPROVALL_FOR_ALL, TOKEN_COUNT,
    },
    events::events_ces::Burn,
    modalities::TokenIdentifier,
};

fn should_burn_minted_token(reporting: OwnerReverseLookupMode) {
    let mut builder = genesis();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(100u64)
            .with_ownership_mode(OwnershipMode::Transferable)
            .with_reporting_mode(reporting)
            .build();

    builder
        .exec(install_request_builder)
        .expect_success()
        .commit();

    let nft_contract_hash: ContractHash = get_nft_contract_hash(&builder).into();
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let token_owner: Key = *DEFAULT_ACCOUNT_KEY;
    let token_id = 0u64;

    let reverse_lookup_enabled: bool = reporting == OwnerReverseLookupMode::Complete;
    if reverse_lookup_enabled {
        let mint_session_call = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            MINT_SESSION_WASM,
            runtime_args! {
                ARG_NFT_CONTRACT_HASH => nft_contract_key,
                ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
                ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
                ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
            },
        )
        .build();

        builder.exec(mint_session_call).expect_success().commit();

        let token_page = support::get_token_page_by_id(
            &builder,
            &nft_contract_key,
            &DEFAULT_ACCOUNT_KEY,
            token_id,
        );

        assert!(token_page[0]);
    } else {
        let mint_runtime_args = runtime_args! {
            ARG_TOKEN_OWNER => token_owner,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        };

        let minting_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash.into(),
            ENTRY_POINT_MINT,
            mint_runtime_args,
        )
        .build();

        builder.exec(minting_request).expect_success().commit();
    }

    let actual_balance_before_burn = support::get_dictionary_value_from_key::<u64>(
        &builder,
        &nft_contract_key,
        TOKEN_COUNT,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    let expected_balance_before_burn = 1u64;
    assert_eq!(actual_balance_before_burn, expected_balance_before_burn);

    let burn_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
        },
    )
    .build();
    builder.exec(burn_request).expect_success().commit();

    // This will error if token is not registered as burnt.
    support::get_dictionary_value_from_key::<()>(
        &builder,
        &nft_contract_key,
        BURNT_TOKENS,
        &token_id.to_string(),
    );

    // This will error if token is not registered as burnt
    let actual_balance = support::get_dictionary_value_from_key::<u64>(
        &builder,
        &nft_contract_key,
        TOKEN_COUNT,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    let expected_balance = 0u64;
    assert_eq!(actual_balance, expected_balance);

    // Expect Burn event.
    // let expected_event = Burn::new(token_owner, TokenIdentifier::Index(token_id), token_owner);
    // let actual_event: Burn = support::get_event(&builder, &nft_contract_key, 1).unwrap();
    // assert_eq!(actual_event, expected_event, "Expected Burn event.");
}

#[test]
fn should_burn_minted_token_with_complete_reporting() {
    should_burn_minted_token(OwnerReverseLookupMode::Complete);
}

#[test]
fn should_burn_minted_token_with_transfer_only_reporting() {
    should_burn_minted_token(OwnerReverseLookupMode::TransfersOnly);
}

#[test]
fn should_not_burn_previously_burnt_token() {
    let mut builder = genesis();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(100u64)
            .with_ownership_mode(OwnershipMode::Transferable)
            .build();

    builder
        .exec(install_request_builder)
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_page =
        support::get_token_page_by_id(&builder, &nft_contract_key, &DEFAULT_ACCOUNT_KEY, 0u64);

    assert!(token_page[0]);

    let burn_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => 0u64,
        },
    )
    .build();

    builder.exec(burn_request).expect_success().commit();

    let re_burn_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => 0u64,
        },
    )
    .build();

    builder.exec(re_burn_request).expect_failure();

    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(
        actual_error,
        42u16,
        "should disallow burning of previously burnt token",
    );
}

#[test]
fn should_return_expected_error_when_burning_non_existing_token() {
    let mut builder = genesis();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(100u64)
            .with_ownership_mode(OwnershipMode::Transferable)
            .build();

    builder
        .exec(install_request_builder)
        .expect_success()
        .commit();

    let token_id = 0u64;

    let burn_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
        },
    )
    .build();

    builder.exec(burn_request).expect_failure();

    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(
        actual_error,
        155u16,
        "should return InvalidTokenID error when trying to burn a non_existing token",
    );
}

#[test]
fn should_return_expected_error_burning_of_others_users_token() {
    let mut builder = genesis();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(100u64)
            .with_ownership_mode(OwnershipMode::Transferable)
            .build();

    builder
        .exec(install_request_builder)
        .expect_success()
        .commit();

    let nft_contract_hash: ContractHash = get_nft_contract_hash(&builder).into();
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let account_user_1 = ACCOUNT_1_ADDR.to_owned();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_page =
        support::get_token_page_by_id(&builder, &nft_contract_key, &DEFAULT_ACCOUNT_KEY, 0u64);

    assert!(token_page[0]);

    let incorrect_burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        nft_contract_hash.into(),
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => 0u64,
        },
    )
    .build();

    builder.exec(incorrect_burn_request).expect_failure();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(error, 6u16, "should disallow burning of other users' token");
}

#[test]
fn should_allow_contract_to_burn_token() {
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
        .with_minting_mode(MintingMode::Acl)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_acl_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        ARG_REVERSE_LOOKUP => false,
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

    let current_token_balance = get_dictionary_value_from_key::<u64>(
        &builder,
        &nft_contract_key,
        TOKEN_COUNT,
        &minting_contract_hash.to_string(),
    );

    assert_eq!(1u64, current_token_balance);

    let burn_via_contract_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash.into(),
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_ID => 0u64
        },
    )
    .build();

    builder
        .exec(burn_via_contract_call)
        .expect_success()
        .commit();

    let updated_token_balance = get_dictionary_value_from_key::<u64>(
        &builder,
        &nft_contract_key,
        TOKEN_COUNT,
        &minting_contract_hash.to_string(),
    );

    assert_eq!(updated_token_balance, 0u64)
}

#[test]
fn should_not_burn_in_non_burn_mode() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_burn_mode(BurnMode::NonBurnable)
        .with_reporting_mode(OwnerReverseLookupMode::Complete)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);
    let burn_mode: u8 = builder
        .query(None, nft_contract_key, &[BURN_MODE.to_string()])
        .unwrap()
        .as_cl_value()
        .unwrap()
        .to_owned()
        .into_t::<u8>()
        .unwrap();

    assert_eq!(burn_mode, BurnMode::NonBurnable as u8);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_id = 0u64;
    let burn_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
        },
    )
    .build();
    builder.exec(burn_request).expect_failure();

    let error = builder.get_error().expect("burn must have failed");
    support::assert_expected_error(error, 106, "InvalidBurnMode(106) must have been raised");
}

#[test]
fn should_let_account_operator_burn_tokens_with_operator_burn_mode() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_reporting_mode(OwnerReverseLookupMode::Complete)
        .with_operator_burn_mode(true)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash: ContractHash = get_nft_contract_hash(&builder).into();
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let token_owner: Key = *DEFAULT_ACCOUNT_KEY;

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => token_owner,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_id = 0u64;
    let operator = ACCOUNT_1_ADDR.to_owned();
    let operator_key = *ACCOUNT_1_KEY;

    let burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        operator,
        nft_contract_hash.into(),
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
        },
    )
    .build();

    builder.exec(burn_request).expect_failure();

    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(
        actual_error,
        6u16,
        "InvalidTokenOwner should not allow burn by non operator",
    );

    let approve_all_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash.into(),
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => operator_key
        },
    )
    .build();

    builder.exec(approve_all_request).expect_success().commit();

    let burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        operator,
        nft_contract_hash.into(),
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
        },
    )
    .build();
    builder.exec(burn_request).expect_success().commit();

    // This will error if token is not registered as burnt.
    support::get_dictionary_value_from_key::<()>(
        &builder,
        &nft_contract_key,
        BURNT_TOKENS,
        &token_id.to_string(),
    );

    // This will error if token is not registered as burnt
    let actual_balance = support::get_dictionary_value_from_key::<u64>(
        &builder,
        &nft_contract_key,
        TOKEN_COUNT,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    let expected_balance = 0u64;
    assert_eq!(actual_balance, expected_balance);

    // Expect Burn event. Mint is first event, burn is second event.
    let actual_event_index = 2;
    let actual_event: Burn =
        support::get_event(&builder, &nft_contract_key, actual_event_index).unwrap();

    let burner = operator_key; // Burner is operator account

    let expected_event = Burn::new(token_owner, &TokenIdentifier::Index(token_id), burner);
    assert_eq!(actual_event, expected_event, "Expected Burn event.");
}

#[test]
fn should_let_contract_operator_burn_tokens_with_operator_burn_mode() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_reporting_mode(OwnerReverseLookupMode::Complete)
        .with_operator_burn_mode(true)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash: ContractHash = get_nft_contract_hash(&builder).into();
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let token_owner: Key = *DEFAULT_ACCOUNT_KEY;

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => token_owner,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_id = 0u64;

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
    let operator = minting_contract_hash;
    let account_user_1 = ACCOUNT_1_ADDR.to_owned();

    let burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        minting_contract_hash,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
        },
    )
    .build();

    builder.exec(burn_request).expect_failure();

    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(
        actual_error,
        6u16,
        "InvalidTokenOwner should not allow burn by non operator",
    );

    let approve_all_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash.into(),
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => Key::Hash(operator.value())
        },
    )
    .build();

    builder.exec(approve_all_request).expect_success().commit();

    let burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        minting_contract_hash,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
        },
    )
    .build();
    builder.exec(burn_request).expect_success().commit();

    // This will error if token is not registered as burnt.
    support::get_dictionary_value_from_key::<()>(
        &builder,
        &nft_contract_key,
        BURNT_TOKENS,
        &token_id.to_string(),
    );

    // This will error if token is not registered as burnt
    let actual_balance = support::get_dictionary_value_from_key::<u64>(
        &builder,
        &nft_contract_key,
        TOKEN_COUNT,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    let expected_balance = 0u64;
    assert_eq!(actual_balance, expected_balance);

    // Expect Burn event. Mint is first event, burn is second event.
    let actual_event_index = 2;
    let actual_event: Burn =
        support::get_event(&builder, &nft_contract_key, actual_event_index).unwrap();

    let burner = Key::Hash(minting_contract_hash.value()); // Burner is contract not session caller ACCOUNT_USER_1

    let expected_event = Burn::new(token_owner, &TokenIdentifier::Index(token_id), burner);
    assert_eq!(actual_event, expected_event, "Expected Burn event.");
}

#[test]
fn should_let_package_operator_burn_tokens_with_contract_package_mode_and_operator_burn_mode() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_reporting_mode(OwnerReverseLookupMode::Complete)
        .with_package_operator_mode(true)
        .with_operator_burn_mode(true)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash: ContractHash = get_nft_contract_hash(&builder).into();
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let token_owner: Key = *DEFAULT_ACCOUNT_KEY;

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => token_owner,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_id = 0u64;

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
    let operator = ContractPackageHash::new(minting_contract_package_hash.value());
    let account_user_1 = ACCOUNT_1_ADDR.to_owned();

    let burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        minting_contract_hash,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
        },
    )
    .build();

    builder.exec(burn_request).expect_failure();

    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(
        actual_error,
        6u16,
        "InvalidTokenOwner should not allow burn by non operator",
    );

    let approve_all_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash.into(),
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => Key::from(operator)
        },
    )
    .build();

    builder.exec(approve_all_request).expect_success().commit();

    let burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        minting_contract_hash,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
        },
    )
    .build();
    builder.exec(burn_request).expect_success().commit();

    // This will error if token is not registered as burnt.
    support::get_dictionary_value_from_key::<()>(
        &builder,
        &nft_contract_key,
        BURNT_TOKENS,
        &token_id.to_string(),
    );

    // This will error if token is not registered as burnt
    let actual_balance = support::get_dictionary_value_from_key::<u64>(
        &builder,
        &nft_contract_key,
        TOKEN_COUNT,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    let expected_balance = 0u64;
    assert_eq!(actual_balance, expected_balance);

    // Expect Burn event. Mint is first event, burn is second event.
    let actual_event_index = 2;
    let actual_event: Burn =
        support::get_event(&builder, &nft_contract_key, actual_event_index).unwrap();

    let burner = Key::Hash(minting_contract_hash.value()); // Burner is contract not its package nor session caller ACCOUNT_USER_1

    let expected_event = Burn::new(token_owner, &TokenIdentifier::Index(token_id), burner);
    assert_eq!(actual_event, expected_event, "Expected Burn event.");
}

#[test]
fn should_burn_token_in_hash_identifier_mode() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_reporting_mode(OwnerReverseLookupMode::Complete)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash: ContractHash = get_nft_contract_hash(&builder).into();
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA ,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_hash: String =
        base16::encode_lower(&support::create_blake2b_hash(TEST_PRETTY_721_META_DATA));

    let burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash.into(),
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_HASH => token_hash,
        },
    )
    .build();

    builder.exec(burn_request).expect_success().commit();
}
