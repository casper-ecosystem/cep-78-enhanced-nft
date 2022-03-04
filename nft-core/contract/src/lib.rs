#![no_std]

extern crate alloc;

//use alloc::format;
use alloc::string::{String, ToString};
use alloc::{vec, vec::Vec};
use casper_types::U256;
use core::mem::MaybeUninit;

use casper_contract::{
    contract_api::{self, runtime, storage},
    ext_ffi,
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    account::AccountHash,
    api_error,
    bytesrepr::{self, FromBytes, ToBytes},
    ApiError, CLTyped, CLValue, Key, URef,
};

//use sha2::{Digest, Sha256};
//use blake3::Hasher;

pub const ARG_COLLECTION_NAME: &str = "collection_name";
pub const ARG_COLLECTION_SYMBOL: &str = "collection_symbol";
pub const ARG_TOTAL_TOKEN_SUPPLY: &str = "total_token_supply"; // <-- Think about if mutable or not...

pub const ARG_TOKEN_OWNER: &str = "token_owner";
pub const ARG_TOKEN_NAME: &str = "token_name";

pub const ARG_TOKEN_ID: &str = "token_id";
pub const ARG_TO_ACCOUNT_HASH: &str = "to_account_hash";
pub const ARG_FROM_ACCOUNT_HASH: &str = "from_account_hash";
pub const ARG_ALLOW_MINTING: &str = "allow_minting";
pub const ARG_PUBLIC_MINTING: &str = "public_minting";
pub const ARG_ACCOUNT_HASH: &str = "account_hash";
pub const ARG_TOKEN_META_DATA: &str = "token_meta_data";
pub const ARG_APPROVE_TRANSFER_FOR_ACCOUNT_HASH: &str = "approve_transfer_for_account_hash";
pub const ARG_APPROVE_ALL: &str = "approve_all";
pub const ARG_OPERATOR: &str = "operator";

// STORAGE is the list of all NFTS
// Owners is a dictionary owner --> nfts
//pub const STORAGE: &str = "storage";
//pub const OWNERS: &str = "owners";
pub const APPROVED_FOR_TRANSFER: &str = "approved_for_transfer"; //Change name?
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
pub const ENTRY_POINT_APPROVE: &str = "approve";
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
    MissingAccountHash = 30,
    InvalidAccountHash = 31,
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
    MissingApprovedAccountHash = 49,
    InvalidApprovedAccountHash = 50,
    MissingApprovedTokensDictionary = 51,
    TokenAlreadyApproved = 52,
    MissingApproveAll = 53,
    InvalidApproveAll = 54,
    MissingOperator = 55,
    InvalidOperator = 56,
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
    let number_of_minted_tokens = storage::new_uref(U256::zero());

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

// set_variables allows the user to set any variable or any combination of variables simultaneously.
// set variables defines what variables are mutable and immutable.
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

    // Only the installing account can change the mutable variables.
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

    let installer_account_hash = runtime::get_key(INSTALLER)
        .unwrap_or_revert_with(NFTCoreError::MissingInstallerKey)
        .into_account()
        .unwrap_or_revert_with(NFTCoreError::FailedToConvertToAccountHash)
        .to_string();

    let caller = runtime::get_caller().to_string();

    let (public_minting, _public_minting_uref) = get_stored_value_with_user_errors::<bool>(
        PUBLIC_MINTING,
        NFTCoreError::MissingPublicMinting,
        NFTCoreError::InvalidPublicMinting,
    );

    // Revert if public minting is disallowed and caller is not installer
    if !public_minting && caller != installer_account_hash {
        runtime::revert(NFTCoreError::InvalidMinter)
    }

    // Get token metadata
    let token_meta_data: String = get_named_arg_with_user_errors(
        ARG_TOKEN_META_DATA,
        NFTCoreError::MissingTokenMetaData,
        NFTCoreError::InvalidTokenMetaData,
    )
    .unwrap_or_revert();

    // Set token_meta data dictionary and revert if token_metadata already exists (unnecessary check??)
    match get_dictionary_value_from_key::<String>(
        TOKEN_META_DATA,
        &number_of_minted_tokens.to_string(),
    ) {
        (Some(_), _) => runtime::revert(NFTCoreError::FatalTokenIDDuplication),
        (None, token_meta_data_seed_uref) => storage::dictionary_put(
            token_meta_data_seed_uref,
            &number_of_minted_tokens.to_string(),
            token_meta_data,
        ),
    }

    //Revert if owner already exists (fatal error of contract implementation), or store minter as owner under token_id
    match get_dictionary_value_from_key::<String>(
        TOKEN_OWNERS,
        &number_of_minted_tokens.to_string(),
    ) {
        (Some(_), _) => runtime::revert(NFTCoreError::FatalTokenIDDuplication),
        (None, token_owners_seed_uref) => storage::dictionary_put(
            token_owners_seed_uref,
            &number_of_minted_tokens.to_string(),
            caller.clone(),
        ),
    };

    // Update owned tokens dictionary
    let (maybe_owned_tokens, owned_tokens_seed_uref) =
        get_dictionary_value_from_key::<Vec<U256>>(OWNED_TOKENS, &caller);

    let updated_owned_tokens = match maybe_owned_tokens {
        Some(mut owned_tokens) => {
            if owned_tokens.contains(&number_of_minted_tokens) {
                runtime::revert(NFTCoreError::FatalTokenIDDuplication); //<<---better error?
            }

            owned_tokens.push(number_of_minted_tokens);
            owned_tokens
        }
        None => vec![number_of_minted_tokens],
    };

    //Store the udated owned tokens vec in dictionary
    storage::dictionary_put(owned_tokens_seed_uref, &caller, updated_owned_tokens);

    // Increment number_of_minted_tokens by one
    number_of_minted_tokens += U256::one();
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

    let caller: String = runtime::get_caller().to_string();

    // Revert if caller is not token_owner. This seems to be the only check we need to do.
    match get_dictionary_value_from_key::<String>(TOKEN_OWNERS, &token_id.to_string()) {
        (Some(token_owner_account_hash), _) => {
            if token_owner_account_hash != caller {
                runtime::revert(NFTCoreError::InvalidTokenOwner)
            }
        }
        (None, _) => runtime::revert(NFTCoreError::InvalidTokenID),
    };

    // It makes sense to keep this token as owned by the caller. It just happens that the caller
    // owns a burnt token. That's all. Similarly, we should probably also not change the owned_tokens
    // dictionary.

    // Mark the token as burnt by adding the token_id to the burnt tokens dictionary.
    match get_dictionary_value_from_key::<()>(BURNT_TOKENS, &token_id.to_string()) {
        (Some(_), _) => runtime::revert(NFTCoreError::PreviouslyBurntToken),
        (None, burnt_tokens_seed_uref) => {
            storage::dictionary_put(burnt_tokens_seed_uref, &token_id.to_string(), ())
        }
    }

    // Should we also update approved dictionary?
}

// approve marks a token as approved for transfer by an account
#[no_mangle]
fn approve() {
    let caller = runtime::get_caller().to_string();
    let token_id = get_named_arg_with_user_errors::<U256>(
        ARG_TOKEN_ID,
        NFTCoreError::MissingTokenID,
        NFTCoreError::InvalidTokenID,
    )
    .unwrap_or_revert();

    let (number_of_minted_tokens, _) = get_stored_value_with_user_errors::<U256>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    // Revert if token_id is out of bounds
    if token_id >= number_of_minted_tokens {
        runtime::revert(NFTCoreError::InvalidTokenID);
    }

    let token_owner =
        match get_dictionary_value_from_key::<String>(TOKEN_OWNERS, &token_id.to_string()) {
            (Some(token_owner), _) => token_owner,
            (None, _) => runtime::revert(NFTCoreError::InvalidAccountHash),
        };

    // Revert if caller is not the token_owner.
    if token_owner != caller {
        runtime::revert(NFTCoreError::InvalidAccountHash);
    }

    // We assume a burnt token cannot be approved
    if let (Some(_), _) = get_dictionary_value_from_key::<()>(BURNT_TOKENS, &token_id.to_string()) {
        runtime::revert(NFTCoreError::PreviouslyBurntToken);
    }

    let approve_for_account_hash = get_named_arg_with_user_errors::<String>(
        ARG_APPROVE_TRANSFER_FOR_ACCOUNT_HASH,
        NFTCoreError::MissingApprovedAccountHash,
        NFTCoreError::InvalidApprovedAccountHash,
    )
    .unwrap_or_revert();

    // If token_owner tries to approve themselves that's probably a mistake and we revert.
    if token_owner == approve_for_account_hash {
        runtime::revert(NFTCoreError::InvalidAccount); //Do we need a better error here? ::AlreadyOwner ??
    }

    let approved_uref = get_uref_with_user_errors(
        APPROVED_FOR_TRANSFER,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );

    storage::dictionary_put(
        approved_uref,
        &token_id.to_string(),
        Some(approve_for_account_hash),
    );
}

#[no_mangle]
fn set_approval_for_all() {
    // If approve_all is true we approve operator for all caller_owned tokens.
    // If false, it's not clear to me.
    let approve_all = get_named_arg_with_user_errors::<bool>(
        ARG_APPROVE_ALL,
        NFTCoreError::MissingApproveAll,
        NFTCoreError::InvalidApproveAll,
    )
    .unwrap_or_revert();

    let operator = get_named_arg_with_user_errors::<String>(
        ARG_OPERATOR,
        NFTCoreError::MissingOperator,
        NFTCoreError::InvalidOperator,
    )
    .unwrap_or_revert();

    let caller = runtime::get_caller().to_string();
    let approved_uref = get_uref_with_user_errors(
        APPROVED_FOR_TRANSFER,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );

    if let (Some(owned_tokens), _) =
        get_dictionary_value_from_key::<Vec<U256>>(APPROVED_FOR_TRANSFER, &caller)
    {
        // Depending on approve_all we either approve all or disapprove all.
        for t in owned_tokens {
            if approve_all {
                storage::dictionary_put(approved_uref, &t.to_string(), Some(operator.clone()));
            } else {
                storage::dictionary_put(approved_uref, &t.to_string(), Option::<String>::None);
            }
        }
    };
}

#[no_mangle]
fn transfer() {
    ////////////////////////////
    // ERC721 transfer logic
    // throw if _from is not _owner
    // throw is _token_id is not a valid token id.
    // throw if _to is the zero address  This probably is there to prevent the transfer function to be used to burn tokens.

    // Proceed with transfer if
    //  caller == owner || caller is approved || caller is approved address
    // If not we throw error
    //////////////////////////////

    // Stragegy
    // Is token burnt?
    // get token_owner
    // Is caller = spender token_owner? If yes go ahead and do the transfer
    // Is caller approved?
    // The idea is that you can transfer the token to the receiver if the caller is either the token owner or if you are approved

    //caller, sender, receiver

    // if caller == owner we can transfer
    // If caller != owner but

    // sender != caller && caller is not approved

    // Get token_id argument
    let token_id: U256 = get_named_arg_with_user_errors(
        ARG_TOKEN_ID,
        NFTCoreError::MissingTokenID,
        NFTCoreError::InvalidTokenID,
    )
    .unwrap_or_revert();

    // We assume we cannot transfer burnt tokens
    if let (Some(_), _) = get_dictionary_value_from_key::<()>(BURNT_TOKENS, &token_id.to_string()) {
        runtime::revert(NFTCoreError::PreviouslyBurntToken);
    }

    // Get token_owner and revert if not found (all tokens must have an owner)
    let token_owner =
        match get_dictionary_value_from_key::<String>(TOKEN_OWNERS, &token_id.to_string()) {
            (Some(token_owner), _) => token_owner,
            (None, _) => runtime::revert(NFTCoreError::InvalidTokenID),
        };

    let from_account_hash = get_named_arg_with_user_errors::<String>(
        ARG_FROM_ACCOUNT_HASH,
        NFTCoreError::MissingAccountHash,
        NFTCoreError::InvalidAccountHash,
    )
    .unwrap_or_revert();

    // Revert if from account is not the token_owner
    if from_account_hash != token_owner {
        runtime::revert(NFTCoreError::InvalidAccount);
    }

    let caller = runtime::get_caller().to_string();

    // Check if caller is approved to execute transfer
    let (is_approved, _, _) =
        match get_dictionary_value_from_key::<String>(APPROVED_FOR_TRANSFER, &token_id.to_string())
        {
            (Some(approved_account_hash), approved_uref) => (
                approved_account_hash == caller,
                approved_uref,
                Some(approved_account_hash),
            ),
            (None, approved_uref) => (false, approved_uref, None),
        };

    // Revert if caller is not owner or not approved. (CEP47 transfer logic looks incorrect to me...)
    if caller != token_owner && !is_approved {
        runtime::revert(NFTCoreError::InvalidAccount);
    }

    let to_account_hash: String = get_named_arg_with_user_errors(
        ARG_TO_ACCOUNT_HASH,
        NFTCoreError::MissingAccountHash,
        NFTCoreError::InvalidAccountHash,
    )
    .unwrap_or_revert();

    // Updated token_owners dictionary. Revert if token_owner not found.
    match get_dictionary_value_from_key::<String>(TOKEN_OWNERS, &token_id.to_string()) {
        (Some(token_actual_owner_account_hash), token_owners_seed_uref) => {
            if token_actual_owner_account_hash != from_account_hash {
                runtime::revert(NFTCoreError::InvalidTokenOwner)
            }

            storage::dictionary_put(
                token_owners_seed_uref,
                &token_id.to_string(),
                to_account_hash.clone(),
            );
        }
        (None, _) => runtime::revert(NFTCoreError::InvalidTokenID),
    }

    // Update to_account owned_tokens. Revert if owned_tokens list is not found
    match get_dictionary_value_from_key::<Vec<U256>>(OWNED_TOKENS, &from_account_hash) {
        (Some(mut owned_tokens), owned_tokens_seed_uref) => {
            // Check that token_id is in owned tokens list. If so remove token_id from list
            // If not revert.
            if let Some(id) = owned_tokens.iter().position(|id| *id == token_id) {
                owned_tokens.remove(id);
            } else {
                runtime::revert(NFTCoreError::InvalidTokenOwner)
            }

            // Store updated
            storage::dictionary_put(
                owned_tokens_seed_uref,
                &from_account_hash.to_string(),
                owned_tokens,
            );
        }
        (None, _) => runtime::revert(NFTCoreError::InvalidTokenID), // Better error?
    }

    // Update to_account owned_tokens
    match get_dictionary_value_from_key::<Vec<U256>>(OWNED_TOKENS, &to_account_hash) {
        (Some(mut owned_tokens), owned_tokens_seed_uref) => {
            if owned_tokens.iter().any(|id| *id == token_id) {
                runtime::revert(NFTCoreError::FatalTokenIDDuplication)
            } else {
                owned_tokens.push(token_id);
            }

            storage::dictionary_put(owned_tokens_seed_uref, &to_account_hash, owned_tokens);
        }
        (None, owned_tokens_seed_uref) => {
            let owned_tokens = vec![token_id];
            storage::dictionary_put(owned_tokens_seed_uref, &to_account_hash, owned_tokens);
        }
    }

    // // Finally we must changed the approved dictionary if token_id mapped to an approved account_hash
    // if maybe_approved_uref.is_some() {
    //     storage::dictionary_put(approved_uref, &token_id.to_string(), Option::<String>::None);
    // }
}

// Returns the length of the Vec<U256> in OWNED_TOKENS dictionary. If key is not found
// it returns 0.
#[no_mangle]
fn balance_of() {
    let account_hash = get_named_arg_with_user_errors::<String>(
        ARG_ACCOUNT_HASH,
        NFTCoreError::MissingAccountHash,
        NFTCoreError::InvalidAccountHash,
    )
    .unwrap_or_revert();

    let (maybe_owned_tokens, _) =
        get_dictionary_value_from_key::<Vec<U256>>(OWNED_TOKENS, &account_hash);

    let balance = match maybe_owned_tokens {
        Some(owned_tokens) => owned_tokens.len(),
        None => 0,
    };

    let balance_cl_value = CLValue::from_t(U256::from(balance))
        .unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);
    runtime::ret(balance_cl_value);
}

#[no_mangle]
fn owner_of() {
    let token_id = get_named_arg_with_user_errors::<U256>(
        ARG_TOKEN_ID,
        NFTCoreError::MissingTokenID,
        NFTCoreError::InvalidTokenID,
    )
    .unwrap_or_revert();

    let (number_of_minted_tokens, _) = get_stored_value_with_user_errors::<U256>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    if token_id <= number_of_minted_tokens {
        runtime::revert(NFTCoreError::InvalidTokenID); // Do we really want to revert here?
    }

    let (maybe_token_owner, _) =
        get_dictionary_value_from_key::<String>(TOKEN_OWNERS, &token_id.to_string());

    let token_owner = match maybe_token_owner {
        Some(token_owner) => token_owner,
        None => "".to_string(),
    };

    let token_owner_cl_value =
        CLValue::from_t(token_owner).unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);

    runtime::ret(token_owner_cl_value);
}

// Returns true if token has been minted and not burnt; false otherwise
#[no_mangle]
fn token_exists() {
    let token_id = get_named_arg_with_user_errors::<U256>(
        ARG_TOKEN_ID,
        NFTCoreError::MissingTokenID,
        NFTCoreError::InvalidTokenID,
    )
    .unwrap_or_revert();

    let (number_of_minted_tokens, _) = get_stored_value_with_user_errors::<U256>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    let token_exists = if token_id < number_of_minted_tokens {
        let (maybe_burnt, _) =
            get_dictionary_value_from_key::<()>(BURNT_TOKENS, &token_id.to_string());
        maybe_burnt.is_none()
    } else {
        false
    };

    let token_exists_cl_value =
        CLValue::from_t(token_exists).unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);
    runtime::ret(token_exists_cl_value);
}

// Returns approved account_hash from token_id, throws error if token id is not valid
#[no_mangle]
fn get_approved() {
    let token_id = get_optional_named_arg_with_user_errors::<U256>(
        ARG_TOKEN_ID,
        NFTCoreError::MissingTokenID,
        NFTCoreError::InvalidTokenID,
    )
    .unwrap_or_revert();

    if let (Some(_), _) = get_dictionary_value_from_key::<()>(BURNT_TOKENS, &token_id.to_string()) {
        runtime::revert(NFTCoreError::PreviouslyBurntToken);
    }

    let (maybe_approved, _) =
        get_dictionary_value_from_key::<String>(APPROVED_FOR_TRANSFER, &token_id.to_string());

    let approved = match maybe_approved {
        Some(approved) => approved,
        None => runtime::revert(NFTCoreError::InvalidTokenID),
    };

    let approved_cl_value =
        CLValue::from_t(approved).unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);
    runtime::ret(approved_cl_value);
}

// We use the universal set_variables instead.
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

// fn read_optional_account_with_user_errors(key: Key, invalid: NFTCoreError) -> Option<Account> {
//     let (key_ptr, key_size, _bytes) = to_ptr(key);

//     let value_size = {
//         let mut value_size = MaybeUninit::uninit();
//         let ret = unsafe { ext_ffi::casper_read_value(key_ptr, key_size, value_size.as_mut_ptr()) };
//         match api_error::result_from(ret) {
//             Ok(_) => unsafe { value_size.assume_init() },
//             Err(ApiError::ValueNotFound) => return None,
//             Err(e) => runtime::revert(e),
//         }
//     };

//     let value_bytes = read_host_buffer(value_size).unwrap_or_revert();
//     match bytesrepr::deserialize::<Account>(value_bytes) {
//         Ok(account) => Some(account),
//         Err(_) => {
//             runtime::revert(invalid);
//         }
//     }
// }

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
