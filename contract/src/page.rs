use alloc::format;

use bit_vec::BitVec;

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    Key,
};

use crate::{
    constants::{PAGE_DICTIONARY_PREFIX, PAGE_TABLE},
    error::NFTCoreError,
    modalities::TokenIdentifier,
    utils,
    utils::PAGE_SIZE,
};

pub(crate) const ALLOCATED: bool = true;
pub(crate) const OWNED: bool = true;
pub(crate) const NOT_OWNED: bool = false;

#[derive(PartialEq)]
pub(crate) enum Operation {
    Mint,
    Transfer,
}

pub(crate) fn mark_allocation_in_page_table(token_owner_key: Key, page_table_entry: u64) {
    let page_table_uref = utils::get_uref(
        PAGE_TABLE,
        NFTCoreError::MissingPageTableURef,
        NFTCoreError::InvalidPageTableURef,
    );
    let encoded_page_table = storage::dictionary_get::<u64>(
        page_table_uref,
        &utils::get_owned_tokens_dictionary_item_key(token_owner_key),
    )
    .unwrap_or_revert()
    .unwrap_or(0u64);
    let mut decoded_page_table = BitVec::from_bytes(
        &encoded_page_table
            .to_bytes()
            .unwrap_or_revert_with(NFTCoreError::FailedToDecodePageTable),
    );
    decoded_page_table.set(page_table_entry as usize, ALLOCATED);
    let allocated_page_table = {
        let (decimal_representation, _remainder) = u64::from_bytes(&decoded_page_table.to_bytes())
            .unwrap_or_revert_with(NFTCoreError::FailedToEncodePageTable);
        decimal_representation
    };
    storage::dictionary_put(
        page_table_uref,
        &utils::get_owned_tokens_dictionary_item_key(token_owner_key),
        allocated_page_table,
    );
}

pub(crate) fn mark_page_entry_as_owned(
    token_owner_key: Key,
    token_identifier: &TokenIdentifier,
    operation: Operation,
) {
    modify_page(token_owner_key, token_identifier, operation, OWNED)
}

pub(crate) fn mark_page_entry_as_not_owned(
    token_owner_key: Key,
    token_identifier: &TokenIdentifier,
    operation: Operation,
) {
    modify_page(token_owner_key, token_identifier, operation, NOT_OWNED)
}

fn modify_page(
    token_owner_key: Key,
    token_identifier: &TokenIdentifier,
    operation: Operation,
    ownership_state: bool,
) {
    let token_number = utils::get_token_index(token_identifier);
    let page_table_entry = token_number / PAGE_SIZE;
    let page_address = token_number % PAGE_SIZE;
    let page_uref = utils::get_uref(
        &format!("{}{}", PAGE_DICTIONARY_PREFIX, page_table_entry),
        NFTCoreError::MissingPageUref,
        NFTCoreError::InvalidPageUref,
    );
    let encoded_page_table = storage::dictionary_get::<u64>(
        utils::get_uref(
            PAGE_TABLE,
            NFTCoreError::MissingPageTableURef,
            NFTCoreError::InvalidPageTableURef,
        ),
        &utils::get_owned_tokens_dictionary_item_key(token_owner_key),
    )
    .unwrap_or_revert()
    .unwrap_or(0u64);
    let decoded_page_table = BitVec::from_bytes(
        &encoded_page_table
            .to_bytes()
            .unwrap_or_revert_with(NFTCoreError::FailedToDecodePageTable),
    );
    let encoded_page = match storage::dictionary_get::<u32>(
        page_uref,
        &utils::get_owned_tokens_dictionary_item_key(token_owner_key),
    )
    .unwrap_or_revert()
    {
        None => {
            if decoded_page_table[page_table_entry as usize] == ALLOCATED
                && operation == Operation::Transfer
                && !ownership_state
            {
                runtime::revert(NFTCoreError::MissingPage)
            } else {
                0u32
            }
        }
        Some(encoded_page) => encoded_page,
    };
    let mut decoded_page = BitVec::from_bytes(
        &encoded_page
            .to_bytes()
            .unwrap_or_revert_with(NFTCoreError::FailedToDecodePage),
    );
    if ownership_state == NOT_OWNED && decoded_page[page_address as usize] != OWNED {
        runtime::revert(NFTCoreError::InvalidTokenOwner)
    }
    decoded_page.set(page_address as usize, ownership_state);
    let modified_page = {
        let (decimal_representation, _remainder) = u32::from_bytes(&decoded_page.to_bytes())
            .unwrap_or_revert_with(NFTCoreError::FailedToEncodePage);
        decimal_representation
    };
    storage::dictionary_put(
        page_uref,
        &utils::get_owned_tokens_dictionary_item_key(token_owner_key),
        modified_page,
    );
}
