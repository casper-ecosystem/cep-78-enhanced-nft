use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{account::AccountHash, runtime_args, CLValue, Key, RuntimeArgs};

use crate::utility::{
    constants::{
        ACCESS_KEY_NAME, ACCOUNT_USER_1, ARG_IS_HASH_IDENTIFIER_MODE, ARG_NFT_CONTRACT_HASH,
        ARG_NFT_PACKAGE_HASH, ARG_SOURCE_KEY, ARG_TARGET_KEY, ARG_TOKEN_HASH, ARG_TOKEN_META_DATA,
        ARG_TOKEN_OWNER, BACKFILLED_TOKEN_TRACKER, CONTRACT_1_0_0_WASM, MAX_PAGE_NUMBER,
        MIGRATE_WASM, MINT_SESSION_WASM, NFT_CONTRACT_WASM, NFT_TEST_COLLECTION, NFT_TEST_SYMBOL,
        PAGE_SIZE, RECEIPT_NAME, TOKEN_COUNT_AT_UPGRADE, TRANSFER_SESSION_WASM,
    },
    installer_request_builder::{
        InstallerRequestBuilder, MetadataMutability, NFTIdentifierMode, NFTMetadataKind,
        OwnershipMode,
    },
    support,
};

const OWNED_TOKENS: &str = "owned_tokens";

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

    builder.exec(install_request).expect_success().commit();

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
        .get(ACCESS_KEY_NAME)
        .is_some();

    assert!(maybe_access_named_key);

    let package_hash = support::get_nft_contract_package_hash(&builder);

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_PACKAGE_HASH => package_hash
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let actual_page_record_width = builder
        .query(None, nft_contract_key, &[MAX_PAGE_NUMBER.to_string()])
        .expect("must have the stored value")
        .as_cl_value()
        .map(|page_cl_value| CLValue::into_t::<u64>(page_cl_value.clone()))
        .unwrap()
        .expect("must convert");

    let expected_page_record_width = 100u64 / PAGE_SIZE;

    assert_eq!(expected_page_record_width, actual_page_record_width);

    let actual_page = support::get_token_page_by_id(
        &builder,
        &nft_contract_key,
        &Key::Account(*DEFAULT_ACCOUNT_ADDR),
        0u64,
    );

    let expected_page = {
        let mut page = vec![false; 10];
        for page_entry in page.iter_mut().take(3) {
            *page_entry = true;
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

    builder.exec(mint_request).expect_success().commit();
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

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash_1_0_0 = support::get_nft_contract_hash(&builder);
    let nft_contract_key_1_0_0: Key = nft_contract_hash_1_0_0.into();

    let mut expected_metadata: Vec<String> = vec![];

    for i in 0..3 {
        let token_metadata = support::CEP78Metadata::with_random_checksum(
            "Some Name".to_string(),
            format!("https://www.foobar.com/{}", i),
        );

        let json_token_metadata =
            serde_json::to_string_pretty(&token_metadata).expect("must convert to string");

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

        builder.exec(mint_request).expect_success().commit();
    }

    let previous_token_representation = support::get_dictionary_value_from_key::<Vec<String>>(
        &builder,
        &nft_contract_key_1_0_0,
        OWNED_TOKENS,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    assert_eq!(previous_token_representation, expected_metadata);

    let maybe_access_named_key = builder
        .query(None, Key::Account(*DEFAULT_ACCOUNT_ADDR), &[])
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
        runtime_args! {
            ARG_NFT_PACKAGE_HASH => support::get_nft_contract_package_hash(&builder),
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let token_hash_tracker = support::get_stored_value_from_global_state::<u64>(
        &builder,
        nft_contract_key,
        vec![BACKFILLED_TOKEN_TRACKER.to_string()],
    )
    .expect("must get u64 value");

    assert_eq!(token_hash_tracker, 0);

    let number_of_tokens_at_upgrade = support::get_stored_value_from_global_state::<u64>(
        &builder,
        nft_contract_key,
        vec![TOKEN_COUNT_AT_UPGRADE.to_string()],
    )
    .expect("must get u64 value");

    assert_eq!(number_of_tokens_at_upgrade, 3);

    let token_metadata = support::CEP78Metadata::with_random_checksum(
        "Some Name".to_string(),
        format!("https://www.foobar.com/{}", 90),
    );

    let json_token_metadata =
        serde_json::to_string(&token_metadata).expect("must convert to string");

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

    builder
        .exec(post_upgrade_mint_request)
        .expect_success()
        .commit();

    let actual_page = support::get_token_page_by_hash(
        &builder,
        &nft_contract_key,
        &Key::Account(*DEFAULT_ACCOUNT_ADDR),
        expected_metadata[0].clone(),
    );

    let expected_page = {
        let mut page = vec![false; 10];
        for page_ownership in page.iter_mut().take(4) {
            *page_ownership = true;
        }
        page
    };
    assert_eq!(actual_page, expected_page);

    let transfer_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TARGET_KEY => Key::Account(AccountHash::new(ACCOUNT_USER_1)),
            ARG_SOURCE_KEY => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_IS_HASH_IDENTIFIER_MODE => true,
            ARG_TOKEN_HASH => expected_metadata[0].clone()
        },
    )
    .build();

    builder.exec(transfer_request).expect_success().commit();

    let actual_page = support::get_token_page_by_hash(
        &builder,
        &nft_contract_key,
        &Key::Account(AccountHash::new(ACCOUNT_USER_1)),
        expected_metadata[0].clone(),
    );

    assert!(actual_page[0])
}

#[test]
fn should_update_receipts_post_upgrade() {
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

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash_1_0_0 = support::get_nft_contract_hash(&builder);
    let nft_contract_key_1_0_0: Key = nft_contract_hash_1_0_0.into();

    for _ in 0..20 {
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

        builder.exec(mint_request).expect_success().commit();
    }

    let nft_contract_package_hash = support::get_nft_contract_package_hash(&builder);

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_PACKAGE_HASH => nft_contract_package_hash
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_package_key: Key = nft_contract_package_hash.into();

    let migrate_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MIGRATE_WASM,
        runtime_args! {
            ARG_NFT_PACKAGE_HASH => nft_package_key
        },
    )
    .build();

    builder.exec(migrate_request).expect_success().commit();

    let nft_contract_key: Key = support::get_nft_contract_hash(&builder).into();

    let nft_receipt: String = support::get_stored_value_from_global_state(
        &builder,
        nft_contract_key,
        vec![RECEIPT_NAME.to_string()],
    )
    .expect("must have receipt");

    let default_account = builder
        .query(None, Key::Account(*DEFAULT_ACCOUNT_ADDR), &[])
        .unwrap()
        .as_account()
        .unwrap()
        .clone();

    let receipt_page_0 = *default_account
        .named_keys()
        .get(&format!("{}-m-{}-p-{}", nft_receipt, PAGE_SIZE, 0))
        .expect("must have page 0 receipt");

    let expected_page = vec![true; PAGE_SIZE as usize];

    let actual_page_0 = support::get_stored_value_from_global_state::<Vec<bool>>(&builder, receipt_page_0, vec![])
        .expect("must get actual page");

    assert_eq!(expected_page, actual_page_0);

    let receipt_page_1 = *default_account
        .named_keys()
        .get(&format!("{}-m-{}-p-{}", nft_receipt, PAGE_SIZE, 1))
        .expect("must have page 0 receipt");

    let actual_page_1 = support::get_stored_value_from_global_state::<Vec<bool>>(&builder, receipt_page_1, vec![])
        .expect("must get actual page");

    assert_eq!(expected_page, actual_page_1);
}
