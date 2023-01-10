
use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{bytesrepr::ToBytes, runtime_args, Key, RuntimeArgs, system::mint};
use casper_types::account::AccountHash;

use crate::utility::{
    constants::{
        ARG_ALL_EVENTS, ARG_IS_HASH_IDENTIFIER_MODE, ARG_NFT_CONTRACT_HASH, ARG_STARTING_EVENT_ID,
        ARG_TOKEN_HASH, ARG_TOKEN_ID, ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, GET_TOKEN_EVENTS_WASM,
        MINT_SESSION_WASM, NFT_CONTRACT_WASM, RECEIPT_NAME, TEST_PRETTY_CEP78_METADATA,
        ARG_TARGET_KEY, ARG_SOURCE_KEY
    },
    installer_request_builder::{
        InstallerRequestBuilder, MetadataMutability, NFTIdentifierMode, NFTMetadataKind,
        OwnershipMode,
    },
    support,
};
use crate::utility::constants::{ACCOUNT_USER_2, ENTRY_POINT_BURN, TRANSFER_SESSION_WASM};

const EVENTS: &str = "events";
const EVENT_ID_TRACKER: &str = "id_tracker";


#[repr(u8)]
#[derive(PartialEq)]
pub(crate) enum TokenEvent {
    Minted = 0,
    Transferred = 1,
    Burned = 2,
    Approved = 3,
}

impl ToString for TokenEvent {
    fn to_string(&self) -> String {
        match self {
            TokenEvent::Minted => "Minted".to_string(),
            TokenEvent::Transferred => "Transferred".to_string(),
            TokenEvent::Burned => "Burned".to_string(),
            TokenEvent::Approved => "Approved".to_string(),
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
    assert_eq!(TokenEvent::Minted as u8, latest_event);

    let nft_reciept: String = support::query_stored_value(
        &mut builder,
        nft_contract_key,
        vec![RECEIPT_NAME.to_string()],
    );

    let mut get_events_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_STARTING_EVENT_ID => 0u64,
        ARG_ALL_EVENTS => true
    };

    match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            get_events_runtime_args.insert(ARG_TOKEN_ID, 0u64)
                .expect("must insert the token id runtime argument");
            get_events_runtime_args.insert(ARG_IS_HASH_IDENTIFIER_MODE, false)
                .expect("must insert identifier mode flag argument");
        }
        NFTIdentifierMode::Hash => {
            let token_hash: String =
                base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA));
            get_events_runtime_args.insert(ARG_TOKEN_HASH, token_hash)
                .expect("must insert the token hash runtime argument");
            get_events_runtime_args.insert(ARG_IS_HASH_IDENTIFIER_MODE, true)
                .expect("must insert identifier mode flag argument");
        }
    };

    let get_events_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        GET_TOKEN_EVENTS_WASM,
        get_events_runtime_args
    ).build();

    builder.exec(get_events_call).expect_success().commit();

    let actual_string_events: Vec<String> = support::query_stored_value(
        &mut builder,
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        vec![format!("events-{}", nft_reciept)],
    );
    let expected_string_events: Vec<String> = vec![TokenEvent::Minted.to_string()];
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
    ).build();

    builder.exec(native_transfer_request).expect_success().commit();


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
            nft_transfer_args.insert(ARG_TOKEN_ID, 0u64)
                .expect("must insert the token id runtime argument");
            nft_transfer_args.insert(ARG_IS_HASH_IDENTIFIER_MODE, false)
                .expect("must insert identifier mode flag argument");
        }
        NFTIdentifierMode::Hash => {
            let token_hash: String =
                base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA));
            nft_transfer_args.insert(ARG_TOKEN_HASH, token_hash)
                .expect("must insert the token hash runtime argument");
            nft_transfer_args.insert(ARG_IS_HASH_IDENTIFIER_MODE, true)
                .expect("must insert identifier mode flag argument");
        }
    };

    let nft_transfer_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_SESSION_WASM,
        nft_transfer_args
    ).build();

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
        burn_transfer_args
    ).build();

    builder.exec(nft_burn_request).expect_success().commit();

    let mut get_event_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_STARTING_EVENT_ID => 0u64,
        ARG_ALL_EVENTS => true
    };

    match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            get_event_runtime_args.insert(ARG_TOKEN_ID, 0u64)
                .expect("must insert the token id runtime argument");
            get_event_runtime_args.insert(ARG_IS_HASH_IDENTIFIER_MODE, false)
                .expect("must insert hash identifier flag");
        }
        NFTIdentifierMode::Hash => {
            get_event_runtime_args.insert(ARG_TOKEN_HASH, base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_CEP78_METADATA)))
                .expect("must insert the token hash runtime argument");
            get_event_runtime_args.insert(ARG_IS_HASH_IDENTIFIER_MODE, true)
                .expect("must insert hash identifier flag");
        }
    }

    let get_events_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        GET_TOKEN_EVENTS_WASM,
        get_event_runtime_args
    ).build();

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
    let expected_string_events: Vec<String> = vec![TokenEvent::Minted.to_string(), TokenEvent::Transferred.to_string(), TokenEvent::Burned.to_string()];
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