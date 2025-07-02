#![allow(unused)]
use super::{
    constants::{
        ACCOUNT_1_PUBLIC_KEY, ACCOUNT_2_PUBLIC_KEY, ACCOUNT_3_PUBLIC_KEY, CONTRACT_PACKAGE,
        MINTING_CONTRACT_PACKAGE_NAME,
    },
    installer_request_builder::InstallerRequestBuilder,
};
use crate::utility::constants::{
    ARG_KEY_NAME, ARG_NFT_CONTRACT_HASH, CONTRACT_NAME, MINTING_CONTRACT_NAME, PAGE_SIZE,
    TRANSFER_FILTER_CONTRACT_NAME,
};
use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use casper_engine_test_support::{
    utils::create_run_genesis_request, ChainspecConfig, ExecuteRequestBuilder, LmdbWasmTestBuilder,
    DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_PUBLIC_KEY,
};
use casper_execution_engine::{engine_state::Error as EngineStateError, execution::ExecError};
use casper_types::{
    account::AccountHash,
    bytesrepr::{Bytes, FromBytes},
    contracts::{ContractHash, ContractPackageHash},
    AddressableEntityHash, ApiError, CLTyped, CLValueError, EntityAddr, GenesisAccount, Key, Motes,
    PackageHash, RuntimeArgs, URef, BLAKE2B_DIGEST_LENGTH, U512,
};
use cep78::constants::{HASH_KEY_NAME_1_0_0, INDEX_BY_HASH, PREFIX_PAGE_DICTIONARY};
use rand::prelude::*;
use rand::random;
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::fmt::Debug;

pub(crate) fn genesis() -> LmdbWasmTestBuilder {
    let mut builder = LmdbWasmTestBuilder::default();
    // TODO Set enable_addressable_entity as param
    builder.with_chainspec(ChainspecConfig::default().with_enable_addressable_entity(false));
    builder.run_genesis(create_run_genesis_request(vec![
        GenesisAccount::Account {
            public_key: DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
            balance: Motes::new(U512::from(5_000_000_000_000_u64)),
            validator: None,
        },
        GenesisAccount::Account {
            public_key: ACCOUNT_1_PUBLIC_KEY.clone(),
            balance: Motes::new(U512::from(5_000_000_000_000_u64)),
            validator: None,
        },
        GenesisAccount::Account {
            public_key: ACCOUNT_2_PUBLIC_KEY.clone(),
            balance: Motes::new(U512::from(5_000_000_000_000_u64)),
            validator: None,
        },
        GenesisAccount::Account {
            public_key: ACCOUNT_3_PUBLIC_KEY.clone(),
            balance: Motes::new(U512::from(5_000_000_000_000_u64)),
            validator: None,
        },
    ]));
    builder
}

pub(crate) fn get_nft_contract_hash(builder: &LmdbWasmTestBuilder) -> AddressableEntityHash {
    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap();
    let named_keys = account.named_keys();

    named_keys
        .get(CONTRACT_NAME)
        .expect("must have this entry in named keys")
        .into_entity_hash()
        .expect("must get entity_hash")
}

pub(crate) fn get_nft_contract_hash_key(builder: &LmdbWasmTestBuilder) -> Key {
    let nft_contract_hash: ContractHash = get_nft_contract_hash(builder).into();
    // With entities enabled
    //  let nft_contract_key: Key = Key::contract_entity_key(nft_contract_hash.into()); // As AddressableEntityHash
    Key::Hash(nft_contract_hash.value()) // As Key::Hash
}

pub(crate) fn get_nft_contract_package_hash(builder: &LmdbWasmTestBuilder) -> ContractPackageHash {
    let nft_hash_addr = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap()
        .named_keys()
        .get(HASH_KEY_NAME_1_0_0)
        .expect("must have this entry in named keys")
        .into_package_addr()
        .expect("must get package addr");

    ContractPackageHash::new(nft_hash_addr)
}

pub(crate) fn get_nft_contract_package_hash_cep78(
    builder: &LmdbWasmTestBuilder,
) -> ContractPackageHash {
    let nft_hash_addr = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap()
        .named_keys()
        .get(CONTRACT_PACKAGE)
        .expect("must have this entry in named keys")
        .into_hash_addr()
        .expect("must get hash_addr");

    ContractPackageHash::new(nft_hash_addr)
}

pub(crate) fn get_minting_contract_hash(builder: &LmdbWasmTestBuilder) -> AddressableEntityHash {
    builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap()
        .named_keys()
        .get(MINTING_CONTRACT_NAME)
        .expect("must have minting contract hash entry in named keys")
        .into_entity_hash()
        .expect("must get hash_addr")
}

pub(crate) fn get_minting_contract_hash_key(builder: &LmdbWasmTestBuilder) -> Key {
    let minting_contract_hash: ContractHash = get_minting_contract_hash(builder).into();
    // With entities enabled
    // let minting_contract_key: Key = Key::contract_entity_key(minting_contract_hash.into());
    Key::Hash(minting_contract_hash.value())
}

pub(crate) fn get_minting_contract_package_hash(builder: &LmdbWasmTestBuilder) -> PackageHash {
    builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap()
        .named_keys()
        .get(MINTING_CONTRACT_PACKAGE_NAME)
        .expect("must have minting contract package hash entry in named keys")
        .into_package_hash()
        .expect("must get hash_addr")
}

pub(crate) fn get_transfer_filter_contract_hash(
    builder: &LmdbWasmTestBuilder,
) -> AddressableEntityHash {
    builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap()
        .named_keys()
        .get(TRANSFER_FILTER_CONTRACT_NAME)
        .expect("must have transfer filter hash entry in named keys")
        .into_entity_hash()
        .expect("must get hash_addr")
}

pub(crate) fn get_dictionary_value_from_key<T: CLTyped + FromBytes>(
    builder: &LmdbWasmTestBuilder,
    nft_contract_key: &Key,
    dictionary_name: &str,
    dictionary_key: &str,
) -> T {
    let named_key = match nft_contract_key.into_entity_hash() {
        Some(hash) => {
            let entity_with_named_keys = builder
                .get_entity_with_named_keys_by_entity_hash(hash)
                .expect("should be named key from entity hash");
            let named_keys = entity_with_named_keys.named_keys();
            named_keys
                .get(dictionary_name)
                .expect("must have key")
                .to_owned()
        }
        None => match nft_contract_key.into_account() {
            Some(account_hash) => {
                let entity_with_named_keys = builder
                    .get_entity_with_named_keys_by_account_hash(account_hash)
                    .expect("should be named key from account hash");
                let named_keys = entity_with_named_keys.named_keys();
                named_keys
                    .get(dictionary_name)
                    .expect("must have key")
                    .to_owned()
            }
            None => {
                let named_keys = builder.get_named_keys(EntityAddr::SmartContract(
                    nft_contract_key
                        .into_hash_addr()
                        .expect("should be entity addr"),
                ));
                named_keys
                    .get(dictionary_name)
                    .expect("must have key")
                    .to_owned()
            }
        },
    };

    let seed_uref = named_key.as_uref().expect("must convert to seed uref");

    builder
        .query_dictionary_item(None, *seed_uref, dictionary_key)
        .expect("should have dictionary value")
        .as_cl_value()
        .expect("T should be CLValue")
        .to_owned()
        .into_t()
        .unwrap()
}

pub(crate) fn assert_expected_invalid_installer_request(
    install_request_builder: InstallerRequestBuilder,
    expected_error_code: u16,
    reason: &str,
) {
    let mut builder = genesis();
    builder.exec(install_request_builder.build());
    let error = builder.get_error().expect("should have an error");
    assert_expected_error(error, expected_error_code, reason);
}

pub(crate) fn assert_expected_error(actual_error: EngineStateError, error_code: u16, reason: &str) {
    let actual = format!("{actual_error:?}");
    let expected = format!(
        "{:?}",
        EngineStateError::Exec(ExecError::Revert(ApiError::User(error_code)))
    );

    assert_eq!(
        actual, expected,
        "Error should match {error_code} with reason: {reason}"
    )
}

pub(crate) fn _get_uref(builder: &LmdbWasmTestBuilder, key: &str) -> URef {
    builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap()
        .named_keys()
        .get(key)
        .expect("must have this entry as a result of calling mint")
        .into_uref()
        .unwrap()
}

pub(crate) fn query_stored_value<T: CLTyped + FromBytes>(
    builder: &LmdbWasmTestBuilder,
    base_key: Key,
    name: &str,
) -> T {
    let stored = builder.query(None, base_key, &[name.to_string()]);
    let cl_value = stored
        .expect("must have stored value")
        .as_cl_value()
        .cloned()
        .expect("must have cl value");

    cl_value.into_t::<T>().expect("must get value")
}

pub(crate) fn call_session_code_with_ret<T: CLTyped + FromBytes>(
    builder: &mut LmdbWasmTestBuilder,
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
    query_stored_value::<T>(builder, account_hash.into(), key_name)
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
            checksum,
        }
    }

    pub(crate) fn with_random_checksum(name: String, token_uri: String) -> Self {
        let checksum: String = digest(random::<u64>().to_string());
        Self::new(name, token_uri, checksum)
    }
}

fn make_page_dictionary_item_key(token_owner_key: &Key) -> String {
    match token_owner_key {
        Key::Account(token_owner_account_hash) => token_owner_account_hash.to_string(),
        Key::Hash(token_owner_hash_addr) => ContractHash::new(*token_owner_hash_addr).to_string(),
        Key::AddressableEntity(token_owner_entity_addr) => match token_owner_entity_addr {
            EntityAddr::System(_) => panic!("invalid key type"),
            EntityAddr::Account(hash_addr) => AddressableEntityHash::new(*hash_addr),
            EntityAddr::SmartContract(hash_addr) => AddressableEntityHash::new(*hash_addr),
        }
        .to_string(),
        Key::SmartContract(token_owner_package_addr) => {
            PackageHash::new(*token_owner_package_addr).to_string()
        }
        _ => panic!("invalid key type"),
    }
}

pub(crate) fn get_token_page_by_id(
    builder: &LmdbWasmTestBuilder,
    nft_contract_key: &Key,
    token_owner_key: &Key,
    token_id: u64,
) -> Vec<bool> {
    let page_number = token_id / PAGE_SIZE;
    let token_page_item_key = make_page_dictionary_item_key(token_owner_key);
    get_dictionary_value_from_key(
        builder,
        nft_contract_key,
        &format!("{PREFIX_PAGE_DICTIONARY}_{page_number}"),
        &token_page_item_key,
    )
}

pub(crate) fn get_token_page_by_hash(
    builder: &LmdbWasmTestBuilder,
    nft_contract_key: &Key,
    token_owner_key: &Key,
    token_hash: String,
) -> Vec<bool> {
    let token_number: u64 =
        get_dictionary_value_from_key(builder, nft_contract_key, INDEX_BY_HASH, &token_hash);
    get_token_page_by_id(builder, nft_contract_key, token_owner_key, token_number)
}

pub(crate) fn get_stored_value_from_global_state<T: CLTyped + FromBytes>(
    builder: &LmdbWasmTestBuilder,
    query_key: Key,
    path: Vec<String>,
) -> Result<T, CLValueError> {
    builder
        .query(None, query_key, &path)
        .unwrap()
        .as_cl_value()
        .unwrap()
        .clone()
        .into_t::<T>()
}

pub(crate) fn get_receipt_name(nft_receipt: String, page_table_entry: u64) -> String {
    format!("{nft_receipt}_m_{PAGE_SIZE}_p_{page_table_entry}")
}

pub fn get_event<T: FromBytes + CLTyped + Debug>(
    builder: &LmdbWasmTestBuilder,
    nft_contract_key: &Key,
    index: u32,
) -> Result<T, String> {
    let bytes: Bytes = get_dictionary_value_from_key(
        builder,
        nft_contract_key,
        casper_event_standard::EVENTS_DICT,
        &index.to_string(),
    );

    let (event, _): (T, Bytes) = match T::from_bytes(&bytes) {
        Ok((event, bytes_remaining)) if bytes_remaining.is_empty() => {
            (event, bytes_remaining.into())
        }
        Err(err) => {
            let error = err.to_string();
            return Err(format!("Error Failed to decode event {error}"));
        }
        _ => unimplemented!(),
    };

    Ok(event)
}
