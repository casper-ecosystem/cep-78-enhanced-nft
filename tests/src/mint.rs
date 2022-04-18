use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};

use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
use casper_types::{runtime_args, system::mint, Key, RuntimeArgs, U256};

use crate::utility::constants::{
    ARG_APPROVE_ALL, ARG_KEY_NAME, ARG_OPERATOR, ARG_TOKEN_ID, ARG_TOKEN_URI,
    BALANCE_OF_SESSION_WASM, ENTRY_POINT_APPROVE, ENTRY_POINT_SET_APPROVE_FOR_ALL,
    MINT_SESSION_WASM, OWNED_TOKENS_DICTIONARY_KEY, TEST_URI,
};
use crate::utility::installer_request_builder::OwnershipMode;
use crate::utility::support::{
    call_entry_point_with_ret, create_dummy_key_pair, get_nft_contract_hash,
};
use crate::utility::{
    constants::{
        ACCOUNT_USER_1, ACCOUNT_USER_2, ARG_NFT_CONTRACT_HASH, ARG_PUBLIC_MINTING,
        ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, BALANCES, CONTRACT_NAME, ENTRY_POINT_MINT,
        NFT_CONTRACT_WASM, NUMBER_OF_MINTED_TOKENS, OWNED_TOKENS, TEST_META_DATA, TOKEN_ISSUERS,
        TOKEN_META_DATA, TOKEN_OWNERS,
    },
    installer_request_builder::InstallerRequestBuilder,
    support,
};

fn get_builder(
    total_token_supply: Option<U256>,
    allowing_minting: Option<bool>,
) -> WasmTestBuilder<InMemoryGlobalState> {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let mut install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
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
    let mut builder = get_builder(Some(U256::from(2u64)), Some(false));

    let mint_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
        },
    )
    .build();
    builder.exec(mint_request).expect_failure();

    // Error should be MintingIsPaused=59
    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(
        actual_error,
        59u16,
        "should now allow minting when minting is disabled",
    );
}

#[test]
fn entry_points_with_ret_should_return_correct_value() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(U256::from(2u64))
            .with_ownership_mode(OwnershipMode::TransferableUnchecked);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => get_nft_contract_hash(&builder),
            ARG_KEY_NAME => Some(OWNED_TOKENS_DICTIONARY_KEY.to_string()),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
            ARG_TOKEN_URI => TEST_URI.to_string(),
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let account_hash = *DEFAULT_ACCOUNT_ADDR;

    let actual_balance: U256 = call_entry_point_with_ret(
        &mut builder,
        account_hash,
        nft_contract_hash,
        runtime_args! {
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
        },
        BALANCE_OF_SESSION_WASM,
        "balance_of",
    );

    let expected_balance = U256::one();
    assert_eq!(
        actual_balance, expected_balance,
        "actual and expected balances should be equal"
    );

    let actual_owner: Key = call_entry_point_with_ret(
        &mut builder,
        account_hash,
        nft_contract_hash,
        runtime_args! {
            ARG_TOKEN_ID => U256::zero(),
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
            ARG_TOKEN_ID => U256::zero(),
            ARG_OPERATOR => Key::Account(operator_public_key.to_account_hash())
        },
    )
    .build();
    builder.exec(approve_request).expect_success().commit();

    let actual_operator: Option<Key> = call_entry_point_with_ret(
        &mut builder,
        account_hash,
        nft_contract_hash,
        runtime_args! {
            ARG_TOKEN_ID => U256::zero(),
        },
        "get_approved_call.wasm",
        "get_approved",
    );

    let expected_operator = Key::Account(operator_public_key.to_account_hash());
    assert_eq!(
        actual_operator,
        Some(expected_operator),
        "actual and expected owner should be equal"
    );
}

#[test]
fn should_call_mint_via_session_code() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(U256::from(2u64));
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_hash,
            ARG_KEY_NAME => Some(OWNED_TOKENS_DICTIONARY_KEY.to_string()),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
            ARG_TOKEN_URI => TEST_URI.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();
}

#[test]
fn mint_should_return_dictionary_key_to_callers_owned_tokens() {
    let mut builder = get_builder(Some(U256::from(2u64)), None);

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_hash,
            ARG_KEY_NAME => Some(OWNED_TOKENS_DICTIONARY_KEY.to_string()),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
            ARG_TOKEN_URI => TEST_URI.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);

    let (_, owned_tokens_key) = account
        .named_keys()
        .get_key_value(OWNED_TOKENS_DICTIONARY_KEY)
        .expect("should have owned_tokens_key");

    match builder.query(None, *owned_tokens_key, &[]).unwrap() {
        casper_types::StoredValue::CLValue(val) => {
            let expected = val.into_t::<Vec<U256>>().expect("should be Vec<U256>");
            println!("{:?}", expected);
        }
        _ => panic!("wrong stored value type"),
    }

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_hash,
            ARG_KEY_NAME => Some(OWNED_TOKENS_DICTIONARY_KEY.to_string()),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
            ARG_TOKEN_URI => TEST_URI.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    match builder.query(None, *owned_tokens_key, &[]).unwrap() {
        casper_types::StoredValue::CLValue(val) => {
            let expected = val
                .into_t::<Vec<U256>>()
                .expect("should still be Vec<U256>");
            println!("{:?}", expected);
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
            .with_total_token_supply(U256::from(2u64));
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => get_nft_contract_hash(&builder),
            ARG_KEY_NAME => Some(OWNED_TOKENS_DICTIONARY_KEY.to_string()),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
            ARG_TOKEN_URI => TEST_URI.to_string()
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
    let query_result: U256 = support::query_stored_value(
        &mut builder,
        *nft_contract_key,
        vec![NUMBER_OF_MINTED_TOKENS.to_string()],
    );

    assert_eq!(
        query_result,
        U256::one(),
        "number_of_minted_tokens initialized at installation should have incremented by one"
    );

    let actual_token_meta_data = support::get_dictionary_value_from_key::<String>(
        &builder,
        nft_contract_key,
        TOKEN_META_DATA,
        &U256::zero().to_string(),
    );

    assert_eq!(actual_token_meta_data, TEST_META_DATA);

    let minter_account_hash = support::get_dictionary_value_from_key::<Key>(
        &builder,
        nft_contract_key,
        TOKEN_OWNERS,
        &U256::zero().to_string(),
    )
    .into_account()
    .unwrap();

    assert_eq!(DEFAULT_ACCOUNT_ADDR.clone(), minter_account_hash);

    let actual_token_ids = support::get_dictionary_value_from_key::<Vec<U256>>(
        &builder,
        nft_contract_key,
        OWNED_TOKENS,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    let expected_token_ids = vec![U256::zero()];
    assert_eq!(expected_token_ids, actual_token_ids);

    // If total_token_supply is initialized to 1 the following test should fail.
    // If we set total_token_supply > 1 it should pass

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => get_nft_contract_hash(&builder),
            ARG_KEY_NAME => Some(OWNED_TOKENS_DICTIONARY_KEY.to_string()),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
             ARG_TOKEN_URI => TEST_URI.to_string()
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();
}

#[test]
fn mint_should_correctly_set_meta_data() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(U256::from(2u32));
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => get_nft_contract_hash(&builder),
            ARG_KEY_NAME => Some(OWNED_TOKENS_DICTIONARY_KEY.to_string()),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
            ARG_TOKEN_URI => TEST_URI.to_string()
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
        TOKEN_META_DATA,
        &U256::zero().to_string(),
    );

    assert_eq!(actual_token_meta_data, TEST_META_DATA);
}

#[test]
fn mint_should_correctly_set_issuer() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(U256::from(2u32));
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => get_nft_contract_hash(&builder),
            ARG_KEY_NAME => Some(OWNED_TOKENS_DICTIONARY_KEY.to_string()),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
            ARG_TOKEN_URI => TEST_URI.to_string()
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
        &U256::zero().to_string(),
    )
    .into_account()
    .unwrap();

    assert_eq!(actual_token_issuer, DEFAULT_ACCOUNT_ADDR.clone());
}

#[test]
fn mint_should_correctly_update_balances() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(U256::from(2u32));
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => get_nft_contract_hash(&builder),
            ARG_KEY_NAME => Some(OWNED_TOKENS_DICTIONARY_KEY.to_string()),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
            ARG_TOKEN_URI => TEST_URI.to_string()
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

    let actual_minter_balance = support::get_dictionary_value_from_key::<U256>(
        &builder,
        nft_contract_key,
        BALANCES,
        &token_owner,
    );
    let expected_minter_balance = U256::one();
    assert_eq!(actual_minter_balance, expected_minter_balance);
}

#[test]
fn should_allow_public_minting_with_flag_set_to_true() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(U256::from(100u64))
        .with_public_minting(Some(true))
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

    let public_minting_status = support::query_stored_value::<bool>(
        &mut builder,
        *nft_contract_key,
        vec![ARG_PUBLIC_MINTING.to_string()],
    );

    assert!(
        public_minting_status,
        "public minting should be set to true"
    );

    let mint_session_call = ExecuteRequestBuilder::standard(
        account_1_public_key.to_account_hash(),
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => get_nft_contract_hash(&builder),
            ARG_KEY_NAME => Some(OWNED_TOKENS_DICTIONARY_KEY.to_string()),
            ARG_TOKEN_OWNER => Key::Account(account_1_public_key.to_account_hash()),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
            ARG_TOKEN_URI => TEST_URI.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let minter_account_hash = support::get_dictionary_value_from_key::<Key>(
        &builder,
        nft_contract_key,
        TOKEN_OWNERS,
        &U256::zero().to_string(),
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
        .with_total_token_supply(U256::from(100u64))
        .with_public_minting(Some(false))
        .build();
    builder.exec(install_request).expect_success().commit();

    let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = installing_account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");
    let nft_contract_hash = nft_contract_key
        .into_hash()
        .expect("must convert to hash addr");

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

    let public_minting_status = support::query_stored_value::<bool>(
        &mut builder,
        *nft_contract_key,
        vec![ARG_PUBLIC_MINTING.to_string()],
    );

    assert!(
        !public_minting_status,
        "public minting should be set to false"
    );

    let mint_session_call = ExecuteRequestBuilder::standard(
        account_1_public_key.to_account_hash(),
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => get_nft_contract_hash(&builder),
            ARG_TOKEN_OWNER => Key::Account(account_1_public_key.to_account_hash()),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
            ARG_TOKEN_URI => TEST_URI.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_failure();
}

#[test]
fn should_allow_minting_for_different_public_key_with_public_minting_set_to_true() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(U256::from(100u64))
        .with_public_minting(Some(true))
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

    let public_minting_status = support::query_stored_value::<bool>(
        &mut builder,
        *nft_contract_key,
        vec![ARG_PUBLIC_MINTING.to_string()],
    );

    assert!(
        public_minting_status,
        "public minting should be set to true"
    );

    let mint_session_call = ExecuteRequestBuilder::standard(
        account_1_public_key.to_account_hash(),
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => get_nft_contract_hash(&builder),
            ARG_KEY_NAME => Some(OWNED_TOKENS_DICTIONARY_KEY.to_string()),
            ARG_TOKEN_OWNER => Key::Account(account_1_public_key.to_account_hash()),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
            ARG_TOKEN_URI => TEST_URI.to_string()
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
        .with_total_token_supply(U256::from(100u64))
        .with_ownership_mode(OwnershipMode::TransferableChecked)
        .build();
    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_hash,
            ARG_KEY_NAME => Some(OWNED_TOKENS_DICTIONARY_KEY.to_string()),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
            ARG_TOKEN_URI => TEST_URI.to_string()
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_hash,
            ARG_KEY_NAME => Some(OWNED_TOKENS_DICTIONARY_KEY.to_string()),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
            ARG_TOKEN_URI => TEST_URI.to_string()
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
}
