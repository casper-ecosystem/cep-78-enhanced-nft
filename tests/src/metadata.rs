use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{account::AccountHash, runtime_args, ContractHash, Key, RuntimeArgs};

use crate::utility::{
    constants::{
        ARG_CONTRACT_WHITELIST, ARG_NFT_CONTRACT_HASH, ARG_TOKEN_HASH, ARG_TOKEN_ID,
        ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, ENTRY_POINT_METADATA, ENTRY_POINT_MINT,
        ENTRY_POINT_SET_TOKEN_METADATA, MALFORMED_META_DATA, METADATA_CEP78,
        METADATA_CUSTOM_VALIDATED, METADATA_NFT721, METADATA_RAW, MINTING_CONTRACT_WASM,
        MINT_SESSION_WASM, NFT_CONTRACT_WASM, TEST_PRETTY_721_META_DATA,
        TEST_PRETTY_CEP78_METADATA, TEST_PRETTY_UPDATED_721_META_DATA,
        TEST_PRETTY_UPDATED_CEP78_METADATA, TOKEN_OWNERS,
    },
    installer_request_builder::{
        InstallerRequestBuilder, MetadataMutability, MintingMode, NFTHolderMode, NFTIdentifierMode,
        NFTMetadataKind, OwnershipMode, WhitelistMode, TEST_CUSTOM_METADATA,
        TEST_CUSTOM_METADATA_SCHEMA, TEST_CUSTOM_UPDATED_METADATA,
    },
    support,
    support::{
        assert_expected_error, get_minting_contract_hash, get_nft_contract_hash, query_stored_value,
    },
};

#[test]
fn should_prevent_update_in_immutable_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(10u64)
        .with_nft_metadata_kind(NFTMetadataKind::NFT721)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = support::get_nft_contract_hash(&builder).into();

    let mint_token_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA
        },
    )
    .build();

    builder.exec(mint_token_request).expect_success().commit();

    let token_hash: String =
        base16::encode_lower(&support::create_blake2b_hash(&TEST_PRETTY_721_META_DATA));

    let update_token_metadata_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        support::get_nft_contract_hash(&builder),
        ENTRY_POINT_SET_TOKEN_METADATA,
        runtime_args! {
            ARG_TOKEN_HASH => token_hash,
            ARG_TOKEN_META_DATA => TEST_PRETTY_UPDATED_721_META_DATA
        },
    )
    .build();

    builder.exec(update_token_metadata_request).expect_failure();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(error, 104, "must match ForbiddenMetadataUpdate(104)")
}

#[test]
fn should_prevent_install_with_hash_identifier_in_mutable_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(10u64)
        .with_nft_metadata_kind(NFTMetadataKind::NFT721)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_metadata_mutability(MetadataMutability::Mutable)
        .build();

    builder.exec(install_request).expect_failure();

    let error = builder.get_error().expect("must fail at installation");

    assert_expected_error(error, 102, "Should raise InvalidMetadataMutability(102)")
}

#[test]
fn should_prevent_update_for_invalid_metadata() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(10u64)
        .with_nft_metadata_kind(NFTMetadataKind::NFT721)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_metadata_mutability(MetadataMutability::Mutable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = support::get_nft_contract_hash(&builder).into();

    let mint_token_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA
        },
    )
    .build();

    builder.exec(mint_token_request).expect_success().commit();

    let original_metadata = support::get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_NFT721,
        &0u64.to_string(),
    );

    assert_eq!(TEST_PRETTY_721_META_DATA, original_metadata);

    let update_token_metadata_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        support::get_nft_contract_hash(&builder),
        ENTRY_POINT_SET_TOKEN_METADATA,
        runtime_args! {
            ARG_TOKEN_ID => 0u64,
            ARG_TOKEN_META_DATA => MALFORMED_META_DATA
        },
    )
    .build();

    builder.exec(update_token_metadata_request).expect_failure();
}

#[test]
fn should_prevent_metadata_update_by_non_owner_key() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(10u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_nft_metadata_kind(NFTMetadataKind::NFT721)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_metadata_mutability(MetadataMutability::Mutable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = support::get_nft_contract_hash(&builder).into();

    let nft_owner_account_key = Key::Account(AccountHash::new([4u8; 32]));

    let mint_token_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => nft_owner_account_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA
        },
    )
    .build();

    builder.exec(mint_token_request).expect_success().commit();

    let original_metadata = support::get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_NFT721,
        &0u64.to_string(),
    );

    assert_eq!(TEST_PRETTY_721_META_DATA, original_metadata);

    let token_owner_key = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &0u64.to_string(),
    );

    assert_eq!(token_owner_key, nft_owner_account_key);

    let update_token_metadata_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        support::get_nft_contract_hash(&builder),
        ENTRY_POINT_SET_TOKEN_METADATA,
        runtime_args! {
            ARG_TOKEN_ID => 0u64,
            ARG_TOKEN_META_DATA => TEST_PRETTY_UPDATED_721_META_DATA
        },
    )
    .build();

    builder.exec(update_token_metadata_request).expect_failure();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(error, 6, "must match InvalidTokenOwner(6)")
}

fn should_allow_update_for_valid_metadata_based_on_kind(
    nft_metadata_kind: NFTMetadataKind,
    identifier_mode: NFTIdentifierMode,
) {
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
            ARG_TOKEN_META_DATA => original_metadata.to_string()
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

    assert_eq!(actual_updated_metadata, updated_metadata.to_string())
}

#[test]
fn should_update_metadata_for_nft721_using_token_id() {
    should_allow_update_for_valid_metadata_based_on_kind(
        NFTMetadataKind::NFT721,
        NFTIdentifierMode::Ordinal,
    )
}

#[test]
fn should_update_metadata_for_cep78_using_token_id() {
    should_allow_update_for_valid_metadata_based_on_kind(
        NFTMetadataKind::CEP78,
        NFTIdentifierMode::Ordinal,
    )
}

#[test]
fn should_update_metadata_for_custom_validated_using_token_id() {
    should_allow_update_for_valid_metadata_based_on_kind(
        NFTMetadataKind::CustomValidated,
        NFTIdentifierMode::Ordinal,
    )
}

#[test]
fn should_get_metadata_using_token_id() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let minting_contract_hash = get_minting_contract_hash(&builder);
    let minting_contract_key: Key = minting_contract_hash.into();

    let contract_whitelist = vec![minting_contract_hash];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_minting_mode(MintingMode::Installer as u8)
        .with_contract_whitelist(contract_whitelist.clone())
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let actual_contract_whitelist: Vec<ContractHash> = query_stored_value(
        &mut builder,
        nft_contract_key,
        vec![ARG_CONTRACT_WHITELIST.to_string()],
    );

    assert_eq!(actual_contract_whitelist, contract_whitelist);

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => minting_contract_key,
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
    };

    let minting_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder.exec(minting_request).expect_success().commit();

    let token_id = 0u64.to_string();
    let minted_metadata: String = support::get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        METADATA_NFT721,
        &token_id,
    );
    assert_eq!(minted_metadata, TEST_PRETTY_721_META_DATA);

    let get_metadata_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_METADATA,
        runtime_args! {
            ARG_TOKEN_ID => 0u64,
            ARG_NFT_CONTRACT_HASH => nft_contract_key
        },
    )
    .build();

    builder.exec(get_metadata_request);
}

#[test]
fn get_schema() {
    println!(
        "{}",
        serde_json::to_string_pretty(&*TEST_CUSTOM_METADATA).unwrap()
    )
}
