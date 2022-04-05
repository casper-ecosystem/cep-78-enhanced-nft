use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_execution_engine::{
    core::{engine_state::Error as EngineStateError, execution},
    storage::global_state::in_memory::InMemoryGlobalState,
};
use casper_types::{
    account::AccountHash, bytesrepr::FromBytes, runtime_args, ApiError, CLTyped, ContractHash, Key,
    PublicKey, RuntimeArgs, SecretKey, URef,
};

use super::{
    constants::{CONTRACT_NAME, ENTRY_POINT_SESSION_WASM},
    installer_request_builder::InstallerRequestBuilder,
};

const ARG_ENTRY_POINT_NAME: &str = "entry_point_name";
const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";

pub(crate) fn get_nft_contract_hash(
    builder: &WasmTestBuilder<InMemoryGlobalState>,
) -> ContractHash {
    let nft_hash_addr = builder
        .get_expected_account(*DEFAULT_ACCOUNT_ADDR)
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have this entry in named keys")
        .into_hash()
        .expect("must get hash_addr");

    ContractHash::new(nft_hash_addr)
}

pub(crate) fn get_dictionary_value_from_key<T: CLTyped + FromBytes>(
    builder: &WasmTestBuilder<InMemoryGlobalState>,
    nft_contract_key: &Key,
    dictionary_name: &str,
    dictionary_key: &str,
) -> T {
    let seed_uref = *builder
        .query(None, *nft_contract_key, &[])
        .expect("must have nft contract")
        .as_contract()
        .expect("must convert contract")
        .named_keys()
        .get(dictionary_name)
        .expect("must have key")
        .as_uref()
        .expect("must convert to seed uref");

    builder
        .query_dictionary_item(None, seed_uref, dictionary_key)
        .expect("should have dictionary value")
        .as_cl_value()
        .expect("T should be CLValue")
        .to_owned()
        .into_t()
        .unwrap()
}

pub(crate) fn create_dummy_key_pair(account_string: [u8; 32]) -> (SecretKey, PublicKey) {
    let secrete_key =
        SecretKey::ed25519_from_bytes(account_string).expect("failed to create secret key");
    let public_key = PublicKey::from(&secrete_key);
    (secrete_key, public_key)
}

pub(crate) fn assert_expected_invalid_installer_request(
    install_request_builder: InstallerRequestBuilder,
    expected_error_code: u16,
    reason: &str,
) {
    let mut builder = InMemoryWasmTestBuilder::default();

    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
    builder
        .exec(install_request_builder.build())
        .expect_failure(); // Should test against expected error

    let error = builder.get_error().expect("should have an error");
    assert_expected_error(error, expected_error_code, reason);
}

pub(crate) fn assert_expected_error(actual_error: EngineStateError, error_code: u16, reason: &str) {
    let actual = format!("{:?}", actual_error);
    let expected = format!(
        "{:?}",
        EngineStateError::Exec(execution::Error::Revert(ApiError::User(error_code)))
    );

    assert_eq!(
        actual, expected,
        "Error should match {} with reason: {}",
        error_code, reason
    )
}

pub(crate) fn get_uref(builder: &WasmTestBuilder<InMemoryGlobalState>, key: &str) -> URef {
    builder
        .get_expected_account(*DEFAULT_ACCOUNT_ADDR)
        .named_keys()
        .get(key)
        .expect("must have this entry as a result of calling mint")
        .into_uref()
        .unwrap()
}

pub(crate) fn query_stored_value<T: CLTyped + FromBytes>(
    builder: &mut InMemoryWasmTestBuilder,
    base_key: Key,
    path: Vec<String>,
) -> T {
    builder
        .query(None, base_key, &path)
        .expect("must have stored value")
        .as_cl_value()
        .cloned()
        .expect("must have cl value")
        .into_t::<T>()
        .expect("must get value")
}

pub(crate) fn call_entry_point_with_ret<T: CLTyped + FromBytes>(
    builder: &mut InMemoryWasmTestBuilder,
    account_hash: AccountHash,
    nft_contract_hash: ContractHash,
    mut runtime_args: RuntimeArgs,
    entry_point_name: &str,
) -> T {
    runtime_args
        .insert(ARG_ENTRY_POINT_NAME, entry_point_name.to_string())
        .unwrap();
    runtime_args
        .insert(ARG_NFT_CONTRACT_HASH, nft_contract_hash)
        .unwrap();

    let entry_point_session_call =
        ExecuteRequestBuilder::standard(account_hash, ENTRY_POINT_SESSION_WASM, runtime_args)
            .build();

    builder
        .exec(entry_point_session_call)
        .expect_success()
        .commit();

    query_stored_value::<T>(
        builder,
        account_hash.into(),
        [entry_point_name.to_string()].into(),
    )
}
