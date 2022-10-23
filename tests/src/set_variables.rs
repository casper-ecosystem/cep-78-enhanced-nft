use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{runtime_args, ContractHash, Key, RuntimeArgs};

use crate::utility::{
    constants::{
        ACCOUNT_USER_1, ARG_ALLOW_MINTING, CONTRACT_NAME, ENTRY_POINT_SET_VARIABLES,
        NFT_CONTRACT_WASM, NFT_TEST_COLLECTION, NFT_TEST_SYMBOL,
    },
    installer_request_builder::InstallerRequestBuilder,
    support,
};

#[test]
fn only_installer_should_be_able_to_toggle_allow_minting() {
    let (_, other_user_public_key) = support::create_dummy_key_pair(ACCOUNT_USER_1); //<-- Choose MINTER2 for failing red test
    let other_user_account = other_user_public_key.to_account_hash();
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_allowing_minting(false)
        .build();

    builder.exec(install_request).expect_success().commit();

    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_hash = account
        .named_keys()
        .get(CONTRACT_NAME)
        .cloned()
        .and_then(Key::into_hash)
        .map(ContractHash::new)
        .expect("failed to find nft contract");

    //Account other than installer account should not be able to change allow_minting
    // Red test
    let other_user_set_variables_request = ExecuteRequestBuilder::contract_call_by_hash(
        other_user_account,
        nft_contract_hash,
        ENTRY_POINT_SET_VARIABLES,
        runtime_args! { ARG_ALLOW_MINTING => Some(true) },
    )
    .build();

    builder
        .exec(other_user_set_variables_request)
        .expect_failure()
        .commit();

    // Don't just use expect_failure. Match and actual error!
    // let error = builder.get_error().expect("must have error");
    // assert_expected_error(error, NFTCoreError::InvalidAccount as u16);

    //Installer account should be able to change allow_minting
    // Green test
    let installer_set_variables_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_VARIABLES,
        runtime_args! { ARG_ALLOW_MINTING => Some(true) },
    )
    .build();

    builder
        .exec(installer_set_variables_request)
        .expect_success()
        .commit();
}
