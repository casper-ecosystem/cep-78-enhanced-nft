use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::{convert::TryInto, mem::MaybeUninit};

use casper_contract::{
    contract_api::{self, runtime, storage},
    ext_ffi,
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    account::AccountHash,
    api_error,
    bytesrepr::{self, FromBytes, ToBytes},
    system::CallStackElement,
    ApiError, CLTyped, ContractHash, Key, URef,
};

use crate::{
    constants::{ARG_TOKEN_HASH, ARG_TOKEN_ID, HOLDER_MODE, OWNED_TOKENS, OWNERSHIP_MODE},
    error::NFTCoreError,
    modalities::{NFTHolderMode, NFTIdentifierMode, OwnershipMode, TokenIdentifier},
    BurnMode, BURNT_TOKENS, BURN_MODE,
};

pub(crate) fn upsert_dictionary_value_from_key<T: CLTyped + FromBytes + ToBytes>(
    dictionary_name: &str,
    key: &str,
    value: T,
) {
    let seed_uref = get_uref(
        dictionary_name,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );

    match storage::dictionary_get::<T>(seed_uref, key) {
        Ok(None | Some(_)) => storage::dictionary_put(seed_uref, key, value),
        Err(error) => runtime::revert(error),
    }
}

pub(crate) fn get_ownership_mode() -> Result<OwnershipMode, NFTCoreError> {
    get_stored_value_with_user_errors::<u8>(
        OWNERSHIP_MODE,
        NFTCoreError::MissingOwnershipMode,
        NFTCoreError::InvalidOwnershipMode,
    )
    .try_into()
}

pub(crate) fn get_holder_mode() -> Result<NFTHolderMode, NFTCoreError> {
    get_stored_value_with_user_errors::<u8>(
        HOLDER_MODE,
        NFTCoreError::MissingHolderMode,
        NFTCoreError::InvalidHolderMode,
    )
    .try_into()
}

pub(crate) fn get_owned_tokens_dictionary_item_key(token_owner_key: Key) -> String {
    match token_owner_key {
        Key::Account(token_owner_account_hash) => token_owner_account_hash.to_string(),
        Key::Hash(token_owner_hash_addr) => ContractHash::new(token_owner_hash_addr).to_string(),
        _ => runtime::revert(NFTCoreError::InvalidKey),
    }
}

pub(crate) fn get_dictionary_value_from_key<T: CLTyped + FromBytes>(
    dictionary_name: &str,
    key: &str,
) -> Option<T> {
    let seed_uref = get_uref(
        dictionary_name,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );

    match storage::dictionary_get::<T>(seed_uref, key) {
        Ok(maybe_value) => maybe_value,
        Err(error) => runtime::revert(error),
    }
}

pub(crate) fn get_stored_value_with_user_errors<T: CLTyped + FromBytes>(
    name: &str,
    missing: NFTCoreError,
    invalid: NFTCoreError,
) -> T {
    let uref = get_uref(name, missing, invalid);
    read_with_user_errors(uref, missing, invalid)
}

pub(crate) fn get_named_arg_size(name: &str) -> Option<usize> {
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

pub(crate) fn get_optional_named_arg_with_user_errors<T: FromBytes>(
    name: &str,
    invalid: NFTCoreError,
) -> Option<T> {
    match get_named_arg_with_user_errors(name, NFTCoreError::Phantom, invalid) {
        Ok(val) => val,
        Err(_) => None,
    }
}

pub(crate) fn get_named_arg_with_user_errors<T: FromBytes>(
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

pub(crate) fn get_account_hash(
    name: &str,
    missing: NFTCoreError,
    invalid: NFTCoreError,
) -> AccountHash {
    let key = get_key_with_user_errors(name, missing, invalid);
    key.into_account()
        .unwrap_or_revert_with(NFTCoreError::UnexpectedKeyVariant)
}

pub(crate) fn get_uref(name: &str, missing: NFTCoreError, invalid: NFTCoreError) -> URef {
    let key = get_key_with_user_errors(name, missing, invalid);
    key.into_uref()
        .unwrap_or_revert_with(NFTCoreError::UnexpectedKeyVariant)
}

pub(crate) fn named_uref_exists(name: &str) -> bool {
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

    api_error::result_from(ret).is_ok()
}

pub(crate) fn get_key_with_user_errors(
    name: &str,
    missing: NFTCoreError,
    invalid: NFTCoreError,
) -> Key {
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

pub(crate) fn read_with_user_errors<T: CLTyped + FromBytes>(
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

pub(crate) fn read_host_buffer_into(dest: &mut [u8]) -> Result<usize, ApiError> {
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

pub(crate) fn read_host_buffer(size: usize) -> Result<Vec<u8>, ApiError> {
    let mut dest: Vec<u8> = if size == 0 {
        Vec::new()
    } else {
        let bytes_non_null_ptr = contract_api::alloc_bytes(size);
        unsafe { Vec::from_raw_parts(bytes_non_null_ptr.as_ptr(), size, size) }
    };
    read_host_buffer_into(&mut dest)?;
    Ok(dest)
}

pub(crate) fn to_ptr<T: ToBytes>(t: T) -> (*const u8, usize, Vec<u8>) {
    let bytes = t.into_bytes().unwrap_or_revert();
    let ptr = bytes.as_ptr();
    let size = bytes.len();
    (ptr, size, bytes)
}

pub(crate) fn get_verified_caller() -> Result<Key, NFTCoreError> {
    let holder_mode = get_holder_mode()?;
    match *runtime::get_call_stack()
        .iter()
        .nth_back(1)
        .to_owned()
        .unwrap_or_revert()
    {
        CallStackElement::Session {
            account_hash: calling_account_hash,
        } => {
            if let NFTHolderMode::Contracts = holder_mode {
                return Err(NFTCoreError::InvalidHolderMode);
            }
            Ok(Key::Account(calling_account_hash))
        }
        CallStackElement::StoredSession { contract_hash, .. }
        | CallStackElement::StoredContract { contract_hash, .. } => {
            if let NFTHolderMode::Accounts = holder_mode {
                return Err(NFTCoreError::InvalidHolderMode);
            }
            Ok(contract_hash.into())
        }
    }
}

pub(crate) fn get_token_identifier_from_runtime_args(
    identifier_mode: &NFTIdentifierMode,
) -> TokenIdentifier {
    match identifier_mode {
        NFTIdentifierMode::Ordinal => get_named_arg_with_user_errors::<u64>(
            ARG_TOKEN_ID,
            NFTCoreError::MissingTokenID,
            NFTCoreError::InvalidTokenIdentifier,
        )
        .map(TokenIdentifier::new_index)
        .unwrap_or_revert(),
        NFTIdentifierMode::Hash => get_named_arg_with_user_errors::<String>(
            ARG_TOKEN_HASH,
            NFTCoreError::MissingTokenID,
            NFTCoreError::InvalidTokenIdentifier,
        )
        .map(TokenIdentifier::new_hash)
        .unwrap_or_revert(),
    }
}

pub(crate) fn get_token_identifiers_from_dictionary(
    identifier_mode: &NFTIdentifierMode,
    owners_item_key: &str,
) -> Option<Vec<TokenIdentifier>> {
    match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            get_dictionary_value_from_key::<Vec<u64>>(OWNED_TOKENS, owners_item_key).map(
                |token_indices| {
                    token_indices
                        .into_iter()
                        .map(TokenIdentifier::new_index)
                        .collect()
                },
            )
        }
        NFTIdentifierMode::Hash => {
            get_dictionary_value_from_key::<Vec<String>>(OWNED_TOKENS, owners_item_key).map(
                |token_hashes| {
                    token_hashes
                        .into_iter()
                        .map(TokenIdentifier::new_hash)
                        .collect()
                },
            )
        }
    }
}

pub(crate) fn upsert_token_identifiers(
    identifier_mode: &NFTIdentifierMode,
    owners_item_key: &str,
    token_identifiers: Vec<TokenIdentifier>,
) -> Result<(), NFTCoreError> {
    match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            let token_indices: Vec<u64> = token_identifiers
                .into_iter()
                .map(|token_identifier| {
                    token_identifier
                        .get_index()
                        .unwrap_or_revert_with(NFTCoreError::InvalidIdentifierMode)
                })
                .collect();
            upsert_dictionary_value_from_key(OWNED_TOKENS, owners_item_key, token_indices);
            Ok(())
        }
        NFTIdentifierMode::Hash => {
            let token_hashes: Vec<String> = token_identifiers
                .into_iter()
                .map(|token_identifier| {
                    token_identifier
                        .get_hash()
                        .unwrap_or_revert_with(NFTCoreError::InvalidIdentifierMode)
                })
                .collect();
            upsert_dictionary_value_from_key(OWNED_TOKENS, owners_item_key, token_hashes);
            Ok(())
        }
    }
}

pub(crate) fn get_burn_mode() -> BurnMode {
    let burn_mode: BurnMode = get_stored_value_with_user_errors::<u8>(
        BURN_MODE,
        NFTCoreError::MissingBurnMode,
        NFTCoreError::InvalidBurnMode,
    )
    .try_into()
    .unwrap_or_revert();
    burn_mode
}

pub(crate) fn is_token_burned(token_identifier: &TokenIdentifier) -> bool {
    get_dictionary_value_from_key::<()>(BURNT_TOKENS, &token_identifier.get_dictionary_item_key())
        .is_some()
}
