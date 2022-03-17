use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{runtime_args, system::mint, ContractHash, RuntimeArgs, U256};

use crate::utility::{
    constants::{
        ACCOUNT_USER_1, ACCOUNT_USER_2, ARG_PUBLIC_MINTING, ARG_TOKEN_META_DATA, BALANCES,
        CONTRACT_NAME, ENTRY_POINT_MINT, NFT_CONTRACT_WASM, NUMBER_OF_MINTED_TOKENS, OWNED_TOKENS,
        TEST_META_DATA, TOKEN_META_DATA, TOKEN_OWNERS,
    },
    installer_request_builder::InstallerRequestBuilder,
    support,
};

#[test]
fn should_disallow_minting_when_allow_minting_is_set_to_false() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(U256::from(2u64))
            .with_allowing_minting(Some(false));

    builder
        .exec(install_request_builder.build())
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
    builder.exec(mint_request).expect_failure();

    // Error should be MintingIsPaused=59
    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(actual_error, 59u16);
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

    let minter_account_hash = support::get_dictionary_value_from_key::<String>(
        &builder,
        nft_contract_key,
        TOKEN_OWNERS,
        &U256::zero().to_string(),
    );

    assert_eq!(
        DEFAULT_ACCOUNT_ADDR.clone().to_string(),
        minter_account_hash
    );

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

    let mint_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_META_DATA=> TEST_META_DATA.to_string(),
        },
    )
    .build();
    builder.exec(mint_request).expect_success().commit();

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

    let mint_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_META_DATA=> TEST_META_DATA.to_string(),
        },
    )
    .build();
    builder.exec(mint_request).expect_success().commit();

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
        public_minting_status,
        "public minting should be set to true"
    );

    let nft_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        account_1_public_key.to_account_hash(),
        ContractHash::new(nft_contract_hash),
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(nft_mint_request).expect_success().commit();

    let minter_public_key = support::get_dictionary_value_from_key::<String>(
        &builder,
        nft_contract_key,
        TOKEN_OWNERS,
        &U256::zero().to_string(),
    );

    assert_eq!(
        account_1_public_key.to_account_hash().to_string(),
        minter_public_key
    );
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

    let nft_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        account_1_public_key.to_account_hash(),
        ContractHash::new(nft_contract_hash),
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(nft_mint_request).expect_failure();
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
    let nft_contract_hash = nft_contract_key
        .into_hash()
        .expect("must convert to hash addr");

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

    let incorrect_nft_minting_request = ExecuteRequestBuilder::contract_call_by_hash(
        account_1_public_key.to_account_hash(),
        ContractHash::new(nft_contract_hash),
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
        },
    )
    .build();

    builder
        .exec(incorrect_nft_minting_request)
        .expect_success()
        .commit();
}
