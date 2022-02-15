#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::{vec, vec::Vec};
use casper_types::account::Account;
use casper_types::{CLType, PublicKey, U256};
use core::mem::MaybeUninit;

use casper_contract::contract_api::runtime::revert;
use casper_contract::{
    contract_api::{self, runtime, storage, system},
    ext_ffi,
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::bytesrepr::Error;
use casper_types::{
    account::AccountHash,
    api_error,
    bytesrepr::{self, FromBytes, ToBytes},
    ApiError, BlockTime, CLTyped, CLValue, Key, URef, U512,
};

//use sha2::{Digest, Sha256};
//use blake3::Hasher;

pub const ARG_COLLECTION_NAME: &str = "collection_name";
pub const ARG_COLLECTION_SYMBOL: &str = "collection_symbol";
pub const ARG_TOTAL_TOKEN_SUPPLY: &str = "total_token_supply"; // <-- Think about if mutable or not...

pub const ARG_TOKEN_OWNER: &str = "token_owner";
pub const ARG_TOKEN_NAME: &str = "token_name";

pub const ARG_TOKEN_ID: &str = "token_id";
pub const ARG_TOKEN_RECEIVER: &str = "token_receiver";
pub const ARG_TOKEN_SENDER: &str = "token_sender";
pub const ARG_ALLOW_MINTING: &str = "allow_minting";
pub const ARG_PUBLIC_MINTING: &str = "public_minting";
pub const ARG_PUBLIC_KEY: &str = "public_key";
pub const ARG_TOKEN_META_DATA: &str = "token_meta_data";
pub const ARG_APPROVE_TRANSFER_FOR_PUBLIC_KEY: &str = "approve_transfer_for_public_key";

// STORAGE is the list of all NFTS
// Owners is a dictionary owner --> nfts
pub const STORAGE: &str = "storage";
//pub const OWNERS: &str = "owners";
pub const APPROVED_FOR_TRANSFER: &str = "approved_for_transfer";
pub const NUMBER_OF_MINTED_TOKENS: &str = "number_of_minted_tokens";

pub const INSTALLER: &str = "installer";
pub const CONTRACT_NAME: &str = "nft_contract";
pub const HASH_KEY_NAME: &str = "nft_contract_package";
pub const ACCESS_KEY_NAME: &str = "nft_contract_package_access";
pub const CONTRACT_VERSION: &str = "contract_version";
pub const COLLECTION_NAME: &str = "collection_name";
pub const COLLECTION_SYMBOL: &str = "collection_symbol";
pub const TOTAL_TOKEN_SUPPLY: &str = "total_token_supply";
pub const ALLOW_MINTING: &str = "allow_minting";
pub const PUBLIC_MINTING: &str = "public_minting";
pub const TOKEN_OWNERS: &str = "token_owners";
pub const TOKEN_META_DATA: &str = "token_meta_data";
pub const OWNED_TOKENS: &str = "owned_tokens";
pub const BURNT_TOKENS: &str = "burnt_tokens";

pub const ENTRY_POINT_INIT: &str = "init";
pub const ENTRY_POINT_SET_VARIABLES: &str = "set_variables";
pub const ENTRY_POINT_MINT: &str = "mint";
pub const ENTRY_POINT_BURN: &str = "burn";
pub const ENTRY_POINT_TRANSFER: &str = "transfer";
pub const ENTRY_POINT_APPROVE_TRANSFER: &str = "approve_transfer";
pub const ENTRY_POINT_BALANCE_OF: &str = "balance_of";
pub const ENTRY_POINT_COLLECTION_NAME: &str = "collection_name";
pub const ENTRY_POINT_SET_ALLOW_MINTING: &str = "set_allow_minting";

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum NFTCoreError {
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
    FailedToSerializeMetaData = 19,
    MissingAccount = 20,
    MissingMintingStatus = 21,
    InvalidMintingStatus = 22,
    MissingCollectionSymbol = 23,
    InvalidCollectionSymbol = 24,
    MissingTotalTokenSupply = 25,
    InvalidTotalTokenSupply = 26,
    MissingTokenID = 27,
    InvalidTokenID = 28,
    MissingTokenOwners = 29,
    MissingPublicKey = 30,
    InvalidPublicKey = 31,
    TokenSupplyDepleted = 32,
    MissingOwnedTokensDictionary = 33,
    TokenAlreadyBelongsToMinterFatal = 34,
    FatalTokenIDDuplication = 35,
    InvalidMinter = 36,
    MissingPublicMinting = 37,
    InvalidPublicMinting = 38,
    MissingInstallerKey = 39,
    FailedToConvertToAccountHash = 40,
    InvalidBurner = 41,
    PreviouslyBurntToken = 42,
    MissingAllowMinting = 43,
    InvalidAllowMinting = 44,
    MissingNumberOfMintedTokens = 45,
    InvalidNumberOfMintedTokens = 46,
    MissingTokenMetaData = 47,
    InvalidTokenMetaData = 48,
    MissingApprovedPublicKey = 49,
    InvalidApprovedPublicKey = 50,
    MissingApprovedTokensDictionary = 51,
    TokenAlreadyApproved = 52,
}

impl From<NFTCoreError> for ApiError {
    fn from(e: NFTCoreError) -> Self {
        ApiError::User(e as u16)
    }
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

    let collection_name: String = get_named_arg_with_user_errors(
        ARG_COLLECTION_NAME,
        NFTCoreError::MissingCollectionName,
        NFTCoreError::InvalidCollectionName,
    )
    .unwrap_or_revert();

    let collection_symbol: String = get_named_arg_with_user_errors(
        ARG_COLLECTION_SYMBOL,
        NFTCoreError::MissingCollectionSymbol,
        NFTCoreError::InvalidCollectionSymbol,
    )
    .unwrap_or_revert();

    let total_token_supply: U256 = get_named_arg_with_user_errors(
        ARG_TOTAL_TOKEN_SUPPLY,
        NFTCoreError::MissingTotalTokenSupply,
        NFTCoreError::InvalidTotalTokenSupply,
    )
    .unwrap_or_revert();

    let allow_minting: bool = get_named_arg_with_user_errors(
        ARG_ALLOW_MINTING,
        NFTCoreError::MissingMintingStatus,
        NFTCoreError::InvalidMintingStatus,
    )
    .unwrap_or_revert();

    let public_minting: bool = get_named_arg_with_user_errors(
        ARG_PUBLIC_MINTING,
        NFTCoreError::MissingPublicMinting,
        NFTCoreError::InvalidPublicMinting,
    )
    .unwrap_or_revert();

    let collection_name_uref = storage::new_uref(collection_name);
    let collection_symbol_uref = storage::new_uref(collection_symbol);
    let total_token_supply_uref = storage::new_uref(total_token_supply);
    let allow_minting_uref = storage::new_uref(allow_minting);
    let public_minting_uref = storage::new_uref(public_minting);

    let token_owners_seed_uref = storage::new_dictionary(TOKEN_OWNERS)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);

    let token_meta_data_seed_uref = storage::new_dictionary(TOKEN_META_DATA)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);

    let owned_tokens_seed_uref = storage::new_dictionary(OWNED_TOKENS)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);

    let approved_for_transfer_seed_uref = storage::new_dictionary(APPROVED_FOR_TRANSFER)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);

    let burn_tokens_uref = storage::new_dictionary(BURNT_TOKENS)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);

    let number_of_minted_tokens = storage::new_uref(U256::zero());

    runtime::put_key(NUMBER_OF_MINTED_TOKENS, number_of_minted_tokens.into());
    runtime::put_key(TOKEN_OWNERS, token_owners_seed_uref.into());
    runtime::put_key(TOKEN_META_DATA, token_meta_data_seed_uref.into());
    runtime::put_key(OWNED_TOKENS, owned_tokens_seed_uref.into());
    runtime::put_key(
        APPROVED_FOR_TRANSFER,
        approved_for_transfer_seed_uref.into(),
    );
    runtime::put_key(BURNT_TOKENS, burn_tokens_uref.into());
    runtime::put_key(COLLECTION_NAME, collection_name_uref.into());
    runtime::put_key(COLLECTION_SYMBOL, collection_symbol_uref.into());
    runtime::put_key(TOTAL_TOKEN_SUPPLY, total_token_supply_uref.into());
    runtime::put_key(ALLOW_MINTING, allow_minting_uref.into());
    runtime::put_key(PUBLIC_MINTING, public_minting_uref.into());
}

#[no_mangle]
pub fn set_variables() {
    // TODO: check for anything that would break invariants here.
    // anything we set here is mutable to the caller,
    // make sure that things that shouldn't be mutable aren't
    let installer = get_account_hash_with_user_errors(
        INSTALLER,
        NFTCoreError::MissingInstaller,
        NFTCoreError::InvalidInstaller,
    );

    if installer != runtime::get_caller() {
        runtime::revert(NFTCoreError::InvalidAccount);
    }

    if let Some(allow_minting) = get_optional_named_arg_with_user_errors::<bool>(
        ARG_ALLOW_MINTING,
        NFTCoreError::MissingAllowMinting, // Think about if these are appropriate errors...
        NFTCoreError::InvalidAllowMinting,
    ) {
        let allow_minting_uref = get_uref_with_user_errors(
            ALLOW_MINTING,
            NFTCoreError::MissingAllowMinting,
            NFTCoreError::MissingAllowMinting,
        );
        storage::write(allow_minting_uref, allow_minting);
    }
}

#[no_mangle]
pub fn mint() {
    let (total_token_supply, _): (U256, URef) = get_stored_value_with_user_errors(
        TOTAL_TOKEN_SUPPLY,
        NFTCoreError::MissingTotalTokenSupply,
        NFTCoreError::InvalidTotalTokenSupply,
    );
    let (mut number_of_minted_tokens, number_of_minted_tokens_uref) =
        get_stored_value_with_user_errors::<U256>(
            NUMBER_OF_MINTED_TOKENS,
            NFTCoreError::MissingNumberOfMintedTokens,
            NFTCoreError::InvalidNumberOfMintedTokens,
        );

    // Revert if we do not have any more tokens to mint
    if number_of_minted_tokens >= total_token_supply {
        runtime::revert(NFTCoreError::TokenSupplyDepleted);
    }

    let minter_public_key: PublicKey = get_named_arg_with_user_errors(
        ARG_PUBLIC_KEY,
        NFTCoreError::MissingPublicKey,
        NFTCoreError::InvalidPublicKey,
    )
    .unwrap_or_revert();

    let token_meta_data: String = get_named_arg_with_user_errors(
        ARG_TOKEN_META_DATA,
        NFTCoreError::MissingTokenMetaData,
        NFTCoreError::InvalidTokenMetaData,
    )
    .unwrap_or_revert();

    let token_meta_data_seed_uref = get_uref_with_user_errors(
        TOKEN_META_DATA,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );

    //Store the new value
    storage::dictionary_put(
        token_meta_data_seed_uref,
        &number_of_minted_tokens.to_string(),
        token_meta_data.clone(),
    );

    let minter_account_hash = minter_public_key.to_account_hash();
    let installer_account_hash = runtime::get_key(INSTALLER)
        .unwrap_or_revert_with(NFTCoreError::MissingInstallerKey)
        .into_account()
        .unwrap_or_revert_with(NFTCoreError::FailedToConvertToAccountHash);
    let current_caller = runtime::get_caller();
    let (public_minting, _public_minting_uref) = get_stored_value_with_user_errors::<bool>(
        PUBLIC_MINTING,
        NFTCoreError::MissingPublicMinting,
        NFTCoreError::InvalidPublicMinting,
    );

    if public_minting {
        if current_caller != minter_account_hash {
            runtime::revert(NFTCoreError::InvalidMinter)
        }
    } else {
        if installer_account_hash != current_caller {
            runtime::revert(NFTCoreError::InvalidMinter)
        }
    }

    //Revert if owner already exists, or store minter as owner under token_id
    match get_dictionary_value_from_key::<PublicKey>(
        TOKEN_OWNERS,
        &number_of_minted_tokens.to_string(),
    ) {
        (Some(_), token_owners_seed_uref) => runtime::revert(NFTCoreError::FatalTokenIDDuplication),
        (None, token_owners_seed_uref) => storage::dictionary_put(
            token_owners_seed_uref,
            &number_of_minted_tokens.to_string(),
            minter_public_key.clone(),
        ),
    };

    let (mut maybe_owned_tokens, owned_tokens_seed_uref) =
        get_dictionary_value_from_key::<Vec<U256>>(OWNED_TOKENS, &minter_public_key.to_string());

    let updated_owned_tokens = match maybe_owned_tokens {
        Some(mut owned_tokens) => {
            if owned_tokens.contains(&number_of_minted_tokens) {
                runtime::revert(NFTCoreError::FatalTokenIDDuplication); //<<--- change to correct error
            }

            owned_tokens.push(number_of_minted_tokens);
            owned_tokens
        }
        None => vec![number_of_minted_tokens],
    };

    //Store the new value
    storage::dictionary_put(
        owned_tokens_seed_uref,
        &minter_public_key.to_string(),
        updated_owned_tokens.clone(),
    );

    // Increment number_of_minted_tokens by one
    number_of_minted_tokens = number_of_minted_tokens + U256::one();
    storage::write(number_of_minted_tokens_uref, number_of_minted_tokens);
}

#[no_mangle]
fn burn() {
    let token_id: U256 = get_named_arg_with_user_errors(
        ARG_TOKEN_ID,
        NFTCoreError::MissingTokenID,
        NFTCoreError::InvalidTokenID,
    )
    .unwrap_or_revert();

    let burner_public_key: PublicKey = get_named_arg_with_user_errors(
        ARG_PUBLIC_KEY,
        NFTCoreError::MissingPublicKey,
        NFTCoreError::InvalidPublicKey,
    )
    .unwrap_or_revert();

    //Revert if burner is not caller
    if runtime::get_caller() != burner_public_key.to_account_hash() {
        runtime::revert(NFTCoreError::InvalidBurner)
    }

    // Revert if burner is not token_owner
    match get_dictionary_value_from_key::<PublicKey>(TOKEN_OWNERS, &token_id.to_string()) {
        (Some(token_owner_public_key), _) => {
            if token_owner_public_key != burner_public_key {
                runtime::revert(NFTCoreError::InvalidTokenOwner)
            }
        }
        (None, _) => runtime::revert(NFTCoreError::InvalidTokenID),
    };

    match get_dictionary_value_from_key::<()>(BURNT_TOKENS, &token_id.to_string()) {
        (Some(_), _) => runtime::revert(NFTCoreError::PreviouslyBurntToken),
        (None, burnt_tokens_seed_uref) => {
            storage::dictionary_put(burnt_tokens_seed_uref, &token_id.to_string(), ())
        }
    }
}

#[no_mangle]
fn transfer() {
    let token_id: U256 = get_named_arg_with_user_errors(
        ARG_TOKEN_ID,
        NFTCoreError::MissingTokenID,
        NFTCoreError::InvalidTokenID,
    )
    .unwrap_or_revert();

    if let (Some(_), _) = get_dictionary_value_from_key::<()>(BURNT_TOKENS, &token_id.to_string()) {
        runtime::revert(NFTCoreError::PreviouslyBurntToken);
    }

    let sender_public_key: PublicKey = get_named_arg_with_user_errors(
        ARG_TOKEN_SENDER,
        NFTCoreError::MissingPublicKey,
        NFTCoreError::InvalidPublicKey,
    )
    .unwrap_or_revert();

    let receiver_public_key: PublicKey = get_named_arg_with_user_errors(
        ARG_TOKEN_RECEIVER,
        NFTCoreError::MissingPublicKey,
        NFTCoreError::InvalidPublicKey,
    )
    .unwrap_or_revert();

    match get_dictionary_value_from_key::<PublicKey>(TOKEN_OWNERS, &token_id.to_string()) {
        (Some(token_actual_owner_public_key), token_owners_seed_uref) => {
            if token_actual_owner_public_key != sender_public_key {
                runtime::revert(NFTCoreError::InvalidTokenOwner)
            }

            storage::dictionary_put(
                token_owners_seed_uref,
                &token_id.to_string(),
                receiver_public_key.clone(),
            );
        }
        (None, _) => runtime::revert(NFTCoreError::InvalidTokenID),
    }

    match get_dictionary_value_from_key::<Vec<U256>>(OWNED_TOKENS, &sender_public_key.to_string()) {
        (Some(mut owned_tokens), owned_tokens_seed_uref) => {
            if let Some(id) = owned_tokens.iter().position(|id| *id == token_id) {
                owned_tokens.remove(id);
            } else {
                runtime::revert(NFTCoreError::InvalidTokenOwner)
            }

            storage::dictionary_put(
                owned_tokens_seed_uref,
                &sender_public_key.to_string(),
                owned_tokens,
            );
        }
        (None, owned_tokens_seed_uref) => runtime::revert(NFTCoreError::InvalidTokenID),
    }

    match get_dictionary_value_from_key::<Vec<U256>>(OWNED_TOKENS, &receiver_public_key.to_string())
    {
        (Some(mut owned_tokens), owned_tokens_seed_uref) => {
            if let Some(_) = owned_tokens.iter().position(|id| *id == token_id) {
                runtime::revert(NFTCoreError::FatalTokenIDDuplication)
            } else {
                owned_tokens.push(token_id);
            }

            storage::dictionary_put(
                owned_tokens_seed_uref,
                &receiver_public_key.to_string(),
                owned_tokens,
            );
        }
        (None, owned_tokens_seed_uref) => {
            let owned_tokens = vec![token_id];
            storage::dictionary_put(
                owned_tokens_seed_uref,
                &receiver_public_key.to_string(),
                owned_tokens,
            );
        }
    }
}

#[no_mangle]
fn approve_transfer() {
    let token_owner = get_named_arg_with_user_errors::<PublicKey>(
        ARG_TOKEN_OWNER,
        NFTCoreError::MissingTokenOwner,
        NFTCoreError::InvalidTokenOwner,
    )
    .unwrap_or_revert();

    //Revert if incorrect caller
    if token_owner.to_account_hash() != runtime::get_caller() {
        runtime::revert(NFTCoreError::InvalidAccount);
    }

    let token_id = get_named_arg_with_user_errors::<U256>(
        ARG_TOKEN_ID,
        NFTCoreError::MissingTokenID,
        NFTCoreError::InvalidTokenID,
    )
    .unwrap_or_revert();

    if let (Some(_), _) = get_dictionary_value_from_key::<()>(BURNT_TOKENS, &token_id.to_string()) {
        runtime::revert(NFTCoreError::PreviouslyBurntToken);
    }

    let (number_of_minted_tokens, _) = get_stored_value_with_user_errors::<U256>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    // Revert if token_id is out of bounds
    if token_id >= number_of_minted_tokens {
        runtime::revert(NFTCoreError::InvalidTokenID);
    }

    let token_owner = get_named_arg_with_user_errors::<PublicKey>(
        ARG_TOKEN_OWNER,
        NFTCoreError::MissingTokenOwner,
        NFTCoreError::InvalidTokenOwner,
    )
    .unwrap_or_revert();

    let approve_for_public_key = get_named_arg_with_user_errors::<PublicKey>(
        ARG_APPROVE_TRANSFER_FOR_PUBLIC_KEY,
        NFTCoreError::MissingApprovedPublicKey,
        NFTCoreError::InvalidApprovedPublicKey,
    )
    .unwrap_or_revert();

    if token_owner == approve_for_public_key {
        runtime::revert(NFTCoreError::InvalidAccount); //Do we need a better error here?
    }

    match get_dictionary_value_from_key::<PublicKey>(APPROVED_FOR_TRANSFER, &token_id.to_string()) {
        (Some(approved), _) => runtime::revert(NFTCoreError::TokenAlreadyApproved),
        (None, approved_uref) => {
            storage::dictionary_put(approved_uref, &token_id.to_string(), approve_for_public_key);
        }
    }
}

// #[no_mangle]
// fn set_allow_minting() {
//     // Only installing account should be able to mutate allow_minting.
//     // So we revert if caller is not the installer.
//     let (installer, _) = get_stored_value_with_user_errors::<AccountHash>(
//         INSTALLER,
//         NFTCoreError::MissingInstaller,
//         NFTCoreError::InvalidInstaller,
//     );
//     if runtime::get_caller() != installer {
//         runtime::revert(NFTCoreError::InvalidAccount);
//     }

//     let value = runtime::get_named_arg::<bool>(ARG_ALLOW_MINTING);

//     let (_, uref) = get_stored_value_with_user_errors::<bool>(
//         ALLOW_MINTING,
//         NFTCoreError::MissingAllowMinting,
//         NFTCoreError::InvalidAllowMinting,
//     );
//     storage::write(uref, value);
// }

////////////////////////////////////////////////////////////////////////////////
//////////////////////////////// Helper methods ////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

fn get_dictionary_value_from_key<T: CLTyped + FromBytes>(
    dictionary_name: &str,
    key: &str,
) -> (Option<T>, URef) {
    let seed_uref = get_uref_with_user_errors(
        dictionary_name,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );

    match storage::dictionary_get::<T>(seed_uref, key) {
        Ok(value) => (value, seed_uref),
        Err(error) => runtime::revert(error),
    }
}

fn get_stored_value_with_user_errors<T: CLTyped + FromBytes>(
    name: &str,
    missing: NFTCoreError,
    invalid: NFTCoreError,
) -> (T, URef) {
    let uref = get_uref_with_user_errors(name, missing, invalid);
    (read_with_user_errors(uref, missing, invalid), uref)
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

pub fn get_optional_named_arg_with_user_errors<T: FromBytes>(
    name: &str,
    missing: NFTCoreError,
    invalid: NFTCoreError,
) -> Option<T> {
    match get_named_arg_with_user_errors(name, missing, invalid) {
        Ok(val) => val,
        Err(err @ NFTCoreError::InvalidInstaller) => runtime::revert(err),
        Err(e) => runtime::revert(e),
    }
}

pub fn get_named_arg_with_user_errors<T: FromBytes>(
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

// Todo: write tests
// Test1: check to see if created account exists
// Test2: check to see if not-created account exists

fn read_optional_account_with_user_errors(key: Key, invalid: NFTCoreError) -> Option<Account> {
    let (key_ptr, key_size, _bytes) = to_ptr(key);

    let value_size = {
        let mut value_size = MaybeUninit::uninit();
        let ret = unsafe { ext_ffi::casper_read_value(key_ptr, key_size, value_size.as_mut_ptr()) };
        match api_error::result_from(ret) {
            Ok(_) => unsafe { value_size.assume_init() },
            Err(ApiError::ValueNotFound) => return None,
            Err(e) => runtime::revert(e),
        }
    };

    let value_bytes = read_host_buffer(value_size).unwrap_or_revert();
    match bytesrepr::deserialize::<Account>(value_bytes) {
        Ok(account) => return Some(account),
        Err(_) => {
            runtime::revert(invalid);
        }
    }
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

// #[no_mangle]
// fn burn() {
//     //From token_owner get token_ids list and remove token from there
//     // Get STORAGE and remove NFT from there.let
//     let token_owner: PublicKey = runtime::get_named_arg(ARG_TOKEN_OWNER);
//     let token_id: U256 = runtime::get_named_arg(ARG_TOKEN_ID);

//     let owners_seed_uref = get_uref_with_user_errors(
//         OWNERS,
//         NFTCoreError::MissingStorageUref,
//         NFTCoreError::InvalidStorageUref,
//     );

//     let mut token_ids: TokenIDs =
//         storage::dictionary_get::<TokenIDs>(owners_seed_uref, &token_owner.to_string())
//             .unwrap_or_revert_with(NFTCoreError::FailedToAccessOwnershipDictionary)
//             .unwrap_or_revert_with(NFTCoreError::InvalidTokenOwner);

//     //Remove the token_id from the token_ids of the owner
//     let index = token_ids
//         .iter()
//         .position(|id| *id == token_id)
//         .unwrap_or_revert_with(NFTCoreError::InvalidTokenOwner);

//     token_ids.remove(index);
//     storage::dictionary_put(owners_seed_uref, &token_owner.to_string(), token_ids);

//     let storage_seed_uref = get_uref_with_user_errors(
//         STORAGE,
//         NFTCoreError::MissingStorageUref,
//         NFTCoreError::InvalidStorageUref,
//     );

//     //Remove from storage dictionary
//     storage::dictionary_put(
//         storage_seed_uref,
//         &token_id.to_string(),
//         Option::<U256>::None,
//     );
// }

// #[no_mangle]
// fn mint() {
//     // Should the NFT immediately belong to the given owner, or should it
//     // belong to the contract and then be transferred to an owner at a later time?
//     let token_owner: PublicKey = runtime::get_named_arg(ARG_TOKEN_OWNER);

//     let token_meta: String = runtime::get_named_arg(ARG_TOKEN_META);

//     let owner_account_hash = token_owner.to_account_hash();
//     let nft = NFT::new_without_id(token_owner.clone(), token_name, token_meta);

//     let token_id = nft.id();

//     let storage_seed_uref = get_uref_with_user_errors(
//         STORAGE,
//         NFTCoreError::MissingStorageUref,
//         NFTCoreError::InvalidStorageUref,
//     );

//     // If nft with id already exists we revert and return with error
//     if let Some(_) = storage::dictionary_get::<NFT>(storage_seed_uref, &nft.id().to_string())
//         .unwrap_or_revert_with(NFTCoreError::FailedToAccessStorageDictionary)
//     {
//         runtime::revert(NFTCoreError::DuplicateMinted);
//     }

//     let owners_seed_uref = get_uref_with_user_errors(
//         OWNERS,
//         NFTCoreError::MissingStorageUref,
//         NFTCoreError::InvalidStorageUref,
//     );

//     let current_token_ids: TokenIDs =
//         match storage::dictionary_get::<TokenIDs>(owners_seed_uref, &token_owner.to_string())
//             .unwrap_or_revert_with(NFTCoreError::FailedToAccessOwnershipDictionary)
//         {
//             Some(mut token_ids) => {
//                 if token_ids.contains(&token_id) {
//                     runtime::revert(NFTCoreError::DuplicateMinted)
//                 }
//                 token_ids.push(token_id);
//                 token_ids
//             }
//             None => vec![token_id],
//         };

//     //<ID,NFT>

//     //128
//     //0..128
//     // List of token ids
//     //<usize,NFT>

//     storage::dictionary_put(storage_seed_uref, &token_id.to_string(), nft);
//     storage::dictionary_put(
//         owners_seed_uref,
//         &owner_account_hash.to_string(),
//         current_token_ids,
//     );
// }

// // balance_of implies an amount and NFTs are not amount-based.
// #[no_mangle]
// fn balance_of() {
//     let token_owner: PublicKey = runtime::get_named_arg(ARG_TOKEN_OWNER);

//     let owners_seed_uref = get_uref_with_user_errors(
//         OWNERS,
//         NFTCoreError::MissingStorageUref,
//         NFTCoreError::InvalidStorageUref,
//     );

//     let current_token_ids: TokenIDs =
//         match storage::dictionary_get::<TokenIDs>(owners_seed_uref, &token_owner.to_string())
//             .unwrap_or_revert_with(NFTCoreError::FailedToAccessOwnershipDictionary)
//         {
//             Some(token_ids) => token_ids,
//             None => vec![],
//         };

//     let current_token_ids = CLValue::from_t(current_token_ids)
//         .unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);
//     runtime::ret(current_token_ids);
// }

// All NFTs dictionary: <U256,NFT>
// Owners dictionary: <Account,Vec<U256>>
// U256 numbers of tokens.

// We mint one: 24 --> new token

// #[no_mangle]
// fn get_tokens_with_collection_name() {
//     let collection_name: String = runtime::get_named_arg(ARG_COLLECTION_NAME);
//     //
//     let storage_seed_uref = get_uref_with_user_errors(
//         STORAGE,
//         NFTCoreError::MissingStorageUref,
//         NFTCoreError::InvalidStorageUref,
//     );

//     //runtime::get_key(name)
//     // Here we will need a list of all nfts.
//     //let all_tokens = storage::dictionary_get(storage_seed_uref, dictionary_item_key)
// }

// #[no_mangle]
// pub fn transfer() {
//     let token_owner: PublicKey = runtime::get_named_arg(ARG_TOKEN_OWNER);
//     let token_receiver: PublicKey = runtime::get_named_arg(ARG_TOKEN_RECEIVER);
//     let token_id: String = runtime::get_named_arg(ARG_TOKEN_ID);

//     let owners_seed_uref = get_uref_with_user_errors(
//         OWNERS,
//         NFTCoreError::MissingStorageUref,
//         NFTCoreError::InvalidStorageUref,
//     );

//     let mut owner_token_ids =
//         storage::dictionary_get::<TokenIDs>(owners_seed_uref, &token_owner.to_string())
//             .unwrap_or_revert_with(NFTCoreError::FailedToAccessOwnershipDictionary)
//             .unwrap_or_revert_with(NFTCoreError::InvalidTokenOwner); //<-- Better error?

//     // Check ownser actually owns the token with id. IF not we revert with error
//     let index = owner_token_ids
//         .iter()
//         .position(|id| *id == token_id)
//         .unwrap_or_revert_with(NFTCoreError::InvalidTokenOwner);

//     //Remove token from owner and modify
//     let _ = owner_token_ids.remove(index);
//     storage::dictionary_put(owners_seed_uref, &token_owner.to_string(), owner_token_ids);

//     // Check if receiver account exists and revert if not.
//     let account_key: Key = token_owner.to_account_hash().into();
//     if read_optional_account_with_user_errors(account_key, NFTCoreError::InvalidAccount).is_none() {
//         runtime::revert(NFTCoreError::MissingAccount);
//     }

//     let receiver_tokens =
//         match storage::dictionary_get::<TokenIDs>(owners_seed_uref, &token_receiver.to_string())
//             .unwrap_or_revert_with(NFTCoreError::FailedToAccessOwnershipDictionary)
//         {
//             Some(mut token_ids) => {
//                 // Add token to receiver token_ids
//                 // Using a Vec to represent token_ids is not ideal as a token_id should
//                 // only appear once. Logically a HashSet would be a better choice. However,
//                 // this may not be ideal from a gas cost perspective.

//                 if token_ids.contains(&token_id) {
//                     runtime::revert(NFTCoreError::DuplicateMinted); //<--- Create new error type!!
//                 }

//                 token_ids.push(token_id);
//                 token_ids
//             }
//             None => {
//                 // Create new token_ids vec
//                 vec![token_id]
//             }
//         };

//     storage::dictionary_put(
//         owners_seed_uref,
//         &token_receiver.to_string(),
//         receiver_tokens,
//     );
// }
