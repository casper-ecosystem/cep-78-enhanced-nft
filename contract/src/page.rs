use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::convert::TryInto;

use crate::{utils, NFTCoreError, NFTIdentifierMode, TokenIdentifier, IDENTIFIER_MODE, NUMBER_OF_MINTED_TOKENS, TOKEN_TRACKER, REVERSE_TOKEN_TRACKER};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::Key;

const PAGE_SIZE: u64 = 10;

// This is a very gas expensive operation and should not be used often.
pub(crate) fn get_all_owned_token_identifiers(token_owner: &Key) -> Vec<TokenIdentifier> {
    let current_number_of_minted_tokens = utils::get_stored_value_with_user_errors::<u64>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );
    let max_number_of_pages = (current_number_of_minted_tokens / PAGE_SIZE as u64) + 1;
    let token_owner_seed_uref = utils::get_uref(
        &token_owner.to_formatted_string(),
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );
    let forward_lookup_seed_uref = utils::get_uref(
        TOKEN_TRACKER,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );
    let identifier_mode: NFTIdentifierMode = utils::get_stored_value_with_user_errors::<u8>(
        IDENTIFIER_MODE,
        NFTCoreError::MissingIdentifierMode,
        NFTCoreError::InvalidIdentifierMode,
    )
    .try_into()
    .unwrap_or_revert();
    let mut token_identifiers = vec![];

    for page_number in 0..=max_number_of_pages {
        if let Some(page) =
            storage::dictionary_get::<Vec<bool>>(token_owner_seed_uref, &page_number.to_string())
                .unwrap_or_revert()
        {
            for (index, does_own) in page.iter().enumerate() {
                if *does_own {
                    let token_index = (page_number * PAGE_SIZE as u64) + (index as u64);
                    let token_identifer_string = storage::dictionary_get::<String>(
                        forward_lookup_seed_uref,
                        &token_index.to_string(),
                    )
                    .unwrap_or_revert()
                    .unwrap_or_revert_with(NFTCoreError::InvalidTokenIdentifier);
                    match identifier_mode {
                        NFTIdentifierMode::Ordinal => {
                            let ordinal_identifier =
                                token_identifer_string.parse::<u64>().unwrap_or_else(|_| {
                                    runtime::revert(NFTCoreError::FailedToParseString)
                                });
                            token_identifiers.push(TokenIdentifier::new_index(ordinal_identifier))
                        }
                        NFTIdentifierMode::Hash => token_identifiers
                            .push(TokenIdentifier::new_hash(token_identifer_string)),
                    }
                }
            }
        }
    }
    token_identifiers
}

pub(crate) fn get_token_tracking_index(token_identifier: &TokenIdentifier) -> Option<u64> {
    let reverse_lookup_uref = utils::get_uref(
        REVERSE_TOKEN_TRACKER,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );

    storage::dictionary_get(
        reverse_lookup_uref,
        &token_identifier.get_dictionary_item_key(),
    )
        .unwrap_or_revert()
}

pub(crate) fn _get_token_owner_page(token_index: u64, token_owner: &Key) -> Option<Vec<bool>> {
    let page_number = token_index / PAGE_SIZE;
    let token_owner_dictionary_uref = utils::get_uref(
        &token_owner.to_formatted_string(),
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );
    storage::dictionary_get::<Vec<bool>>(token_owner_dictionary_uref, &page_number.to_string())
        .unwrap_or_revert()
}

pub(crate) fn get_token_page_index(token_index: u64) -> usize {
    let page_index = token_index % PAGE_SIZE;
    if token_index >= PAGE_SIZE as u64{
        runtime::revert(NFTCoreError::InvalidPageIndex)
    }
    page_index as usize
}

pub(crate) fn manage_token_owner_page(
    token_index: u64,
    token_owner: &Key,
    new_ownership_state: bool,
) {
    let page_number = (token_index / PAGE_SIZE).to_string();
    let page_index = get_token_page_index(token_index);
    // If the new_ownership_state is false, it means a previously owned
    // token is being transferred out and/or burnt. This implies the
    // page for the token must previously exist
    let token_owner_seed_uref = match runtime::get_key(&token_owner.to_formatted_string()) {
        Some(key) => key.into_uref().unwrap_or_revert(),
        None => {
            if !new_ownership_state {
                runtime::revert(NFTCoreError::MissingStorageUref)
            }
            storage::new_dictionary(&token_owner.to_formatted_string())
                .unwrap_or_revert()
        }
    };

    let mut token_owner_page = storage::dictionary_get(token_owner_seed_uref, &page_number)
        .unwrap_or_revert()
        .unwrap_or_else(|| {
            if !new_ownership_state {
                runtime::revert(NFTCoreError::InvalidTokenOwner)
            } else {
                vec![false; PAGE_SIZE as usize]
            }
        });

    if token_owner_page[page_index] == new_ownership_state {
        runtime::revert(NFTCoreError::InvalidPageIndex)
    }
    let _ = core::mem::replace(&mut token_owner_page[page_index], new_ownership_state);

    storage::dictionary_put(token_owner_seed_uref, &page_number, token_owner_page)
}

pub(crate) fn get_token_page_dictionary_key(token_index: u64, token_owner: &Key) -> Key {
    let page_number = (token_index / PAGE_SIZE).to_string();
    let owner_seed_uref = utils::get_uref(
        &token_owner.to_formatted_string(),
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );
    Key::dictionary(owner_seed_uref, page_number.as_bytes())
}