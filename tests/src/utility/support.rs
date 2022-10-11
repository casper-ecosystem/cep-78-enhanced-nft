use crate::utility::constants::{ARG_KEY_NAME, ARG_NFT_CONTRACT_HASH, MINTING_CONTRACT_NAME, PAGE_SIZE, TOKEN_TRACKER};
use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use serde::{Serialize, Deserialize};
use rand::prelude::*;
use sha256::digest;

use super::{constants::{CONTRACT_NAME, NUMBER_OF_MINTED_TOKENS}, installer_request_builder::InstallerRequestBuilder};
use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_execution_engine::{
    core::{engine_state::Error as EngineStateError, execution},
    storage::global_state::in_memory::InMemoryGlobalState,
};
use casper_types::{account::AccountHash, bytesrepr::FromBytes, ApiError, CLTyped, ContractHash, Key, PublicKey, RuntimeArgs, SecretKey, URef, BLAKE2B_DIGEST_LENGTH, CLValue};


#[derive(Serialize, Deserialize)]
pub(crate) struct CEP78Metadata {
    name: String,
    token_uri: String,
    checksum: String,
}

impl CEP78Metadata {
    pub(crate) fn new(name: String, token_uri: String, checksum: String) -> Self {
        Self {
            name,
            token_uri,
            checksum
        }
    }

    pub(crate) fn with_random_checksum(name: String, token_uri: String) -> Self {
        let checksum: String = digest(random::<u64>().to_string());
        Self::new(name, token_uri, checksum)
    }
}

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

pub(crate) fn get_minting_contract_hash(
    builder: &WasmTestBuilder<InMemoryGlobalState>,
) -> ContractHash {
    let minting_contract_hash = builder
        .get_expected_account(*DEFAULT_ACCOUNT_ADDR)
        .named_keys()
        .get(MINTING_CONTRACT_NAME)
        .expect("must have minting contract hash entry in named keys")
        .into_hash()
        .expect("must get hash_addr");

    ContractHash::new(minting_contract_hash)
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

pub(crate) fn _get_uref(builder: &WasmTestBuilder<InMemoryGlobalState>, key: &str) -> URef {
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
    nft_contract_key: Key,
    mut runtime_args: RuntimeArgs,
    wasm_file_name: &str,
    key_name: &str,
) -> T {
    runtime_args
        .insert(ARG_NFT_CONTRACT_HASH, nft_contract_key)
        .unwrap();
    runtime_args
        .insert(ARG_KEY_NAME, key_name.to_string())
        .unwrap();
    let session_call =
        ExecuteRequestBuilder::standard(account_hash, wasm_file_name, runtime_args).build();
    builder.exec(session_call).expect_success().commit();
    query_stored_value::<T>(builder, account_hash.into(), [key_name.to_string()].into())
}

pub(crate) fn create_blake2b_hash<T: AsRef<[u8]>>(data: T) -> [u8; BLAKE2B_DIGEST_LENGTH] {
    let mut result = [0; BLAKE2B_DIGEST_LENGTH];
    // NOTE: Assumed safe as `BLAKE2B_DIGEST_LENGTH` is a valid value for a hasher
    let mut hasher = VarBlake2b::new(BLAKE2B_DIGEST_LENGTH).expect("should create hasher");

    hasher.update(data);
    hasher.finalize_variable(|slice| {
        result.copy_from_slice(slice);
    });
    result
}


fn get_token_id_strings(builder: &WasmTestBuilder<InMemoryGlobalState>, nft_contract_key: Key, token_owner: &Key) -> Vec<String> {
    let number_of_minted_tokens = builder.query(None, nft_contract_key, &[NUMBER_OF_MINTED_TOKENS.to_string()])
        .expect("must have number of minted tokens")
        .as_cl_value()
        .map(|cl_value| CLValue::into_t::<u64>(cl_value.clone()))
        .unwrap()
        .expect("must get u64");

    let nft_contract_named_keys = builder.query(None, nft_contract_key, &vec![])
        .expect("must get contract as stored_value")
        .as_contract()
        .unwrap()
        .named_keys()
        .clone();

    let forward_lookup_uref = nft_contract_named_keys
        .get(TOKEN_TRACKER)
        .expect("must have uref as key")
        .into_uref()
        .unwrap();

    let token_owner_seed_uref = nft_contract_named_keys
        .get(&token_owner.to_formatted_string())
        .expect("must have uref as key")
        .into_uref()
        .unwrap();

    let mut owned_tokens = vec![];

    for i in 0..number_of_minted_tokens {
        let page_number = (i / PAGE_SIZE).to_string();
        let page_index = (i % PAGE_SIZE) as usize;
        let page = builder.query_dictionary_item(None, token_owner_seed_uref, &page_number)
            .expect("must get page as stored value")
            .as_cl_value()
            .map(|page_as_cl_value| CLValue::into_t::<Vec<bool>>(page_as_cl_value.clone()))
            .unwrap()
            .expect("must get page from cl_value");
        if page[page_index] {
            let token_id_as_string = builder.query_dictionary_item(None, forward_lookup_uref, &i.to_string())
                .unwrap()
                .as_cl_value()
                .map(|token_string| CLValue::into_t::<String>(token_string.clone()))
                .unwrap()
                .expect("must get token id as string");
            owned_tokens.push(token_id_as_string)

        }
    }
    owned_tokens
}

pub(crate) fn get_token_id(builder: &WasmTestBuilder<InMemoryGlobalState>, nft_contract_key: Key, token_owner: &Key) -> Vec<u64> {
    let token_id_as_strings = get_token_id_strings(builder, nft_contract_key, token_owner);
    token_id_as_strings
        .into_iter()
        .map(|id_string| id_string.parse::<u64>().unwrap())
        .collect()
}

pub(crate) fn get_token_hashes(builder: &WasmTestBuilder<InMemoryGlobalState>, nft_contract_key: Key, token_owner: &Key) -> Vec<String> {
    get_token_id_strings(builder, nft_contract_key, token_owner)
}


