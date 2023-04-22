use std::collections::BTreeMap;

use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{account::AccountHash, runtime_args, Key, RuntimeArgs};

use contract::{
    constants::{
        ACCESS_KEY_NAME_1_0_0, APPROVED, ARG_APPROVE_ALL, ARG_COLLECTION_NAME, ARG_EVENTS_MODE,
        ARG_NAMED_KEY_CONVENTION, ARG_OPERATOR, ARG_SOURCE_KEY, ARG_SPENDER, ARG_TARGET_KEY,
        ARG_TOKEN_HASH, ARG_TOKEN_ID, ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, BURNT_TOKENS,
        ENTRY_POINT_APPROVE, ENTRY_POINT_BURN, ENTRY_POINT_REGISTER_OWNER,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL, ENTRY_POINT_SET_TOKEN_METADATA, EVENTS, EVENT_TYPE,
        METADATA_CEP78, METADATA_CUSTOM_VALIDATED, METADATA_NFT721, METADATA_RAW, OPERATOR, OWNER,
        PREFIX_CEP78, PREFIX_HASH_KEY_NAME, RECIPIENT, TOKEN_COUNT, TOKEN_ID,
    },
    modalities::{EventsMode, NamedKeyConventionMode},
};

use crate::utility::{
    constants::{
        ARG_IS_HASH_IDENTIFIER_MODE, ARG_KEY_NAME, ARG_NFT_CONTRACT_HASH,
        ARG_NFT_CONTRACT_PACKAGE_HASH, CONTRACT_1_0_0_WASM, CONTRACT_NAME,
        IS_APPROVED_FOR_ALL_WASM, MINT_1_0_0_WASM, MINT_SESSION_WASM, NFT_CONTRACT_WASM,
        NFT_TEST_COLLECTION, NFT_TEST_SYMBOL, TEST_PRETTY_721_META_DATA,
        TEST_PRETTY_CEP78_METADATA, TEST_PRETTY_UPDATED_721_META_DATA,
        TEST_PRETTY_UPDATED_CEP78_METADATA, TRANSFER_SESSION_WASM,
    },
    installer_request_builder::{
        InstallerRequestBuilder, MetadataMutability, NFTIdentifierMode, NFTMetadataKind,
        OwnerReverseLookupMode, OwnershipMode, TEST_CUSTOM_METADATA, TEST_CUSTOM_METADATA_SCHEMA,
        TEST_CUSTOM_UPDATED_METADATA,
    },
    support::{
        self, call_session_code_with_ret, create_funded_dummy_account,
        get_dictionary_value_from_key, get_nft_contract_hash, get_token_page_by_id,
        query_stored_value,
    },
};

// cep47 event style
#[test]
fn should_record_cep47_dictionary_style_mint_event() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_nft_metadata_kind(NFTMetadataKind::CEP78)
            .with_ownership_mode(OwnershipMode::Transferable)
            .with_total_token_supply(2u64)
            .with_events_mode(EventsMode::CEP47);
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
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let event = get_dictionary_value_from_key::<BTreeMap<String, String>>(
        &builder,
        &nft_contract_key,
        EVENTS,
        "0",
    );

    let collection_name: String = query_stored_value(
        &builder,
        nft_contract_key,
        vec![ARG_COLLECTION_NAME.to_string()],
    );

    let package = query_stored_value::<String>(
        &builder,
        nft_contract_key,
        vec![format!("{PREFIX_CEP78}_{collection_name}")],
    );

    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();
    expected_event.insert(EVENT_TYPE.to_string(), "Mint".to_string());
    expected_event.insert(PREFIX_HASH_KEY_NAME.to_string(), package);
    expected_event.insert(
        RECIPIENT.to_string(),
        "Key::Account(58b891759929bd4ed5a9cce20b9d6e3c96a66c21386bed96040e17dd07b79fa7)"
            .to_string(),
    );
    expected_event.insert(TOKEN_ID.to_string(), "0".to_string());
    assert_eq!(event, expected_event);
}

#[test]
fn should_record_cep47_dictionary_style_transfer_token_event_in_hash_identifier_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_total_token_supply(10u64)
        .with_events_mode(EventsMode::CEP47)
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

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => Key::Account(AccountHash::new([3u8;32]))
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let transfer_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_IS_HASH_IDENTIFIER_MODE => true,
            ARG_TOKEN_HASH => token_hash,
            ARG_SOURCE_KEY => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TARGET_KEY =>  Key::Account(AccountHash::new([3u8;32])),
        },
    )
    .build();

    builder.exec(transfer_request).expect_success().commit();

    let event = get_dictionary_value_from_key::<BTreeMap<String, String>>(
        &builder,
        &nft_contract_key,
        EVENTS,
        "1",
    );

    let collection_name: String = query_stored_value(
        &builder,
        nft_contract_key,
        vec![ARG_COLLECTION_NAME.to_string()],
    );

    let package = query_stored_value::<String>(
        &builder,
        nft_contract_key,
        vec![format!("{PREFIX_CEP78}_{collection_name}")],
    );

    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();

    expected_event.insert(EVENT_TYPE.to_string(), "Transfer".to_string());
    expected_event.insert(PREFIX_HASH_KEY_NAME.to_string(), package);
    expected_event.insert(
        RECIPIENT.to_string(),
        "Key::Account(0303030303030303030303030303030303030303030303030303030303030303)"
            .to_string(),
    );
    expected_event.insert(
        "sender".to_string(),
        "Key::Account(58b891759929bd4ed5a9cce20b9d6e3c96a66c21386bed96040e17dd07b79fa7)"
            .to_string(),
    );
    expected_event.insert(
        TOKEN_ID.to_string(),
        "69fe422f3b0d0ba4d911323451a490bdd679c437e889127700b7bf83123b2d0c".to_string(),
    );
    assert_eq!(event, expected_event);
}

#[test]
fn should_record_cep47_dictionary_style_metadata_update_event_for_nft721_using_token_id() {
    let nft_metadata_kind = NFTMetadataKind::NFT721;
    let identifier_mode = NFTIdentifierMode::Ordinal;

    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(10u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_metadata_mutability(MetadataMutability::Mutable)
        .with_identifier_mode(identifier_mode)
        .with_nft_metadata_kind(nft_metadata_kind)
        .with_json_schema(
            serde_json::to_string(&*TEST_CUSTOM_METADATA_SCHEMA)
                .expect("must convert to json schema"),
        )
        .with_events_mode(EventsMode::CEP47)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let custom_metadata = serde_json::to_string_pretty(&*TEST_CUSTOM_METADATA)
        .expect("must convert to json metadata");

    let original_metadata = match &nft_metadata_kind {
        NFTMetadataKind::CEP78 => TEST_PRETTY_CEP78_METADATA,
        NFTMetadataKind::NFT721 => TEST_PRETTY_721_META_DATA,
        NFTMetadataKind::Raw => "",
        NFTMetadataKind::CustomValidated => &custom_metadata,
    };

    let mint_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => original_metadata.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_request).expect_success().commit();

    let dictionary_name = match nft_metadata_kind {
        NFTMetadataKind::CEP78 => METADATA_CEP78,
        NFTMetadataKind::NFT721 => METADATA_NFT721,
        NFTMetadataKind::Raw => METADATA_RAW,
        NFTMetadataKind::CustomValidated => METADATA_CUSTOM_VALIDATED,
    };

    let actual_metadata = match identifier_mode {
        NFTIdentifierMode::Ordinal => get_dictionary_value_from_key::<String>(
            &builder,
            &nft_contract_key,
            dictionary_name,
            &0u64.to_string(),
        ),
        NFTIdentifierMode::Hash => get_dictionary_value_from_key(
            &builder,
            &nft_contract_key,
            dictionary_name,
            &base16::encode_lower(&support::create_blake2b_hash(original_metadata)),
        ),
    };

    assert_eq!(actual_metadata, original_metadata.to_string());

    let custom_updated_metadata = serde_json::to_string_pretty(&*TEST_CUSTOM_UPDATED_METADATA)
        .expect("must convert to json metadata");

    let updated_metadata = match &nft_metadata_kind {
        NFTMetadataKind::CEP78 => TEST_PRETTY_UPDATED_CEP78_METADATA,
        NFTMetadataKind::NFT721 => TEST_PRETTY_UPDATED_721_META_DATA,
        NFTMetadataKind::Raw => "",
        NFTMetadataKind::CustomValidated => &custom_updated_metadata,
    };

    let update_metadata_runtime_args = {
        let mut args = runtime_args! {
            ARG_TOKEN_META_DATA => updated_metadata.to_string(),
        };
        match identifier_mode {
            NFTIdentifierMode::Ordinal => args.insert(ARG_TOKEN_ID, 0u64).expect("must get args"),
            NFTIdentifierMode::Hash => args
                .insert(
                    ARG_TOKEN_HASH,
                    base16::encode_lower(&support::create_blake2b_hash(original_metadata)),
                )
                .expect("must get args"),
        }
        args
    };

    let update_metadata_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        get_nft_contract_hash(&builder),
        ENTRY_POINT_SET_TOKEN_METADATA,
        update_metadata_runtime_args,
    )
    .build();

    builder
        .exec(update_metadata_request)
        .expect_success()
        .commit();

    let actual_updated_metadata = match identifier_mode {
        NFTIdentifierMode::Ordinal => get_dictionary_value_from_key::<String>(
            &builder,
            &nft_contract_key,
            dictionary_name,
            &0u64.to_string(),
        ),
        NFTIdentifierMode::Hash => get_dictionary_value_from_key(
            &builder,
            &nft_contract_key,
            dictionary_name,
            &base16::encode_lower(&support::create_blake2b_hash(original_metadata)),
        ),
    };

    assert_eq!(actual_updated_metadata, updated_metadata.to_string());

    let event = get_dictionary_value_from_key::<BTreeMap<String, String>>(
        &builder,
        &nft_contract_key,
        EVENTS,
        "1",
    );

    let collection_name: String = query_stored_value(
        &builder,
        nft_contract_key,
        vec![ARG_COLLECTION_NAME.to_string()],
    );

    let package = query_stored_value::<String>(
        &builder,
        nft_contract_key,
        vec![format!("{PREFIX_CEP78}_{collection_name}")],
    );

    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();
    expected_event.insert(EVENT_TYPE.to_string(), "MetadataUpdate".to_string());
    expected_event.insert(PREFIX_HASH_KEY_NAME.to_string(), package);
    expected_event.insert(TOKEN_ID.to_string(), "0".to_string());
    assert_eq!(event, expected_event);
}

#[test]
fn should_cep47_dictionary_style_burn_event() {
    let token_id = 0u64;
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(100u64)
            .with_ownership_mode(OwnershipMode::Transferable)
            .with_reporting_mode(OwnerReverseLookupMode::Complete)
            .with_events_mode(EventsMode::CEP47)
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

    let token_page = get_token_page_by_id(
        &builder,
        nft_contract_key,
        &Key::Account(*DEFAULT_ACCOUNT_ADDR),
        token_id,
    );

    assert!(token_page[0]);

    let actual_balance_before_burn = get_dictionary_value_from_key::<u64>(
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
    get_dictionary_value_from_key::<()>(
        &builder,
        nft_contract_key,
        BURNT_TOKENS,
        &token_id.to_string(),
    );

    // This will error of token is not registered as
    let actual_balance = get_dictionary_value_from_key::<u64>(
        &builder,
        nft_contract_key,
        TOKEN_COUNT,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    let expected_balance = 0u64;
    assert_eq!(actual_balance, expected_balance);

    let event = get_dictionary_value_from_key::<BTreeMap<String, String>>(
        &builder,
        nft_contract_key,
        EVENTS,
        "1",
    );

    let collection_name: String = query_stored_value(
        &builder,
        *nft_contract_key,
        vec![ARG_COLLECTION_NAME.to_string()],
    );

    let package = query_stored_value::<String>(
        &builder,
        *nft_contract_key,
        vec![format!("{PREFIX_CEP78}_{collection_name}")],
    );

    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();
    expected_event.insert(EVENT_TYPE.to_string(), "Burn".to_string());
    expected_event.insert(PREFIX_HASH_KEY_NAME.to_string(), package);
    expected_event.insert(
        OWNER.to_string(),
        "Key::Account(58b891759929bd4ed5a9cce20b9d6e3c96a66c21386bed96040e17dd07b79fa7)"
            .to_string(),
    );
    expected_event.insert(TOKEN_ID.to_string(), "0".to_string());
    assert_eq!(event, expected_event);
}

#[test]
fn should_cep47_dictionary_style_approve_event_in_hash_identifier_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_events_mode(EventsMode::CEP47)
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

    let maybe_approved_spender = get_dictionary_value_from_key::<Option<Key>>(
        &builder,
        &nft_contract_key,
        APPROVED,
        &token_hash,
    );

    assert_eq!(maybe_approved_spender, Some(spender));

    let event = get_dictionary_value_from_key::<BTreeMap<String, String>>(
        &builder,
        &nft_contract_key,
        EVENTS,
        "1",
    );

    let collection_name: String = query_stored_value(
        &builder,
        nft_contract_key,
        vec![ARG_COLLECTION_NAME.to_string()],
    );

    let package = query_stored_value::<String>(
        &builder,
        nft_contract_key,
        vec![format!("{PREFIX_CEP78}_{collection_name}")],
    );
    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();
    expected_event.insert(EVENT_TYPE.to_string(), "Approve".to_string());
    expected_event.insert(PREFIX_HASH_KEY_NAME.to_string(), package);
    expected_event.insert(
        OWNER.to_string(),
        "Key::Account(58b891759929bd4ed5a9cce20b9d6e3c96a66c21386bed96040e17dd07b79fa7)"
            .to_string(),
    );
    expected_event.insert(
        "spender".to_string(),
        "Key::Account(0707070707070707070707070707070707070707070707070707070707070707)"
            .to_string(),
    );
    expected_event.insert(
        TOKEN_ID.to_string(),
        "69fe422f3b0d0ba4d911323451a490bdd679c437e889127700b7bf83123b2d0c".to_string(),
    );
    assert_eq!(event, expected_event);
}

#[test]
fn should_cep47_dictionary_style_approvall_for_all_event() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_nft_metadata_kind(NFTMetadataKind::CEP78)
            .with_ownership_mode(OwnershipMode::Transferable)
            .with_total_token_supply(1u64)
            .with_events_mode(EventsMode::CEP47);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let operator = create_funded_dummy_account(&mut builder, None);
    let operator_key = Key::Account(operator);

    let set_approve_all_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => operator_key,
        },
    )
    .build();

    builder
        .exec(set_approve_all_request)
        .expect_success()
        .commit();

    let is_operator = call_session_code_with_ret::<bool>(
        &mut builder,
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_key,
        runtime_args! {
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_OPERATOR => operator_key,
        },
        IS_APPROVED_FOR_ALL_WASM,
        ARG_KEY_NAME,
    );

    assert!(is_operator, "expected operator to be approved for all");

    let event = get_dictionary_value_from_key::<BTreeMap<String, String>>(
        &builder,
        &nft_contract_key,
        EVENTS,
        "1",
    );

    let collection_name: String = query_stored_value(
        &builder,
        nft_contract_key,
        vec![ARG_COLLECTION_NAME.to_string()],
    );

    let package = query_stored_value::<String>(
        &builder,
        nft_contract_key,
        vec![format!("{PREFIX_CEP78}_{collection_name}")],
    );
    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();
    expected_event.insert(EVENT_TYPE.to_string(), "ApprovalForAll".to_string());
    expected_event.insert(PREFIX_HASH_KEY_NAME.to_string(), package);
    expected_event.insert(
        OWNER.to_string(),
        "Key::Account(58b891759929bd4ed5a9cce20b9d6e3c96a66c21386bed96040e17dd07b79fa7)"
            .to_string(),
    );
    expected_event.insert(
        OPERATOR.to_string(),
        "Key::Account(3d5de8c609159a0954e773dd686fb7724428316cb30e00bdc899976127747f55)"
            .to_string(),
    );
    assert_eq!(event, expected_event);
}

#[test]
fn should_cep47_dictionary_style_revoked_for_all_event() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_nft_metadata_kind(NFTMetadataKind::CEP78)
            .with_ownership_mode(OwnershipMode::Transferable)
            .with_total_token_supply(1u64)
            .with_events_mode(EventsMode::CEP47);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let operator = create_funded_dummy_account(&mut builder, None);
    let operator_key = Key::Account(operator);

    let set_approve_all_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => operator_key,
        },
    )
    .build();

    builder
        .exec(set_approve_all_request)
        .expect_success()
        .commit();

    let is_operator = call_session_code_with_ret::<bool>(
        &mut builder,
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_key,
        runtime_args! {
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_OPERATOR => operator_key,
        },
        IS_APPROVED_FOR_ALL_WASM,
        ARG_KEY_NAME,
    );

    assert!(is_operator, "expected operator to be approved for all");

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

    let event = get_dictionary_value_from_key::<BTreeMap<String, String>>(
        &builder,
        &nft_contract_key,
        EVENTS,
        "2",
    );

    let collection_name: String = query_stored_value(
        &builder,
        nft_contract_key,
        vec![ARG_COLLECTION_NAME.to_string()],
    );

    let package = query_stored_value::<String>(
        &builder,
        nft_contract_key,
        vec![format!("{PREFIX_CEP78}_{collection_name}")],
    );
    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();
    expected_event.insert(EVENT_TYPE.to_string(), "RevokedForAll".to_string());
    expected_event.insert(PREFIX_HASH_KEY_NAME.to_string(), package);
    expected_event.insert(
        OWNER.to_string(),
        "Key::Account(58b891759929bd4ed5a9cce20b9d6e3c96a66c21386bed96040e17dd07b79fa7)"
            .to_string(),
    );
    expected_event.insert(
        OPERATOR.to_string(),
        "Key::Account(3d5de8c609159a0954e773dd686fb7724428316cb30e00bdc899976127747f55)"
            .to_string(),
    );
    assert_eq!(event, expected_event);
}

#[test]
fn should_record_migration_event_in_cep47() {
    const OWNED_TOKENS: &str = "owned_tokens";

    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, CONTRACT_1_0_0_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1000u64)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash_1_0_0 = support::get_nft_contract_hash_1_0_0(&builder);
    let nft_contract_key_1_0_0: Key = nft_contract_hash_1_0_0.into();

    let number_of_tokens_pre_migration = 3usize;

    for _ in 0..number_of_tokens_pre_migration {
        let mint_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            MINT_1_0_0_WASM,
            runtime_args! {
                ARG_NFT_CONTRACT_HASH => nft_contract_key_1_0_0,
                ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
                ARG_TOKEN_META_DATA => "",
            },
        )
        .build();

        builder.exec(mint_request).expect_success().commit();
    }

    let previous_token_representation = support::get_dictionary_value_from_key::<Vec<u64>>(
        &builder,
        &nft_contract_key_1_0_0,
        OWNED_TOKENS,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    assert_eq!(previous_token_representation, vec![0, 1, 2]);

    let maybe_access_named_key = builder
        .query(None, Key::Account(*DEFAULT_ACCOUNT_ADDR), &[])
        .unwrap()
        .as_account()
        .unwrap()
        .named_keys()
        .get(ACCESS_KEY_NAME_1_0_0)
        .is_some();

    assert!(maybe_access_named_key);

    let contract_package_hash = support::get_nft_contract_package_hash(&builder);

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_PACKAGE_HASH => contract_package_hash,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_EVENTS_MODE => EventsMode::CEP47 as u8
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let latest_cep47_event_id =
        get_dictionary_value_from_key::<u64>(&builder, &nft_contract_key, EVENTS, "len") - 1u64;

    let event = get_dictionary_value_from_key::<BTreeMap<String, String>>(
        &builder,
        &nft_contract_key,
        EVENTS,
        &latest_cep47_event_id.to_string(),
    );

    let collection_name: String = query_stored_value(
        &builder,
        nft_contract_key,
        vec![ARG_COLLECTION_NAME.to_string()],
    );

    let package = query_stored_value::<String>(
        &builder,
        nft_contract_key,
        vec![format!("{PREFIX_CEP78}_{collection_name}")],
    );
    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();
    expected_event.insert(EVENT_TYPE.to_string(), "Migration".to_string());
    expected_event.insert(PREFIX_HASH_KEY_NAME.to_string(), package);
    assert_eq!(event, expected_event);
}

#[test]
#[should_panic]
fn should_not_record_events_in_no_events_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(100u64)
            .with_ownership_mode(OwnershipMode::Transferable)
            .with_reporting_mode(OwnerReverseLookupMode::Complete)
            .with_events_mode(EventsMode::NoEvents)
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

    // This will error of token is not registered as
    let actual_balance = get_dictionary_value_from_key::<u64>(
        &builder,
        nft_contract_key,
        TOKEN_COUNT,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    let expected_balance = 0u64;
    assert_eq!(actual_balance, expected_balance);

    // Query for the Mint event here and expect failure
    // as no events are being written to global state.
    get_dictionary_value_from_key::<BTreeMap<String, String>>(
        &builder,
        nft_contract_key,
        EVENTS,
        "1",
    );
}
