use cep78::{
    constants::{
        APPROVED, ARG_APPROVE_ALL, ARG_COLLECTION_NAME, ARG_MINTING_MODE, ARG_OPERATOR,
        ARG_SOURCE_KEY, ARG_SPENDER, ARG_TARGET_KEY, ARG_TOKEN_HASH, ARG_TOKEN_ID,
        ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, ENTRY_POINT_APPROVE, ENTRY_POINT_MINT,
        ENTRY_POINT_REGISTER_OWNER, ENTRY_POINT_SET_APPROVALL_FOR_ALL, METADATA_CEP78,
        METADATA_CUSTOM_VALIDATED, METADATA_NFT721, METADATA_RAW, NUMBER_OF_MINTED_TOKENS,
        PAGE_TABLE, RECEIPT_NAME, TOKEN_COUNT, TOKEN_ISSUERS, TOKEN_OWNERS,
    },
    events::events_ces::{ApprovalForAll, Mint, RevokedForAll},
    modalities::TokenIdentifier,
};
use serde::{Deserialize, Serialize};

use casper_engine_test_support::{
    ExecuteRequestBuilder, LmdbWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
};
use casper_types::{account::AccountHash, runtime_args, CLValue, Key};

use crate::utility::{
    constants::{
        ACCOUNT_1_ADDR, ACCOUNT_1_KEY, ACCOUNT_2_ADDR, ACCOUNT_3_ADDR, ACCOUNT_3_KEY,
        ARG_IS_HASH_IDENTIFIER_MODE, ARG_KEY_NAME, ARG_NFT_CONTRACT_HASH, BALANCE_OF_SESSION_WASM,
        CONTRACT_NAME, DEFAULT_ACCOUNT_KEY, GET_APPROVED_WASM, IS_APPROVED_FOR_ALL_WASM,
        MALFORMED_META_DATA, MINT_SESSION_WASM, NFT_CONTRACT_WASM, NFT_TEST_COLLECTION,
        OWNER_OF_SESSION_WASM, PAGE_SIZE, TEST_COMPACT_META_DATA, TEST_PRETTY_721_META_DATA,
        TEST_PRETTY_CEP78_METADATA, TEST_PRETTY_UPDATED_CEP78_METADATA, TRANSFER_SESSION_WASM,
    },
    installer_request_builder::{
        InstallerRequestBuilder, MetadataMutability, MintingMode, NFTHolderMode, NFTIdentifierMode,
        NFTMetadataKind, OwnerReverseLookupMode, OwnershipMode, WhitelistMode,
        TEST_CUSTOM_METADATA, TEST_CUSTOM_METADATA_SCHEMA,
    },
    support::{
        self, assert_expected_error, call_session_code_with_ret, genesis,
        get_dictionary_value_from_key, get_nft_contract_hash, get_nft_contract_hash_key,
        get_token_page_by_hash,
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
) -> LmdbWasmTestBuilder {
    let mut builder = genesis();

    let mut install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_collection_name(NFT_TEST_COLLECTION.to_string())
            .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
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
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
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
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(2u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

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

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let account_hash = *DEFAULT_ACCOUNT_ADDR;

    let actual_balance: u64 = call_session_code_with_ret(
        &mut builder,
        account_hash,
        nft_contract_key,
        runtime_args! {
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
        },
        BALANCE_OF_SESSION_WASM,
        ARG_KEY_NAME,
    );

    let expected_balance = 1u64;
    assert_eq!(
        actual_balance, expected_balance,
        "actual and expected balances should be equal"
    );

    let token_id = 0u64;

    let actual_owner: Key = call_session_code_with_ret(
        &mut builder,
        account_hash,
        nft_contract_key,
        runtime_args! {
            ARG_IS_HASH_IDENTIFIER_MODE => false,
            ARG_TOKEN_ID => token_id,
        },
        OWNER_OF_SESSION_WASM,
        ARG_KEY_NAME,
    );

    let expected_owner = *DEFAULT_ACCOUNT_KEY;
    assert_eq!(
        actual_owner, expected_owner,
        "actual and expected owner should be equal"
    );

    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_SPENDER => *ACCOUNT_1_KEY
        },
    )
    .build();
    builder.exec(approve_request).expect_success().commit();

    let actual_approved_account: Option<Key> = call_session_code_with_ret(
        &mut builder,
        account_hash,
        nft_contract_key,
        runtime_args! {
            ARG_IS_HASH_IDENTIFIER_MODE => false,
            ARG_TOKEN_ID => token_id,
        },
        GET_APPROVED_WASM,
        ARG_KEY_NAME,
    );

    let expected_approved_account = *ACCOUNT_1_KEY;
    assert_eq!(
        actual_approved_account,
        Some(expected_approved_account),
        "actual and expected approved account should be equal"
    );
}

#[test]
fn should_mint() {
    let mut builder = genesis();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_ownership_mode(OwnershipMode::Transferable)
            .with_nft_metadata_kind(NFTMetadataKind::CEP78)
            .with_total_token_supply(2u64);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);
    let token_owner: Key = *DEFAULT_ACCOUNT_KEY;

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => token_owner,
            ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    // Expect Mint event.
    let expected_event = Mint::new(
        token_owner,
        &TokenIdentifier::Index(0),
        TEST_PRETTY_CEP78_METADATA.to_string(),
    );
    let actual_event: Mint = support::get_event(&builder, &nft_contract_key, 0).unwrap();
    assert_eq!(actual_event, expected_event, "Expected Mint event.");
}

#[test]
fn mint_should_return_dictionary_key_to_callers_owned_tokens() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_allowing_minting(true)
        .build();

    builder.exec(install_request).expect_success().commit();

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

    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap();

    let nft_receipt: String = support::query_stored_value(&builder, nft_contract_key, RECEIPT_NAME);

    let account_receipt = *account
        .named_keys()
        .get(&format!("{nft_receipt}_m_{PAGE_SIZE}_p_{}", 0))
        .expect("must have receipt");

    let actual_page = builder
        .query(None, account_receipt, &[])
        .expect("must have stored_value")
        .as_cl_value()
        .map(|page_cl_value| CLValue::into_t::<Vec<bool>>(page_cl_value.clone()))
        .unwrap()
        .unwrap();

    let expected_page = {
        let mut page = vec![false; PAGE_SIZE as usize];
        let _ = std::mem::replace(&mut page[0], true);
        page
    };

    assert_eq!(actual_page, expected_page);
}

#[test]
fn mint_should_increment_number_of_minted_tokens_by_one_and_add_public_key_to_token_owners() {
    let mut builder = genesis();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64)
            .with_ownership_mode(OwnershipMode::Transferable);
    builder
        .exec(install_request_builder.build())
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

    //mint should have incremented number_of_minted_tokens by one
    let query_result: u64 =
        support::query_stored_value(&builder, nft_contract_key, NUMBER_OF_MINTED_TOKENS);

    assert_eq!(
        query_result, 1u64,
        "number_of_minted_tokens initialized at installation should have incremented by one"
    );

    let token_id = 0u64;

    let actual_token_meta_data = support::get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_NFT721,
        &token_id.to_string(),
    );

    assert_eq!(actual_token_meta_data, TEST_PRETTY_721_META_DATA);

    let minter_account_hash = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    )
    .into_account()
    .unwrap();

    assert_eq!(DEFAULT_ACCOUNT_ADDR.clone(), minter_account_hash);

    let token_page =
        support::get_token_page_by_id(&builder, &nft_contract_key, &DEFAULT_ACCOUNT_KEY, token_id);

    assert!(token_page[0]);

    // If total_token_supply is initialized to 1 the following test should fail.
    // If we set total_token_supply > 1 it should pass

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
}

#[test]
fn should_set_meta_data() {
    let mut builder = genesis();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64)
            .with_ownership_mode(OwnershipMode::Transferable);
    builder
        .exec(install_request_builder.build())
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

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let token_id = 0u64;

    let actual_token_meta_data = support::get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_NFT721,
        &token_id.to_string(),
    );

    assert_eq!(actual_token_meta_data, TEST_PRETTY_721_META_DATA);
}

#[test]
fn should_set_issuer() {
    let mut builder = genesis();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64)
            .with_ownership_mode(OwnershipMode::Transferable);
    builder
        .exec(install_request_builder.build())
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

    //Let's start querying
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);
    let token_id = 0u64;

    let actual_token_issuer = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_ISSUERS,
        &token_id.to_string(),
    )
    .into_account()
    .unwrap();

    assert_eq!(actual_token_issuer, DEFAULT_ACCOUNT_ADDR.clone());
}

#[test]
fn should_set_issuer_with_different_owner() {
    let mut builder = genesis();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64)
            .with_ownership_mode(OwnershipMode::Transferable);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let account_user_1 = ACCOUNT_1_ADDR.to_owned();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(account_user_1),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    //Let's start querying
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let token_id = 0u64;

    let actual_token_issuer = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_ISSUERS,
        &token_id.to_string(),
    )
    .into_account()
    .unwrap();

    assert_eq!(actual_token_issuer, *DEFAULT_ACCOUNT_ADDR);
}

#[test]
fn should_track_token_balance_by_owner() {
    let mut builder = genesis();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64)
            .with_ownership_mode(OwnershipMode::Transferable);
    builder
        .exec(install_request_builder.build())
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

    let token_owner = DEFAULT_ACCOUNT_ADDR.to_string();
    let actual_minter_balance = support::get_dictionary_value_from_key::<u64>(
        &builder,
        &nft_contract_key,
        TOKEN_COUNT,
        &token_owner,
    );
    let expected_minter_balance = 1u64;
    assert_eq!(actual_minter_balance, expected_minter_balance);
}

#[test]
fn should_allow_public_minting_with_flag_set_to_true() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_minting_mode(MintingMode::Public)
        .build();
    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let public_minting_status =
        support::query_stored_value::<u8>(&builder, nft_contract_key, ARG_MINTING_MODE);

    assert_eq!(
        public_minting_status,
        MintingMode::Public as u8,
        "public minting should be set to true"
    );

    let mint_session_call = ExecuteRequestBuilder::standard(
        *ACCOUNT_1_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *ACCOUNT_1_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_id = 0u64;

    let minter_account_hash = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    )
    .into_account()
    .unwrap();

    assert_eq!(*ACCOUNT_1_ADDR, minter_account_hash);
}

#[test]
fn should_disallow_public_minting_with_flag_set_to_false() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_minting_mode(MintingMode::Installer)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();
    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let account_user_1 = ACCOUNT_1_ADDR.to_owned();

    let public_minting_status =
        support::query_stored_value::<u8>(&builder, nft_contract_key, ARG_MINTING_MODE);

    assert_eq!(
        public_minting_status,
        MintingMode::Installer as u8,
        "public minting should be set to false"
    );

    let mint_session_call = ExecuteRequestBuilder::standard(
        account_user_1,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(account_user_1),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_failure();
}

#[test]
fn should_allow_minting_for_different_public_key_with_minting_mode_set_to_public() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_minting_mode(MintingMode::Public)
        .build();
    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let account_user_1 = ACCOUNT_1_ADDR.to_owned();
    let account_user_2 = ACCOUNT_2_ADDR.to_owned();

    let public_minting_status =
        support::query_stored_value::<u8>(&builder, nft_contract_key, ARG_MINTING_MODE);

    assert_eq!(
        public_minting_status,
        MintingMode::Public as u8,
        "minting mode should be set to public"
    );

    let mint_session_call = ExecuteRequestBuilder::standard(
        account_user_1,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(account_user_1),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let mint_session_call = ExecuteRequestBuilder::standard(
        account_user_2,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(account_user_2),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();
}

#[test]
fn should_set_approval_for_all() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();
    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let owner_key = *DEFAULT_ACCOUNT_KEY;

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => owner_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    let operator = ACCOUNT_3_ADDR.to_owned();
    let operator_key = *ACCOUNT_3_KEY;

    let set_approve_for_all_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => operator_key
        },
    )
    .build();

    builder
        .exec(set_approve_for_all_request)
        .expect_success()
        .commit();

    let is_operator = call_session_code_with_ret::<bool>(
        &mut builder,
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_key,
        runtime_args! {
            ARG_TOKEN_OWNER => owner_key,
            ARG_OPERATOR => operator_key,
        },
        IS_APPROVED_FOR_ALL_WASM,
        ARG_KEY_NAME,
    );

    assert!(is_operator, "expected operator to be approved for all");

    // Expect ApprovalForAll event.
    let expected_event = ApprovalForAll::new(owner_key, operator_key);
    let actual_event: ApprovalForAll = support::get_event(&builder, &nft_contract_key, 1).unwrap();
    assert_eq!(
        actual_event, expected_event,
        "Expected ApprovalForAll event."
    );

    // Test if two minted tokens are transferable by operator
    let token_receiver = *ACCOUNT_1_ADDR;
    let token_receiver_key = *ACCOUNT_1_KEY;

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_receiver_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let token_id = 0u64;

    // Transfer first minted token by operator
    let transfer_request = ExecuteRequestBuilder::standard(
        operator,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_ID => token_id,
            ARG_IS_HASH_IDENTIFIER_MODE => false,
            ARG_SOURCE_KEY => owner_key,
            ARG_TARGET_KEY => token_receiver_key,
        },
    )
    .build();
    builder.exec(transfer_request).expect_success().commit();

    let actual_token_owner = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    )
    .into_account()
    .unwrap();

    assert_eq!(actual_token_owner, token_receiver);

    // Second mint by owner
    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => owner_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    let token_id = 1u64;

    // Transfer second minted token by operator
    let transfer_request = ExecuteRequestBuilder::standard(
        operator,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_ID => token_id,
            ARG_IS_HASH_IDENTIFIER_MODE => false,
            ARG_SOURCE_KEY => owner_key,
            ARG_TARGET_KEY => token_receiver_key,
        },
    )
    .build();
    builder.exec(transfer_request).expect_success().commit();

    let actual_token_owner = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    )
    .into_account()
    .unwrap();

    assert_eq!(actual_token_owner, token_receiver);
}

#[test]
fn should_revoke_approval_for_all() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();
    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);
    let owner_key = *DEFAULT_ACCOUNT_KEY;

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => owner_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    let operator = ACCOUNT_3_ADDR.to_owned();
    let operator_key = Key::Account(operator);

    let set_approve_for_all_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => operator_key
        },
    )
    .build();

    builder
        .exec(set_approve_for_all_request)
        .expect_success()
        .commit();

    let is_operator = call_session_code_with_ret::<bool>(
        &mut builder,
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_key,
        runtime_args! {
            ARG_TOKEN_OWNER => owner_key,
            ARG_OPERATOR => operator_key,
        },
        IS_APPROVED_FOR_ALL_WASM,
        ARG_KEY_NAME,
    );

    assert!(is_operator, "expected operator to be approved for all");

    // Expect ApprovalForAll event.
    let expected_event = ApprovalForAll::new(owner_key, operator_key);
    let actual_event: ApprovalForAll = support::get_event(&builder, &nft_contract_key, 1).unwrap();
    assert_eq!(
        actual_event, expected_event,
        "Expected ApprovalForAll event."
    );

    let revoke_approve_for_all_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => false,
            ARG_OPERATOR => operator_key
        },
    )
    .build();

    builder
        .exec(revoke_approve_for_all_request)
        .expect_success()
        .commit();

    let is_operator = call_session_code_with_ret::<bool>(
        &mut builder,
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_key,
        runtime_args! {
            ARG_TOKEN_OWNER => owner_key,
            ARG_OPERATOR => operator_key,
        },
        IS_APPROVED_FOR_ALL_WASM,
        ARG_KEY_NAME,
    );

    assert!(!is_operator, "expected operator not to be approved for all");

    // Expect RevokedForAll event.
    let expected_event = RevokedForAll::new(owner_key, operator_key);
    let actual_event: RevokedForAll = support::get_event(&builder, &nft_contract_key, 2).unwrap();
    assert_eq!(
        actual_event, expected_event,
        "Expected RevokedForAll event."
    );
}

#[test]
fn should_not_mint_with_invalid_nft721_metadata() {
    let mut builder = genesis();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64)
            .with_ownership_mode(OwnershipMode::Transferable);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,

            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => MALFORMED_META_DATA,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
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
    let mut builder = genesis();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64)
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
            ARG_TOKEN_META_DATA => TEST_COMPACT_META_DATA,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_id = 0u64;

    let actual_metadata = get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_NFT721,
        &token_id.to_string(),
    );

    assert_eq!(TEST_PRETTY_721_META_DATA, actual_metadata)
}

#[test]
fn should_mint_with_valid_cep99_metadata() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(2u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_nft_metadata_kind(NFTMetadataKind::CEP78)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_id = 0u64;

    let actual_metadata = get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_CEP78,
        &token_id.to_string(),
    );

    assert_eq!(TEST_PRETTY_CEP78_METADATA, actual_metadata)
}

#[test]
fn should_mint_with_custom_metadata_validation() {
    let mut builder = genesis();

    let custom_json_schema =
        serde_json::to_string(&*TEST_CUSTOM_METADATA_SCHEMA).expect("must convert to json schema");

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(2u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_nft_metadata_kind(NFTMetadataKind::CustomValidated)
        .with_json_schema(custom_json_schema)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_TOKEN_META_DATA => serde_json::to_string(&*TEST_CUSTOM_METADATA).expect("must convert to json metadata") ,
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_id = 0u64;

    let actual_metadata = get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_CUSTOM_VALIDATED,
        &token_id.to_string(),
    );

    let pretty_custom_metadata = serde_json::to_string_pretty(&*TEST_CUSTOM_METADATA)
        .expect("must convert to json metadata");

    assert_eq!(pretty_custom_metadata, actual_metadata)
}

#[test]
fn should_mint_with_raw_metadata() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(2u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,

            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => "raw_string".to_string() ,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_id = 0u64;

    let actual_metadata = get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_RAW,
        &token_id.to_string(),
    );

    assert_eq!("raw_string".to_string(), actual_metadata)
}

#[test]
fn should_mint_with_hash_identifier_mode() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_total_token_supply(10u64)
        .build();

    builder.exec(install_request).expect_success().commit();

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

    let token_id_hash: String =
        base16::encode_lower(&support::create_blake2b_hash(TEST_PRETTY_721_META_DATA));

    let token_page = get_token_page_by_hash(
        &builder,
        &nft_contract_key,
        &DEFAULT_ACCOUNT_KEY,
        token_id_hash,
    );

    assert!(token_page[0])
}

#[test]
fn should_fail_to_mint_when_immediate_caller_is_account_in_contract_mode() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(2u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Unlocked)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_COMPACT_META_DATA,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_failure();

    let error = builder.get_error().expect("must have error");

    assert_expected_error(error, 76, "InvalidHolderMode(76) must have been raised");
}

#[test]
fn should_approve_in_hash_identifier_mode() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
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

    let spender = Key::Account(AccountHash::new([7u8; 32]));

    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_HASH => token_hash.clone(),
            ARG_SPENDER => spender
        },
    )
    .build();

    builder.exec(approve_request).expect_success().commit();

    let maybe_approved_account = support::get_dictionary_value_from_key::<Option<Key>>(
        &builder,
        &nft_contract_key,
        APPROVED,
        &token_hash,
    );

    assert_eq!(maybe_approved_account, Some(spender))
}

#[test]
fn should_mint_without_returning_receipts_and_flat_gas_cost() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(1000u64)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => "",
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let first_mint_gas_cost = builder.last_exec_gas_consumed();

    let mint_session_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(AccountHash::new([3u8;32])),
            ARG_TOKEN_META_DATA => "",
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let second_mint_gas_cost = builder.last_exec_gas_consumed();

    let mint_session_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(AccountHash::new([4u8;32])),
            ARG_TOKEN_META_DATA => "",
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let third_mint_gas_cost = builder.last_exec_gas_consumed();

    // In this case there is no first time allocation of a page.
    // Therefore the second and first mints must have equivalent gas costs.
    assert_eq!(first_mint_gas_cost, second_mint_gas_cost);
    assert_eq!(second_mint_gas_cost, third_mint_gas_cost)
}

// A test to ensure that the page table allocation is preserved
// even if the "register_owner" is called twice.
#[test]
fn should_maintain_page_table_despite_invoking_register_owner() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(1000u64)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => "",
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let actual_page_table = support::get_dictionary_value_from_key::<Vec<bool>>(
        &builder,
        &nft_contract_key,
        PAGE_TABLE,
        &DEFAULT_ACCOUNT_ADDR.to_string(),
    );

    assert_eq!(actual_page_table.len(), 1);

    // The mint WASM will register the owner, now we re-invoke the same entry point
    // and ensure that the page table doesn't mutate.
    let register_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY
        },
    )
    .build();

    builder.exec(register_call).expect_success().commit();

    let table_post_register = support::get_dictionary_value_from_key::<Vec<bool>>(
        &builder,
        &nft_contract_key,
        PAGE_TABLE,
        &DEFAULT_ACCOUNT_ADDR.to_string(),
    );

    assert_eq!(actual_page_table, table_post_register)
}

#[test]
fn should_prevent_mint_to_unregistered_owner() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(1000u64)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_reporting_mode(OwnerReverseLookupMode::Complete)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => "",
        },
    )
    .build();

    builder.exec(mint_session_call).expect_failure();

    let error = builder.get_error().expect("must have error");

    assert_expected_error(error, 127u16, "must raise unregistered owner in mint");
}

#[test]
fn should_mint_with_two_required_metadata_kind() {
    let mut builder = genesis();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(1000u64)
            .with_identifier_mode(NFTIdentifierMode::Ordinal)
            .with_metadata_mutability(MetadataMutability::Immutable)
            .with_ownership_mode(OwnershipMode::Transferable)
            .with_reporting_mode(OwnerReverseLookupMode::Complete)
            .with_nft_metadata_kind(NFTMetadataKind::CEP78)
            .with_additional_required_metadata(vec![NFTMetadataKind::Raw as u8]);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA,
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let meta_78 = support::get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_CEP78,
        &0u64.to_string(),
    );

    let meta_raw = support::get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_RAW,
        &0u64.to_string(),
    );

    assert_eq!(meta_78, TEST_PRETTY_CEP78_METADATA);
    assert_eq!(meta_raw, TEST_PRETTY_CEP78_METADATA);
}

#[test]
fn should_mint_with_one_required_one_optional_metadata_kind_without_optional() {
    let mut builder = genesis();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_nft_metadata_kind(NFTMetadataKind::CEP78)
            .with_optional_metadata(vec![NFTMetadataKind::Raw as u8])
            .with_total_token_supply(1000u64)
            .with_identifier_mode(NFTIdentifierMode::Ordinal)
            .with_metadata_mutability(MetadataMutability::Immutable)
            .with_ownership_mode(OwnershipMode::Transferable)
            .with_reporting_mode(OwnerReverseLookupMode::Complete);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA,
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    let meta_78 = support::get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_CEP78,
        &0u64.to_string(),
    );

    let meta_raw = support::get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_RAW,
        &0u64.to_string(),
    );

    assert_eq!(meta_78, TEST_PRETTY_CEP78_METADATA);
    assert_eq!(meta_raw, TEST_PRETTY_CEP78_METADATA);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA,
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let meta_78 = support::get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_CEP78,
        &1u64.to_string(),
    );

    assert_eq!(meta_78, TEST_PRETTY_CEP78_METADATA);
}

#[test]
fn should_not_mint_with_missing_required_metadata() {
    let mut builder = genesis();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(1000u64)
            .with_identifier_mode(NFTIdentifierMode::Ordinal)
            .with_metadata_mutability(MetadataMutability::Immutable)
            .with_ownership_mode(OwnershipMode::Transferable)
            .with_reporting_mode(OwnerReverseLookupMode::Complete)
            .with_nft_metadata_kind(NFTMetadataKind::CEP78)
            .with_additional_required_metadata(vec![NFTMetadataKind::NFT721 as u8]);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_failure();
    let error = builder.get_error().expect("mint request must have failed");
    assert_expected_error(
        error,
        88,
        "NFT721 metadata does not satisfy the required CEP78 requirement.",
    )
}

#[test]
fn should_mint_with_transfer_only_reporting() {
    let mut builder = genesis();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_nft_metadata_kind(NFTMetadataKind::CEP78)
            .with_ownership_mode(OwnershipMode::Transferable)
            .with_reporting_mode(OwnerReverseLookupMode::TransfersOnly)
            .with_total_token_supply(2u64);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);
    let nft_contract_hash = get_nft_contract_hash(&builder);

    let mint_runtime_args = runtime_args! {
        ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
        ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA.to_string(),
    };

    let minting_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder.exec(minting_request).expect_success().commit();

    let actual_balance_after_mint = support::get_dictionary_value_from_key::<u64>(
        &builder,
        &nft_contract_key,
        TOKEN_COUNT,
        &DEFAULT_ACCOUNT_ADDR.to_string(),
    );

    let expected_balance_after_mint = 1u64;
    assert_eq!(actual_balance_after_mint, expected_balance_after_mint);
}

#[test]
fn should_approve_all_in_hash_identifier_mode() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(1000u64)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_nft_metadata_kind(NFTMetadataKind::CEP78)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let owner_key = *DEFAULT_ACCOUNT_KEY;
    let operator_key = Key::Account(AccountHash::new([7u8; 32]));

    let mint_session_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => owner_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA,
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let mint_session_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => owner_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_UPDATED_CEP78_METADATA,
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let approval_all_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => operator_key,
        },
    )
    .build();

    builder.exec(approval_all_request).expect_success().commit();

    let is_operator = call_session_code_with_ret::<bool>(
        &mut builder,
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_key,
        runtime_args! {
            ARG_TOKEN_OWNER => owner_key,
            ARG_OPERATOR => operator_key,
        },
        IS_APPROVED_FOR_ALL_WASM,
        ARG_KEY_NAME,
    );

    assert!(is_operator, "expected operator to be approved for all");

    // Expect ApprovalForAll event.
    let expected_event = ApprovalForAll::new(owner_key, operator_key);
    let expected_event_index = 2;
    let actual_event: ApprovalForAll =
        support::get_event(&builder, &nft_contract_key, expected_event_index).unwrap();
    assert_eq!(
        actual_event, expected_event,
        "Expected ApprovalForAll event."
    );
}

#[test]
fn should_approve_all_with_flat_gas_cost() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();
    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);
    let owner_key = *DEFAULT_ACCOUNT_KEY;

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => owner_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    let operator = ACCOUNT_1_ADDR.to_owned();
    let operator_key = Key::Account(operator);

    let set_approve_for_all_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => operator_key
        },
    )
    .build();

    builder
        .exec(set_approve_for_all_request)
        .expect_success()
        .commit();

    let first_set_approve_for_all_gas_cost = builder.last_exec_gas_consumed();

    let is_operator = call_session_code_with_ret::<bool>(
        &mut builder,
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_key,
        runtime_args! {
            ARG_TOKEN_OWNER => owner_key,
            ARG_OPERATOR => operator_key,
        },
        IS_APPROVED_FOR_ALL_WASM,
        ARG_KEY_NAME,
    );

    assert!(is_operator, "expected operator to be approved for all");

    let other_operator = ACCOUNT_2_ADDR.to_owned();
    let other_operator_key = Key::Account(other_operator);

    let set_approve_for_all_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => other_operator_key
        },
    )
    .build();

    builder
        .exec(set_approve_for_all_request)
        .expect_success()
        .commit();

    let second_set_approve_for_all_gas_cost = builder.last_exec_gas_consumed();

    let is_also_operator = call_session_code_with_ret::<bool>(
        &mut builder,
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_key,
        runtime_args! {
            ARG_TOKEN_OWNER => owner_key,
            ARG_OPERATOR => other_operator_key,
        },
        IS_APPROVED_FOR_ALL_WASM,
        ARG_KEY_NAME,
    );

    assert!(
        is_also_operator,
        "expected other operator to be approved for all"
    );

    // Operator approval should have flat gas costs
    // Therefore the second and first set_approve_for_all must have equivalent gas costs.
    assert_eq!(
        first_set_approve_for_all_gas_cost,
        second_set_approve_for_all_gas_cost
    )
}
