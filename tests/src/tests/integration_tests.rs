use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_ACCOUNT_PUBLIC_KEY, DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{runtime_args, system::mint, ContractHash, Key, RuntimeArgs, U256};

use super::{
    constants::{
        ACCOUNT_USER_1, ACCOUNT_USER_2, APPROVED_FOR_TRANSFER, ARG_ALLOW_MINTING,
        ARG_APPROVE_TRANSFER_FOR_ACCOUNT_HASH, ARG_FROM_ACCOUNT_HASH, ARG_TOKEN_ID,
        ARG_TOKEN_META_DATA, ARG_TO_ACCOUNT_HASH, CONTRACT_NAME, ENTRY_POINT_APPROVE,
        ENTRY_POINT_MINT, ENTRY_POINT_SET_VARIABLES, ENTRY_POINT_TRANSFER, NFT_CONTRACT_WASM,
        NFT_TEST_COLLECTION, NFT_TEST_SYMBOL, OWNED_TOKENS, TEST_META_DATA, TOKEN_OWNERS,
    },
    installer_request_builder::InstallerRequestBuilder,
    utils,
};

#[test]
fn only_installer_should_be_able_to_toggle_allow_minting() {
    let (_, other_user_public_key) = utils::create_dummy_key_pair(ACCOUNT_USER_1); //<-- Choose MINTER2 for failing red test
    let other_user_account = other_user_public_key.to_account_hash();
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(U256::one())
        .with_allowing_minting(Some(false))
        .build();

    builder.exec(install_request).expect_success().commit();

    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_hash = account
        .named_keys()
        .get(CONTRACT_NAME)
        .cloned()
        .and_then(Key::into_hash)
        .map(ContractHash::new)
        .expect("failed to find nft contract");

    //Account other than installer account should not be able to change allow_minting
    // Red test
    let other_user_set_variables_request = ExecuteRequestBuilder::contract_call_by_hash(
        other_user_account,
        nft_contract_hash,
        ENTRY_POINT_SET_VARIABLES,
        runtime_args! { ARG_ALLOW_MINTING => Some(true) },
    )
    .build();

    builder
        .exec(other_user_set_variables_request)
        .expect_failure()
        .commit();

    // Don't just use expect_failure. Match and actual error!
    // let error = builder.get_error().expect("must have error");
    // assert_expected_error(error, NFTCoreError::InvalidAccount as u16);

    //Installer account should be able to change allow_minting
    // Green test
    let installer_set_variables_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_VARIABLES,
        runtime_args! { ARG_ALLOW_MINTING => Some(true) },
    )
    .build();

    builder
        .exec(installer_set_variables_request)
        .expect_success()
        .commit();
}

#[test]
fn should_transfer_token_from_sender_to_receiver() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(U256::from(1u64))
        .build();

    builder.exec(install_request).expect_success().commit();

    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_hash = account
        .named_keys()
        .get(CONTRACT_NAME)
        .cloned()
        .and_then(Key::into_hash)
        .map(ContractHash::new)
        .expect("failed to find nft contract");

    let token_owner = DEFAULT_ACCOUNT_PUBLIC_KEY
        .clone()
        .to_account_hash()
        .to_string();

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_request).expect_success().commit();

    let (_, token_receiver) = utils::create_dummy_key_pair(ACCOUNT_USER_1);
    let transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_TRANSFER,
        runtime_args! {
            ARG_TOKEN_ID => U256::zero(),// We need mint to return the token_id!!
            ARG_FROM_ACCOUNT_HASH => token_owner,
            ARG_TO_ACCOUNT_HASH => token_receiver.to_account_hash().to_string(),
        },
    )
    .build();
    builder.exec(transfer_request).expect_success().commit();

    //let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = installing_account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let actual_token_owner: String = utils::get_dictionary_value_from_key(
        &builder,
        nft_contract_key,
        TOKEN_OWNERS,
        &U256::zero().to_string(),
    );

    assert_eq!(
        actual_token_owner,
        token_receiver.to_account_hash().to_string()
    ); // Change  token_receiver to token_owner for red test

    let actual_owned_tokens: Vec<U256> = utils::get_dictionary_value_from_key(
        &builder,
        nft_contract_key,
        OWNED_TOKENS,
        &token_receiver.to_account_hash().to_string(),
    );

    let expected_owned_tokens = vec![U256::zero()]; //Change zero() to one() for red test
    assert_eq!(actual_owned_tokens, expected_owned_tokens);
}

#[test]
fn approve_token_for_transfer_should_add_entry_to_approved_dictionary() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(U256::one())
        .build();

    builder.exec(install_request).expect_success().commit();

    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_hash = account
        .named_keys()
        .get(CONTRACT_NAME)
        .cloned()
        .and_then(Key::into_hash)
        .map(ContractHash::new)
        .expect("failed to find nft contract");

    // let token_owner = DEFAULT_ACCOUNT_PUBLIC_KEY.clone();
    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_request).expect_success().commit();

    let (_, approve_public_key) = utils::create_dummy_key_pair(ACCOUNT_USER_1);
    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash,
            ENTRY_POINT_APPROVE,
            runtime_args! {
                ARG_TOKEN_ID => U256::zero(),
                ARG_APPROVE_TRANSFER_FOR_ACCOUNT_HASH => approve_public_key.to_account_hash().to_string()
            },
        )
        .build();
    builder.exec(approve_request).expect_success().commit();

    let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = installing_account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let actual_approved_account_hash: Option<String> = utils::get_dictionary_value_from_key(
        &builder,
        nft_contract_key,
        APPROVED_FOR_TRANSFER,
        &U256::zero().to_string(),
    );

    assert_eq!(
        actual_approved_account_hash,
        Some(approve_public_key.to_account_hash().to_string())
    );
}

#[test]
fn should_be_able_to_transfer_token_using_approved_operator() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(U256::one())
        .build();

    builder.exec(install_request).expect_success().commit();

    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_hash = account
        .named_keys()
        .get(CONTRACT_NAME)
        .cloned()
        .and_then(Key::into_hash)
        .map(ContractHash::new)
        .expect("failed to find nft contract");

    // mint token for DEFAULT_ACCOUNT_ADDR
    let token_owner = DEFAULT_ACCOUNT_PUBLIC_KEY.clone();
    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
        },
    )
    .build();
    builder.exec(mint_request).expect_success().commit();

    // Create operator account and transfer funds
    let (_, operator) = utils::create_dummy_key_pair(ACCOUNT_USER_1);
    let transfer_to_operator = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            mint::ARG_AMOUNT => 100_000_000_000_000u64,
            mint::ARG_TARGET => operator.to_account_hash(),
            mint::ARG_ID => Option::<u64>::None,
        },
    )
    .build();
    builder.exec(transfer_to_operator).expect_success().commit();

    // Approve operator
    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_ID => U256::zero(),
            ARG_APPROVE_TRANSFER_FOR_ACCOUNT_HASH => operator.to_account_hash().to_string()
        },
    )
    .build();
    builder.exec(approve_request).expect_success().commit();

    // Create to_account and transfer minted token using operator
    let (_, to_account_hash) = utils::create_dummy_key_pair(ACCOUNT_USER_2);
    let transfer_to_to_account = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            mint::ARG_AMOUNT => 100_000_000_000_000u64,
            mint::ARG_TARGET => to_account_hash.to_account_hash(),
            mint::ARG_ID => Option::<u64>::None,
        },
    )
    .build();
    builder
        .exec(transfer_to_to_account)
        .expect_success()
        .commit();

    let transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
        operator.to_account_hash(),
        nft_contract_hash,
        ENTRY_POINT_TRANSFER,
        runtime_args! {
            ARG_TOKEN_ID => U256::zero(),
            ARG_FROM_ACCOUNT_HASH => token_owner.to_account_hash().to_string(),
            ARG_TO_ACCOUNT_HASH => to_account_hash.to_account_hash().to_string(),
        },
    )
    .build();
    builder.exec(transfer_request).expect_success().commit();

    // // Start querying...
    // let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    // let nft_contract_key = installing_account
    //     .named_keys()
    //     .get(CONTRACT_NAME)
    //     .expect("must have key in named keys");

    // let actual_approved_account_hash: Option<String> = get_dictionary_value_from_key(
    //     &builder,
    //     nft_contract_key,
    //     APPROVED_FOR_TRANSFER,
    //     &U256::zero().to_string(),
    // );

    // assert_eq!(
    //     actual_approved_account_hash,
    //     Some(operator.to_account_hash().to_string())
    // );
}
