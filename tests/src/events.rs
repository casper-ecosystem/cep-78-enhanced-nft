use std::collections::BTreeMap;

use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{
    account::AccountHash, bytesrepr::ToBytes, runtime_args, system::mint, Key, RuntimeArgs,
};

use crate::utility::{
    constants::{
        ACCOUNT_USER_2, ACCOUNT_USER_3, ARG_ALL_EVENTS, ARG_COLLECTION_NAME, ARG_GET_LATEST_ONLY,
        ARG_IS_HASH_IDENTIFIER_MODE, ARG_LAST_EVENT_ID, ARG_NFT_CONTRACT_HASH, ARG_OPERATOR,
        ARG_SOURCE_KEY, ARG_STARTING_EVENT_ID, ARG_TARGET_KEY, ARG_TOKEN_HASH, ARG_TOKEN_ID,
        ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, BALANCES, BURNT_TOKENS, CONTRACT_NAME,
        ENTRY_POINT_APPROVE, ENTRY_POINT_BURN, ENTRY_POINT_REGISTER_OWNER,
        ENTRY_POINT_SET_TOKEN_METADATA, EVENTS, EVENT_ID_TRACKER, GET_TOKEN_EVENTS_WASM,
        METADATA_CEP78, METADATA_CUSTOM_VALIDATED, METADATA_NFT721, METADATA_RAW,
        MINT_SESSION_WASM, NFT_CONTRACT_WASM, NFT_TEST_COLLECTION, OPERATOR, RECEIPT_NAME,
        TEST_PRETTY_721_META_DATA, TEST_PRETTY_CEP78_METADATA, TEST_PRETTY_UPDATED_721_META_DATA,
        TEST_PRETTY_UPDATED_CEP78_METADATA, TRANSFER_SESSION_WASM,
    },
    installer_request_builder::{
        EventsMode, InstallerRequestBuilder, MetadataMutability, NFTIdentifierMode,
        NFTMetadataKind, OwnerReverseLookupMode, OwnershipMode, TEST_CUSTOM_METADATA,
        TEST_CUSTOM_METADATA_SCHEMA, TEST_CUSTOM_UPDATED_METADATA,
    },
    support::{self, get_dictionary_value_from_key, get_nft_contract_hash, query_stored_value},
};

#[repr(u8)]
#[derive(PartialEq, Eq)]
pub(crate) enum TokenEvent {
    Mint = 0,
    Transfer = 1,
    Burn = 2,
    // TODO
    // Approve = 3,
}

impl ToString for TokenEvent {
    fn to_string(&self) -> String {
        match self {
            TokenEvent::Mint => "Mint".to_string(),
            TokenEvent::Transfer => "Transfer".to_string(),
            TokenEvent::Burn => "Burn".to_string(),
            // TODO
            // TokenEvent::Approve => "Approve".to_string(),
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

// TODO Check usage or destination of that function
fn _get_events_token_identifier_args(identifier_mode: NFTIdentifierMode) -> RuntimeArgs {
    match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            runtime_args! {
                ARG_TOKEN_ID => 0u64,
                ARG_IS_HASH_IDENTIFIER_MODE => false,
            }
        }
        NFTIdentifierMode::Hash => {
            runtime_args! {
                ARG_TOKEN_HASH => base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA)),
                ARG_IS_HASH_IDENTIFIER_MODE => true,
            }
        }
    }
}

fn should_get_single_events_by_identifier(identifier_mode: NFTIdentifierMode) {
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

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
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

    let event_dictionary_item_key = match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            let latest_event_id = support::get_dictionary_value_from_key::<u64>(
                &builder,
                &nft_contract_key,
                EVENT_ID_TRACKER,
                &0u64.to_string(),
            );
            assert_eq!(0u64, latest_event_id);
            get_event_item_key_from_token_index(0u64, 0u64)
        }
        NFTIdentifierMode::Hash => {
            let token_hash: String =
                base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA));
            let latest_event_id = support::get_dictionary_value_from_key::<u64>(
                &builder,
                &nft_contract_key,
                EVENT_ID_TRACKER,
                &token_hash,
            );
            assert_eq!(0u64, latest_event_id);
            get_event_item_key_from_token_hash(token_hash, 0u64)
        }
    };

    let latest_event: u8 = support::get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        EVENTS,
        &event_dictionary_item_key,
    );
    assert_eq!(TokenEvent::Mint as u8, latest_event);

    let nft_receipt: String = support::query_stored_value(
        &mut builder,
        nft_contract_key,
        vec![RECEIPT_NAME.to_string()],
    );

    let mut get_events_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_STARTING_EVENT_ID => 0u64,
        ARG_ALL_EVENTS => true,
        ARG_GET_LATEST_ONLY => false,
    };

    match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            get_events_runtime_args
                .insert(ARG_TOKEN_ID, 0u64)
                .expect("must insert the token id runtime argument");
            get_events_runtime_args
                .insert(ARG_IS_HASH_IDENTIFIER_MODE, false)
                .expect("must insert identifier mode flag argument");
        }
        NFTIdentifierMode::Hash => {
            let token_hash: String =
                base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA));
            get_events_runtime_args
                .insert(ARG_TOKEN_HASH, token_hash)
                .expect("must insert the token hash runtime argument");
            get_events_runtime_args
                .insert(ARG_IS_HASH_IDENTIFIER_MODE, true)
                .expect("must insert identifier mode flag argument");
        }
    };

    let get_events_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        GET_TOKEN_EVENTS_WASM,
        get_events_runtime_args,
    )
    .build();

    builder.exec(get_events_call).expect_success().commit();

    let actual_string_events: Vec<String> = support::query_stored_value(
        &mut builder,
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        vec![format!("{EVENTS}-{nft_receipt}")],
    );
    let expected_string_events: Vec<String> = vec![TokenEvent::Mint.to_string()];
    assert_eq!(actual_string_events, expected_string_events)
}

#[test]
fn should_get_single_event_by_token_id() {
    should_get_single_events_by_identifier(NFTIdentifierMode::Ordinal)
}

#[test]
fn should_get_token_events_by_token_hash() {
    should_get_single_events_by_identifier(NFTIdentifierMode::Hash)
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

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
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
                .insert(ARG_TOKEN_ID, 0u64)
                .expect("must insert the token id runtime argument");
            nft_transfer_args
                .insert(ARG_IS_HASH_IDENTIFIER_MODE, false)
                .expect("must insert identifier mode flag argument");
        }
        NFTIdentifierMode::Hash => {
            let token_hash: String =
                base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA));
            nft_transfer_args
                .insert(ARG_TOKEN_HASH, token_hash)
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
                ARG_TOKEN_ID => 0u64
            }
        }
        NFTIdentifierMode::Hash => {
            runtime_args! {
                        ARG_TOKEN_HASH =>
            base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA))
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

    let mut get_event_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_STARTING_EVENT_ID => 0u64,
        ARG_ALL_EVENTS => true,
        ARG_GET_LATEST_ONLY => false,
    };

    match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            get_event_runtime_args
                .insert(ARG_TOKEN_ID, 0u64)
                .expect("must insert the token id runtime argument");
            get_event_runtime_args
                .insert(ARG_IS_HASH_IDENTIFIER_MODE, false)
                .expect("must insert hash identifier flag");
        }
        NFTIdentifierMode::Hash => {
            get_event_runtime_args
                .insert(
                    ARG_TOKEN_HASH,
                    base16::encode_lower(&support::create_blake2b_hash(
                        &TEST_PRETTY_CEP78_METADATA,
                    )),
                )
                .expect("must insert the token hash runtime argument");
            get_event_runtime_args
                .insert(ARG_IS_HASH_IDENTIFIER_MODE, true)
                .expect("must insert hash identifier flag");
        }
    }

    let get_events_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        GET_TOKEN_EVENTS_WASM,
        get_event_runtime_args,
    )
    .build();

    builder.exec(get_events_request).expect_success().commit();

    let nft_receipt: String = support::query_stored_value(
        &mut builder,
        nft_contract_key,
        vec![RECEIPT_NAME.to_string()],
    );

    let actual_string_events: Vec<String> = support::query_stored_value(
        &mut builder,
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        vec![format!("{EVENTS}-{nft_receipt}")],
    );
    let expected_string_events: Vec<String> = vec![
        TokenEvent::Mint.to_string(),
        TokenEvent::Transfer.to_string(),
        TokenEvent::Burn.to_string(),
    ];
    assert_eq!(actual_string_events, expected_string_events)
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

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
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
                .insert(ARG_TOKEN_ID, 0u64)
                .expect("must insert the token id runtime argument");
            nft_transfer_args_1
                .insert(ARG_IS_HASH_IDENTIFIER_MODE, false)
                .expect("must insert identifier mode flag argument");
        }
        NFTIdentifierMode::Hash => {
            let token_hash: String =
                base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA));
            nft_transfer_args_1
                .insert(ARG_TOKEN_HASH, token_hash)
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
                .insert(ARG_TOKEN_ID, 0u64)
                .expect("must insert the token id runtime argument");
            nft_transfer_args_2
                .insert(ARG_IS_HASH_IDENTIFIER_MODE, false)
                .expect("must insert identifier mode flag argument");
        }
        NFTIdentifierMode::Hash => {
            let token_hash: String =
                base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA));
            nft_transfer_args_2
                .insert(ARG_TOKEN_HASH, token_hash)
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
                ARG_TOKEN_ID => 0u64
            }
        }
        NFTIdentifierMode::Hash => {
            runtime_args! {
                ARG_TOKEN_HASH =>
            base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA))
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

    let mut get_event_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_STARTING_EVENT_ID => 1u64,
        ARG_LAST_EVENT_ID => 2u64,
        ARG_ALL_EVENTS => false,
        ARG_GET_LATEST_ONLY => false,
    };

    match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            get_event_runtime_args
                .insert(ARG_TOKEN_ID, 0u64)
                .expect("must insert the token id runtime argument");
            get_event_runtime_args
                .insert(ARG_IS_HASH_IDENTIFIER_MODE, false)
                .expect("must insert hash identifier flag");
        }
        NFTIdentifierMode::Hash => {
            get_event_runtime_args
                .insert(
                    ARG_TOKEN_HASH,
                    base16::encode_lower(&support::create_blake2b_hash(
                        &TEST_PRETTY_CEP78_METADATA,
                    )),
                )
                .expect("must insert the token hash runtime argument");
            get_event_runtime_args
                .insert(ARG_IS_HASH_IDENTIFIER_MODE, true)
                .expect("must insert hash identifier flag");
        }
    }

    let get_events_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        GET_TOKEN_EVENTS_WASM,
        get_event_runtime_args,
    )
    .build();

    builder.exec(get_events_request).expect_success().commit();

    let nft_receipt: String = support::query_stored_value(
        &mut builder,
        nft_contract_key,
        vec![RECEIPT_NAME.to_string()],
    );

    let actual_string_events: Vec<String> = support::query_stored_value(
        &mut builder,
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        vec![format!("{EVENTS}-{nft_receipt}")],
    );
    let expected_string_events: Vec<String> = vec![
        TokenEvent::Transfer.to_string(),
        TokenEvent::Transfer.to_string(),
    ];
    assert_eq!(actual_string_events, expected_string_events)
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

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
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

    let burn_runtime_args = match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            runtime_args! {
                ARG_TOKEN_ID => 0u64
            }
        }
        NFTIdentifierMode::Hash => {
            runtime_args! {
                        ARG_TOKEN_HASH =>
            base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA))
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

    let get_latest_token_event_request = match identifier_mode {
        NFTIdentifierMode::Ordinal => ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            GET_TOKEN_EVENTS_WASM,
            runtime_args! {
                ARG_NFT_CONTRACT_HASH => nft_contract_key,
                ARG_IS_HASH_IDENTIFIER_MODE => false,
                ARG_GET_LATEST_ONLY => true,
                ARG_TOKEN_ID => 0u64,
            },
        )
        .build(),
        NFTIdentifierMode::Hash => ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            GET_TOKEN_EVENTS_WASM,
            runtime_args! {
                    ARG_NFT_CONTRACT_HASH => nft_contract_key,
                    ARG_IS_HASH_IDENTIFIER_MODE => true,
                    ARG_GET_LATEST_ONLY => true,
                    ARG_TOKEN_HASH =>
            base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA))},
        )
        .build(),
    };

    builder
        .exec(get_latest_token_event_request)
        .expect_success()
        .commit();

    let nft_receipt: String = support::query_stored_value(
        &mut builder,
        nft_contract_key,
        vec![RECEIPT_NAME.to_string()],
    );

    let actual_string_event: String = support::query_stored_value(
        &mut builder,
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        vec![format!("{EVENTS}-{nft_receipt}")],
    );

    assert_eq!(actual_string_event, TokenEvent::Burn.to_string())
}

#[test]
fn should_get_latest_token_event_by_token_id() {
    should_get_latest_token_event_by_token_identifier(NFTIdentifierMode::Ordinal)
}

#[test]
fn should_get_latest_token_even_by_token_hash() {
    should_get_latest_token_event_by_token_identifier(NFTIdentifierMode::Hash)
}

#[test]
fn should_record_cep47_style_mint_event() {
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

    let event = query_stored_value::<BTreeMap<String, String>>(
        &mut builder,
        nft_contract_key,
        vec!["latest_event".to_string()],
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
fn should_record_cep47_style_transfer_token_event_in_hash_identifier_mode() {
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

    let event = query_stored_value::<BTreeMap<String, String>>(
        &mut builder,
        nft_contract_key,
        vec!["latest_event".to_string()],
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
fn should_record_cep47_style_metadata_update_event_for_nft721_using_token_id() {
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

    let nft_contract_key: Key = support::get_nft_contract_hash(&builder).into();

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
        NFTIdentifierMode::Ordinal => support::get_dictionary_value_from_key::<String>(
            &builder,
            &nft_contract_key,
            dictionary_name,
            &0u64.to_string(),
        ),
        NFTIdentifierMode::Hash => support::get_dictionary_value_from_key(
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
        support::get_nft_contract_hash(&builder),
        ENTRY_POINT_SET_TOKEN_METADATA,
        update_metadata_runtime_args,
    )
    .build();

    builder
        .exec(update_metadata_request)
        .expect_success()
        .commit();

    let actual_updated_metadata = match identifier_mode {
        NFTIdentifierMode::Ordinal => support::get_dictionary_value_from_key::<String>(
            &builder,
            &nft_contract_key,
            dictionary_name,
            &0u64.to_string(),
        ),
        NFTIdentifierMode::Hash => support::get_dictionary_value_from_key(
            &builder,
            &nft_contract_key,
            dictionary_name,
            &base16::encode_lower(&support::create_blake2b_hash(original_metadata)),
        ),
    };

    assert_eq!(actual_updated_metadata, updated_metadata.to_string());

    let event = query_stored_value::<BTreeMap<String, String>>(
        &mut builder,
        nft_contract_key,
        vec!["latest_event".to_string()],
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
fn should_cep47_style_burn_event() {
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

    let token_page = support::get_token_page_by_id(
        &builder,
        nft_contract_key,
        &Key::Account(*DEFAULT_ACCOUNT_ADDR),
        token_id,
    );

    assert!(token_page[0]);

    let actual_balance_before_burn = support::get_dictionary_value_from_key::<u64>(
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
    let _ = support::get_dictionary_value_from_key::<()>(
        &builder,
        nft_contract_key,
        BURNT_TOKENS,
        &token_id.to_string(),
    );

    // This will error of token is not registered as
    let actual_balance = support::get_dictionary_value_from_key::<u64>(
        &builder,
        nft_contract_key,
        BALANCES,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    let expected_balance = 0u64;
    assert_eq!(actual_balance, expected_balance);

    let event = query_stored_value::<BTreeMap<String, String>>(
        &mut builder,
        *nft_contract_key,
        vec!["latest_event".to_string()],
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
fn should_cep47_style_approve_event_in_hash_identifier_mode() {
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

    let maybe_approved_operator = support::get_dictionary_value_from_key::<Option<Key>>(
        &builder,
        &nft_contract_key,
        OPERATOR,
        &token_hash,
    );

    assert_eq!(maybe_approved_operator, Some(operator));

    let event = query_stored_value::<BTreeMap<String, String>>(
        &mut builder,
        nft_contract_key,
        vec!["latest_event".to_string()],
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
            .with_events_mode(EventsMode::CEP47Dict);
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
        .with_events_mode(EventsMode::CEP47Dict)
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
        .with_events_mode(EventsMode::CEP47Dict)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = support::get_nft_contract_hash(&builder).into();

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
        NFTIdentifierMode::Ordinal => support::get_dictionary_value_from_key::<String>(
            &builder,
            &nft_contract_key,
            dictionary_name,
            &0u64.to_string(),
        ),
        NFTIdentifierMode::Hash => support::get_dictionary_value_from_key(
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
        support::get_nft_contract_hash(&builder),
        ENTRY_POINT_SET_TOKEN_METADATA,
        update_metadata_runtime_args,
    )
    .build();

    builder
        .exec(update_metadata_request)
        .expect_success()
        .commit();

    let actual_updated_metadata = match identifier_mode {
        NFTIdentifierMode::Ordinal => support::get_dictionary_value_from_key::<String>(
            &builder,
            &nft_contract_key,
            dictionary_name,
            &0u64.to_string(),
        ),
        NFTIdentifierMode::Hash => support::get_dictionary_value_from_key(
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
            .with_events_mode(EventsMode::CEP47Dict)
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

    let token_page = support::get_token_page_by_id(
        &builder,
        nft_contract_key,
        &Key::Account(*DEFAULT_ACCOUNT_ADDR),
        token_id,
    );

    assert!(token_page[0]);

    let actual_balance_before_burn = support::get_dictionary_value_from_key::<u64>(
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
    let _ = support::get_dictionary_value_from_key::<()>(
        &builder,
        nft_contract_key,
        BURNT_TOKENS,
        &token_id.to_string(),
    );

    // This will error of token is not registered as
    let actual_balance = support::get_dictionary_value_from_key::<u64>(
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
        .with_events_mode(EventsMode::CEP47Dict)
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

    let maybe_approved_operator = support::get_dictionary_value_from_key::<Option<Key>>(
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
