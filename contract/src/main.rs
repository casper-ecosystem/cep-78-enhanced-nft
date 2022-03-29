#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

mod constants;
mod error;
mod utils;

extern crate alloc;
use core::convert::TryInto;

use alloc::{string::String, string::ToString, vec, vec::Vec};

use casper_types::{
    account::AccountHash, contracts::NamedKeys, runtime_args, CLType, CLValue, ContractHash,
    ContractVersion, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Key, Parameter,
    PublicKey, RuntimeArgs, U256,
};

use casper_contract::contract_api::system;
use casper_contract::{
    contract_api::{
        runtime,
        storage::{self},
    },
    unwrap_or_revert::UnwrapOrRevert,
};

use constants::*;
use error::NFTCoreError;
use utils::*;

#[no_mangle]
pub extern "C" fn init() {
    // We only allow the init() entrypoint to be called once.
    // If COLLECTION_NAME uref already exists we revert since this implies that
    // the init() entrypoint has already been called.
    if named_uref_exists(COLLECTION_NAME) {
        runtime::revert(NFTCoreError::ContractAlreadyInitialized);
    }

    // Only the installing account may call this method. All other callers are erroneous.
    let installing_account = get_account_hash(
        INSTALLER,
        NFTCoreError::MissingInstaller,
        NFTCoreError::InvalidInstaller,
    );

    if installing_account != runtime::get_caller() {
        runtime::revert(NFTCoreError::InvalidAccount)
    }

    // Start collecting the runtime arguments.
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

    let ownership_mode: OwnershipMode = get_named_arg_with_user_errors::<u8>(
        ARG_OWNERSHIP_MODE,
        NFTCoreError::MissingOwnershipMode,
        NFTCoreError::InvalidOwnershipMode,
    )
    .unwrap_or_revert()
    .try_into()
    .unwrap_or_revert();

    // Put all created URefs into the contract's context (necessary to retain access rights,
    // for future use).
    // Initialize contract with URefs for all invariant values, which can never be changed.
    runtime::put_key(COLLECTION_NAME, storage::new_uref(collection_name).into());
    runtime::put_key(
        COLLECTION_SYMBOL,
        storage::new_uref(collection_symbol).into(),
    );
    runtime::put_key(
        TOTAL_TOKEN_SUPPLY,
        storage::new_uref(total_token_supply).into(),
    );
    runtime::put_key(
        OWNERSHIP_MODE,
        storage::new_uref(ownership_mode as u8).into(),
    );

    // Initialize contract with variables which must be present but maybe set to
    // different values after initialization.
    runtime::put_key(ALLOW_MINTING, storage::new_uref(allow_minting).into());
    runtime::put_key(PUBLIC_MINTING, storage::new_uref(public_minting).into());
    // This is an internal variable that the installing account cannot change
    // but is incremented by the contract itself.
    runtime::put_key(
        NUMBER_OF_MINTED_TOKENS,
        storage::new_uref(U256::zero()).into(),
    );

    // Create the data dictionaries to store essential values, topically.
    storage::new_dictionary(TOKEN_OWNERS)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(TOKEN_ISSUERS)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(TOKEN_META_DATA)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(OWNED_TOKENS)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(APPROVED_FOR_TRANSFER)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(BURNT_TOKENS)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(TOKEN_COUNTS)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
}

// set_variables allows the user to set any variable or any combination of variables simultaneously.
// set variables defines what variables are mutable and immutable.
#[no_mangle]
pub extern "C" fn set_variables() {
    // TODO: check for anything that would break invariants here.
    // anything we set here is mutable to the caller,
    // make sure that things that shouldn't be mutable aren't
    let installer = get_account_hash(
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
        let allow_minting_uref = get_uref(
            ALLOW_MINTING,
            NFTCoreError::MissingAllowMinting,
            NFTCoreError::MissingAllowMinting,
        );
        storage::write(allow_minting_uref, allow_minting);
    }
}

#[no_mangle]
pub extern "C" fn mint() {
    // The contract owner can toggle the minting behavior on and off over time.
    // The contract is toggled on by default.
    let minting_status = get_stored_value_with_user_errors::<bool>(
        ALLOW_MINTING,
        NFTCoreError::MissingAllowMinting,
        NFTCoreError::InvalidAllowMinting,
    );

    // If contract minting behavior is currently toggled off; exit.
    if !minting_status {
        runtime::revert(NFTCoreError::MintingIsPaused);
    }

    let total_token_supply = get_stored_value_with_user_errors::<U256>(
        TOTAL_TOKEN_SUPPLY,
        NFTCoreError::MissingTotalTokenSupply,
        NFTCoreError::InvalidTotalTokenSupply,
    );

    let mut next_index = get_stored_value_with_user_errors::<U256>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    // Revert if the token supply has been exhausted.
    if next_index >= total_token_supply {
        runtime::revert(NFTCoreError::TokenSupplyDepleted);
    }

    let caller = runtime::get_caller();

    let private_minting = !get_stored_value_with_user_errors::<bool>(
        PUBLIC_MINTING,
        NFTCoreError::MissingPublicMinting,
        NFTCoreError::InvalidPublicMinting,
    );

    if private_minting {
        let installer_account_hash = runtime::get_key(INSTALLER)
            .unwrap_or_revert_with(NFTCoreError::MissingInstallerKey)
            .into_account()
            .unwrap_or_revert_with(NFTCoreError::FailedToConvertToAccountHash);

        // Revert if private minting is required and caller is not installer.
        if caller != installer_account_hash {
            runtime::revert(NFTCoreError::InvalidMinter)
        }
    }

    // The contract's ownership behavior (determined at installation) determines,
    // who owns the NFT we are about to mint.
    let ownership_mode = utils::get_ownership_mode().unwrap_or_revert();
    let token_owner = {
        match ownership_mode {
            OwnershipMode::Minter => caller,
            OwnershipMode::Assigned | OwnershipMode::TransferableUnchecked => {
                runtime::get_named_arg::<PublicKey>(ARG_TOKEN_OWNER).to_account_hash()
            }
            OwnershipMode::TransferableChecked => {
                // // TODO: We need to be able to determine account existence, which would need to be
                // let foo = storage::read_foo(token_owner_key)
                // runtime::revert("This is unsupported currently, I know, sucks to suck.")

                runtime::get_named_arg::<PublicKey>(ARG_TOKEN_OWNER).to_account_hash()
            }
        }
    };

    // Get token metadata
    let token_meta_data: String = get_named_arg_with_user_errors(
        ARG_TOKEN_META_DATA,
        NFTCoreError::MissingTokenMetaData,
        NFTCoreError::InvalidTokenMetaData,
    )
    .unwrap_or_revert();

    let dictionary_item_key = &next_index.to_string();

    let token_owner_key = Key::Account(token_owner);
    let token_owner_dictionary_item_key = token_owner_key.to_string();

    upsert_dictionary_value_from_key(TOKEN_OWNERS, dictionary_item_key, token_owner_key);
    upsert_dictionary_value_from_key(TOKEN_META_DATA, dictionary_item_key, token_meta_data);
    upsert_dictionary_value_from_key(TOKEN_ISSUERS, &next_index.to_string(), caller);

    // Update owned tokens dictionary
    let maybe_owned_tokens =
        get_dictionary_value_from_key::<Vec<U256>>(OWNED_TOKENS, &token_owner_dictionary_item_key);

    let updated_owned_tokens = match maybe_owned_tokens {
        Some(mut owned_tokens) => {
            if owned_tokens.contains(&next_index) {
                runtime::revert(NFTCoreError::FatalTokenIdDuplication); //<<---better error?
            }

            owned_tokens.push(next_index);
            owned_tokens
        }
        None => vec![next_index],
    };

    // Update the value under the owned_tokens_uref.
    let owned_tokens_uref = match runtime::get_key(&token_owner_dictionary_item_key) {
        Some(key) => {
            let owned_tokens_uref = key.into_uref().unwrap_or_revert();
            storage::write(owned_tokens_uref, updated_owned_tokens);
            owned_tokens_uref
        }
        None => {
            let uref = storage::new_uref(updated_owned_tokens.clone());
            upsert_dictionary_value_from_key(OWNED_TOKENS, &token_owner_dictionary_item_key, uref);
            uref
        }
    };

    //Increment the count of owned tokens.
    let updated_token_count =
        match get_dictionary_value_from_key::<U256>(TOKEN_COUNTS, &token_owner_dictionary_item_key)
        {
            Some(balance) => balance + U256::one(),
            None => U256::one(),
        };
    upsert_dictionary_value_from_key(
        TOKEN_COUNTS,
        &token_owner_dictionary_item_key,
        updated_token_count,
    );

    // Increment number_of_minted_tokens by one
    next_index += U256::one();
    let number_of_minted_tokens_uref = get_uref(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingTotalTokenSupply,
        NFTCoreError::InvalidTotalTokenSupply,
    );
    storage::write(number_of_minted_tokens_uref, next_index);
    // Return the URef under which the minter's owned token indexes with read ONLY access.
    runtime::ret(CLValue::from_t(owned_tokens_uref.into_read()).unwrap_or_revert())
}

#[no_mangle]
pub extern "C" fn burn() {
    let token_id: U256 = get_named_arg_with_user_errors(
        ARG_TOKEN_ID,
        NFTCoreError::MissingTokenID,
        NFTCoreError::InvalidTokenID,
    )
    .unwrap_or_revert();

    let caller: AccountHash = runtime::get_caller();

    // Revert if caller is not token_owner. This seems to be the only check we need to do.
    // TODO: Decide whether an approved operator should be allowed to burn token?
    let token_owner =
        match get_dictionary_value_from_key::<PublicKey>(TOKEN_OWNERS, &token_id.to_string()) {
            Some(token_owner) => {
                if token_owner.to_account_hash() != caller {
                    runtime::revert(NFTCoreError::InvalidTokenOwner)
                }
                token_owner
            }
            None => runtime::revert(NFTCoreError::InvalidTokenID),
        };

    // It makes sense to keep this token as owned by the caller. It just happens that the caller
    // owns a burnt token. That's all. Similarly, we should probably also not change the owned_tokens
    // dictionary.

    // Should an operator be allow to burn tokens?

    if get_dictionary_value_from_key::<()>(BURNT_TOKENS, &token_id.to_string()).is_some() {
        runtime::revert(NFTCoreError::PreviouslyBurntToken);
    }

    // Mark the token as burnt by adding the token_id to the burnt tokens dictionary.
    upsert_dictionary_value_from_key::<()>(BURNT_TOKENS, &token_id.to_string(), ());

    // Should we also update approved dictionary?
    let updated_balance =
        match get_dictionary_value_from_key::<U256>(TOKEN_COUNTS, &token_owner.to_string()) {
            Some(balance) => {
                if balance > U256::zero() {
                    balance - U256::one()
                } else {
                    // This should never happen if contract is implemented correctly.
                    runtime::revert(NFTCoreError::FatalTokenIdDuplication);
                }
            }
            None => {
                // This should never happen if contract is implemented correctly.
                runtime::revert(NFTCoreError::FatalTokenIdDuplication);
            }
        };

    upsert_dictionary_value_from_key(TOKEN_COUNTS, &caller.to_string(), updated_balance);
}

// approve marks a token as approved for transfer by an account
#[no_mangle]
pub extern "C" fn approve() {
    let caller = runtime::get_caller();
    let token_id = get_named_arg_with_user_errors::<U256>(
        ARG_TOKEN_ID,
        NFTCoreError::MissingTokenID,
        NFTCoreError::InvalidTokenID,
    )
    .unwrap_or_revert();

    let number_of_minted_tokens = get_stored_value_with_user_errors::<U256>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    // Revert if token_id is out of bounds
    if token_id >= number_of_minted_tokens {
        runtime::revert(NFTCoreError::InvalidTokenID);
    }

    let token_owner =
        match get_dictionary_value_from_key::<PublicKey>(TOKEN_OWNERS, &token_id.to_string()) {
            Some(token_owner) => token_owner,
            None => runtime::revert(NFTCoreError::InvalidAccountHash),
        };

    // Revert if caller is not the token_owner. Only the token owner can approve an operator
    if token_owner.to_account_hash() != caller {
        runtime::revert(NFTCoreError::InvalidAccountHash);
    }

    // We assume a burnt token cannot be approved
    if get_dictionary_value_from_key::<()>(BURNT_TOKENS, &token_id.to_string()).is_some() {
        runtime::revert(NFTCoreError::PreviouslyBurntToken);
    }

    let approve_for_public_key = get_named_arg_with_user_errors::<PublicKey>(
        ARG_APPROVE_TRANSFER_FOR_ACCOUNT_HASH,
        NFTCoreError::MissingApprovedAccountHash,
        NFTCoreError::InvalidApprovedAccountHash,
    )
    .unwrap_or_revert();

    // If token_owner tries to approve themselves that's probably a mistake and we revert.
    if token_owner == approve_for_public_key {
        runtime::revert(NFTCoreError::InvalidAccount); //Do we need a better error here? ::AlreadyOwner ??
    }

    let approved_uref = get_uref(
        APPROVED_FOR_TRANSFER,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );

    storage::dictionary_put(
        approved_uref,
        &token_id.to_string(),
        Some(approve_for_public_key),
    );
}

#[no_mangle]
pub extern "C" fn set_approval_for_all() {
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
    let approved_uref = get_uref(
        APPROVED_FOR_TRANSFER,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );

    if let Some(owned_tokens) =
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
pub extern "C" fn transfer() {
    // Get token_id argument
    let token_id: U256 = get_named_arg_with_user_errors(
        ARG_TOKEN_ID,
        NFTCoreError::MissingTokenID,
        NFTCoreError::InvalidTokenID,
    )
    .unwrap_or_revert();

    // We assume we cannot transfer burnt tokens
    if get_dictionary_value_from_key::<()>(BURNT_TOKENS, &token_id.to_string()).is_some() {
        runtime::revert(NFTCoreError::PreviouslyBurntToken);
    }

    // Get token_owner and revert if not found (all tokens must have an owner)
    // let token_owner = {
    //     let token_owner_account_hash_string =
    //         match get_dictionary_value_from_key::<String>(TOKEN_OWNERS, &token_id.to_string()) {
    //             (Some(token_owner), _) => token_owner,
    //             (None, _) => runtime::revert(NFTCoreError::InvalidTokenID),
    //         };

    //     match AccountHash::from_formatted_str(&token_owner_account_hash_string) {
    //         Ok(token_owner_account_hash) => token_owner_account_hash,
    //         Err(_) => runtime::revert(NFTCoreError::FailureToParseAccountHash),
    //     }
    // };

    let token_owner =
        match get_dictionary_value_from_key::<PublicKey>(TOKEN_OWNERS, &token_id.to_string()) {
            Some(token_owner) => token_owner,
            None => runtime::revert(NFTCoreError::InvalidTokenID),
        };

    let from_public_key = get_named_arg_with_user_errors::<PublicKey>(
        ARG_FROM_ACCOUNT_HASH,
        NFTCoreError::MissingAccountHash,
        NFTCoreError::InvalidAccountHash,
    )
    .unwrap_or_revert();

    // Revert if from account is not the token_owner
    if from_public_key != token_owner {
        runtime::revert(NFTCoreError::InvalidAccount);
    }

    let caller = runtime::get_caller();

    // Check if caller is approved to execute transfer
    let is_approved = match get_dictionary_value_from_key::<Option<PublicKey>>(
        APPROVED_FOR_TRANSFER,
        &token_id.to_string(),
    ) {
        Some(Some(approved_public_key)) => approved_public_key.to_account_hash() == caller,
        _ => false,
    };

    // Revert if caller is not owner and not approved. (CEP47 transfer logic looks incorrect to me...)
    if caller != token_owner.to_account_hash() && !is_approved {
        runtime::revert(NFTCoreError::InvalidAccount); // InvalidCaller better error?
    }

    let to_account_public_key: PublicKey = get_named_arg_with_user_errors(
        ARG_TO_ACCOUNT_HASH,
        NFTCoreError::MissingAccountHash,
        NFTCoreError::InvalidAccountHash,
    )
    .unwrap_or_revert();

    // Updated token_owners dictionary. Revert if token_owner not found.
    match get_dictionary_value_from_key::<PublicKey>(TOKEN_OWNERS, &token_id.to_string()) {
        Some(token_actual_owner) => {
            let token_owners_seed_uref = get_uref(
                TOKEN_OWNERS,
                NFTCoreError::MissingStorageUref,
                NFTCoreError::InvalidStorageUref,
            );
            if token_actual_owner != from_public_key {
                runtime::revert(NFTCoreError::InvalidTokenOwner)
            }

            storage::dictionary_put(
                token_owners_seed_uref,
                &token_id.to_string(),
                to_account_public_key.clone(),
            );
        }
        None => runtime::revert(NFTCoreError::InvalidTokenID),
    }

    // Update to_account owned_tokens. Revert if owned_tokens list is not found
    match get_dictionary_value_from_key::<Vec<U256>>(OWNED_TOKENS, &from_public_key.to_string()) {
        Some(mut owned_tokens) => {
            // Check that token_id is in owned tokens list. If so remove token_id from list
            // If not revert.

            let owned_tokens_seed_uref = get_uref(
                OWNED_TOKENS,
                NFTCoreError::MissingStorageUref,
                NFTCoreError::InvalidStorageUref,
            );

            if let Some(id) = owned_tokens.iter().position(|id| *id == token_id) {
                owned_tokens.remove(id);
            } else {
                runtime::revert(NFTCoreError::InvalidTokenOwner)
            }

            // Store updated
            storage::dictionary_put(
                owned_tokens_seed_uref,
                &from_public_key.to_string(),
                owned_tokens,
            );
        }
        None => runtime::revert(NFTCoreError::InvalidTokenID), // Better error?
    }

    // Update the from_account balance
    let updated_from_account_balance =
        match get_dictionary_value_from_key::<U256>(TOKEN_COUNTS, &from_public_key.to_string()) {
            Some(balance) => {
                if balance > U256::zero() {
                    balance - U256::one()
                } else {
                    // This should never happen...
                    runtime::revert(NFTCoreError::FatalTokenIdDuplication);
                }
            }
            None => {
                // This should never happen...
                runtime::revert(NFTCoreError::FatalTokenIdDuplication);
            }
        };
    upsert_dictionary_value_from_key(
        TOKEN_COUNTS,
        &from_public_key.to_string(),
        updated_from_account_balance,
    );

    // Update to_account owned_tokens
    match get_dictionary_value_from_key::<Vec<U256>>(
        OWNED_TOKENS,
        &to_account_public_key.to_string(),
    ) {
        Some(mut owned_tokens) => {
            let owned_tokens_seed_uref = get_uref(
                OWNED_TOKENS,
                NFTCoreError::MissingStorageUref,
                NFTCoreError::InvalidStorageUref,
            );

            if owned_tokens.iter().any(|id| *id == token_id) {
                runtime::revert(NFTCoreError::FatalTokenIdDuplication)
            } else {
                owned_tokens.push(token_id);
            }

            storage::dictionary_put(
                owned_tokens_seed_uref,
                &to_account_public_key.to_string(),
                owned_tokens,
            );
        }
        None => {
            let owned_tokens_seed_uref = get_uref(
                OWNED_TOKENS,
                NFTCoreError::MissingStorageUref,
                NFTCoreError::InvalidStorageUref,
            );

            let owned_tokens = vec![token_id];
            storage::dictionary_put(
                owned_tokens_seed_uref,
                &to_account_public_key.to_string(),
                owned_tokens,
            );
        }
    }

    // Update the to_account balance
    let updated_to_account_balance = match get_dictionary_value_from_key::<U256>(
        TOKEN_COUNTS,
        &to_account_public_key.to_string(),
    ) {
        Some(balance) => balance + U256::one(),
        None => U256::one(),
    };
    upsert_dictionary_value_from_key(
        TOKEN_COUNTS,
        &to_account_public_key.to_string(),
        updated_to_account_balance,
    );
}

// Returns the length of the Vec<U256> in OWNED_TOKENS dictionary. If key is not found
// it returns 0.
#[no_mangle]
pub extern "C" fn balance_of() {
    let account_hash = get_named_arg_with_user_errors::<String>(
        ARG_ACCOUNT_HASH,
        NFTCoreError::MissingAccountHash,
        NFTCoreError::InvalidAccountHash,
    )
    .unwrap_or_revert();

    let balance = match get_dictionary_value_from_key(TOKEN_COUNTS, &account_hash) {
        Some(balance) => balance,
        None => U256::zero(),
    };

    // Convert balance usize to CLValue::U256
    let balance_cl_value =
        CLValue::from_t(balance).unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);
    runtime::ret(balance_cl_value);
}

#[no_mangle]
pub extern "C" fn owner_of() {
    let token_id = get_named_arg_with_user_errors::<U256>(
        ARG_TOKEN_ID,
        NFTCoreError::MissingTokenID,
        NFTCoreError::InvalidTokenID,
    )
    .unwrap_or_revert();

    let number_of_minted_tokens = get_stored_value_with_user_errors::<U256>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    // Check if token_id is out of bounds.
    if token_id >= number_of_minted_tokens {
        runtime::revert(NFTCoreError::InvalidTokenID);
    }

    let maybe_token_owner =
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
pub extern "C" fn metadata() {
    let token_id = get_named_arg_with_user_errors::<U256>(
        ARG_TOKEN_ID,
        NFTCoreError::MissingTokenID,
        NFTCoreError::InvalidTokenID,
    )
    .unwrap_or_revert();

    let number_of_minted_tokens = get_stored_value_with_user_errors::<U256>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    // Check if token_id is out of bounds.
    if token_id >= number_of_minted_tokens {
        runtime::revert(NFTCoreError::InvalidTokenID);
    }

    let maybe_token_metadata =
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
pub extern "C" fn get_approved() {
    let token_id =
        get_optional_named_arg_with_user_errors::<U256>(ARG_TOKEN_ID, NFTCoreError::InvalidTokenID)
            .unwrap_or_revert();

    if get_dictionary_value_from_key::<()>(BURNT_TOKENS, &token_id.to_string()).is_some() {
        runtime::revert(NFTCoreError::PreviouslyBurntToken);
    }

    let number_of_minted_tokens = get_stored_value_with_user_errors::<U256>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    // Check if token_id is out of bounds.
    if token_id >= number_of_minted_tokens {
        runtime::revert(NFTCoreError::InvalidTokenID);
    }

    let maybe_approved =
        get_dictionary_value_from_key::<String>(APPROVED_FOR_TRANSFER, &token_id.to_string())
            .unwrap_or_revert();

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
            vec![
                Parameter::new(ARG_TOKEN_OWNER, CLType::PublicKey),
                Parameter::new(ARG_TOKEN_META_DATA, CLType::String),
            ],
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

    let ownership_mode: u8 = get_named_arg_with_user_errors(
        ARG_OWNERSHIP_MODE,
        NFTCoreError::MissingOwnershipMode,
        NFTCoreError::InvalidOwnershipMode,
    )
    .unwrap_or_revert();

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
             ARG_OWNERSHIP_MODE => ownership_mode,
        },
    );
}
