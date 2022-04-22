use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{runtime_args, CLValue, RuntimeArgs, U256};

use crate::utility::{
    constants::{
        ARG_ALLOW_MINTING, ARG_COLLECTION_NAME, ARG_COLLECTION_SYMBOL, ARG_PUBLIC_MINTING,
        ARG_TOTAL_TOKEN_SUPPLY, CONTRACT_NAME, ENTRY_POINT_INIT, NFT_CONTRACT_WASM,
        NFT_TEST_COLLECTION, NFT_TEST_SYMBOL, NUMBER_OF_MINTED_TOKENS,
    },
    installer_request_builder::InstallerRequestBuilder,
    support,
};

#[test]
fn should_install_contract() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(U256::from(1u64))
        .build();

    builder.exec(install_request).expect_success().commit();

    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let query_result: String = support::query_stored_value(
        &mut builder,
        *nft_contract_key,
        vec![ARG_COLLECTION_NAME.to_string()],
    );

    assert_eq!(
        query_result,
        NFT_TEST_COLLECTION.to_string(),
        "collection_name initialized at installation should exist"
    );

    let query_result: String = support::query_stored_value(
        &mut builder,
        *nft_contract_key,
        vec![ARG_COLLECTION_SYMBOL.to_string()],
    );

    assert_eq!(
        query_result,
        NFT_TEST_SYMBOL.to_string(),
        "collection_symbol initialized at installation should exist"
    );

    let query_result: U256 = support::query_stored_value(
        &mut builder,
        *nft_contract_key,
        vec![ARG_TOTAL_TOKEN_SUPPLY.to_string()],
    );

    assert_eq!(
        query_result,
        U256::from(1u64),
        "total_token_supply initialized at installation should exist"
    );

    let query_result: bool = support::query_stored_value(
        &mut builder,
        *nft_contract_key,
        vec![ARG_ALLOW_MINTING.to_string()],
    );

    assert!(query_result, "Allow minting should default to true");

    let query_result: bool = support::query_stored_value(
        &mut builder,
        *nft_contract_key,
        vec![ARG_PUBLIC_MINTING.to_string()],
    );

    assert!(!query_result, "public minting should default to false");

    let query_result: U256 = support::query_stored_value(
        &mut builder,
        *nft_contract_key,
        vec![NUMBER_OF_MINTED_TOKENS.to_string()],
    );

    assert_eq!(
        query_result,
        U256::zero(),
        "number_of_minted_tokens initialized at installation should exist"
    );
}

#[test]
fn should_only_allow_init_during_installation_session() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(U256::from(2u64));
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let init_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_INIT,
        runtime_args! {
            ARG_COLLECTION_NAME => "collection_name".to_string(),
            ARG_COLLECTION_SYMBOL => "collection_symbol".to_string(),
            ARG_TOTAL_TOKEN_SUPPLY => "total_token_supply".to_string(),
            ARG_ALLOW_MINTING => true,
            ARG_PUBLIC_MINTING => false,
        },
    )
    .build();
    builder.exec(init_request).expect_failure();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(
        error,
        58u16,
        "should not allow calls to init() after installation",
    );
}

// This test needs some love...
#[test]
fn should_install_with_allow_minting_set_to_false() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(U256::from(1u64))
        .build();

    builder.exec(install_request).expect_success().commit();

    let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = installing_account.named_keys().get(CONTRACT_NAME);

    // TODO: THis test needs to be finished: --> should_install_with_optional_arguments_set_to_default_values()
    // Query and make sure that defaults values have expected values.
}

#[test]
fn should_reject_invalid_collection_name() {
    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_invalid_collection_name(
                CLValue::from_t::<U256>(U256::zero()).expect("expected CLValue"),
            );

    support::assert_expected_invalid_installer_request(
        install_request_builder,
        18,
        "should reject installation when given an invalid collection name",
    );
}

#[test]
fn should_reject_invalid_collection_symbol() {
    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_invalid_collection_symbol(
                CLValue::from_t::<U256>(U256::zero()).expect("expected CLValue"),
            );

    support::assert_expected_invalid_installer_request(
        install_request_builder,
        24,
        "should reject installation when given an invalid collection symbol",
    );
}

#[test]
fn should_reject_non_numerical_total_token_supply_value() {
    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_invalid_total_token_supply(
                CLValue::from_t::<String>("".to_string()).expect("expected CLValue"),
            );
    support::assert_expected_invalid_installer_request(
        install_request_builder,
        26,
        "should reject installation when given an invalid total supply value",
    );
}
