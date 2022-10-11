use crate::{
    constants::OWNED_TOKENS, utils, NFTCoreError, TokenIdentifier, MINTED_TOKENS_AT_UPGRADE,
    NUMBER_OF_MINTED_TOKENS, PAGE_SIZE, REVERSE_TOKEN_TRACKER, TOKEN_HASH_TRACKER, TOKEN_OWNERS,
    TOKEN_TRACKER,
};
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{Key, URef};

pub(crate) fn break_up_owned_tokens() {
    let mut searched_token_keys: Vec<Key> = vec![];
    let mut searched_token_ids: Vec<u64> = vec![];

    let number_of_minted_tokens = utils::get_stored_value_with_user_errors::<u64>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    let forward_tracker = utils::get_uref(
        TOKEN_TRACKER,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );

    let reverse_tracker = utils::get_uref(
        REVERSE_TOKEN_TRACKER,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );

    for token_id in 0..number_of_minted_tokens {
        if !searched_token_ids.contains(&token_id) {
            let token_identifier = TokenIdentifier::new_index(token_id);
            let token_owner = utils::get_dictionary_value_from_key::<Key>(
                TOKEN_OWNERS,
                &token_identifier.get_dictionary_item_key(),
            )
            .unwrap_or_revert_with(NFTCoreError::MissingTokenOwner);

            let owned_tokens = utils::get_dictionary_value_from_key::<Vec<u64>>(
                OWNED_TOKENS,
                &utils::get_owned_tokens_dictionary_item_key(token_owner),
            )
            .unwrap_or_revert_with(NFTCoreError::MissingTokenID);

            let token_owner_seed_uref = match runtime::get_key(&token_owner.to_formatted_string()) {
                Some(dictionary_seed_key) => dictionary_seed_key
                    .into_uref()
                    .unwrap_or_revert_with(NFTCoreError::InvalidKey),
                None => {
                    storage::new_dictionary(&token_owner.to_formatted_string()).unwrap_or_revert()
                }
            };

            for owned_token_id in owned_tokens.iter() {
                let page_number = (*owned_token_id / PAGE_SIZE).to_string();
                let page_index = (*owned_token_id % PAGE_SIZE) as usize;
                let mut page =
                    match storage::dictionary_get::<Vec<bool>>(token_owner_seed_uref, &page_number)
                        .unwrap_or_revert()
                    {
                        Some(page) => page,
                        None => vec![false; PAGE_SIZE as usize],
                    };
                let _ = core::mem::replace(&mut page[page_index], true);

                storage::dictionary_put(token_owner_seed_uref, &page_number, page);

                storage::dictionary_put(forward_tracker, &token_id.to_string(), token_id);
                storage::dictionary_put(
                    reverse_tracker,
                    &token_identifier.get_dictionary_item_key(),
                    token_id,
                );

                searched_token_ids.push(*owned_token_id);
                searched_token_keys.push(token_owner);
            }
        }
    }
}

pub(crate) fn break_up_individual_owned_token_hashes(token_owner: &Key) -> URef {
    let mut latest_tracked_token = utils::get_stored_value_with_user_errors::<u64>(
        TOKEN_HASH_TRACKER,
        NFTCoreError::MissingTokenHashTracker,
        NFTCoreError::InvalidTokenHashTracker,
    );

    let number_of_minted_tokens_at_upgrade = utils::get_stored_value_with_user_errors::<u64>(
        MINTED_TOKENS_AT_UPGRADE,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    if latest_tracked_token > number_of_minted_tokens_at_upgrade {
        runtime::revert(NFTCoreError::InvalidNumberOfMintedTokens)
    }

    let forward_tracker = utils::get_uref(
        TOKEN_TRACKER,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );

    let reverse_tracker = utils::get_uref(
        REVERSE_TOKEN_TRACKER,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );

    let token_owner_dictionary_item_key = utils::get_owned_tokens_dictionary_item_key(token_owner.clone());

    let owned_tokens: Vec<TokenIdentifier> =
        utils::get_dictionary_value_from_key::<Vec<String>>(OWNED_TOKENS, &token_owner_dictionary_item_key)
            .unwrap_or_revert_with(NFTCoreError::MissingOwnedTokens)
            .into_iter()
            .map(|token_hash| TokenIdentifier::new_hash(token_hash))
            .collect();

    let token_owner_seed_uref =
        storage::new_dictionary(&token_owner.to_formatted_string()).unwrap_or_revert();

    // We will backfill the token hashes foregoing any attempt to preserve
    // the historical order in which the tokens were minted
    for token_hash in owned_tokens {
        storage::dictionary_put(
            forward_tracker,
            &latest_tracked_token.to_string(),
            token_hash.get_dictionary_item_key(),
        );
        storage::dictionary_put(
            reverse_tracker,
            &token_hash.get_dictionary_item_key(),
            latest_tracked_token,
        );
        let current_page_number = latest_tracked_token / PAGE_SIZE;
        let current_page_index = latest_tracked_token % PAGE_SIZE;
        let mut page = match storage::dictionary_get::<Vec<bool>>(
            token_owner_seed_uref,
            &current_page_number.to_string(),
        )
        .unwrap_or_revert()
        {
            Some(page) => page,
            None => vec![false; PAGE_SIZE as usize],
        };
        let _ = core::mem::replace(&mut page[current_page_index as usize], true);
        storage::dictionary_put(
            token_owner_seed_uref,
            &current_page_number.to_string(),
            page,
        );
        latest_tracked_token += 1;
    }

    let token_hash_uref = utils::get_uref(
        TOKEN_HASH_TRACKER,
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );
    storage::write(token_hash_uref, latest_tracked_token);

    token_owner_seed_uref
}
