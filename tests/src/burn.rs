use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    PRODUCTION_RUN_GENESIS_REQUEST,
};
use casper_types::{account::AccountHash, runtime_args, ContractHash, Key, RuntimeArgs};
use contract::{
    constants::{
        ARG_APPROVE_ALL, ARG_COLLECTION_NAME, ARG_OPERATOR, ARG_TOKEN_HASH, ARG_TOKEN_ID,
        ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, BURNT_TOKENS, BURN_MODE, ENTRY_POINT_BURN,
        ENTRY_POINT_MINT, ENTRY_POINT_SET_APPROVALL_FOR_ALL, TOKEN_COUNT,
    },
    events::events_ces::Burn,
    modalities::TokenIdentifier,
};

use crate::utility::{
    constants::{
        ACCOUNT_USER_1, ARG_NFT_CONTRACT_HASH, ARG_REVERSE_LOOKUP, CONTRACT_NAME,
        MINTING_CONTRACT_WASM, MINT_SESSION_WASM, NFT_CONTRACT_WASM, NFT_TEST_COLLECTION,
        TEST_PRETTY_721_META_DATA,
    },
    installer_request_builder::{
        BurnMode, InstallerRequestBuilder, MetadataMutability, MintingMode, NFTHolderMode,
        NFTIdentifierMode, OwnerReverseLookupMode, OwnershipMode, WhitelistMode,
    },
    support::{
        self, get_dictionary_value_from_key, get_minting_contract_hash, get_nft_contract_hash,
    },
};

fn should_burn_minted_token(reporting: OwnerReverseLookupMode) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

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

    let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = installing_account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let token_owner: Key = Key::Account(*DEFAULT_ACCOUNT_ADDR);
    let token_id = 0u64;

    let reverse_lookup_enabled: bool = reporting == OwnerReverseLookupMode::Complete;
    if reverse_lookup_enabled {
        let mint_session_call = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            MINT_SESSION_WASM,
            runtime_args! {
                ARG_NFT_CONTRACT_HASH => Key::Hash(nft_contract_hash.value()),
                ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
                ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
                ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
            },
        )
        .build();

        builder.exec(mint_session_call).expect_success().commit();

        let token_page = support::get_token_page_by_id(
            &builder,
            nft_contract_key,
            &Key::Account(*DEFAULT_ACCOUNT_ADDR),
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
            nft_contract_hash,
            ENTRY_POINT_MINT,
            mint_runtime_args,
        )
        .build();

        builder.exec(minting_request).expect_success().commit();
    }

    let actual_balance_before_burn = support::get_dictionary_value_from_key::<u64>(
        &builder,
        nft_contract_key,
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

    //This will error of token is not registered as burnt.
    support::get_dictionary_value_from_key::<()>(
        &builder,
        nft_contract_key,
        BURNT_TOKENS,
        &token_id.to_string(),
    );

    // This will error of token is not registered as
    let actual_balance = support::get_dictionary_value_from_key::<u64>(
        &builder,
        nft_contract_key,
        TOKEN_COUNT,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    let expected_balance = 0u64;
    assert_eq!(actual_balance, expected_balance);

    // Expect Burn event.
    let expected_event = Burn::new(token_owner, TokenIdentifier::Index(0));
    let actual_event: Burn = support::get_event(&builder, nft_contract_key, 1);
    assert_eq!(actual_event, expected_event, "Expected Burn event.");
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
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(100u64)
            .with_ownership_mode(OwnershipMode::Transferable)
            .build();

    builder
        .exec(install_request_builder)
        .expect_success()
        .commit();

    let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = installing_account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => Key::Hash(nft_contract_hash.value()),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_page = support::get_token_page_by_id(
        &builder,
        nft_contract_key,
        &Key::Account(*DEFAULT_ACCOUNT_ADDR),
        0u64,
    );

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
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

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
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(100u64)
            .with_ownership_mode(OwnershipMode::Transferable)
            .build();

    builder
        .exec(install_request_builder)
        .expect_success()
        .commit();

    let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = installing_account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");
    let nft_contract_hash = nft_contract_key
        .into_hash()
        .expect("must convert to hash addr");

    let account_user_1 = support::create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1));

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => Key::Hash(nft_contract_hash),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_page = support::get_token_page_by_id(
        &builder,
        nft_contract_key,
        &Key::Account(*DEFAULT_ACCOUNT_ADDR),
        0u64,
    );

    assert!(token_page[0]);

    let incorrect_burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        ContractHash::new(nft_contract_hash),
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
fn should_return_expected_error_when_burning_not_owned_token() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(100u64)
            .with_ownership_mode(OwnershipMode::Transferable)
            .build();

    builder
        .exec(install_request_builder)
        .expect_success()
        .commit();

    let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = installing_account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let nft_contract_hash = nft_contract_key
        .into_hash()
        .expect("must convert to hash addr");

    let account_user_1 = support::create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1));

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => Key::Hash(nft_contract_hash),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_page = support::get_token_page_by_id(
        &builder,
        nft_contract_key,
        &Key::Account(*DEFAULT_ACCOUNT_ADDR),
        0u64,
    );

    assert!(token_page[0]);

    let incorrect_burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        ContractHash::new(nft_contract_hash),
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => 0u64
        },
    )
    .build();

    builder.exec(incorrect_burn_request).expect_failure();

    let actual_error = builder.get_error().expect("must get error");
    support::assert_expected_error(
        actual_error,
        6u16,
        "should disallow burning on mismatch of owner key",
    );
}

#[test]
fn should_allow_contract_to_burn_token() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

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
        .with_ownership_mode(OwnershipMode::Minter)
        .with_minting_mode(MintingMode::Installer)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_contract_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

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
        minting_contract_hash,
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
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_burn_mode(BurnMode::NonBurnable)
        .with_reporting_mode(OwnerReverseLookupMode::Complete)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();
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
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
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

// This test is no longer relevant as approve_for_all does not check burnt tokens anymore
#[test]
fn should_let_approve_all_with_burnt_tokens() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_reporting_mode(OwnerReverseLookupMode::Complete)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    for _ in 0..3 {
        let mint_session_call = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            MINT_SESSION_WASM,
            runtime_args! {
                ARG_NFT_CONTRACT_HASH => nft_contract_key,
                ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
                ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
                ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
            },
        )
        .build();

        builder.exec(mint_session_call).expect_success().commit();
    }

    let token_id = 0u64;
    let burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
        },
    )
    .build();

    builder.exec(burn_request).expect_success().commit();

    let operator: Key = Key::Account(AccountHash::new([7u8; 32]));

    let approve_all_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => operator
        },
    )
    .build();

    builder.exec(approve_all_request).expect_success().commit();
}

#[test]
fn should_burn_token_in_hash_identifier_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_reporting_mode(OwnerReverseLookupMode::Complete)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
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
        nft_contract_hash,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_HASH => token_hash,
        },
    )
    .build();

    builder.exec(burn_request).expect_success().commit();
}
