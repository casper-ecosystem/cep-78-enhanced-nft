use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
use casper_types::{account::AccountHash, runtime_args, CLValue, ContractHash, Key, RuntimeArgs};
use contract::events::Migration;

use crate::utility::{
    constants::{
        ACCESS_KEY_NAME_1_0_0, ACCOUNT_USER_1, ARG_ACCESS_KEY_NAME_1_0_0, ARG_COLLECTION_NAME,
        ARG_HASH_KEY_NAME_1_0_0, ARG_IS_HASH_IDENTIFIER_MODE, ARG_NAMED_KEY_CONVENTION,
        ARG_NFT_CONTRACT_HASH, ARG_NFT_PACKAGE_HASH, ARG_SOURCE_KEY, ARG_TARGET_KEY,
        ARG_TOKEN_HASH, ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, CONTRACT_1_0_0_WASM,
        ENTRY_POINT_REGISTER_OWNER, MANGLE_NAMED_KEYS, MINT_1_0_0_WASM, MINT_SESSION_WASM,
        NFT_CONTRACT_WASM, NFT_TEST_COLLECTION, NFT_TEST_SYMBOL, PAGE_LIMIT, PAGE_SIZE,
        RECEIPT_NAME, TRANSFER_SESSION_WASM, UNMATCHED_HASH_COUNT, UPDATED_RECEIPTS_WASM,
    },
    installer_request_builder::{
        InstallerRequestBuilder, MetadataMutability, NFTIdentifierMode, NFTMetadataKind,
        NamedKeyConventionMode, OwnershipMode,
    },
    support,
};

const OWNED_TOKENS: &str = "owned_tokens";
const MANGLED_ACCESS_KEY_NAME: &str = "mangled_access_key";
const MANGLED_HASH_KEY_NAME: &str = "mangled_hash_key";

fn get_nft_contract_hash_1_0_0(builder: &WasmTestBuilder<InMemoryGlobalState>) -> ContractHash {
    let nft_hash_addr = builder
        .get_expected_account(*DEFAULT_ACCOUNT_ADDR)
        .named_keys()
        .get("nft_contract")
        .expect("must have this entry in named keys")
        .into_hash()
        .expect("must get hash_addr");

    ContractHash::new(nft_hash_addr)
}

#[test]
fn should_safely_upgrade_in_ordinal_identifier_mode() {
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

    let nft_contract_hash_1_0_0 = get_nft_contract_hash_1_0_0(&builder);
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

    let package_hash = support::get_nft_contract_package_hash(&builder);

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_PACKAGE_HASH => package_hash,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_ACCESS_KEY_NAME_1_0_0 => ACCESS_KEY_NAME_1_0_0.to_string()
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_contract_hash = support::get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

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
    let actual_event: Migration = support::get_event(&builder, &nft_contract_key, 0);
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

    let nft_contract_hash_1_0_0 = get_nft_contract_hash_1_0_0(&builder);
    let nft_contract_key_1_0_0: Key = nft_contract_hash_1_0_0.into();

    let mut expected_metadata: Vec<String> = vec![];

    let number_of_tokens_pre_migration = 3usize;

    // Build of prestate before migration.
    for i in 0..number_of_tokens_pre_migration {
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
            ARG_NFT_PACKAGE_HASH => support::get_nft_contract_package_hash(&builder),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_ACCESS_KEY_NAME_1_0_0 => ACCESS_KEY_NAME_1_0_0.to_string()
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

    let nft_contract_hash_1_0_0 = get_nft_contract_hash_1_0_0(&builder);
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

    let nft_contract_package_hash = support::get_nft_contract_package_hash(&builder);

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_PACKAGE_HASH => nft_contract_package_hash,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_ACCESS_KEY_NAME_1_0_0 => ACCESS_KEY_NAME_1_0_0.to_string()
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let nft_package_key: Key = nft_contract_package_hash.into();

    let migrate_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        UPDATED_RECEIPTS_WASM,
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

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_PACKAGE_HASH => support::get_nft_contract_package_hash(&builder),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_ACCESS_KEY_NAME_1_0_0 => ACCESS_KEY_NAME_1_0_0.to_string()
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    // Once the new contract version has been added to the package
    // calling the updated_receipts entrypoint should cause an error to be returned.
    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_PACKAGE_HASH => support::get_nft_contract_package_hash(&builder),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_ACCESS_KEY_NAME_1_0_0 => ACCESS_KEY_NAME_1_0_0.to_string()
        },
    )
    .build();
    builder.exec(upgrade_request).expect_failure();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(error, 126u16, "must have previously migrated error");
}

#[test]
fn should_not_migrate_contracts_with_zero_token_issuance() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

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
            ARG_NFT_PACKAGE_HASH => support::get_nft_contract_package_hash(&builder),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_ACCESS_KEY_NAME_1_0_0 => ACCESS_KEY_NAME_1_0_0.to_string()
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

    let nft_contract_hash_1_0_0 = get_nft_contract_hash_1_0_0(&builder);
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
