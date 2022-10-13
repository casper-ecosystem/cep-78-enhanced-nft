use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_ACCOUNT_PUBLIC_KEY, DEFAULT_RUN_GENESIS_REQUEST, MINIMUM_ACCOUNT_CREATION_BALANCE,
};
use casper_types::{account::AccountHash, runtime_args, system::mint, ContractHash, Key, PublicKey, RuntimeArgs, SecretKey, U512, CLValue};

use crate::utility::{
    constants::{
        ARG_NFT_CONTRACT_HASH, ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, MINT_SESSION_WASM,
        NFT_CONTRACT_WASM, NFT_TEST_COLLECTION, NFT_TEST_SYMBOL, TRANSFER_SESSION_WASM,
        ARG_IS_HASH_IDENTIFIER_MODE, ARG_SOURCE_KEY, ARG_TARGET_KEY, ARG_TOKEN_ID, ACCESS_KEY_NAME, CONTRACT_1_0_0_WASM, OWNED_TOKENS, PAGE_SIZE,
        ACCOUNT_USER_1, ACCOUNT_USER_2, ARG_TOKEN_HASH
    },
    installer_request_builder,
    installer_request_builder::{
        InstallerRequestBuilder, NFTIdentifierMode, NFTMetadataKind, OwnershipMode,
    },
    support,
};
use crate::utility::constants::{MINTED_TOKENS_AT_UPGRADE, TOKEN_HASH_TRACKER};
use crate::utility::installer_request_builder::MetadataMutability;


#[test]
fn should_safely_upgrade_in_ordinal_identifier_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, CONTRACT_1_0_0_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request)
        .expect_success()
        .commit();

    let nft_contract_hash_1_0_0 = support::get_nft_contract_hash(&builder);
    let nft_contract_key_1_0_0: Key = nft_contract_hash_1_0_0.into();

    for _ in 0..3 {
        let mint_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            MINT_SESSION_WASM,
            runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key_1_0_0,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => "",
        },
        )
            .build();

        builder
            .exec(mint_request)
            .expect_success()
            .commit();
    }

    let previous_token_representation = support::get_dictionary_value_from_key::<Vec<u64>>(
        &builder,
        &nft_contract_key_1_0_0,
        OWNED_TOKENS,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string()
    );

    assert_eq!(previous_token_representation, vec![0,1,2]);

    let maybe_access_named_key = builder
        .query(None, Key::Account(*DEFAULT_ACCOUNT_ADDR), &vec![])
        .unwrap()
        .as_account()
        .unwrap()
        .named_keys()
        .get(ACCESS_KEY_NAME)
        .is_some();

    assert!(maybe_access_named_key);

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {}
    ).build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    assert!(builder.query(None, nft_contract_key, &vec![]).unwrap()
        .as_contract()
        .unwrap()
        .named_keys()
        .get(&Key::Account(*DEFAULT_ACCOUNT_ADDR).to_formatted_string())
        .is_some());

    let page_number = (previous_token_representation[0 as usize] / PAGE_SIZE).to_string();

    let actual_page = support::get_dictionary_value_from_key::<Vec<bool>>(
        &builder,
        &nft_contract_key,
        &Key::Account(*DEFAULT_ACCOUNT_ADDR).to_formatted_string(),
        &page_number
    );

    let expected_page = {
        let mut page = vec![false; 10];
        for index in 0..3 {
            let _ = std::mem::replace(&mut page[index], true);
        }
        page
    };
    assert_eq!(actual_page, expected_page);

    let mint_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => "",
        },
    )
        .build();

    builder
        .exec(mint_request)
        .expect_success()
        .commit();

}

#[test]
fn should_safely_upgrade_in_hash_identifier_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, CONTRACT_1_0_0_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_nft_metadata_kind(NFTMetadataKind::CEP78)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .build();

    builder.exec(install_request)
        .expect_success()
        .commit();

    let nft_contract_hash_1_0_0 = support::get_nft_contract_hash(&builder);
    let nft_contract_key_1_0_0: Key = nft_contract_hash_1_0_0.into();

    let mut expected_metadata: Vec<String> = vec![];

    for i in 0..3 {
        let token_metadata = support::CEP78Metadata::with_random_checksum(
            "Some Name".to_string(),
            format!("https://www.foobar.com/{}", i),
        );

        let json_token_metadata = serde_json::to_string_pretty(&token_metadata)
            .expect("must convert to string");

        let token_hash = base16::encode_lower(&support::create_blake2b_hash(&json_token_metadata));

        expected_metadata.push(token_hash);

        let mint_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            MINT_SESSION_WASM,
            runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key_1_0_0,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => json_token_metadata,
        },
        )
            .build();

        builder
            .exec(mint_request)
            .expect_success()
            .commit();
    }

    let previous_token_representation = support::get_dictionary_value_from_key::<Vec<String>>(
        &builder,
        &nft_contract_key_1_0_0,
        OWNED_TOKENS,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string()
    );

    assert_eq!(previous_token_representation, expected_metadata);

    let maybe_access_named_key = builder
        .query(None, Key::Account(*DEFAULT_ACCOUNT_ADDR), &vec![])
        .unwrap()
        .as_account()
        .unwrap()
        .named_keys()
        .get(ACCESS_KEY_NAME)
        .is_some();

    assert!(maybe_access_named_key);

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {}
    ).build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let token_hash_tracker = builder.query(None, nft_contract_key, &vec![TOKEN_HASH_TRACKER.to_string()])
        .unwrap()
        .as_cl_value()
        .map(|tracker_cl_value| CLValue::into_t::<u64>(tracker_cl_value.clone()))
        .unwrap()
        .expect("must get u64 value");

    assert_eq!(token_hash_tracker, 0);

    let number_of_tokens_at_upgrade = builder.query(None, nft_contract_key, &vec![MINTED_TOKENS_AT_UPGRADE.to_string()])
        .unwrap()
        .as_cl_value()
        .map(|tracker_cl_value| CLValue::into_t::<u64>(tracker_cl_value.clone()))
        .unwrap()
        .expect("must get u64 value");

    assert_eq!(number_of_tokens_at_upgrade, 3);

    let token_metadata = support::CEP78Metadata::with_random_checksum(
        "Some Name".to_string(),
        format!("https://www.foobar.com/{}", 90),
    );

    let json_token_metadata = serde_json::to_string(&token_metadata)
        .expect("must convert to string");

    let contracts_named_keys = builder.query(None, nft_contract_key, &[])
        .unwrap()
        .as_contract()
        .expect("must have contract")
        .named_keys()
        .clone();

    let default_token_key = Key::Account(*DEFAULT_ACCOUNT_ADDR);

    // The dictionary should not have been created before the mint request.
    assert!(contracts_named_keys.get(&default_token_key.to_formatted_string()).is_none());

    let post_upgrade_mint_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => json_token_metadata,
        },
    )
        .build();

    builder.exec(post_upgrade_mint_request)
        .expect_success()
        .commit();

    let contracts_named_keys = builder.query(None, nft_contract_key, &[])
        .unwrap()
        .as_contract()
        .expect("must have contract")
        .named_keys()
        .clone();

    // The dictionary should have been created as part of breaking the list of owned tokens.
    assert!(contracts_named_keys.get(&default_token_key.to_formatted_string()).is_some());

    let actual_page = support::get_dictionary_value_from_key::<Vec<bool>>(
        &builder,
        &nft_contract_key,
        &Key::Account(*DEFAULT_ACCOUNT_ADDR).to_formatted_string(),
        "0"
    );

    let expected_page = {
        let mut page = vec![false; 10];
        for index in 0..4 {
            let _ = std::mem::replace(&mut page[index], true);
        }
        page
    };
    assert_eq!(actual_page, expected_page);

    let transfer_request =  ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TARGET_KEY => Key::Account(AccountHash::new(ACCOUNT_USER_1)),
            ARG_SOURCE_KEY => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_IS_HASH_IDENTIFIER_MODE => true,
            ARG_TOKEN_HASH => expected_metadata[0].clone()
        }
    ).build();

    builder.exec(transfer_request)
        .expect_success()
        .commit();

    let actual_page = support::get_dictionary_value_from_key::<Vec<bool>>(
        &builder,
        &nft_contract_key,
        &Key::Account(AccountHash::new(ACCOUNT_USER_1)).to_formatted_string(),
        "0"
    );

    assert!(actual_page[0])
}

