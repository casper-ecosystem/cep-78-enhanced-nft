#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;
use alloc::{string::String, string::ToString, vec, vec::Vec};

use casper_types::contracts::NamedKeys;
use casper_types::{
    runtime_args, CLType, ContractHash, ContractVersion, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, Parameter, RuntimeArgs, U256,
};

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
pub const ARG_APPROVE_TRANSFER_FOR_ACCOUNT_HASH: &str = "approve_transfer_for_account_hash"; //Change name?
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
pub const ENTRY_POINT_OWNER_OF: &str = "owner_of";
pub const ENTRY_POINT_GET_APPROVED: &str = "get_approved";
pub const ENTRY_POINT_METADATA: &str = "metadata";
pub const ENTRY_POINT_OWNED_TOKENS: &str = "owned_tokens";

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
    Phantom = 57,
    ContractAlreadyInitialized = 58,
    MintingIsPaused = 59,
}

impl From<NFTCoreError> for ApiError {
    fn from(e: NFTCoreError) -> Self {
        ApiError::User(e as u16)
    }
}

#[no_mangle]
pub fn init() {
    // We only allow the init() entrypoint to be called once.
    // If ARG_COLLECTION_NAME uref already exists we revert since this implies that
    // the init() entrypoint has already been called.
    if named_uref_exists(ARG_COLLECTION_NAME) {
        runtime::revert(NFTCoreError::ContractAlreadyInitialized);
    }

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
    if let (false, _) = get_stored_value_with_user_errors::<bool>(
        ALLOW_MINTING,
        NFTCoreError::MissingAllowMinting,
        NFTCoreError::InvalidAllowMinting,
    ) {
        runtime::revert(NFTCoreError::MintingIsPaused);
    }

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
            // ??? Do we want to allow the approved account (the operator to burn its tokens?)
            if token_owner_account_hash != caller {
                runtime::revert(NFTCoreError::InvalidTokenOwner)
            }
        }
        (None, _) => runtime::revert(NFTCoreError::InvalidTokenID),
    };

    // It makes sense to keep this token as owned by the caller. It just happens that the caller
    // owns a burnt token. That's all. Similarly, we should probably also not change the owned_tokens
    // dictionary.

    // Should an operator be allow to burn tokens?

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
    let is_approved = match get_dictionary_value_from_key::<Option<String>>(
        APPROVED_FOR_TRANSFER,
        &token_id.to_string(),
    ) {
        (Some(maybe_approved_account_hash), _) => {
            if let Some(approved_account_hash) = maybe_approved_account_hash {
                approved_account_hash == caller
            } else {
                false
            }
        }
        (None, _) => false,
    };

    // Revert if caller is not owner and not approved. (CEP47 transfer logic looks incorrect to me...)
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

    // The number of owned tokens is the the length of the owned_tokens array
    // If no array is found we return 0.
    let balance = match maybe_owned_tokens {
        Some(owned_tokens) => owned_tokens.len(),
        None => 0,
    };

    // Convert balance usize to CLValue::U256
    let balance_cl_value = CLValue::from_t(U256::from(balance))
        .unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);
    runtime::ret(balance_cl_value);
}

// Returns the length of the Vec<U256> in OWNED_TOKENS dictionary. If key is not found
// it returns 0.
// #[no_mangle]
// fn owned_tokens() {
//     let account_hash = get_named_arg_with_user_errors::<String>(
//         ARG_ACCOUNT_HASH,
//         NFTCoreError::MissingAccountHash,
//         NFTCoreError::InvalidAccountHash,
//     )
//     .unwrap_or_revert();

//     let (maybe_owned_tokens, _) =
//         get_dictionary_value_from_key::<Vec<U256>>(OWNED_TOKENS, &account_hash);

//     // Convert balance usize to CLValue::U256
//     let maybe_owned_tokens_cl_value = CLValue::from_t(maybe_owned_tokens)
//         .unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);
//     runtime::ret(maybe_owned_tokens_cl_value);
// }

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

    // Check if token_id is out of bounds.
    if token_id >= number_of_minted_tokens {
        runtime::revert(NFTCoreError::InvalidTokenID);
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

#[no_mangle]
fn metadata() {
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

    // Check if token_id is out of bounds.
    if token_id >= number_of_minted_tokens {
        runtime::revert(NFTCoreError::InvalidTokenID);
    }

    let (maybe_token_metadata, _) =
        get_dictionary_value_from_key::<String>(TOKEN_META_DATA, &token_id.to_string());

    if let Some(metadata) = maybe_token_metadata {
        let metadata_cl_value =
            CLValue::from_t(metadata).unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);

        runtime::ret(metadata_cl_value);
    } else {
        runtime::revert(NFTCoreError::InvalidTokenID) //<-- better error!
    }
}

// Returns true if token has been minted and not burnt; false otherwise
// #[no_mangle]
// fn token_exists() {
//     let token_id = get_named_arg_with_user_errors::<U256>(
//         ARG_TOKEN_ID,
//         NFTCoreError::MissingTokenID,
//         NFTCoreError::InvalidTokenID,
//     )
//     .unwrap_or_revert();

//     let (number_of_minted_tokens, _) = get_stored_value_with_user_errors::<U256>(
//         NUMBER_OF_MINTED_TOKENS,
//         NFTCoreError::MissingNumberOfMintedTokens,
//         NFTCoreError::InvalidNumberOfMintedTokens,
//     );

//     let token_exists = if token_id < number_of_minted_tokens {
//         let (maybe_burnt, _) =
//             get_dictionary_value_from_key::<()>(BURNT_TOKENS, &token_id.to_string());
//         maybe_burnt.is_none()
//     } else {
//         false
//     };

//     let token_exists_cl_value =
//         CLValue::from_t(token_exists).unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);
//     runtime::ret(token_exists_cl_value);
// }

// Returns approved account_hash from token_id, throws error if token id is not valid
#[no_mangle]
fn get_approved() {
    let token_id =
        get_optional_named_arg_with_user_errors::<U256>(ARG_TOKEN_ID, NFTCoreError::InvalidTokenID)
            .unwrap_or_revert();

    if let (Some(_), _) = get_dictionary_value_from_key::<()>(BURNT_TOKENS, &token_id.to_string()) {
        runtime::revert(NFTCoreError::PreviouslyBurntToken);
    }

    let (number_of_minted_tokens, _) = get_stored_value_with_user_errors::<U256>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    // Check if token_id is out of bounds.
    if token_id >= number_of_minted_tokens {
        runtime::revert(NFTCoreError::InvalidTokenID);
    }

    let (maybe_approved, _) =
        get_dictionary_value_from_key::<String>(APPROVED_FOR_TRANSFER, &token_id.to_string());

    // let approved = match maybe_approved {
    //     Some(approved) => approved,
    //     None => runtime::revert(NFTCoreError::InvalidTokenID), //This should never happen though...
    // };

    let approved_cl_value = CLValue::from_t(maybe_approved)
        .unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);
    runtime::ret(approved_cl_value);
}

fn store() -> (ContractHash, ContractVersion) {
    let entry_points = {
        let mut entry_points = EntryPoints::new();

        // This entrypoint initializes the contract and is required to be called during the session
        // where the contract is installed; immedetialy after the contract has been installed but before
        // exiting session. All parameters are required.
        // This entrypoint is intended to be called exactly once and will error if called more than once.
        let init_contract = EntryPoint::new(
            ENTRY_POINT_INIT,
            vec![
                Parameter::new(ARG_COLLECTION_NAME, CLType::String),
                Parameter::new(ARG_COLLECTION_SYMBOL, CLType::String),
                Parameter::new(ARG_TOTAL_TOKEN_SUPPLY, CLType::U256),
                Parameter::new(ARG_ALLOW_MINTING, CLType::Bool),
                Parameter::new(ARG_PUBLIC_MINTING, CLType::Bool),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        // This entrypoint exposes all variables that can be changed by managing account post installation.
        // Meant to be called by the managing account (INSTALLER) post installation
        // if a variable needs to be changed. Each parameter of the entrypoint
        // should only be passed if that variable is changed.
        // For instance if the allow_minting variable is being changed and nothing else
        // the managing account would send the new allow_minting value as the only argument.
        // If no arguments are provided it is essentially a no-operation, however there
        // is still a gas cost.
        // By switching allow_minting to false we pause minting.
        let set_variables = EntryPoint::new(
            ENTRY_POINT_SET_VARIABLES,
            vec![Parameter::new(ARG_ALLOW_MINTING, CLType::Bool)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        // This entrypoint mints a new token with provided metadata.
        // Meant to be called post installation.
        // Reverts with MintingIsPaused error if allow_minting is false.
        // When a token is minted the calling account is listed as its owner and the token is automatically
        // assigned an U256 ID equal to the current number_of_minted_tokens.
        // Before minting the token the entrypoint checks if number_of_minted_tokens
        // exceed the total_token_supply. If so, it reverts the minting with an error TokenSupplyDepleted.
        // The mint entrypoint also checks whether the calling account is the managing account (the installer)
        // If not, and if public_minting is set to false, it reverts with the error InvalidAccount.
        // The newly minted token is automatically assigned a U256 ID equal to the current number_of_minted_tokens.
        // The account is listed as the token owner, as well as added to the accounts list of owned tokens.
        // After minting is successful the number_of_minted_tokens is incremented by one.
        let mint = EntryPoint::new(
            ENTRY_POINT_MINT,
            vec![Parameter::new(ARG_TOKEN_META_DATA, CLType::String)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        // This entrypoint burns the token with provided token_id argument, after which it is no longer
        // possible to transfer it.
        // Looks up the owner of the supplied token_id arg. If caller is not owner we revert with error
        // InvalidTokenOwner. If token id is invalid (e.g. out of bounds) it reverts with error  InvalidTokenID.
        // If token is listed as already burnt we revert with error PreviouslyBurntTOken. If not the token is then
        // registered as burnt.
        let burn = EntryPoint::new(
            ENTRY_POINT_BURN,
            vec![Parameter::new(ARG_TOKEN_ID, CLType::U256)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        // This entrypoint transfers ownership of token from one account to another.
        // It looks up the owner of the supplied token_id arg. Revert if token is already burnt, token_id
        // is unvalid, or if caller is not owner and not approved operator.
        // If token id is invalid it reverts with error InvalidTokenID.
        let transfer = EntryPoint::new(
            ENTRY_POINT_TRANSFER,
            vec![
                Parameter::new(ARG_TOKEN_ID, CLType::U256),
                Parameter::new(ARG_FROM_ACCOUNT_HASH, CLType::String),
                Parameter::new(ARG_TO_ACCOUNT_HASH, CLType::String),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        // This entrypoint approves another account (an operator) to transfer tokens. It reverts
        // if token_id is invalid, if caller is not the owner, if token has already
        // been burnt, or if caller tries to approve themselves as an operator.
        let approve = EntryPoint::new(
            ENTRY_POINT_APPROVE,
            vec![
                Parameter::new(ARG_TOKEN_ID, CLType::U256),
                Parameter::new(ARG_APPROVE_TRANSFER_FOR_ACCOUNT_HASH, CLType::String),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        // This entrypoint returns the token owner given a token_id. It reverts if token_id
        // is invalid. A burnt token still has an associated owner.
        let owner_of = EntryPoint::new(
            ENTRY_POINT_OWNER_OF,
            vec![Parameter::new(ARG_TOKEN_ID, CLType::U256)],
            CLType::String,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        // This entrypoint returns the operator (if any) associated with the provided token_id
        // Reverts if token has been burnt.
        let get_approved = EntryPoint::new(
            ENTRY_POINT_GET_APPROVED,
            vec![Parameter::new(ARG_TOKEN_ID, CLType::U256)],
            CLType::String,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        // This entrypoint returns number of owned tokens associated with the provided account
        let balance_of = EntryPoint::new(
            ENTRY_POINT_BALANCE_OF,
            vec![Parameter::new(ARG_ACCOUNT_HASH, CLType::String)],
            CLType::U256,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        // // Given the owner account_hash this entrypoint returns number of owned tokens
        // let owned_tokens = EntryPoint::new(
        //     ENTRY_POINT_OWNED_TOKENS,
        //     vec![Parameter::new(ARG_ACCOUNT_HASH, CLType::String)],
        //     CLType::List(U256),
        //     EntryPointAccess::Public,
        //     EntryPointType::Contract,
        // );

        // This entrypoint returns the metadata associated with the provided token_id
        let metadata = EntryPoint::new(
            ENTRY_POINT_METADATA,
            vec![Parameter::new(ARG_TOKEN_ID, CLType::U256)],
            CLType::String,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        entry_points.add_entry_point(init_contract);
        entry_points.add_entry_point(set_variables);
        entry_points.add_entry_point(mint);
        entry_points.add_entry_point(burn);
        entry_points.add_entry_point(transfer);
        entry_points.add_entry_point(approve);
        entry_points.add_entry_point(owner_of);
        entry_points.add_entry_point(balance_of);
        entry_points.add_entry_point(get_approved);
        entry_points.add_entry_point(metadata);

        entry_points
    };

    let named_keys = {
        let mut named_keys = NamedKeys::new();
        named_keys.insert(INSTALLER.to_string(), runtime::get_caller().into());
        named_keys
    };

    storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(HASH_KEY_NAME.to_string()),
        Some(ACCESS_KEY_NAME.to_string()),
    )
}

#[no_mangle]
pub extern "C" fn call() {
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

    let allow_minting: bool = get_optional_named_arg_with_user_errors(
        ARG_ALLOW_MINTING,
        NFTCoreError::InvalidMintingStatus,
    )
    .unwrap_or(true);

    let public_minting: bool = get_optional_named_arg_with_user_errors(
        ARG_PUBLIC_MINTING,
        NFTCoreError::InvalidPublicMinting,
    )
    .unwrap_or(false);

    let (contract_hash, contract_version) = store();

    // Store contract_hash and contract_version under the keys CONTRACT_NAME and CONTRACT_VERSION
    runtime::put_key(CONTRACT_NAME, contract_hash.into());
    runtime::put_key(CONTRACT_VERSION, storage::new_uref(contract_version).into());

    // Call contract to initialize it
    runtime::call_contract::<()>(
        contract_hash,
        ENTRY_POINT_INIT,
        runtime_args! {
             ARG_COLLECTION_NAME => collection_name,
             ARG_COLLECTION_SYMBOL => collection_symbol,
             ARG_TOTAL_TOKEN_SUPPLY => total_token_supply,
             ARG_ALLOW_MINTING => allow_minting,
             ARG_PUBLIC_MINTING => public_minting,
        },
    );
}

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
    invalid: NFTCoreError,
) -> Option<T> {
    match get_named_arg_with_user_errors(name, NFTCoreError::Phantom, invalid) {
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

fn named_uref_exists(name: &str) -> bool {
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
