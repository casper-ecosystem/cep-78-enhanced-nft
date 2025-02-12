use casper_engine_test_support::{
    ExecuteRequestBuilder, LmdbWasmTestBuilder, UpgradeRequestBuilder, DEFAULT_ACCOUNT_ADDR,
};
use casper_fixtures::LmdbFixtureState;
use casper_types::{
    runtime_args, system::MINT, AddressableEntityHash, EraId, Key, ProtocolVersion,
};
use cep78::{
    constants::{
        ARG_COLLECTION_NAME, ARG_EVENTS_MODE, ARG_NAMED_KEY_CONVENTION, ARG_TOKEN_META_DATA,
        ARG_TOKEN_OWNER,
    },
    modalities::{EventsMode, NamedKeyConventionMode},
};

use crate::utility::{
    constants::{
        ARG_NFT_CONTRACT_HASH, ARG_NFT_CONTRACT_PACKAGE_HASH, CONTRACT_NAME, CONTRACT_VERSION,
        DEFAULT_ACCOUNT_KEY, NFT_CONTRACT_WASM, NFT_TEST_COLLECTION,
    },
    support::{get_nft_contract_package_hash_cep78, query_stored_value},
};

pub fn upgrade_v1_5_6_fixture_to_v2_0_0_ee(
    builder: &mut LmdbWasmTestBuilder,
    lmdb_fixture_state: &LmdbFixtureState,
) {
    // state hash in builder and lmdb storage should be the same
    assert_eq!(
        builder.get_post_state_hash(),
        lmdb_fixture_state.post_state_hash
    );

    // we upgrade the execution engines protocol from 1.x to 2.x
    let mut upgrade_config = UpgradeRequestBuilder::new()
        .with_current_protocol_version(lmdb_fixture_state.genesis_protocol_version())
        .with_new_protocol_version(ProtocolVersion::V2_0_0)
        // TODO fix with_enable_addressable_entity ?
        // .with_migrate_legacy_accounts(true)
        // .with_migrate_legacy_contracts(true)
        //.with_enable_addressable_entity(true)
        .with_activation_point(EraId::new(1))
        .build();

    builder
        .upgrade(&mut upgrade_config)
        .expect_upgrade_success()
        .commit();

    // the state hash should now be different
    assert_ne!(
        builder.get_post_state_hash(),
        lmdb_fixture_state.post_state_hash
    );
}

// the difference between the two is that in v1_binary the contract hash is fetched at [u8;32], while in v2_binary it is an AddressaleEntityHash
pub fn get_contract_hash_v1_binary(builder: &LmdbWasmTestBuilder) -> AddressableEntityHash {
    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap();
    let account_named_keys = account.named_keys();

    let cep18_token = account_named_keys
        .get(CONTRACT_NAME)
        .and_then(|key| key.into_hash_addr())
        .map(AddressableEntityHash::new)
        .expect("should have contract hash");

    cep18_token
}

pub fn get_contract_hash_v2_binary(builder: &LmdbWasmTestBuilder) -> AddressableEntityHash {
    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap();
    let account_named_keys = account.named_keys();

    let cep18_token = account_named_keys
        .get(CONTRACT_NAME)
        .and_then(|key| key.into_entity_hash())
        .expect("should have contract hash");

    cep18_token
}

#[test]
fn should_be_able_to_call_1x_contract_in_2x_execution_engine() {
    // load fixture that was created in a previous EE version
    let (mut builder, lmdb_fixture_state, _temp_dir) =
        casper_fixtures::builder_from_global_state_fixture("cep78_1.5.1-ee1.5.6-minted");

    // upgrade the execution engine to the new protocol version
    upgrade_v1_5_6_fixture_to_v2_0_0_ee(&mut builder, &lmdb_fixture_state);

    let nft_contract_key = get_contract_hash_v1_binary(&builder);

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_key,
        MINT,
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
fn should_migrate_1_5_6_to_feat_2_0() {
    // load fixture
    let (mut builder, lmdb_fixture_state, _temp_dir) =
        casper_fixtures::builder_from_global_state_fixture("cep78_1.5.1-ee1.5.6-minted");

    // upgrade engine
    upgrade_v1_5_6_fixture_to_v2_0_0_ee(&mut builder, &lmdb_fixture_state);

    let version_0_major: u32 = 1;
    let version_0_minor: u32 = query_stored_value(&builder, *DEFAULT_ACCOUNT_KEY, CONTRACT_VERSION);
    let contract_package_hash = get_nft_contract_package_hash_cep78(&builder);

    // upgrade the contract itself using a binary built for the new engine
    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_PACKAGE_HASH => contract_package_hash,
            ARG_EVENTS_MODE => EventsMode::CES as u8,
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::DerivedFromCollectionName as u8,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION,
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let version_1_string: String =
        query_stored_value(&builder, *DEFAULT_ACCOUNT_KEY, CONTRACT_VERSION);

    // Split into major and minor parts
    let parts: Vec<&str> = version_1_string.split('.').collect();

    // Parse the major and minor components
    let version_1_major: u32 = parts
        .first()
        .expect("Failed to get the major version")
        .parse()
        .expect("Failed to parse the major version as u32");

    let version_1_minor: u32 = parts
        .get(1)
        .unwrap_or(&"0") // Default to "0" if no minor version exists
        .parse()
        .expect("Failed to parse the minor version as u32");

    assert!(version_0_major < version_1_major);
    assert!(version_0_minor == version_1_minor);

    let nft_contract_key = get_contract_hash_v2_binary(&builder);

    // mint some new tokens in cep-18
    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_key,
        MINT,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => "",
        },
    )
    .build();

    builder.exec(mint_request).expect_success().commit();
}
