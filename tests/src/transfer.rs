use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_ACCOUNT_PUBLIC_KEY, DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{runtime_args, system::mint, ContractHash, Key, RuntimeArgs, U256};

use crate::utility::{
    constants::{
        ACCOUNT_USER_1, ACCOUNT_USER_2, ARG_ENTRY_POINT_NAME, ARG_FROM_ACCOUNT_HASH,
        ARG_NFT_CONTRACT_HASH, ARG_OPERATOR, ARG_TOKEN_ID, ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER,
        ARG_TO_ACCOUNT_HASH, BALANCES, CONTRACT_NAME, ENTRY_POINT_APPROVE, ENTRY_POINT_MINT,
        ENTRY_POINT_SESSION_WASM, ENTRY_POINT_TRANSFER, NFT_CONTRACT_WASM, NFT_TEST_COLLECTION,
        NFT_TEST_SYMBOL, OPERATOR, OWNED_TOKENS, TEST_META_DATA, TOKEN_OWNERS,
    },
    installer_request_builder::{InstallerRequestBuilder, OwnershipMode},
    support::{self, get_dictionary_value_from_key, get_nft_contract_hash},
};

#[test]
fn should_dissallow_transfer_with_minter_or_assigned_ownership_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(U256::from(1u64))
        .with_ownership_mode(OwnershipMode::Minter)
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

    let token_owner = *DEFAULT_ACCOUNT_ADDR;

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        ENTRY_POINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => get_nft_contract_hash(&builder),
            ARG_ENTRY_POINT_NAME => ENTRY_POINT_MINT.to_string(),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = installing_account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let actual_owner_balance: U256 = support::get_dictionary_value_from_key(
        &builder,
        nft_contract_key,
        BALANCES,
        &token_owner.to_string(),
    );
    let expected_owner_balance = U256::one();
    assert_eq!(actual_owner_balance, expected_owner_balance);

    let (_, token_receiver) = support::create_dummy_key_pair(ACCOUNT_USER_1);
    let transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_TRANSFER,
        runtime_args! {
            ARG_TOKEN_ID => U256::zero(),// We need mint to return the token_id!!
            ARG_FROM_ACCOUNT_HASH => Key::Account(token_owner),
            ARG_TO_ACCOUNT_HASH =>  Key::Account( token_receiver.to_account_hash()),
        },
    )
    .build();
    builder.exec(transfer_request).expect_failure();

    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(
        actual_error,
        63u16,
        "should not allow transfer when ownership mode is Assigned or Minter",
    );
}

#[test]
fn should_transfer_token_from_sender_to_receiver() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(U256::from(1u64))
        .with_ownership_mode(OwnershipMode::TransferableUnchecked)
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

    let token_owner = *DEFAULT_ACCOUNT_ADDR;

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        ENTRY_POINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => get_nft_contract_hash(&builder),
            ARG_ENTRY_POINT_NAME => ENTRY_POINT_MINT.to_string(),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = installing_account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let actual_owner_balance: U256 = support::get_dictionary_value_from_key(
        &builder,
        nft_contract_key,
        BALANCES,
        &token_owner.to_string(),
    );
    let expected_owner_balance = U256::one();
    assert_eq!(actual_owner_balance, expected_owner_balance);

    let (_, token_receiver) = support::create_dummy_key_pair(ACCOUNT_USER_1);
    let transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_TRANSFER,
        runtime_args! {
            ARG_TOKEN_ID => U256::zero(),// We need mint to return the token_id!!
            ARG_FROM_ACCOUNT_HASH => Key::Account(token_owner),
            ARG_TO_ACCOUNT_HASH =>  Key::Account( token_receiver.to_account_hash()),
        },
    )
    .build();
    builder.exec(transfer_request).expect_success().commit();

    let actual_token_owner = support::get_dictionary_value_from_key::<Key>(
        &builder,
        nft_contract_key,
        TOKEN_OWNERS,
        &U256::zero().to_string(),
    )
    .into_account()
    .unwrap();

    assert_eq!(actual_token_owner, token_receiver.to_account_hash()); // Change  token_receiver to token_owner for red test

    let actual_owned_tokens: Vec<U256> = support::get_dictionary_value_from_key(
        &builder,
        nft_contract_key,
        OWNED_TOKENS,
        &token_receiver.to_account_hash().to_string(),
    );

    let expected_owned_tokens = vec![U256::zero()]; //Change zero() to one() for red test
    assert_eq!(actual_owned_tokens, expected_owned_tokens);

    let actual_sender_balance: U256 = support::get_dictionary_value_from_key(
        &builder,
        nft_contract_key,
        BALANCES,
        &token_owner.to_string(),
    );
    let expected_sender_balance = U256::zero();
    assert_eq!(actual_sender_balance, expected_sender_balance);

    let actual_receiver_balance: U256 = support::get_dictionary_value_from_key(
        &builder,
        nft_contract_key,
        BALANCES,
        &token_receiver.to_account_hash().to_string(),
    );
    let expected_receiver_balance = U256::one();
    assert_eq!(actual_receiver_balance, expected_receiver_balance);
}

#[test]
fn approve_token_for_transfer_should_add_entry_to_approved_dictionary() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(U256::one())
        .with_ownership_mode(OwnershipMode::TransferableUnchecked)
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

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        ENTRY_POINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => get_nft_contract_hash(&builder),
            ARG_ENTRY_POINT_NAME => ENTRY_POINT_MINT.to_string(),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let (_, approve_public_key) = support::create_dummy_key_pair(ACCOUNT_USER_1);
    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_ID => U256::zero(),
            ARG_OPERATOR => Key::Account(approve_public_key.to_account_hash())
        },
    )
    .build();
    builder.exec(approve_request).expect_success().commit();

    let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = installing_account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let actual_approved_key: Option<Key> = support::get_dictionary_value_from_key(
        &builder,
        nft_contract_key,
        OPERATOR,
        &U256::zero().to_string(),
    );

    assert_eq!(
        actual_approved_key,
        Some(Key::Account(approve_public_key.to_account_hash()))
    );
}

#[test]
fn should_dissallow_approving_when_ownership_mode_is_minter_or_assigned() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(U256::one())
        .with_ownership_mode(OwnershipMode::Assigned)
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

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        ENTRY_POINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => get_nft_contract_hash(&builder),
            ARG_ENTRY_POINT_NAME => ENTRY_POINT_MINT.to_string(),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let (_, approve_public_key) = support::create_dummy_key_pair(ACCOUNT_USER_1);
    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_ID => U256::zero(),
            ARG_OPERATOR => Key::Account(approve_public_key.to_account_hash())
        },
    )
    .build();
    builder.exec(approve_request).expect_failure();

    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(
        actual_error,
        63u16,
        "should not allow transfer when ownership mode is Assigned or Minter",
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
        .with_ownership_mode(OwnershipMode::TransferableUnchecked)
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
    let token_owner = DEFAULT_ACCOUNT_PUBLIC_KEY.clone().to_account_hash();
    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        ENTRY_POINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => get_nft_contract_hash(&builder),
            ARG_ENTRY_POINT_NAME => ENTRY_POINT_MINT.to_string(),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    // Create operator account and transfer funds
    let (_, operator) = support::create_dummy_key_pair(ACCOUNT_USER_1);
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
            ARG_OPERATOR => Key::Account (operator.to_account_hash())
        },
    )
    .build();
    builder.exec(approve_request).expect_success().commit();

    // Create to_account and transfer minted token using operator
    let (_, to_account_public_key) = support::create_dummy_key_pair(ACCOUNT_USER_2);
    let transfer_to_to_account = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            mint::ARG_AMOUNT => 100_000_000_000_000u64,
            mint::ARG_TARGET => to_account_public_key.clone(),
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
            ARG_FROM_ACCOUNT_HASH =>  Key::Account(token_owner),
            ARG_TO_ACCOUNT_HASH => Key::Account( to_account_public_key.to_account_hash()),
        },
    )
    .build();
    builder.exec(transfer_request).expect_success().commit();

    // Start querying...
    let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = installing_account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let actual_approved_account_hash: Option<Key> = get_dictionary_value_from_key(
        &builder,
        nft_contract_key,
        OPERATOR,
        &U256::zero().to_string(),
    );

    assert_eq!(
        actual_approved_account_hash,
        Some(Key::Account(operator.to_account_hash()))
    );
}
