use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_ACCOUNT_PUBLIC_KEY, DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{runtime_args, system::mint, ContractHash, RuntimeArgs, U256};

use crate::utility::{
    constants::{
        ACCOUNT_USER_1, ARG_TOKEN_ID, ARG_TOKEN_META_DATA, BURNT_TOKENS, CONTRACT_NAME,
        ENTRY_POINT_BURN, ENTRY_POINT_MINT, NFT_CONTRACT_WASM, OWNED_TOKENS, TEST_META_DATA,
    },
    installer_request_builder::InstallerRequestBuilder,
    support,
};

#[test]
fn should_burn_minted_token() {
    const TOKEN_ID: U256 = U256::zero();

    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(U256::from(100u64))
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

    let mint_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_request).expect_success().commit();

    let actual_owned_tokens = support::get_dictionary_value_from_key::<Vec<U256>>(
        &builder,
        nft_contract_key,
        OWNED_TOKENS,
        &DEFAULT_ACCOUNT_PUBLIC_KEY
            .clone()
            .to_account_hash()
            .to_string(),
    );

    let expected_owned_tokens = vec![U256::zero()];
    assert_eq!(expected_owned_tokens, actual_owned_tokens);

    let burn_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => U256::zero(),
        },
    )
    .build();

    builder.exec(burn_request).expect_success().commit();

    let _ = support::get_dictionary_value_from_key::<()>(
        &builder,
        nft_contract_key,
        BURNT_TOKENS,
        &TOKEN_ID.to_string(),
    );
}

#[test]
fn should_not_burn_previously_burnt_token() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(U256::from(100u64))
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

    let mint_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_request).expect_success().commit();

    let actual_owned_tokens = support::get_dictionary_value_from_key::<Vec<U256>>(
        &builder,
        nft_contract_key,
        OWNED_TOKENS,
        &DEFAULT_ACCOUNT_PUBLIC_KEY
            .clone()
            .to_account_hash()
            .to_string(),
    );

    let expected_owned_tokens = vec![U256::zero()];
    assert_eq!(
        expected_owned_tokens, actual_owned_tokens,
        "1----------------1"
    );

    let burn_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => U256::zero(),
        },
    )
    .build();

    builder.exec(burn_request).expect_success().commit();

    let re_burn_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => U256::zero(),
        },
    )
    .build();

    builder.exec(re_burn_request).expect_failure();

    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(actual_error, 42u16);
}

#[test]
fn should_not_burn_un_minted_token() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(U256::from(100u64))
            .build();

    builder
        .exec(install_request_builder)
        .expect_success()
        .commit();

    let burn_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => U256::zero(),
        },
    )
    .build();

    builder.exec(burn_request).expect_failure();

    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(actual_error, 28u16);
}

#[test]
fn should_disallow_burning_of_others_users_token() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(U256::from(100u64))
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

    let (_, account_user_1) = support::create_dummy_key_pair(ACCOUNT_USER_1);

    let transfer_to_account_1 = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            mint::ARG_AMOUNT => 100_000_000_000_000u64,
            mint::ARG_TARGET => account_user_1.to_account_hash(),
            mint::ARG_ID => Option::<u64>::None,
        },
    )
    .build();

    builder
        .exec(transfer_to_account_1)
        .expect_success()
        .commit();

    let mint_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_request).expect_success().commit();

    let actual_owned_tokens = support::get_dictionary_value_from_key::<Vec<U256>>(
        &builder,
        nft_contract_key,
        OWNED_TOKENS,
        &DEFAULT_ACCOUNT_PUBLIC_KEY
            .clone()
            .to_account_hash()
            .to_string(),
    );

    let expected_owned_tokens = vec![U256::zero()];
    assert_eq!(expected_owned_tokens, actual_owned_tokens);

    let incorrect_burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1.to_account_hash(),
        ContractHash::new(nft_contract_hash),
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => U256::zero(),
        },
    )
    .build();

    builder.exec(incorrect_burn_request).expect_failure();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(error, 6u16);
}

#[test]
fn should_prevent_burning_on_owner_key_mismatch() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(U256::from(100u64))
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

    let (_, account_user_1) = support::create_dummy_key_pair(ACCOUNT_USER_1);

    let transfer_to_account_1 = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            mint::ARG_AMOUNT => 100_000_000_000_000u64,
            mint::ARG_TARGET => account_user_1.to_account_hash(),
            mint::ARG_ID => Option::<u64>::None,
        },
    )
    .build();

    builder
        .exec(transfer_to_account_1)
        .expect_success()
        .commit();

    let mint_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_request).expect_success().commit();

    let actual_owned_tokens = support::get_dictionary_value_from_key::<Vec<U256>>(
        &builder,
        nft_contract_key,
        OWNED_TOKENS,
        &DEFAULT_ACCOUNT_PUBLIC_KEY
            .clone()
            .to_account_hash()
            .to_string(),
    );

    let expected_owned_tokens = vec![U256::zero()];
    assert_eq!(expected_owned_tokens, actual_owned_tokens);

    let incorrect_burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1.to_account_hash(),
        ContractHash::new(nft_contract_hash),
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_TOKEN_ID => U256::zero()
        },
    )
    .build();

    builder.exec(incorrect_burn_request).expect_failure();

    let actual_error = builder.get_error().expect("must get error");
    support::assert_expected_error(actual_error, 6u16);
}
