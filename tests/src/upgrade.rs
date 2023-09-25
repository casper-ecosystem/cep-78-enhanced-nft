use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    PRODUCTION_RUN_GENESIS_REQUEST,
};

use casper_types::{account::AccountHash, runtime_args, CLValue, ContractHash, Key, RuntimeArgs};
use contract::{
    constants::{
        ACCESS_KEY_NAME_1_0_0, ACL_PACKAGE_MODE, ARG_ACCESS_KEY_NAME_1_0_0, ARG_ACL_PACKAGE_MODE,
        ARG_COLLECTION_NAME, ARG_EVENTS_MODE, ARG_HASH_KEY_NAME_1_0_0, ARG_NAMED_KEY_CONVENTION,
        ARG_OPERATOR_BURN_MODE, ARG_PACKAGE_OPERATOR_MODE, ARG_SOURCE_KEY, ARG_TARGET_KEY,
        ARG_TOKEN_HASH, ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, ARG_TOTAL_TOKEN_SUPPLY,
        ENTRY_POINT_MINT, ENTRY_POINT_REGISTER_OWNER, NUMBER_OF_MINTED_TOKENS, OPERATOR_BURN_MODE,
        PACKAGE_OPERATOR_MODE, PAGE_LIMIT, PREFIX_ACCESS_KEY_NAME, PREFIX_HASH_KEY_NAME,
        RECEIPT_NAME, UNMATCHED_HASH_COUNT,
    },
    events::events_ces::Migration,
    modalities::EventsMode,
};

use crate::utility::{
    constants::{
        ACCOUNT_USER_1, ARG_IS_HASH_IDENTIFIER_MODE, ARG_NFT_CONTRACT_HASH,
        ARG_NFT_CONTRACT_PACKAGE_HASH, CONTRACT_1_0_0_WASM, CONTRACT_1_1_0_WASM,
        CONTRACT_1_2_0_WASM, CONTRACT_1_3_0_WASM, CONTRACT_1_4_0_WASM, MANGLE_NAMED_KEYS,
        MINT_1_0_0_WASM, MINT_SESSION_WASM, NFT_CONTRACT_WASM, NFT_TEST_COLLECTION,
        NFT_TEST_SYMBOL, PAGE_SIZE, TRANSFER_SESSION_WASM, UPDATED_RECEIPTS_WASM,
    },
    installer_request_builder::{
        InstallerRequestBuilder, MetadataMutability, NFTIdentifierMode, NFTMetadataKind,
        NamedKeyConventionMode, OwnerReverseLookupMode, OwnershipMode,
    },
    support::{self},
};

const OWNED_TOKENS: &str = "owned_tokens";
const MANGLED_ACCESS_KEY_NAME: &str = "mangled_access_key";
const MANGLED_HASH_KEY_NAME: &str = "mangled_hash_key";

#[test]
fn should_safely_upgrade_in_ordinal_identifier_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

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
            ARG_EVENTS_MODE => EventsMode::CES as u8
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_contract_key: Key = support::get_nft_contract_hash(&builder).into();

    let actual_page_record_width = builder
        .query(None, nft_contract_key, &[PAGE_LIMIT.to_string()])
        .expect("must have the stored value")
        .as_cl_value()
        .map(|page_cl_value| CLValue::into_t::<u64>(page_cl_value.clone()))
        .unwrap()
        .expect("must convert");

    let expected_page_record_width = 1000u64 / PAGE_SIZE;

    assert_eq!(expected_page_record_width, actual_page_record_width);

    let actual_page = support::get_token_page_by_id(
        &builder,
        &nft_contract_key,
        &Key::Account(*DEFAULT_ACCOUNT_ADDR),
        0u64,
    );

    let expected_page = {
        let mut page = vec![false; PAGE_SIZE as usize];
        for page_entry in page.iter_mut().take(number_of_tokens_pre_migration) {
            *page_entry = true;
        }
        page
    };
    assert_eq!(actual_page, expected_page);

    // Expect Migration event.
    let expected_event = Migration::new();
    let actual_event: Migration = support::get_event(&builder, &nft_contract_key, 0).unwrap();
    assert_eq!(actual_event, expected_event, "Expected Migration event.");

    let mint_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => "",
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_request).expect_success().commit();
}

#[test]
fn should_safely_upgrade_in_hash_identifier_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

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

    let nft_contract_hash_1_0_0 = support::get_nft_contract_hash_1_0_0(&builder);
    let nft_contract_key_1_0_0: Key = nft_contract_hash_1_0_0.into();

    let mut expected_metadata: Vec<String> = vec![];

    let number_of_tokens_pre_migration = 3usize;

    // Build of prestate before migration.
    for i in 0..number_of_tokens_pre_migration {
        let token_metadata = support::CEP78Metadata::with_random_checksum(
            "Some Name".to_string(),
            format!("https://www.foobar.com/{i}"),
        );

        let json_token_metadata =
            serde_json::to_string_pretty(&token_metadata).expect("must convert to string");

        let token_hash = base16::encode_lower(&support::create_blake2b_hash(&json_token_metadata));

        expected_metadata.push(token_hash);

        let mint_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            MINT_1_0_0_WASM,
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
        .get(ACCESS_KEY_NAME_1_0_0)
        .is_some();

    assert!(maybe_access_named_key);

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => support::get_nft_contract_package_hash(&builder),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_TOTAL_TOKEN_SUPPLY => 10u64
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let number_of_tokens_at_upgrade = support::get_stored_value_from_global_state::<u64>(
        &builder,
        nft_contract_key,
        vec![UNMATCHED_HASH_COUNT.to_string()],
    )
    .expect("must get u64 value");

    assert_eq!(number_of_tokens_at_upgrade, 3);

    let total_token_supply_post_upgrade = support::get_stored_value_from_global_state::<u64>(
        &builder,
        nft_contract_key,
        vec![ARG_TOTAL_TOKEN_SUPPLY.to_string()],
    )
    .expect("must get u64 value");

    assert_eq!(total_token_supply_post_upgrade, 10);

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
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
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
        let mut page = vec![false; PAGE_SIZE as usize];
        for page_ownership in page.iter_mut().take(4) {
            *page_ownership = true;
        }
        page
    };
    assert_eq!(actual_page, expected_page);

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => Key::Account(AccountHash::new(ACCOUNT_USER_1))
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let transfer_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_SOURCE_KEY => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TARGET_KEY => Key::Account(AccountHash::new(ACCOUNT_USER_1)),
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

    // Because token hashes are backfilled, during the migration the
    // bits of the page are filled from right to left as the address
    // counts down instead of up. Thus as the `expected_metadata[0]`
    // represents the first hash to be retroactively filled in reverse
    // the address of the token hash is [2] instead of [0]
    assert!(actual_page[2])
}

#[test]
fn should_update_receipts_post_upgrade_paged() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, CONTRACT_1_0_0_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash_1_0_0 = support::get_nft_contract_hash_1_0_0(&builder);
    let nft_contract_key_1_0_0: Key = nft_contract_hash_1_0_0.into();

    let number_of_tokens_pre_migration = 20usize;

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

    let contract_package_hash = support::get_nft_contract_package_hash(&builder);

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let package_hash_key: Key = contract_package_hash.into();

    let migrate_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        UPDATED_RECEIPTS_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_PACKAGE_HASH => package_hash_key
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
        .get(&support::get_receipt_name(nft_receipt, 0))
        .expect("must have page 0 receipt");

    let actual_page_0 =
        support::get_stored_value_from_global_state::<Vec<bool>>(&builder, receipt_page_0, vec![])
            .expect("must get actual page");

    for bit in actual_page_0.iter().take(number_of_tokens_pre_migration) {
        assert!(*bit)
    }
}

#[test]
fn should_not_be_able_to_reinvoke_migrate_entrypoint() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, CONTRACT_1_0_0_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let upgrade_to_1_1_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_1_1_0_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => support::get_nft_contract_package_hash(&builder),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
        },
    )
    .build();

    builder
        .exec(upgrade_to_1_1_request)
        .expect_success()
        .commit();

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_1_2_0_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_PACKAGE_HASH => support::get_nft_contract_package_hash(&builder),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_EVENTS_MODE => EventsMode::CES as u8
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    // Once the new contract version has been added to the package
    // calling the updated_receipts entrypoint should cause an error to be returned.
    let incorrect_upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_1_2_0_WASM,
        runtime_args! {
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_EVENTS_MODE => EventsMode::CES as u8
        },
    )
    .build();

    builder.exec(incorrect_upgrade_request).expect_failure();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(error, 126u16, "must have previously migrated error");
}

#[test]
fn should_not_migrate_contracts_with_zero_token_issuance() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, CONTRACT_1_0_0_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(0u64)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_PACKAGE_HASH => support::get_nft_contract_package_hash(&builder),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
        },
    )
    .build();
    builder.exec(upgrade_request).expect_failure();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(error, 122u16, "cannot upgrade when issuance is 0");
}

#[test]
fn should_upgrade_with_custom_named_keys() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

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

    let mangle_named_keys_request =
        ExecuteRequestBuilder::standard(*DEFAULT_ACCOUNT_ADDR, MANGLE_NAMED_KEYS, runtime_args! {})
            .build();

    builder
        .exec(mangle_named_keys_request)
        .expect_success()
        .commit();

    let maybe_access_named_key = builder
        .query(None, Key::Account(*DEFAULT_ACCOUNT_ADDR), &[])
        .unwrap()
        .as_account()
        .unwrap()
        .named_keys()
        .get(ACCESS_KEY_NAME_1_0_0)
        .is_none();

    assert!(maybe_access_named_key);

    let improper_upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
        },
    )
    .build();

    builder.exec(improper_upgrade_request).expect_failure();

    let proper_upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Custom as u8,
            ARG_ACCESS_KEY_NAME_1_0_0 => MANGLED_ACCESS_KEY_NAME.to_string(),
            ARG_HASH_KEY_NAME_1_0_0 => MANGLED_HASH_KEY_NAME.to_string(),
        },
    )
    .build();

    builder
        .exec(proper_upgrade_request)
        .expect_success()
        .commit();
}

#[test]
fn should_not_upgrade_with_larger_total_token_supply() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

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

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_PACKAGE_HASH => support::get_nft_contract_package_hash(&builder),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_TOTAL_TOKEN_SUPPLY => 1000u64
        },
    )
    .build();

    builder.exec(upgrade_request).expect_failure();
    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(
        error,
        150u16,
        "cannot upgrade when new total token supply is larger than pre-migration one",
    );
}

fn should_safely_upgrade_from_old_version_to_new_version_with_reporting_mode(
    old_version: &str,
    new_version: &str,
    reporting_mode: OwnerReverseLookupMode,
    expected_total_token_supply_post_upgrade: u64,
) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, old_version)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_reporting_mode(reporting_mode)
        .with_metadata_mutability(MetadataMutability::Mutable)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .with_events_mode(EventsMode::CES)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash: ContractHash = support::get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let number_of_tokens_pre_migration = 3usize;

    // Build of prestate before migration.
    for _i in 0..number_of_tokens_pre_migration {
        let register_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash,
            ENTRY_POINT_REGISTER_OWNER,
            runtime_args! {
                ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            },
        )
        .build();

        builder.exec(register_request).expect_success().commit();
        let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
                ARG_TOKEN_META_DATA => "",
            },
        )
        .build();

        builder.exec(mint_request).expect_success().commit();
    }

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        new_version,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Custom as u8,
            ARG_ACCESS_KEY_NAME_1_0_0 => format!("{PREFIX_ACCESS_KEY_NAME}_{NFT_TEST_COLLECTION}"),
            ARG_HASH_KEY_NAME_1_0_0 => format!("{PREFIX_HASH_KEY_NAME}_{NFT_TEST_COLLECTION}"),
            ARG_TOTAL_TOKEN_SUPPLY => 10u64
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_contract_key: Key = support::get_nft_contract_hash(&builder).into();

    let number_of_tokens_at_upgrade = support::get_stored_value_from_global_state::<u64>(
        &builder,
        nft_contract_key,
        vec![NUMBER_OF_MINTED_TOKENS.to_string()],
    )
    .expect("must get u64 value");

    assert_eq!(number_of_tokens_at_upgrade, 3);

    let total_token_supply_post_upgrade = support::get_stored_value_from_global_state::<u64>(
        &builder,
        nft_contract_key,
        vec![ARG_TOTAL_TOKEN_SUPPLY.to_string()],
    )
    .expect("must get u64 value");

    assert_eq!(
        total_token_supply_post_upgrade,
        expected_total_token_supply_post_upgrade
    );

    // Expect No Migration event after 3 Mint events.
    let seed_uref = *builder
        .query(None, nft_contract_key, &[])
        .expect("must have nft contract")
        .as_contract()
        .expect("must convert contract")
        .named_keys()
        .get(casper_event_standard::EVENTS_DICT)
        .expect("must have key")
        .as_uref()
        .expect("must convert to seed uref");

    builder
        .query_dictionary_item(None, seed_uref, "3")
        .expect_err("should not have dictionary value for a third migration event");
}

#[test]
fn should_safely_upgrade_with_acl_package_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

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

    let is_acl_packge_mode = builder
        .query(None, nft_contract_key_1_0_0, &[])
        .expect("must have nft contract")
        .as_contract()
        .expect("must convert contract")
        .named_keys()
        .contains_key(ACL_PACKAGE_MODE);

    assert!(!is_acl_packge_mode);

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_hash_1_0_0,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_EVENTS_MODE => EventsMode::CES as u8,
            ARG_ACL_PACKAGE_MODE => true,
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let is_acl_packge_mode: bool = support::query_stored_value(
        &builder,
        nft_contract_key,
        vec![ACL_PACKAGE_MODE.to_string()],
    );

    assert!(is_acl_packge_mode);

    // Expect Migration event.
    let expected_event = Migration::new();
    let actual_event: Migration = support::get_event(&builder, &nft_contract_key, 0).unwrap();
    assert_eq!(actual_event, expected_event, "Expected Migration event.");
}

#[test]
fn should_safely_upgrade_with_package_operator_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

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

    let is_package_operator_mode = builder
        .query(None, nft_contract_key_1_0_0, &[])
        .expect("must have nft contract")
        .as_contract()
        .expect("must convert contract")
        .named_keys()
        .contains_key(PACKAGE_OPERATOR_MODE);

    assert!(!is_package_operator_mode);

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_hash_1_0_0,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_EVENTS_MODE => EventsMode::CES as u8,
            ARG_PACKAGE_OPERATOR_MODE => true,
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let is_package_operator_mode: bool = support::query_stored_value(
        &builder,
        nft_contract_key,
        vec![PACKAGE_OPERATOR_MODE.to_string()],
    );

    assert!(is_package_operator_mode);

    // Expect Migration event.
    let expected_event = Migration::new();
    let actual_event: Migration = support::get_event(&builder, &nft_contract_key, 0).unwrap();
    assert_eq!(actual_event, expected_event, "Expected Migration event.");
}

#[test]
fn should_safely_upgrade_with_operator_burn_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

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

    let is_operator_burn_mode = builder
        .query(None, nft_contract_key_1_0_0, &[])
        .expect("must have nft contract")
        .as_contract()
        .expect("must convert contract")
        .named_keys()
        .contains_key(OPERATOR_BURN_MODE);

    assert!(!is_operator_burn_mode);

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_hash_1_0_0,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_EVENTS_MODE => EventsMode::CES as u8,
            ARG_OPERATOR_BURN_MODE => true,
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let is_operator_burn_mode: bool = support::query_stored_value(
        &builder,
        nft_contract_key,
        vec![OPERATOR_BURN_MODE.to_string()],
    );

    assert!(is_operator_burn_mode);

    // Expect Migration event.
    let expected_event = Migration::new();
    let actual_event: Migration = support::get_event(&builder, &nft_contract_key, 0).unwrap();
    assert_eq!(actual_event, expected_event, "Expected Migration event.");
}

#[test]
fn should_safely_upgrade_from_1_2_0_to_1_3_0() {
    //* starting total_token_supply 100u64
    let expected_total_token_supply_post_upgrade = 10;
    should_safely_upgrade_from_old_version_to_new_version_with_reporting_mode(
        CONTRACT_1_2_0_WASM,
        CONTRACT_1_3_0_WASM,
        OwnerReverseLookupMode::NoLookUp,
        expected_total_token_supply_post_upgrade,
    );
    let expected_total_token_supply_post_upgrade = 100;
    should_safely_upgrade_from_old_version_to_new_version_with_reporting_mode(
        CONTRACT_1_2_0_WASM,
        CONTRACT_1_3_0_WASM,
        OwnerReverseLookupMode::Complete,
        expected_total_token_supply_post_upgrade,
    );
}

#[test]
fn should_safely_upgrade_from_1_3_0_to_1_4_0() {
    //* starting total_token_supply 100u64
    let expected_total_token_supply_post_upgrade = 10;
    should_safely_upgrade_from_old_version_to_new_version_with_reporting_mode(
        CONTRACT_1_3_0_WASM,
        CONTRACT_1_4_0_WASM,
        OwnerReverseLookupMode::NoLookUp,
        expected_total_token_supply_post_upgrade,
    );
    let expected_total_token_supply_post_upgrade = 100;
    should_safely_upgrade_from_old_version_to_new_version_with_reporting_mode(
        CONTRACT_1_3_0_WASM,
        CONTRACT_1_4_0_WASM,
        OwnerReverseLookupMode::Complete,
        expected_total_token_supply_post_upgrade,
    );
}

#[test]
fn should_safely_upgrade_from_1_4_0_to_current_version() {
    //* starting total_token_supply 100u64
    let expected_total_token_supply_post_upgrade = 10;
    should_safely_upgrade_from_old_version_to_new_version_with_reporting_mode(
        CONTRACT_1_4_0_WASM,
        NFT_CONTRACT_WASM,
        OwnerReverseLookupMode::NoLookUp,
        expected_total_token_supply_post_upgrade,
    );
    let expected_total_token_supply_post_upgrade = 100;
    should_safely_upgrade_from_old_version_to_new_version_with_reporting_mode(
        CONTRACT_1_4_0_WASM,
        NFT_CONTRACT_WASM,
        OwnerReverseLookupMode::Complete,
        expected_total_token_supply_post_upgrade,
    );
}

#[test]
fn should_safely_upgrade_from_1_0_0_to_1_2_0_to_current_version() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, CONTRACT_1_0_0_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_metadata_mutability(MetadataMutability::Mutable)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash_1_0_0 = support::get_nft_contract_hash_1_0_0(&builder);
    let nft_contract_key_1_0_0: Key = nft_contract_hash_1_0_0.into();

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_1_2_0_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key_1_0_0,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_EVENTS_MODE => EventsMode::CES as u8,
            ARG_TOTAL_TOKEN_SUPPLY => 50u64
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let total_token_supply_post_upgrade = support::get_stored_value_from_global_state::<u64>(
        &builder,
        nft_contract_key_1_0_0,
        vec![ARG_TOTAL_TOKEN_SUPPLY.to_string()],
    )
    .expect("must get u64 value");

    assert_eq!(total_token_supply_post_upgrade, 50u64);

    let nft_contract_hash_1_2_0: ContractHash = support::get_nft_contract_hash(&builder);
    let nft_contract_key_1_2_0: Key = nft_contract_hash_1_2_0.into();

    let number_of_tokens_pre_migration = 3usize;

    // Build of prestate before migration.
    for _i in 0..number_of_tokens_pre_migration {
        let mint_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            MINT_SESSION_WASM,
            runtime_args! {
                ARG_NFT_CONTRACT_HASH => nft_contract_key_1_2_0,
                ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
                ARG_TOKEN_META_DATA => "",
                ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
            },
        )
        .build();

        builder.exec(mint_request).expect_success().commit();
    }

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key_1_2_0,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Custom as u8,
            ARG_ACCESS_KEY_NAME_1_0_0 => format!("{PREFIX_ACCESS_KEY_NAME}_{NFT_TEST_COLLECTION}"),
            ARG_HASH_KEY_NAME_1_0_0 => format!("{PREFIX_HASH_KEY_NAME}_{NFT_TEST_COLLECTION}"),
            ARG_TOTAL_TOKEN_SUPPLY => 10u64,
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let number_of_tokens_at_upgrade = support::get_stored_value_from_global_state::<u64>(
        &builder,
        nft_contract_key,
        vec![NUMBER_OF_MINTED_TOKENS.to_string()],
    )
    .expect("must get u64 value");

    assert_eq!(number_of_tokens_at_upgrade, 3);

    let total_token_supply_post_upgrade = support::get_stored_value_from_global_state::<u64>(
        &builder,
        nft_contract_key,
        vec![ARG_TOTAL_TOKEN_SUPPLY.to_string()],
    )
    .expect("must get u64 value");

    assert_eq!(total_token_supply_post_upgrade, 50u64);

    // Expect Migration event.
    let expected_event = Migration::new();
    let actual_event: Migration = support::get_event(&builder, &nft_contract_key, 0).unwrap();
    assert_eq!(actual_event, expected_event, "Expected Migration event.");
}

#[test]
fn should_safely_upgrade_from_1_0_0_to_1_3_0_to_current_version() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, CONTRACT_1_0_0_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_metadata_mutability(MetadataMutability::Mutable)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash_1_0_0 = support::get_nft_contract_hash_1_0_0(&builder);
    let nft_contract_key_1_0_0: Key = nft_contract_hash_1_0_0.into();

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_1_3_0_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key_1_0_0,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_EVENTS_MODE => EventsMode::CES as u8,
            ARG_TOTAL_TOKEN_SUPPLY => 50u64
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let total_token_supply_post_upgrade = support::get_stored_value_from_global_state::<u64>(
        &builder,
        nft_contract_key_1_0_0,
        vec![ARG_TOTAL_TOKEN_SUPPLY.to_string()],
    )
    .expect("must get u64 value");

    assert_eq!(total_token_supply_post_upgrade, 50u64);

    let nft_contract_hash_1_3_0: ContractHash = support::get_nft_contract_hash(&builder);
    let nft_contract_key_1_3_0: Key = nft_contract_hash_1_3_0.into();

    let number_of_tokens_pre_migration = 3usize;

    // Build of prestate before migration.
    for _i in 0..number_of_tokens_pre_migration {
        let mint_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            MINT_SESSION_WASM,
            runtime_args! {
                ARG_NFT_CONTRACT_HASH => nft_contract_key_1_3_0,
                ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
                ARG_TOKEN_META_DATA => "",
                ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
            },
        )
        .build();

        builder.exec(mint_request).expect_success().commit();
    }

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key_1_3_0,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Custom as u8,
            ARG_ACCESS_KEY_NAME_1_0_0 => format!("{PREFIX_ACCESS_KEY_NAME}_{NFT_TEST_COLLECTION}"),
            ARG_HASH_KEY_NAME_1_0_0 => format!("{PREFIX_HASH_KEY_NAME}_{NFT_TEST_COLLECTION}"),
            ARG_TOTAL_TOKEN_SUPPLY => 10u64,
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let number_of_tokens_at_upgrade = support::get_stored_value_from_global_state::<u64>(
        &builder,
        nft_contract_key,
        vec![NUMBER_OF_MINTED_TOKENS.to_string()],
    )
    .expect("must get u64 value");

    assert_eq!(number_of_tokens_at_upgrade, 3);

    let total_token_supply_post_upgrade = support::get_stored_value_from_global_state::<u64>(
        &builder,
        nft_contract_key,
        vec![ARG_TOTAL_TOKEN_SUPPLY.to_string()],
    )
    .expect("must get u64 value");

    assert_eq!(total_token_supply_post_upgrade, 50u64);

    // Expect Migration event.
    let expected_event = Migration::new();
    let actual_event: Migration = support::get_event(&builder, &nft_contract_key, 0).unwrap();
    assert_eq!(actual_event, expected_event, "Expected Migration event.");
}

#[test]
fn should_safely_upgrade_from_1_0_0_to_current_version() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, CONTRACT_1_0_0_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_metadata_mutability(MetadataMutability::Mutable)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash_1_0_0 = support::get_nft_contract_hash_1_0_0(&builder);

    let number_of_tokens_pre_migration = 3usize;

    // Build of prestate before migration.
    for _i in 0..number_of_tokens_pre_migration {
        let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash_1_0_0,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
                ARG_TOKEN_META_DATA => "",
            },
        )
        .build();

        builder.exec(mint_request).expect_success().commit();
    }

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => support::get_nft_contract_package_hash(&builder),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_EVENTS_MODE => EventsMode::CES as u8,
            ARG_TOTAL_TOKEN_SUPPLY => 10u64,
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_contract_key: Key = support::get_nft_contract_hash(&builder).into();

    let number_of_tokens_at_upgrade = support::get_stored_value_from_global_state::<u64>(
        &builder,
        nft_contract_key,
        vec![NUMBER_OF_MINTED_TOKENS.to_string()],
    )
    .expect("must get u64 value");

    assert_eq!(number_of_tokens_at_upgrade, 3);

    let total_token_supply_post_upgrade = support::get_stored_value_from_global_state::<u64>(
        &builder,
        nft_contract_key,
        vec![ARG_TOTAL_TOKEN_SUPPLY.to_string()],
    )
    .expect("must get u64 value");

    assert_eq!(total_token_supply_post_upgrade, 10u64);

    let actual_page_record_width = builder
        .query(None, nft_contract_key, &[PAGE_LIMIT.to_string()])
        .expect("must have the stored value")
        .as_cl_value()
        .map(|page_cl_value| CLValue::into_t::<u64>(page_cl_value.clone()))
        .unwrap()
        .expect("must convert");

    let expected_page_record_width = 1000u64 / PAGE_SIZE;

    assert_eq!(expected_page_record_width, actual_page_record_width);

    let actual_page = support::get_token_page_by_id(
        &builder,
        &nft_contract_key,
        &Key::Account(*DEFAULT_ACCOUNT_ADDR),
        0u64,
    );

    let expected_page = {
        let mut page = vec![false; PAGE_SIZE as usize];
        for page_entry in page.iter_mut().take(number_of_tokens_pre_migration) {
            *page_entry = true;
        }
        page
    };

    assert_eq!(actual_page, expected_page);

    // Expect Migration event.
    let expected_event = Migration::new();
    let actual_event: Migration = support::get_event(&builder, &nft_contract_key, 0).unwrap();
    assert_eq!(actual_event, expected_event, "Expected Migration event.");
}
