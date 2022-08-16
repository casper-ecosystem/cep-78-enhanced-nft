use serde::{Deserialize, Serialize};

use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
use casper_types::{
    account::AccountHash, runtime_args, system::mint, ContractHash, Key, RuntimeArgs,
};

use crate::utility::{
    constants::{
        ACCOUNT_USER_1, ACCOUNT_USER_2, ARG_APPROVE_ALL, ARG_CONTRACT_WHITELIST,
        ARG_IS_HASH_IDENTIFIER_MODE, ARG_MINTING_MODE, ARG_NFT_CONTRACT_HASH, ARG_OPERATOR,
        ARG_TOKEN_HASH, ARG_TOKEN_ID, ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, BALANCES,
        BALANCE_OF_SESSION_WASM, CONTRACT_NAME, ENTRY_POINT_APPROVE, ENTRY_POINT_MINT,
        ENTRY_POINT_SET_APPROVE_FOR_ALL, ENTRY_POINT_SET_VARIABLES, MALFORMED_META_DATA,
        METADATA_CEP78, METADATA_CUSTOM_VALIDATED, METADATA_NFT721, METADATA_RAW,
        MINTING_CONTRACT_WASM, MINT_SESSION_WASM, NFT_CONTRACT_WASM, NUMBER_OF_MINTED_TOKENS,
        OPERATOR, OWNED_TOKENS, RECEIPT_NAME, TEST_COMPACT_META_DATA, TEST_PRETTY_721_META_DATA,
        TEST_PRETTY_CEP78_METADATA, TOKEN_ISSUERS, TOKEN_OWNERS,
    },
    installer_request_builder::{
        InstallerRequestBuilder, MetadataMutability, MintingMode, NFTHolderMode, NFTIdentifierMode,
        NFTMetadataKind, OwnershipMode, WhitelistMode, TEST_CUSTOM_METADATA,
        TEST_CUSTOM_METADATA_SCHEMA,
    },
    support::{
        self, assert_expected_error, call_entry_point_with_ret, create_dummy_key_pair,
        get_dictionary_value_from_key, get_minting_contract_hash, get_nft_contract_hash,
        query_stored_value,
    },
};

#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    name: String,
    symbol: String,
    token_uri: String,
}

fn setup_nft_contract(
    total_token_supply: Option<u64>,
    allowing_minting: bool,
) -> WasmTestBuilder<InMemoryGlobalState> {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let mut install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_collection_name("nft_collection".to_string())
            .with_allowing_minting(allowing_minting);

    if let Some(total_token_supply) = total_token_supply {
        install_request_builder =
            install_request_builder.with_total_token_supply(total_token_supply);
    }

    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();
    builder
}

#[test]
fn should_disallow_minting_when_allow_minting_is_set_to_false() {
    let mut builder = setup_nft_contract(Some(2u64), false);

    let mint_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_META_DATA=>TEST_PRETTY_721_META_DATA.to_string(),
        },
    )
    .build();
    builder.exec(mint_request).expect_failure();

    // Error should be MintingIsPaused=59
    let actual_error = builder.get_error().expect("must have error");
    assert_expected_error(
        actual_error,
        59u16,
        "should now allow minting when minting is disabled",
    );
}

#[test]
fn entry_points_with_ret_should_return_correct_value() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(2u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let account_hash = *DEFAULT_ACCOUNT_ADDR;

    let actual_balance: u64 = call_entry_point_with_ret(
        &mut builder,
        account_hash,
        nft_contract_key,
        runtime_args! {
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
        },
        BALANCE_OF_SESSION_WASM,
        "balance_of",
    );

    let expected_balance = 1u64;
    assert_eq!(
        actual_balance, expected_balance,
        "actual and expected balances should be equal"
    );

    let actual_owner: Key = call_entry_point_with_ret(
        &mut builder,
        account_hash,
        nft_contract_key,
        runtime_args! {
            ARG_IS_HASH_IDENTIFIER_MODE => false,
            ARG_TOKEN_ID => 0u64,
        },
        "owner_of_call.wasm",
        "owner_of",
    );

    let expected_owner = Key::Account(*DEFAULT_ACCOUNT_ADDR);
    assert_eq!(
        actual_owner, expected_owner,
        "actual and expected owner should be equal"
    );

    let (_, operator_public_key) = support::create_dummy_key_pair(ACCOUNT_USER_1);
    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_ID => 0u64,
            ARG_OPERATOR => Key::Account(operator_public_key.to_account_hash())
        },
    )
    .build();
    builder.exec(approve_request).expect_success().commit();

    let actual_operator: Option<Key> = call_entry_point_with_ret(
        &mut builder,
        account_hash,
        nft_contract_key,
        runtime_args! {
            ARG_IS_HASH_IDENTIFIER_MODE => false,
            ARG_TOKEN_ID => 0u64,
        },
        "get_approved_call.wasm",
        "get_approved",
    );

    let expected_operator = Key::Account(operator_public_key.to_account_hash());
    assert_eq!(
        actual_operator,
        Some(expected_operator),
        "actual and expected operator should be equal"
    );
}

#[test]
fn should_mint() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_nft_metadata_kind(NFTMetadataKind::CEP78)
            .with_total_token_supply(2u64);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,

            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA,
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();
}

#[test]
fn mint_should_return_dictionary_key_to_callers_owned_tokens() {
    const NFT_COLLECTION_NAME: &str = "enhanced_nft_collection";
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_COLLECTION_NAME.to_string())
        .with_total_token_supply(100u64)
        .with_allowing_minting(true)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash: Key = get_nft_contract_hash(&builder).into();
    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_hash,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);

    let receipt: String = query_stored_value(
        &mut builder,
        nft_contract_hash,
        vec![RECEIPT_NAME.to_string()],
    );

    let (_, owned_tokens_key) = account
        .named_keys()
        .get_key_value(&receipt)
        .expect("should have owned_tokens_key");

    match builder.query(None, *owned_tokens_key, &[]).unwrap() {
        casper_types::StoredValue::CLValue(val) => {
            let expected = val
                .into_t::<Vec<u64>>()
                .expect("should be Vec<u64> as Identifier defaults to indices");
            assert_eq!(vec![0u64], expected);
        }
        _ => panic!("wrong stored value type"),
    }

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_hash,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    match builder.query(None, *owned_tokens_key, &[]).unwrap() {
        casper_types::StoredValue::CLValue(val) => {
            let expected = val.into_t::<Vec<u64>>().expect("should still be Vec<U256>");
            assert_eq!(vec![0u64, 1u64], expected);
        }
        _ => panic!("also the wrong stored value type"),
    }
}

#[test]
fn mint_should_increment_number_of_minted_tokens_by_one_and_add_public_key_to_token_owners() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,

            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    //Let's start querying
    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    //mint should have incremented number_of_minted_tokens by one
    let query_result: u64 = support::query_stored_value(
        &mut builder,
        *nft_contract_key,
        vec![NUMBER_OF_MINTED_TOKENS.to_string()],
    );

    assert_eq!(
        query_result, 1u64,
        "number_of_minted_tokens initialized at installation should have incremented by one"
    );

    let actual_token_meta_data = support::get_dictionary_value_from_key::<String>(
        &builder,
        nft_contract_key,
        METADATA_NFT721,
        &0u64.to_string(),
    );

    assert_eq!(actual_token_meta_data, TEST_PRETTY_721_META_DATA);

    let minter_account_hash = support::get_dictionary_value_from_key::<Key>(
        &builder,
        nft_contract_key,
        TOKEN_OWNERS,
        &0u64.to_string(),
    )
    .into_account()
    .unwrap();

    assert_eq!(DEFAULT_ACCOUNT_ADDR.clone(), minter_account_hash);

    let actual_token_ids = support::get_dictionary_value_from_key::<Vec<u64>>(
        &builder,
        nft_contract_key,
        OWNED_TOKENS,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    assert_eq!(vec![0u64], actual_token_ids);

    // If total_token_supply is initialized to 1 the following test should fail.
    // If we set total_token_supply > 1 it should pass

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => *nft_contract_key,

            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();
}

#[test]
fn should_set_meta_data() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,

            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    //Let's start querying
    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let actual_token_meta_data = support::get_dictionary_value_from_key::<String>(
        &builder,
        nft_contract_key,
        METADATA_NFT721,
        &0u64.to_string(),
    );

    assert_eq!(actual_token_meta_data, TEST_PRETTY_721_META_DATA);
}

#[test]
fn should_set_issuer() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,

            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    //Let's start querying
    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let actual_token_issuer = support::get_dictionary_value_from_key::<Key>(
        &builder,
        nft_contract_key,
        TOKEN_ISSUERS,
        &0u64.to_string(),
    )
    .into_account()
    .unwrap();

    assert_eq!(actual_token_issuer, DEFAULT_ACCOUNT_ADDR.clone());
}

#[test]
fn should_track_token_balance_by_owner() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,

            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    //Let's start querying
    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let token_owner = DEFAULT_ACCOUNT_ADDR.clone().to_string();

    let actual_minter_balance = support::get_dictionary_value_from_key::<u64>(
        &builder,
        nft_contract_key,
        BALANCES,
        &token_owner,
    );
    let expected_minter_balance = 1u64;
    assert_eq!(actual_minter_balance, expected_minter_balance);
}

#[test]
fn should_allow_public_minting_with_flag_set_to_true() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_minting_mode(MintingMode::Public as u8)
        .build();
    builder.exec(install_request).expect_success().commit();

    let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = installing_account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let (_, account_1_public_key) = support::create_dummy_key_pair(ACCOUNT_USER_1);
    let account_1_account_hash = account_1_public_key.to_account_hash();

    let transfer_to_account_1 = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            mint::ARG_AMOUNT => 100_000_000_000_000u64,
            mint::ARG_TARGET => account_1_account_hash,
            mint::ARG_ID => Option::<u64>::None,
        },
    )
    .build();

    builder
        .exec(transfer_to_account_1)
        .expect_success()
        .commit();

    let public_minting_status = support::query_stored_value::<u8>(
        &mut builder,
        *nft_contract_key,
        vec![ARG_MINTING_MODE.to_string()],
    );

    assert_eq!(
        public_minting_status,
        MintingMode::Public as u8,
        "public minting should be set to true"
    );

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        account_1_public_key.to_account_hash(),
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,

            ARG_TOKEN_OWNER => Key::Account(account_1_public_key.to_account_hash()),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let minter_account_hash = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &0u64.to_string(),
    )
    .into_account()
    .unwrap();

    assert_eq!(account_1_account_hash, minter_account_hash);
}

#[test]
fn should_disallow_public_minting_with_flag_set_to_false() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_minting_mode(MintingMode::Installer as u8)
        .build();
    builder.exec(install_request).expect_success().commit();

    let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = installing_account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let (_account_1_secret_key, account_1_public_key) =
        support::create_dummy_key_pair(ACCOUNT_USER_1);

    let transfer_to_account_1 = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            mint::ARG_AMOUNT => 100_000_000_000_000u64,
            mint::ARG_TARGET => account_1_public_key.to_account_hash(),
            mint::ARG_ID => Option::<u64>::None,
        },
    )
    .build();

    builder
        .exec(transfer_to_account_1)
        .expect_success()
        .commit();

    let public_minting_status = support::query_stored_value::<u8>(
        &mut builder,
        *nft_contract_key,
        vec![ARG_MINTING_MODE.to_string()],
    );

    assert_eq!(
        public_minting_status,
        MintingMode::Installer as u8,
        "public minting should be set to false"
    );

    let mint_session_call = ExecuteRequestBuilder::standard(
        account_1_public_key.to_account_hash(),
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => *nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(account_1_public_key.to_account_hash()),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_session_call).expect_failure();
}

#[test]
fn should_allow_minting_for_different_public_key_with_minting_mode_set_to_public() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_minting_mode(MintingMode::Public as u8)
        .build();
    builder.exec(install_request).expect_success().commit();

    let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = installing_account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let (_account_1_secret_key, account_1_public_key) =
        support::create_dummy_key_pair(ACCOUNT_USER_1);
    let (_account_2_secret_key, _) = support::create_dummy_key_pair(ACCOUNT_USER_2);

    let transfer_to_account_1 = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            mint::ARG_AMOUNT => 100_000_000_000_000u64,
            mint::ARG_TARGET => account_1_public_key.to_account_hash(),
            mint::ARG_ID => Option::<u64>::None,
        },
    )
    .build();

    let transfer_to_account_2 = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            mint::ARG_AMOUNT => 100_000_000_000_000u64,
            mint::ARG_TARGET => account_1_public_key.to_account_hash(),
            mint::ARG_ID => Option::<u64>::None,
        },
    )
    .build();

    let transfer_requests = vec![transfer_to_account_1, transfer_to_account_2];
    for request in transfer_requests {
        builder.exec(request).expect_success().commit();
    }

    let public_minting_status = support::query_stored_value::<u8>(
        &mut builder,
        *nft_contract_key,
        vec![ARG_MINTING_MODE.to_string()],
    );

    assert_eq!(
        public_minting_status,
        MintingMode::Public as u8,
        "minting mode should be set to public"
    );

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        account_1_public_key.to_account_hash(),
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(account_1_public_key.to_account_hash()),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();
}

#[test]
fn should_set_approval_for_all() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
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
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    let (_, operator_public_key) = create_dummy_key_pair(ACCOUNT_USER_1);
    let set_approve_for_all_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_SET_APPROVE_FOR_ALL,
        runtime_args! {ARG_TOKEN_OWNER =>  Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => Key::Account(operator_public_key.to_account_hash())
        },
    )
    .build();

    builder
        .exec(set_approve_for_all_request)
        .expect_success()
        .commit();

    let actual_operator: Option<Key> = call_entry_point_with_ret(
        &mut builder,
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_key,
        runtime_args! {
            ARG_IS_HASH_IDENTIFIER_MODE => false,
            ARG_TOKEN_ID => 0u64,
        },
        "get_approved_call.wasm",
        "get_approved",
    );

    let expected_operator = Key::Account(operator_public_key.to_account_hash());
    assert_eq!(
        actual_operator,
        Some(expected_operator),
        "actual and expected operator should be equal"
    );

    let actual_operator: Option<Key> = call_entry_point_with_ret(
        &mut builder,
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_key,
        runtime_args! {
            ARG_IS_HASH_IDENTIFIER_MODE => false,
            ARG_TOKEN_ID => 0u64,
        },
        "get_approved_call.wasm",
        "get_approved",
    );

    let expected_operator = Key::Account(operator_public_key.to_account_hash());
    assert_eq!(
        actual_operator,
        Some(expected_operator),
        "actual and expected operator should be equal"
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

    let contract_whitelist = vec![minting_contract_hash];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_minting_mode(MintingMode::Installer as u8)
        .with_contract_whitelist(contract_whitelist.clone())
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let actual_contract_whitelist: Vec<ContractHash> = query_stored_value(
        &mut builder,
        nft_contract_key,
        vec![ARG_CONTRACT_WHITELIST.to_string()],
    );

    assert_eq!(actual_contract_whitelist, contract_whitelist);

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
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

    let token_id = 0u64.to_string();

    let actual_token_owner: Key =
        get_dictionary_value_from_key(&builder, &nft_contract_key, TOKEN_OWNERS, &token_id);

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
        ContractHash::from([1u8; 32]),
        ContractHash::from([2u8; 32]),
        ContractHash::from([3u8; 32]),
    ];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_minting_mode(MintingMode::Installer as u8)
        .with_contract_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
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

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Unlocked)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_minting_mode(MintingMode::Installer as u8)
        .with_contract_whitelist(vec![])
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key = nft_contract_hash.into();

    let current_contract_whitelist: Vec<ContractHash> = query_stored_value(
        &mut builder,
        nft_contract_key,
        vec![ARG_CONTRACT_WHITELIST.to_string()],
    );

    assert!(current_contract_whitelist.is_empty());
    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
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

    let updated_contract_whitelist: Vec<ContractHash> = query_stored_value(
        &mut builder,
        nft_contract_key,
        vec![ARG_CONTRACT_WHITELIST.to_string()],
    );

    assert_eq!(vec![minting_contract_hash], updated_contract_whitelist);

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
fn should_not_mint_with_invalid_nft721_metadata() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,

            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => MALFORMED_META_DATA,
        },
    )
    .build();

    builder.exec(mint_session_call).expect_failure();

    let error = builder.get_error().expect("mint request must have failed");
    assert_expected_error(
        error,
        89,
        "FailedToParse721Metadata error (89) must have been raised due to mangled metadata",
    )
}

#[test]
fn should_mint_with_compactified_metadata() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64)
            .build();

    builder
        .exec(install_request_builder)
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,

            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_COMPACT_META_DATA,
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let actual_metadata = get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_NFT721,
        &0u64.to_string(),
    );

    assert_eq!(TEST_PRETTY_721_META_DATA, actual_metadata)
}

#[test]
fn should_mint_with_valid_cep99_metadata() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(2u64)
        .with_nft_metadata_kind(NFTMetadataKind::CEP78)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA,
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let actual_metadata = get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_CEP78,
        &0u64.to_string(),
    );

    assert_eq!(TEST_PRETTY_CEP78_METADATA, actual_metadata)
}

#[test]
fn should_mint_with_custom_metadata_validation() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let custom_json_schema =
        serde_json::to_string(&*TEST_CUSTOM_METADATA_SCHEMA).expect("must convert to json schema");

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(2u64)
        .with_nft_metadata_kind(NFTMetadataKind::CustomValidated)
        .with_json_schema(custom_json_schema)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let custom_metadata =
        serde_json::to_string(&*TEST_CUSTOM_METADATA).expect("must convert to json metadata");

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,

            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => custom_metadata ,
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let actual_metadata = get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_CUSTOM_VALIDATED,
        &0u64.to_string(),
    );

    let pretty_custom_metadata = serde_json::to_string_pretty(&*TEST_CUSTOM_METADATA)
        .expect("must convert to json metadata");

    assert_eq!(pretty_custom_metadata, actual_metadata)
}

#[test]
fn should_mint_with_raw_metadata() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(2u64)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,

            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => "raw_string".to_string() ,
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let actual_metadata = get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_RAW,
        &0u64.to_string(),
    );

    assert_eq!("raw_string".to_string(), actual_metadata)
}

#[test]
fn should_mint_with_hash_identifier_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_total_token_supply(10u64)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA ,
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_id_hash: String =
        base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_721_META_DATA));

    let actual_token_ids = get_dictionary_value_from_key::<Vec<String>>(
        &builder,
        &nft_contract_key,
        OWNED_TOKENS,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    assert_eq!(vec![token_id_hash], actual_token_ids);
}

#[test]
fn should_fail_to_mint_when_immediate_caller_is_account_in_contract_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(2u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Unlocked)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_COMPACT_META_DATA,
        },
    )
    .build();

    builder.exec(mint_session_call).expect_failure();

    let error = builder.get_error().expect("must have error");

    assert_expected_error(error, 76, "InvalidHolderMode(76) must have been raised");
}

#[test]
fn should_approve_in_hash_identifier_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_ownership_mode(OwnershipMode::Transferable)
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
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_hash: String =
        base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_721_META_DATA));

    let operator = Key::Account(AccountHash::new([7u8; 32]));

    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_HASH => token_hash.clone(),
            ARG_OPERATOR => operator
        },
    )
    .build();

    builder.exec(approve_request).expect_success().commit();

    let maybe_approved_operator = support::get_dictionary_value_from_key::<Option<Key>>(
        &builder,
        &nft_contract_key,
        OPERATOR,
        &token_hash,
    );

    assert_eq!(maybe_approved_operator, Some(operator))
}
