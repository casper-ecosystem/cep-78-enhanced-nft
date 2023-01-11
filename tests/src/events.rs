use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{
    account::AccountHash, bytesrepr::ToBytes, runtime_args, system::mint, Key, RuntimeArgs,
};

use crate::utility::{
    constants::{
        ACCOUNT_USER_2, ACCOUNT_USER_3, ARG_ALL_EVENTS, ARG_GET_LATEST_ONLY,
        ARG_IS_HASH_IDENTIFIER_MODE, ARG_LAST_EVENT_ID, ARG_NFT_CONTRACT_HASH, ARG_SOURCE_KEY,
        ARG_STARTING_EVENT_ID, ARG_TARGET_KEY, ARG_TOKEN_HASH, ARG_TOKEN_ID, ARG_TOKEN_META_DATA,
        ARG_TOKEN_OWNER, ENTRY_POINT_BURN, EVENTS, EVENT_ID_TRACKER, GET_TOKEN_EVENTS_WASM,
        MINT_SESSION_WASM, NFT_CONTRACT_WASM, RECEIPT_NAME, TEST_PRETTY_CEP78_METADATA,
        TRANSFER_SESSION_WASM,
    },
    installer_request_builder::{
        InstallerRequestBuilder, MetadataMutability, NFTIdentifierMode, NFTMetadataKind,
        OwnershipMode,
    },
    support,
};

#[repr(u8)]
#[derive(PartialEq, Eq)]
pub(crate) enum TokenEvent {
    Mint = 0,
    Transfer = 1,
    Burn = 2,
    Approve = 3,
}

impl ToString for TokenEvent {
    fn to_string(&self) -> String {
        match self {
            TokenEvent::Mint => "Mint".to_string(),
            TokenEvent::Transfer => "Transfer".to_string(),
            TokenEvent::Burn => "Burn".to_string(),
            TokenEvent::Approve => "Approve".to_string(),
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

fn get_events_token_identifier_args(identifier_mode: NFTIdentifierMode) -> RuntimeArgs {
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

    let nft_reciept: String = support::query_stored_value(
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
        vec![format!("events-{}", nft_reciept)],
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
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let mut nft_transfer_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TARGET_KEY => Key::Account(AccountHash::new(ACCOUNT_USER_2)),
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
                ARG_TOKEN_HASH => base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA))
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

    let nft_reciept: String = support::query_stored_value(
        &mut builder,
        nft_contract_key,
        vec![RECEIPT_NAME.to_string()],
    );

    let actual_string_events: Vec<String> = support::query_stored_value(
        &mut builder,
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        vec![format!("events-{}", nft_reciept)],
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
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let mut nft_transfer_args_1 = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TARGET_KEY => Key::Account(AccountHash::new(ACCOUNT_USER_2)),
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

    let mut nft_transfer_args_2 = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TARGET_KEY => Key::Account(AccountHash::new(ACCOUNT_USER_3)),
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
                ARG_TOKEN_HASH => base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA))
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

#[test]
fn should_get_latest_token_event_by_token_identifier(identifier_mode: NFTIdentifierMode) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_identifier_mode(identifier_mode)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_nft_metadata_kind(NFTMetadataKind::CEP78)
        .with_ownership_mode(OwnershipMode::Transferable)
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
                ARG_TOKEN_HASH => base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA))
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
        NFTIdentifierMode::Ordinal => {
            ExecuteRequestBuilder::standard(
                *DEFAULT_ACCOUNT_ADDR,
                GET_TOKEN_EVENTS_WASM,
                runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_IS_HASH_IDENTIFIER_MODE => false,
            ARG_GET_LATEST_ONLY => true,
            ARG_TOKEN_ID => 0u64,
        }
                ,
            ).build()
        }
        NFTIdentifierMode::Hash => {
            ExecuteRequestBuilder::standard(
                *DEFAULT_ACCOUNT_ADDR,
                GET_TOKEN_EVENTS_WASM,
                runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_IS_HASH_IDENTIFIER_MODE => true,
            ARG_GET_LATEST_ONLY => true,
            ARG_TOKEN_HASH => base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA))
        }
            ).build()
        }
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
fn should_record_cep47_style_mint_event() {}
