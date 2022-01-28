#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::{vec, vec::Vec};
use casper_types::{CLType, PublicKey, U256};
use core::mem::MaybeUninit;

use casper_contract::{
    contract_api::{self, runtime, storage, system},
    ext_ffi,
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    account::AccountHash,
    api_error,
    bytesrepr::{self, FromBytes, ToBytes},
    ApiError, BlockTime, CLTyped, CLValue, Key, URef, U512,
};

pub const ARG_COLLECTION_NAME: &str = "collection_name";
pub const ARG_TOKEN_OWNER: &str = "token_owner";
pub const ARG_TOKEN_NAME: &str = "token_name";
pub const ARG_TOKEN_META: &str = "token_meta";
pub const ARG_TOKEN_ID: &str = "token_id";

pub const STORAGE: &str = "storage";
pub const OWNERS: &str = "owners";

pub const INSTALLER: &str = "installer";
pub const CONTRACT_NAME: &str = "nft_contract";
pub const HASH_KEY_NAME: &str = "nft_contract_package";
pub const ACCESS_KEY_NAME: &str = "nft_contract_package_access";
pub const CONTRACT_VERSION: &str = "contract_version";
pub const COLLECTION_NAME: &str = "collection_name";

pub const ENTRY_POINT_INIT: &str = "init";
pub const ENTRY_POINT_SET_VARIABLES: &str = "set_variables";
pub const ENTRY_POINT_MINT: &str = "mint";
pub const ENTRY_POINT_BALANCE_OF: &str = "balance_of";

#[repr(u16)]
enum NFTCoreError {
    InvalidAccount = 1,
    MissingInstaller = 2,
    InvalidInstaller = 3,
    UnexpectedKeyVariant = 4,
    MissingTokenOwner = 5,
    InvalidTokenOwner = 6,
    FailedToGetArgBytes = 7,
    FailedToCreateDictionary = 8,
    MissingStorageUref = 9,
    InvalidStorageUref = 10,
    MissingOwnersUref = 11,
    InvalidOwnersUref = 12,
    FailedToAccessStorageDictionary = 13,
    FailedToAccessOwnershipDictionary = 14,
    DuplicateMinted = 15,
    FailedToConvertToCLValue = 16,
    MissingCollectionName = 17,
    InvalidCollectionName = 18,
}

impl From<NFTCoreError> for ApiError {
    fn from(e: NFTCoreError) -> Self {
        ApiError::User(e as u16)
    }
}

struct NFT {
    token_owner: PublicKey,
    token_id: U256,
    token_name: String,
    token_meta: String,
}

impl NFT {
    fn new(token_owner: PublicKey, token_id: U256, token_name: String, token_meta: String) -> NFT {
        NFT {
            token_owner,
            token_id,
            token_name,
            token_meta,
        }
    }

    fn id(&self) -> U256 {
        self.token_id
    }
}

impl ToBytes for NFT {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut buffer = bytesrepr::allocate_buffer(self)?;
        buffer.extend(self.token_owner.to_bytes()?);
        buffer.extend(self.token_id.to_bytes()?);
        buffer.extend(self.token_name.to_bytes()?);
        buffer.extend(self.token_meta.to_bytes()?);

        Ok(buffer)
    }

    fn serialized_length(&self) -> usize {
        self.token_owner.serialized_length()
            + self.token_id.serialized_length()
            + self.token_name.serialized_length()
            + self.token_meta.serialized_length()
    }
}

impl FromBytes for NFT {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (token_owner, remainder) = FromBytes::from_bytes(bytes)?;
        let (token_id, remainder) = FromBytes::from_bytes(remainder)?;
        let (token_name, remainder) = FromBytes::from_bytes(remainder)?;
        let (token_meta, remainder) = FromBytes::from_bytes(remainder)?;

        Ok((
            NFT::new(token_owner, token_id, token_name, token_meta),
            remainder,
        ))
    }
}

impl CLTyped for NFT {
    fn cl_type() -> CLType {
        CLType::Any
    }
}

type TokenIDs = Vec<U256>;

#[no_mangle]
fn mint() {
    // should the NFT immediately belong to the given owner, or should it
    // belong to the contract and then be transferred to an owner?
    let token_owner: PublicKey = runtime::get_named_arg(ARG_TOKEN_OWNER);
    let token_id: U256 = runtime::get_named_arg(ARG_TOKEN_ID);
    let token_name: String = runtime::get_named_arg(ARG_TOKEN_NAME);
    let token_meta: String = runtime::get_named_arg(ARG_TOKEN_META);

    let owner_account_hash = token_owner.to_account_hash();

    let nft = NFT::new(token_owner.clone(), token_id, token_name, token_meta);

    //
    let storage_seed_uref = get_uref_with_user_errors(
        STORAGE,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );

    if let Some(_) = storage::dictionary_get::<NFT>(storage_seed_uref, &nft.id().to_string())
        .unwrap_or_revert_with(NFTCoreError::FailedToAccessStorageDictionary)
    {
        runtime::revert(NFTCoreError::DuplicateMinted);
    }

    let owners_seed_uref = get_uref_with_user_errors(
        OWNERS,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );

    let current_token_ids: TokenIDs =
        match storage::dictionary_get::<TokenIDs>(owners_seed_uref, &token_owner.to_string())
            .unwrap_or_revert_with(NFTCoreError::FailedToAccessOwnershipDictionary)
        {
            Some(mut token_ids) => {
                token_ids.push(token_id);

                token_ids
            }
            None => vec![token_id],
        };

    storage::dictionary_put(storage_seed_uref, &token_id.to_string(), nft);
    storage::dictionary_put(
        owners_seed_uref,
        &owner_account_hash.to_string(),
        current_token_ids,
    );
}

// balance_of implies an amount and NFTs are not amount-based.
#[no_mangle]
fn balance_of() {
    let token_owner: PublicKey = runtime::get_named_arg(ARG_TOKEN_OWNER);

    let owners_seed_uref = get_uref_with_user_errors(
        OWNERS,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );

    let current_token_ids: TokenIDs =
        match storage::dictionary_get::<TokenIDs>(owners_seed_uref, &token_owner.to_string())
            .unwrap_or_revert_with(NFTCoreError::FailedToAccessOwnershipDictionary)
        {
            Some(token_ids) => token_ids,
            None => vec![],
        };

    let current_token_ids = CLValue::from_t(current_token_ids)
        .unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);
    runtime::ret(current_token_ids);
}

#[no_mangle]
pub fn init() {
    let installing_account = get_account_hash_with_user_errors(
        INSTALLER,
        NFTCoreError::MissingInstaller,
        NFTCoreError::InvalidInstaller,
    );

    if installing_account != runtime::get_caller() {
        runtime::revert(NFTCoreError::InvalidAccount)
    }

    // Setup the initial variables.
    let collection_name: String = runtime::get_named_arg(ARG_COLLECTION_NAME);
    let collection_name = storage::new_uref(collection_name);
    let storage_seed_uref = storage::new_dictionary(STORAGE)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    let owners_seed_uref = storage::new_dictionary(OWNERS)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);

    runtime::put_key(STORAGE, storage_seed_uref.into());
    runtime::put_key(OWNERS, owners_seed_uref.into());
    runtime::put_key(COLLECTION_NAME, collection_name.into());
}

#[no_mangle]
pub fn set_variables() {
    let installing_account = get_account_hash_with_user_errors(
        INSTALLER,
        NFTCoreError::MissingInstaller,
        NFTCoreError::InvalidInstaller,
    );

    if installing_account != runtime::get_caller() {
        runtime::revert(NFTCoreError::InvalidAccount)
    }

    // // Manipulate the mutable variables.
    // TODO: figure out what things are configurable
    let collection_name: String = runtime::get_named_arg(ARG_COLLECTION_NAME);

    let collection_name_uref = get_uref_with_user_errors(
        COLLECTION_NAME,
        NFTCoreError::MissingCollectionName,
        NFTCoreError::InvalidCollectionName,
    );

    // TODO: should the collection name be mutable?
    storage::write(collection_name_uref, collection_name);
}

fn get_named_arg_size(name: &str) -> Option<usize> {
    let mut arg_size: usize = 0;
    let ret = unsafe {
        ext_ffi::casper_get_named_arg_size(
            name.as_bytes().as_ptr(),
            name.len(),
            &mut arg_size as *mut usize,
        )
    };
    match api_error::result_from(ret) {
        Ok(_) => Some(arg_size),
        Err(ApiError::MissingArgument) => None,
        Err(e) => runtime::revert(e),
    }
}

fn get_optional_named_arg_with_user_errors<T: FromBytes>(
    name: &str,
    missing: NFTCoreError,
    invalid: NFTCoreError,
) -> Option<T> {
    match get_named_arg_with_user_errors(name, missing, invalid) {
        Ok(val) => val,
        Err(err @ NFTCoreError::InvalidInstaller) => runtime::revert(err),
        Err(_) => None,
    }
}

fn get_named_arg_with_user_errors<T: FromBytes>(
    name: &str,
    missing: NFTCoreError,
    invalid: NFTCoreError,
) -> Result<T, NFTCoreError> {
    let arg_size = get_named_arg_size(name).ok_or(missing)?;
    let arg_bytes = if arg_size > 0 {
        let res = {
            let data_non_null_ptr = contract_api::alloc_bytes(arg_size);
            let ret = unsafe {
                ext_ffi::casper_get_named_arg(
                    name.as_bytes().as_ptr(),
                    name.len(),
                    data_non_null_ptr.as_ptr(),
                    arg_size,
                )
            };
            let data =
                unsafe { Vec::from_raw_parts(data_non_null_ptr.as_ptr(), arg_size, arg_size) };
            api_error::result_from(ret).map(|_| data)
        };
        // Assumed to be safe as `get_named_arg_size` checks the argument already
        res.unwrap_or_revert_with(NFTCoreError::FailedToGetArgBytes)
    } else {
        // Avoids allocation with 0 bytes and a call to get_named_arg
        Vec::new()
    };

    bytesrepr::deserialize(arg_bytes).map_err(|_| invalid)
}

fn get_account_hash_with_user_errors(
    name: &str,
    missing: NFTCoreError,
    invalid: NFTCoreError,
) -> AccountHash {
    let key = get_key_with_user_errors(name, missing, invalid);
    key.into_account()
        .unwrap_or_revert_with(NFTCoreError::UnexpectedKeyVariant)
}

fn get_uref_with_user_errors(name: &str, missing: NFTCoreError, invalid: NFTCoreError) -> URef {
    let key = get_key_with_user_errors(name, missing, invalid);
    key.into_uref()
        .unwrap_or_revert_with(NFTCoreError::UnexpectedKeyVariant)
}

fn get_key_with_user_errors(name: &str, missing: NFTCoreError, invalid: NFTCoreError) -> Key {
    let (name_ptr, name_size, _bytes) = to_ptr(name);
    let mut key_bytes = vec![0u8; Key::max_serialized_length()];
    let mut total_bytes: usize = 0;
    let ret = unsafe {
        ext_ffi::casper_get_key(
            name_ptr,
            name_size,
            key_bytes.as_mut_ptr(),
            key_bytes.len(),
            &mut total_bytes as *mut usize,
        )
    };
    match api_error::result_from(ret) {
        Ok(_) => {}
        Err(ApiError::MissingKey) => runtime::revert(missing),
        Err(e) => runtime::revert(e),
    }
    key_bytes.truncate(total_bytes);

    bytesrepr::deserialize(key_bytes).unwrap_or_revert_with(invalid)
}

fn read_with_user_errors<T: CLTyped + FromBytes>(
    uref: URef,
    missing: NFTCoreError,
    invalid: NFTCoreError,
) -> T {
    let key: Key = uref.into();
    let (key_ptr, key_size, _bytes) = to_ptr(key);

    let value_size = {
        let mut value_size = MaybeUninit::uninit();
        let ret = unsafe { ext_ffi::casper_read_value(key_ptr, key_size, value_size.as_mut_ptr()) };
        match api_error::result_from(ret) {
            Ok(_) => unsafe { value_size.assume_init() },
            Err(ApiError::ValueNotFound) => runtime::revert(missing),
            Err(e) => runtime::revert(e),
        }
    };

    let value_bytes = read_host_buffer(value_size).unwrap_or_revert();

    bytesrepr::deserialize(value_bytes).unwrap_or_revert_with(invalid)
}

fn read_host_buffer_into(dest: &mut [u8]) -> Result<usize, ApiError> {
    let mut bytes_written = MaybeUninit::uninit();
    let ret = unsafe {
        ext_ffi::casper_read_host_buffer(dest.as_mut_ptr(), dest.len(), bytes_written.as_mut_ptr())
    };
    // NOTE: When rewriting below expression as `result_from(ret).map(|_| unsafe { ... })`, and the
    // caller ignores the return value, execution of the contract becomes unstable and ultimately
    // leads to `Unreachable` error.
    api_error::result_from(ret)?;
    Ok(unsafe { bytes_written.assume_init() })
}

fn read_host_buffer(size: usize) -> Result<Vec<u8>, ApiError> {
    let mut dest: Vec<u8> = if size == 0 {
        Vec::new()
    } else {
        let bytes_non_null_ptr = contract_api::alloc_bytes(size);
        unsafe { Vec::from_raw_parts(bytes_non_null_ptr.as_ptr(), size, size) }
    };
    read_host_buffer_into(&mut dest)?;
    Ok(dest)
}

fn to_ptr<T: ToBytes>(t: T) -> (*const u8, usize, Vec<u8>) {
    let bytes = t.into_bytes().unwrap_or_revert();
    let ptr = bytes.as_ptr();
    let size = bytes.len();
    (ptr, size, bytes)
}
