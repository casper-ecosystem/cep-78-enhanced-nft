use std::collections::BTreeMap;

use casper_engine_test_support::{ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR};
use casper_event_standard::EVENTS_DICT;
use casper_types::{addressable_entity::EntityKindTag, runtime_args, AddressableEntityHash, Key};

use cep78::{
    constants::{
        APPROVED, ARG_APPROVE_ALL, ARG_COLLECTION_NAME, ARG_OPERATOR, ARG_SOURCE_KEY, ARG_SPENDER,
        ARG_TARGET_KEY, ARG_TOKEN_HASH, ARG_TOKEN_ID, ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, BURNER,
        BURNT_TOKENS, ENTRY_POINT_APPROVE, ENTRY_POINT_BURN, ENTRY_POINT_REGISTER_OWNER,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL, ENTRY_POINT_SET_TOKEN_METADATA, EVENTS, EVENT_TYPE,
        METADATA_CEP78, METADATA_CUSTOM_VALIDATED, METADATA_NFT721, METADATA_RAW, OPERATOR, OWNER,
        PREFIX_CEP78, PREFIX_HASH_KEY_NAME, RECIPIENT, SENDER, SPENDER, TOKEN_COUNT, TOKEN_ID,
    },
    modalities::EventsMode,
};

use crate::utility::{
    constants::{
        ACCOUNT_3_KEY, ARG_IS_HASH_IDENTIFIER_MODE, ARG_KEY_NAME, ARG_NFT_CONTRACT_HASH,
        CONTRACT_NAME, DEFAULT_ACCOUNT_KEY, IS_APPROVED_FOR_ALL_WASM, MINT_SESSION_WASM,
        NFT_CONTRACT_WASM, NFT_TEST_COLLECTION, TEST_PRETTY_721_META_DATA,
        TEST_PRETTY_CEP78_METADATA, TEST_PRETTY_UPDATED_721_META_DATA,
        TEST_PRETTY_UPDATED_CEP78_METADATA, TRANSFER_SESSION_WASM,
    },
    installer_request_builder::{
        InstallerRequestBuilder, MetadataMutability, NFTIdentifierMode, NFTMetadataKind,
        OwnerReverseLookupMode, OwnershipMode, TEST_CUSTOM_METADATA, TEST_CUSTOM_METADATA_SCHEMA,
        TEST_CUSTOM_UPDATED_METADATA,
    },
    support::{
        self, call_session_code_with_ret, genesis, get_dictionary_value_from_key,
        get_nft_contract_hash, get_nft_contract_hash_key, get_token_page_by_id, query_stored_value,
    },
};

// cep47 event style
#[test]
fn should_record_cep47_dictionary_style_mint_event() {
    let mut builder = genesis();

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

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

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

    let collection_name: String =
        query_stored_value(&builder, nft_contract_key, ARG_COLLECTION_NAME);

    let package = query_stored_value::<String>(
        &builder,
        nft_contract_key,
        &format!("{PREFIX_CEP78}_{collection_name}"),
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
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_total_token_supply(10u64)
        .with_events_mode(EventsMode::CEP47)
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

    let owner = Key::addressable_entity_key(
        EntityKindTag::Account,
        AddressableEntityHash::new([3u8; 32]),
    );

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => owner
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
            ARG_SOURCE_KEY => *DEFAULT_ACCOUNT_KEY,
            ARG_TARGET_KEY =>  owner,
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

    let collection_name: String =
        query_stored_value(&builder, nft_contract_key, ARG_COLLECTION_NAME);

    let package = query_stored_value::<String>(
        &builder,
        nft_contract_key,
        &format!("{PREFIX_CEP78}_{collection_name}"),
    );

    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();

    expected_event.insert(EVENT_TYPE.to_string(), "Transfer".to_string());
    expected_event.insert(PREFIX_HASH_KEY_NAME.to_string(), package);
    expected_event.insert(RECIPIENT.to_string(), owner.to_string());
    expected_event.insert(SENDER.to_string(), DEFAULT_ACCOUNT_KEY.to_string());
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

    let mut builder = genesis();

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

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

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
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
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
            &base16::encode_lower(&support::create_blake2b_hash(updated_metadata)),
        ),
    };

    assert_eq!(actual_updated_metadata, updated_metadata.to_string());

    let event = get_dictionary_value_from_key::<BTreeMap<String, String>>(
        &builder,
        &nft_contract_key,
        EVENTS,
        "1",
    );

    let collection_name: String =
        query_stored_value(&builder, nft_contract_key, ARG_COLLECTION_NAME);

    let package = query_stored_value::<String>(
        &builder,
        nft_contract_key,
        &format!("{PREFIX_CEP78}_{collection_name}"),
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
    let mut builder = genesis();

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
        get_token_page_by_id(&builder, &nft_contract_key, &DEFAULT_ACCOUNT_KEY, token_id);

    assert!(token_page[0]);

    let actual_balance_before_burn = get_dictionary_value_from_key::<u64>(
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
    get_dictionary_value_from_key::<()>(
        &builder,
        &nft_contract_key,
        BURNT_TOKENS,
        &token_id.to_string(),
    );

    // This will error if token is not registered as burnt
    let actual_balance = get_dictionary_value_from_key::<u64>(
        &builder,
        &nft_contract_key,
        TOKEN_COUNT,
        &DEFAULT_ACCOUNT_ADDR.to_string(),
    );

    let expected_balance = 0u64;
    assert_eq!(actual_balance, expected_balance);

    let event = get_dictionary_value_from_key::<BTreeMap<String, String>>(
        &builder,
        &nft_contract_key,
        EVENTS,
        "1",
    );

    let collection_name: String =
        query_stored_value(&builder, nft_contract_key, ARG_COLLECTION_NAME);

    let package = query_stored_value::<String>(
        &builder,
        nft_contract_key,
        &format!("{PREFIX_CEP78}_{collection_name}"),
    );

    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();
    expected_event.insert(EVENT_TYPE.to_string(), "Burn".to_string());
    expected_event.insert(PREFIX_HASH_KEY_NAME.to_string(), package);
    expected_event.insert(OWNER.to_string(), DEFAULT_ACCOUNT_KEY.to_string());
    expected_event.insert(TOKEN_ID.to_string(), "0".to_string());
    // Burner is owner
    expected_event.insert(BURNER.to_string(), DEFAULT_ACCOUNT_KEY.to_string());
    assert_eq!(event, expected_event);
}

#[test]
fn should_cep47_dictionary_style_approve_event_in_hash_identifier_mode() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_events_mode(EventsMode::CEP47)
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

    let spender = Key::addressable_entity_key(
        EntityKindTag::Account,
        AddressableEntityHash::new([7u8; 32]),
    );

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

    let collection_name: String =
        query_stored_value(&builder, nft_contract_key, ARG_COLLECTION_NAME);

    let package = query_stored_value::<String>(
        &builder,
        nft_contract_key,
        &format!("{PREFIX_CEP78}_{collection_name}"),
    );
    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();
    expected_event.insert(EVENT_TYPE.to_string(), "Approve".to_string());
    expected_event.insert(PREFIX_HASH_KEY_NAME.to_string(), package);
    expected_event.insert(OWNER.to_string(), DEFAULT_ACCOUNT_KEY.to_string());
    expected_event.insert(SPENDER.to_string(), spender.to_string());
    expected_event.insert(
        TOKEN_ID.to_string(),
        "69fe422f3b0d0ba4d911323451a490bdd679c437e889127700b7bf83123b2d0c".to_string(),
    );
    assert_eq!(event, expected_event);
}

#[test]
fn should_cep47_dictionary_style_approvall_for_all_event() {
    let mut builder = genesis();

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
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

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

    let operator_key = *ACCOUNT_3_KEY;

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
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
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

    let collection_name: String =
        query_stored_value(&builder, nft_contract_key, ARG_COLLECTION_NAME);

    let package = query_stored_value::<String>(
        &builder,
        nft_contract_key,
        &format!("{PREFIX_CEP78}_{collection_name}"),
    );
    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();
    expected_event.insert(EVENT_TYPE.to_string(), "ApprovalForAll".to_string());
    expected_event.insert(PREFIX_HASH_KEY_NAME.to_string(), package);
    expected_event.insert(OWNER.to_string(), DEFAULT_ACCOUNT_KEY.to_string());
    expected_event.insert(OPERATOR.to_string(), operator_key.to_string());
    assert_eq!(event, expected_event);
}

#[test]
fn should_cep47_dictionary_style_revoked_for_all_event() {
    let mut builder = genesis();

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
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

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

    let operator_key = *ACCOUNT_3_KEY;

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
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
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

    let collection_name: String =
        query_stored_value(&builder, nft_contract_key, ARG_COLLECTION_NAME);

    let package = query_stored_value::<String>(
        &builder,
        nft_contract_key,
        &format!("{PREFIX_CEP78}_{collection_name}"),
    );
    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();
    expected_event.insert(EVENT_TYPE.to_string(), "RevokedForAll".to_string());
    expected_event.insert(PREFIX_HASH_KEY_NAME.to_string(), package);
    expected_event.insert(OWNER.to_string(), DEFAULT_ACCOUNT_KEY.to_string());
    expected_event.insert(OPERATOR.to_string(), operator_key.to_string());
    assert_eq!(event, expected_event);
}

#[test]
fn should_not_have_events_dicts_in_no_events_mode() {
    let mut builder = genesis();

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

    let contract_hash = get_nft_contract_hash(&builder);

    // Check dict from EventsMode::CEP47
    let contract = builder
        .get_entity_with_named_keys_by_entity_hash(contract_hash)
        .expect("should have contract");
    let named_keys = contract.named_keys();
    let events = named_keys.get(EVENTS);
    assert_eq!(events, None);

    // Check dict from EventsMode::CES
    let events = named_keys.get(EVENTS_DICT);
    assert_eq!(events, None);
}

#[test]
#[should_panic]
fn should_not_record_events_in_no_events_mode() {
    let mut builder = genesis();

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

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

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

    // This will error if token is not registered as burnt
    let actual_balance = get_dictionary_value_from_key::<u64>(
        &builder,
        &nft_contract_key,
        TOKEN_COUNT,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    let expected_balance = 1u64;
    assert_eq!(actual_balance, expected_balance);

    // Query for the Mint event here and expect failure
    // as no events are being written to global state.
    get_dictionary_value_from_key::<BTreeMap<String, String>>(
        &builder,
        &nft_contract_key,
        EVENTS,
        "1",
    );
}
