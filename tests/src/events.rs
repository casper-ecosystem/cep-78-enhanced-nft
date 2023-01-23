use std::collections::BTreeMap;

use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{
    account::AccountHash, bytesrepr::ToBytes, runtime_args, system::mint, CLValueError, Key,
    RuntimeArgs,
};

use crate::utility::{
    constants::{
        ACCOUNT_USER_2, ACCOUNT_USER_3, ARG_APPROVE_ALL, ARG_COLLECTION_NAME,
        ARG_IS_HASH_IDENTIFIER_MODE, ARG_NFT_CONTRACT_HASH, ARG_OPERATOR, ARG_SOURCE_KEY,
        ARG_TARGET_KEY, ARG_TOKEN_HASH, ARG_TOKEN_ID, ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER,
        BALANCES, BURNT_TOKENS, CONTRACT_NAME, ENTRY_POINT_APPROVE, ENTRY_POINT_BURN,
        ENTRY_POINT_MINT, ENTRY_POINT_REGISTER_OWNER, ENTRY_POINT_SET_APPROVE_FOR_ALL,
        ENTRY_POINT_SET_TOKEN_METADATA, EVENTS, EVENT_ID_TRACKER, METADATA_CEP78,
        METADATA_CUSTOM_VALIDATED, METADATA_NFT721, METADATA_RAW, MINT_SESSION_WASM,
        NFT_CONTRACT_WASM, NFT_TEST_COLLECTION, OPERATOR, TEST_PRETTY_721_META_DATA,
        TEST_PRETTY_CEP78_METADATA, TEST_PRETTY_UPDATED_721_META_DATA,
        TEST_PRETTY_UPDATED_CEP78_METADATA, TRANSFER_SESSION_WASM,
    },
    installer_request_builder::{
        EventsMode, InstallerRequestBuilder, MetadataMutability, NFTIdentifierMode,
        NFTMetadataKind, OwnerReverseLookupMode, OwnershipMode, TEST_CUSTOM_METADATA,
        TEST_CUSTOM_METADATA_SCHEMA, TEST_CUSTOM_UPDATED_METADATA,
    },
    support::{
        self, get_dictionary_value_from_key, get_nft_contract_hash, get_token_page_by_id,
        query_stored_value,
    },
};

use core::convert::TryFrom;

#[repr(u8)]
#[derive(PartialEq, Eq, Debug)]
pub(crate) enum TokenEvent {
    Mint = 0,
    Burn = 1,
    Approve = 2,
    Transfer = 3,
    MetadataUpdate = 4,
}

impl TryFrom<u8> for TokenEvent {
    type Error = CLValueError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TokenEvent::Mint),
            1 => Ok(TokenEvent::Burn),
            2 => Ok(TokenEvent::Approve),
            3 => Ok(TokenEvent::Transfer),
            4 => Ok(TokenEvent::MetadataUpdate),
            _ => panic!("invalid TokenEvent from u8"),
        }
    }
}

fn get_event_item_key_from_token_index(token_index: u64, event_id: u64) -> String {
    let mut preimage = Vec::new();
    let token_index_as_string = token_index.to_string();
    preimage.append(&mut token_index_as_string.to_bytes().unwrap());
    preimage.append(&mut event_id.to_bytes().unwrap());
    base16::encode_lower(&support::create_blake2b_hash(&preimage))
}

fn get_event_item_key_from_token_hash(token_hash: String, event_id: u64) -> String {
    let mut preimage = Vec::new();
    preimage.append(&mut token_hash.to_bytes().unwrap());
    preimage.append(&mut event_id.to_bytes().unwrap());
    base16::encode_lower(&support::create_blake2b_hash(&preimage))
}

fn should_get_single_event_by_token_identifier(identifier_mode: NFTIdentifierMode) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_identifier_mode(identifier_mode)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_nft_metadata_kind(NFTMetadataKind::CEP78)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_events_mode(EventsMode::CEP78)
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
            ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_index = 0u64;
    let token_hash: String =
        base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA));
    let event_id = 0u64;

    let event_dictionary_item_key = match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            let latest_event_id = get_dictionary_value_from_key::<u64>(
                &builder,
                &nft_contract_key,
                EVENT_ID_TRACKER,
                &token_index.to_string(),
            );
            assert_eq!(event_id, latest_event_id);
            get_event_item_key_from_token_index(token_index, event_id)
        }
        NFTIdentifierMode::Hash => {
            let latest_event_id = get_dictionary_value_from_key::<u64>(
                &builder,
                &nft_contract_key,
                EVENT_ID_TRACKER,
                &token_hash,
            );
            assert_eq!(event_id, latest_event_id);
            get_event_item_key_from_token_hash(token_hash.clone(), event_id)
        }
    };

    let latest_event: u8 = get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        EVENTS,
        &event_dictionary_item_key,
    );
    assert_eq!(TokenEvent::Mint as u8, latest_event);

    let event_item_key = match identifier_mode {
        NFTIdentifierMode::Ordinal => get_event_item_key_from_token_index(token_index, event_id),
        NFTIdentifierMode::Hash => get_event_item_key_from_token_hash(token_hash, event_id),
    };

    let actual_event: TokenEvent = TokenEvent::try_from(get_dictionary_value_from_key::<u8>(
        &builder,
        &nft_contract_key,
        EVENTS,
        &event_item_key,
    ))
    .unwrap();
    assert_eq!(actual_event, TokenEvent::Mint)
}

#[test]
fn should_get_single_event_by_token_id() {
    should_get_single_event_by_token_identifier(NFTIdentifierMode::Ordinal)
}

#[test]
fn should_get_token_events_by_token_hash() {
    should_get_single_event_by_token_identifier(NFTIdentifierMode::Hash)
}

fn should_get_multiple_events_by_token_identifier(identifier_mode: NFTIdentifierMode) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let native_transfer_request = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            mint::ARG_AMOUNT => 100_000_000_000_000u64,
            mint::ARG_TARGET => AccountHash::new(ACCOUNT_USER_2),
            mint::ARG_ID => Option::<u64>::None,
        },
    )
    .build();

    builder
        .exec(native_transfer_request)
        .expect_success()
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_identifier_mode(identifier_mode)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_nft_metadata_kind(NFTMetadataKind::CEP78)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_events_mode(EventsMode::CEP78)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();
    let token_index: u64 = 0u64;
    let token_hash: String =
        base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA));

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

    let token_receiver_key = Key::Account(AccountHash::new(ACCOUNT_USER_2));

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        AccountHash::new(ACCOUNT_USER_2),
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_receiver_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let mut nft_transfer_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TARGET_KEY => token_receiver_key,
        ARG_SOURCE_KEY => Key::Account(*DEFAULT_ACCOUNT_ADDR)
    };

    match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            nft_transfer_args
                .insert(ARG_TOKEN_ID, token_index)
                .expect("must insert the token id runtime argument");
            nft_transfer_args
                .insert(ARG_IS_HASH_IDENTIFIER_MODE, false)
                .expect("must insert identifier mode flag argument");
        }
        NFTIdentifierMode::Hash => {
            nft_transfer_args
                .insert(ARG_TOKEN_HASH, token_hash.clone())
                .expect("must insert the token hash runtime argument");
            nft_transfer_args
                .insert(ARG_IS_HASH_IDENTIFIER_MODE, true)
                .expect("must insert identifier mode flag argument");
        }
    };

    let nft_transfer_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_SESSION_WASM,
        nft_transfer_args,
    )
    .build();

    builder.exec(nft_transfer_request).expect_success().commit();

    let burn_transfer_args = match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            runtime_args! {
                ARG_TOKEN_ID => token_index
            }
        }
        NFTIdentifierMode::Hash => {
            runtime_args! {
                ARG_TOKEN_HASH => token_hash.clone()
            }
        }
    };

    let nft_burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        AccountHash::new(ACCOUNT_USER_2),
        nft_contract_hash,
        ENTRY_POINT_BURN,
        burn_transfer_args,
    )
    .build();

    builder.exec(nft_burn_request).expect_success().commit();

    let latest_event_id: u64 = match identifier_mode {
        NFTIdentifierMode::Ordinal => get_dictionary_value_from_key::<u64>(
            &builder,
            &nft_contract_key,
            EVENT_ID_TRACKER,
            &token_index.to_string(),
        ),
        NFTIdentifierMode::Hash => get_dictionary_value_from_key::<u64>(
            &builder,
            &nft_contract_key,
            EVENT_ID_TRACKER,
            &token_hash,
        ),
    };

    let mut actual_events = Vec::<TokenEvent>::new();

    for event_id in 0u64..=latest_event_id {
        let event_item_key = match identifier_mode {
            NFTIdentifierMode::Ordinal => {
                get_event_item_key_from_token_index(token_index, event_id)
            }
            NFTIdentifierMode::Hash => {
                get_event_item_key_from_token_hash(token_hash.clone(), event_id)
            }
        };

        let actual_event: TokenEvent = TokenEvent::try_from(get_dictionary_value_from_key::<u8>(
            &builder,
            &nft_contract_key,
            EVENTS,
            &event_item_key,
        ))
        .unwrap();
        actual_events.push(actual_event);
    }

    let expected_events: Vec<TokenEvent> =
        vec![TokenEvent::Mint, TokenEvent::Transfer, TokenEvent::Burn];

    assert_eq!(actual_events, expected_events)
}

#[test]
fn should_get_multiple_events_using_token_id() {
    should_get_multiple_events_by_token_identifier(NFTIdentifierMode::Ordinal)
}

#[test]
fn should_get_multiple_events_using_token_hash() {
    should_get_multiple_events_by_token_identifier(NFTIdentifierMode::Hash)
}

fn should_get_range_of_events_using_token_identifier(identifier_mode: NFTIdentifierMode) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let native_transfer_request_1 = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            mint::ARG_AMOUNT => 100_000_000_000_000u64,
            mint::ARG_TARGET => AccountHash::new(ACCOUNT_USER_2),
            mint::ARG_ID => Option::<u64>::None,
        },
    )
    .build();

    builder
        .exec(native_transfer_request_1)
        .expect_success()
        .commit();

    let native_transfer_request_2 = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            mint::ARG_AMOUNT => 100_000_000_000_000u64,
            mint::ARG_TARGET => AccountHash::new(ACCOUNT_USER_3),
            mint::ARG_ID => Option::<u64>::None,
        },
    )
    .build();

    builder
        .exec(native_transfer_request_2)
        .expect_success()
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_identifier_mode(identifier_mode)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_nft_metadata_kind(NFTMetadataKind::CEP78)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_events_mode(EventsMode::CEP78)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();
    let token_index: u64 = 0u64;
    let token_hash: String =
        base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA));

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

    let token_receiver_key = Key::Account(AccountHash::new(ACCOUNT_USER_2));

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

    let mut nft_transfer_args_1 = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TARGET_KEY => token_receiver_key,
        ARG_SOURCE_KEY => Key::Account(*DEFAULT_ACCOUNT_ADDR)
    };

    match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            nft_transfer_args_1
                .insert(ARG_TOKEN_ID, token_index)
                .expect("must insert the token id runtime argument");
            nft_transfer_args_1
                .insert(ARG_IS_HASH_IDENTIFIER_MODE, false)
                .expect("must insert identifier mode flag argument");
        }
        NFTIdentifierMode::Hash => {
            nft_transfer_args_1
                .insert(ARG_TOKEN_HASH, token_hash.clone())
                .expect("must insert the token hash runtime argument");
            nft_transfer_args_1
                .insert(ARG_IS_HASH_IDENTIFIER_MODE, true)
                .expect("must insert identifier mode flag argument");
        }
    };

    let nft_transfer_request_1 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_SESSION_WASM,
        nft_transfer_args_1,
    )
    .build();

    builder
        .exec(nft_transfer_request_1)
        .expect_success()
        .commit();

    let token_receiver_key = Key::Account(AccountHash::new(ACCOUNT_USER_3));

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        AccountHash::new(ACCOUNT_USER_2),
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_receiver_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let mut nft_transfer_args_2 = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TARGET_KEY => token_receiver_key,
        ARG_SOURCE_KEY => Key::Account(AccountHash::new(ACCOUNT_USER_2))
    };

    match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            nft_transfer_args_2
                .insert(ARG_TOKEN_ID, token_index)
                .expect("must insert the token id runtime argument");
            nft_transfer_args_2
                .insert(ARG_IS_HASH_IDENTIFIER_MODE, false)
                .expect("must insert identifier mode flag argument");
        }
        NFTIdentifierMode::Hash => {
            nft_transfer_args_2
                .insert(ARG_TOKEN_HASH, token_hash.clone())
                .expect("must insert the token hash runtime argument");
            nft_transfer_args_2
                .insert(ARG_IS_HASH_IDENTIFIER_MODE, true)
                .expect("must insert identifier mode flag argument");
        }
    };

    let nft_transfer_request_2 = ExecuteRequestBuilder::standard(
        AccountHash::new(ACCOUNT_USER_2),
        TRANSFER_SESSION_WASM,
        nft_transfer_args_2,
    )
    .build();

    builder
        .exec(nft_transfer_request_2)
        .expect_success()
        .commit();

    let burn_runtime_args = match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            runtime_args! {
                ARG_TOKEN_ID => token_index
            }
        }
        NFTIdentifierMode::Hash => {
            runtime_args! {
                ARG_TOKEN_HASH => token_hash.clone()
            }
        }
    };

    let nft_burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        AccountHash::new(ACCOUNT_USER_3),
        nft_contract_hash,
        ENTRY_POINT_BURN,
        burn_runtime_args,
    )
    .build();

    builder.exec(nft_burn_request).expect_success().commit();

    for event_id in 1u64..=2u64 {
        let event_item_key = match identifier_mode {
            NFTIdentifierMode::Ordinal => {
                get_event_item_key_from_token_index(token_index, event_id)
            }
            NFTIdentifierMode::Hash => {
                get_event_item_key_from_token_hash(token_hash.clone(), event_id)
            }
        };

        let actual_event: TokenEvent = TokenEvent::try_from(get_dictionary_value_from_key::<u8>(
            &builder,
            &nft_contract_key,
            EVENTS,
            &event_item_key,
        ))
        .unwrap();
        assert_eq!(actual_event, TokenEvent::Transfer)
    }
}

#[test]
fn should_get_range_of_events_by_token_id() {
    should_get_range_of_events_using_token_identifier(NFTIdentifierMode::Ordinal)
}

#[test]
fn should_get_range_of_events_by_token_hash() {
    should_get_range_of_events_using_token_identifier(NFTIdentifierMode::Hash)
}

fn should_get_latest_token_event_by_token_identifier(identifier_mode: NFTIdentifierMode) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_identifier_mode(identifier_mode)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_nft_metadata_kind(NFTMetadataKind::CEP78)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_events_mode(EventsMode::CEP78)
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
            ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA ,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_index = 0u64;
    let token_hash =
        base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA));

    let burn_runtime_args = match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            runtime_args! {
                ARG_TOKEN_ID => token_index
            }
        }
        NFTIdentifierMode::Hash => {
            runtime_args! {
                ARG_TOKEN_HASH =>token_hash.clone()
            }
        }
    };

    let nft_burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_BURN,
        burn_runtime_args,
    )
    .build();

    builder.exec(nft_burn_request).expect_success().commit();

    let event_id = 1u64;
    let event_item_key = match identifier_mode {
        NFTIdentifierMode::Ordinal => get_event_item_key_from_token_index(token_index, event_id),
        NFTIdentifierMode::Hash => get_event_item_key_from_token_hash(token_hash, event_id),
    };

    let actual_event: TokenEvent = TokenEvent::try_from(get_dictionary_value_from_key::<u8>(
        &builder,
        &nft_contract_key,
        EVENTS,
        &event_item_key,
    ))
    .unwrap();
    assert_eq!(actual_event, TokenEvent::Burn)
}

#[test]
fn should_get_latest_token_event_by_token_id() {
    should_get_latest_token_event_by_token_identifier(NFTIdentifierMode::Ordinal)
}

#[test]
fn should_get_latest_token_event_by_token_hash() {
    should_get_latest_token_event_by_token_identifier(NFTIdentifierMode::Hash)
}

fn should_get_approve_event_by_token_identifier(identifier_mode: NFTIdentifierMode) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_identifier_mode(identifier_mode)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_nft_metadata_kind(NFTMetadataKind::CEP78)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_events_mode(EventsMode::CEP78)
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
            ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA ,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_index = 0u64;
    let token_hash =
        base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA));

    let operator = Key::Account(AccountHash::new(ACCOUNT_USER_2));

    let approve_runtime_args = match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            runtime_args! {
                ARG_TOKEN_ID => token_index,
                ARG_OPERATOR => operator
            }
        }
        NFTIdentifierMode::Hash => {
            runtime_args! {
                ARG_TOKEN_HASH => token_hash.clone(),
                ARG_OPERATOR => operator
            }
        }
    };

    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        approve_runtime_args,
    )
    .build();

    builder.exec(approve_request).expect_success().commit();

    let token_identifier = if let NFTIdentifierMode::Ordinal = identifier_mode {
        token_index.to_string()
    } else {
        token_hash.clone()
    };

    let maybe_approved_operator = get_dictionary_value_from_key::<Option<Key>>(
        &builder,
        &nft_contract_key,
        OPERATOR,
        &token_identifier,
    );

    assert_eq!(maybe_approved_operator, Some(operator));

    let event_id = 1u64;
    let event_item_key = match identifier_mode {
        NFTIdentifierMode::Ordinal => get_event_item_key_from_token_index(token_index, event_id),
        NFTIdentifierMode::Hash => get_event_item_key_from_token_hash(token_hash, event_id),
    };

    let actual_event: TokenEvent = TokenEvent::try_from(get_dictionary_value_from_key::<u8>(
        &builder,
        &nft_contract_key,
        EVENTS,
        &event_item_key,
    ))
    .unwrap();
    assert_eq!(actual_event, TokenEvent::Approve)
}

#[test]
fn should_get_approve_event_by_token_id() {
    should_get_approve_event_by_token_identifier(NFTIdentifierMode::Ordinal)
}

#[test]
fn should_get_approve_event_by_token_hash() {
    should_get_approve_event_by_token_identifier(NFTIdentifierMode::Hash)
}

fn should_get_approve_event_on_approve_all_by_token_identifier(identifier_mode: NFTIdentifierMode) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_identifier_mode(identifier_mode)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_events_mode(EventsMode::CEP78)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();
    let token_owner = AccountHash::new(ACCOUNT_USER_2);
    let token_owner_key = Key::Account(token_owner);

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_owner_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let number_of_tokens = 5u64;

    for mint_id in 0u64..number_of_tokens {
        let mint_session_call = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_NFT_CONTRACT_HASH => nft_contract_key,
                ARG_TOKEN_OWNER => token_owner_key,
                ARG_TOKEN_META_DATA => mint_id.to_string(),
            },
        )
        .build();
        builder.exec(mint_session_call).expect_success().commit();
    }

    let native_transfer_request = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            mint::ARG_AMOUNT => 100_000_000_000_000u64,
            mint::ARG_TARGET => token_owner,
            mint::ARG_ID => Option::<u64>::None,
        },
    )
    .build();

    builder
        .exec(native_transfer_request)
        .expect_success()
        .commit();

    let operator = Key::Account(AccountHash::new(ACCOUNT_USER_3));

    let approve_all_request = ExecuteRequestBuilder::contract_call_by_hash(
        token_owner,
        nft_contract_hash,
        ENTRY_POINT_SET_APPROVE_FOR_ALL,
        runtime_args! {
            ARG_TOKEN_OWNER => token_owner,
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => operator
        },
    )
    .build();

    builder.exec(approve_all_request).expect_success().commit();

    let event_id = 1u64;

    for token_index in 0u64..number_of_tokens {
        let event_item_key = match identifier_mode {
            NFTIdentifierMode::Ordinal => {
                get_event_item_key_from_token_index(token_index, event_id)
            }
            NFTIdentifierMode::Hash => {
                let token_hash =
                    base16::encode_lower(&support::create_blake2b_hash(&token_index.to_string()));
                get_event_item_key_from_token_hash(token_hash, event_id)
            }
        };
        let actual_event: TokenEvent = TokenEvent::try_from(get_dictionary_value_from_key::<u8>(
            &builder,
            &nft_contract_key,
            EVENTS,
            &event_item_key,
        ))
        .unwrap();
        assert_eq!(actual_event, TokenEvent::Approve)
    }
}

#[test]
fn should_get_approve_event_on_approve_all_by_token_id() {
    should_get_approve_event_on_approve_all_by_token_identifier(NFTIdentifierMode::Ordinal)
}

#[test]
fn should_get_approve_event_on_approve_all_by_token_hash() {
    should_get_approve_event_on_approve_all_by_token_identifier(NFTIdentifierMode::Hash)
}

#[test]
fn should_record_metadata_update_event_by_token_id() {
    let identifier_mode = NFTIdentifierMode::Ordinal;
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_identifier_mode(identifier_mode)
        .with_metadata_mutability(MetadataMutability::Mutable)
        .with_nft_metadata_kind(NFTMetadataKind::CEP78)
        .with_ownership_mode(OwnershipMode::Transferable)
        // .with_ownership_mode(OwnershipMode::Minter)
        // .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_events_mode(EventsMode::CEP78)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let mint_token_request = ExecuteRequestBuilder::standard(
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

    builder.exec(mint_token_request).expect_success().commit();

    let token_id = 0u64;
    // The Mutable option cannot be used in conjunction with the Hash modality for the NFT
    // identifier so only test Ordinal as identifier
    let update_metadata_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_TOKEN_METADATA,
        runtime_args! {
            ARG_TOKEN_META_DATA => TEST_PRETTY_UPDATED_CEP78_METADATA.to_string(),
            ARG_TOKEN_ID => token_id
        },
    )
    .build();

    builder
        .exec(update_metadata_request)
        .expect_success()
        .commit();

    let event_id = 1u64;
    let dictionary_key = get_event_item_key_from_token_index(token_id, event_id);
    let actual_event: TokenEvent = TokenEvent::try_from(get_dictionary_value_from_key::<u8>(
        &builder,
        &nft_contract_key,
        EVENTS,
        &dictionary_key,
    ))
    .unwrap();
    dbg!(&actual_event);
    assert_eq!(actual_event, TokenEvent::MetadataUpdate)
}

// cep47_dictionary style
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
        "events",
        "0",
    );

    let collection_name: String = query_stored_value(
        &mut builder,
        nft_contract_key,
        vec![ARG_COLLECTION_NAME.to_string()],
    );

    let package = query_stored_value::<String>(
        &mut builder,
        nft_contract_key,
        vec![format!("cep78_{}", collection_name)],
    );

    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();
    expected_event.insert("event_type".to_string(), "Mint".to_string());
    expected_event.insert("nft_contract_package".to_string(), package);
    expected_event.insert(
        "recipient".to_string(),
        "Key::Account(58b891759929bd4ed5a9cce20b9d6e3c96a66c21386bed96040e17dd07b79fa7)"
            .to_string(),
    );
    expected_event.insert("token_id".to_string(), "0".to_string());
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
        base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_721_META_DATA));

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
        "events",
        "1",
    );

    let collection_name: String = query_stored_value(
        &mut builder,
        nft_contract_key,
        vec![ARG_COLLECTION_NAME.to_string()],
    );

    let package = query_stored_value::<String>(
        &mut builder,
        nft_contract_key,
        vec![format!("cep78_{}", collection_name)],
    );

    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();

    expected_event.insert("event_type".to_string(), "Transfer".to_string());
    expected_event.insert("nft_contract_package".to_string(), package);
    expected_event.insert(
        "recipient".to_string(),
        "Key::Account(0303030303030303030303030303030303030303030303030303030303030303)"
            .to_string(),
    );
    expected_event.insert(
        "sender".to_string(),
        "Key::Account(58b891759929bd4ed5a9cce20b9d6e3c96a66c21386bed96040e17dd07b79fa7)"
            .to_string(),
    );
    expected_event.insert(
        "token_id".to_string(),
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
        "events",
        "1",
    );

    let collection_name: String = query_stored_value(
        &mut builder,
        nft_contract_key,
        vec![ARG_COLLECTION_NAME.to_string()],
    );

    let package = query_stored_value::<String>(
        &mut builder,
        nft_contract_key,
        vec![format!("cep78_{}", collection_name)],
    );

    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();
    expected_event.insert("event_type".to_string(), "MetadataUpdate".to_string());
    expected_event.insert("nft_contract_package".to_string(), package);
    expected_event.insert("token_id".to_string(), "0".to_string());
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
        BALANCES,
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
    let _ = get_dictionary_value_from_key::<()>(
        &builder,
        nft_contract_key,
        BURNT_TOKENS,
        &token_id.to_string(),
    );

    // This will error of token is not registered as
    let actual_balance = get_dictionary_value_from_key::<u64>(
        &builder,
        nft_contract_key,
        BALANCES,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    let expected_balance = 0u64;
    assert_eq!(actual_balance, expected_balance);

    let event = get_dictionary_value_from_key::<BTreeMap<String, String>>(
        &builder,
        nft_contract_key,
        "events",
        "1",
    );

    let collection_name: String = query_stored_value(
        &mut builder,
        *nft_contract_key,
        vec![ARG_COLLECTION_NAME.to_string()],
    );

    let package = query_stored_value::<String>(
        &mut builder,
        *nft_contract_key,
        vec![format!("cep78_{}", collection_name)],
    );

    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();
    expected_event.insert("event_type".to_string(), "Burn".to_string());
    expected_event.insert("nft_contract_package".to_string(), package);
    expected_event.insert(
        "owner".to_string(),
        "Key::Account(58b891759929bd4ed5a9cce20b9d6e3c96a66c21386bed96040e17dd07b79fa7)"
            .to_string(),
    );
    expected_event.insert("token_id".to_string(), "0".to_string());
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

    let maybe_approved_operator = get_dictionary_value_from_key::<Option<Key>>(
        &builder,
        &nft_contract_key,
        OPERATOR,
        &token_hash,
    );

    assert_eq!(maybe_approved_operator, Some(operator));

    let event = get_dictionary_value_from_key::<BTreeMap<String, String>>(
        &builder,
        &nft_contract_key,
        "events",
        "1",
    );

    let collection_name: String = query_stored_value(
        &mut builder,
        nft_contract_key,
        vec![ARG_COLLECTION_NAME.to_string()],
    );

    let package = query_stored_value::<String>(
        &mut builder,
        nft_contract_key,
        vec![format!("cep78_{}", collection_name)],
    );
    let mut expected_event: BTreeMap<String, String> = BTreeMap::new();
    expected_event.insert("event_type".to_string(), "Approve".to_string());
    expected_event.insert("nft_contract_package".to_string(), package);
    expected_event.insert(
        "owner".to_string(),
        "Key::Account(58b891759929bd4ed5a9cce20b9d6e3c96a66c21386bed96040e17dd07b79fa7)"
            .to_string(),
    );
    expected_event.insert(
        "spender".to_string(),
        "Key::Account(0707070707070707070707070707070707070707070707070707070707070707)"
            .to_string(),
    );
    expected_event.insert(
        "token_id".to_string(),
        "69fe422f3b0d0ba4d911323451a490bdd679c437e889127700b7bf83123b2d0c".to_string(),
    );
    assert_eq!(event, expected_event);
}
