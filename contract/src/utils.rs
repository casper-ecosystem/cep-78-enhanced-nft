use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use casper_event_standard::Schemas;
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
    ApiError, CLTyped, ContractHash, ContractPackageHash, Key, URef,
};

use crate::{
    constants::{
        ACL_WHITELIST, ARG_TOKEN_HASH, ARG_TOKEN_ID, BURNT_TOKENS, BURN_MODE, CONTRACT_WHITELIST,
        HASH_BY_INDEX, HOLDER_MODE, INDEX_BY_HASH, MIGRATION_FLAG, MINTING_MODE,
        NUMBER_OF_MINTED_TOKENS, OWNED_TOKENS, OWNERSHIP_MODE, PAGE_LIMIT, PAGE_TABLE,
        PREFIX_PAGE_DICTIONARY, RECEIPT_NAME, REPORTING_MODE, RLO_MFLAG, TOKEN_OWNERS,
        TRANSFER_FILTER_CONTRACT, UNMATCHED_HASH_COUNT,
    },
    error::NFTCoreError,
    events::events_ces::{
        Approval, ApprovalForAll, ApprovalRevoked, Burn, MetadataUpdated, Migration, Mint,
        Transfer, VariablesSet,
    },
    modalities::{
        BurnMode, MetadataRequirement, MintingMode, NFTHolderMode, NFTIdentifierMode,
        NFTMetadataKind, OwnerReverseLookupMode, OwnershipMode, Requirement, TokenIdentifier,
    },
    utils,
};

// The size of a given page, it is currently set to 1000
// to ease the math around addressing newly minted tokens.
pub const PAGE_SIZE: u64 = 1000;

pub fn upsert_dictionary_value_from_key<T: CLTyped + FromBytes + ToBytes>(
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

pub fn get_ownership_mode() -> Result<OwnershipMode, NFTCoreError> {
    get_stored_value_with_user_errors::<u8>(
        OWNERSHIP_MODE,
        NFTCoreError::MissingOwnershipMode,
        NFTCoreError::InvalidOwnershipMode,
    )
    .try_into()
}

pub fn get_holder_mode() -> Result<NFTHolderMode, NFTCoreError> {
    get_stored_value_with_user_errors::<u8>(
        HOLDER_MODE,
        NFTCoreError::MissingHolderMode,
        NFTCoreError::InvalidHolderMode,
    )
    .try_into()
}

pub fn encode_dictionary_item_key(key: Key) -> String {
    match key {
        Key::Account(account_hash) => account_hash.to_string(),
        Key::Hash(hash_addr) => ContractHash::new(hash_addr).to_string(),
        _ => runtime::revert(NFTCoreError::InvalidKey),
    }
}

pub fn encode_key_and_value<T: CLTyped + ToBytes>(key: &Key, value: &T) -> String {
    let mut bytes_a = key.to_bytes().unwrap_or_revert();
    let mut bytes_b = value.to_bytes().unwrap_or_revert();

    bytes_a.append(&mut bytes_b);

    let bytes = runtime::blake2b(bytes_a);
    hex::encode(bytes)
}

pub fn get_dictionary_value_from_key<T: CLTyped + FromBytes>(
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

pub fn get_stored_value_with_user_errors<T: CLTyped + FromBytes>(
    name: &str,
    missing: NFTCoreError,
    invalid: NFTCoreError,
) -> T {
    let uref = get_uref(name, missing, invalid);
    read_with_user_errors(uref, missing, invalid)
}

pub fn get_named_arg_size(name: &str) -> Option<usize> {
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

// The optional here is literal and does not co-relate to an Option enum type.
// If the argument has been provided it is accepted, and is then turned into a Some.
// If the argument is not provided at all, then it is considered as None.
pub fn get_optional_named_arg_with_user_errors<T: FromBytes>(
    name: &str,
    invalid: NFTCoreError,
) -> Option<T> {
    match get_named_arg_with_user_errors::<T>(name, NFTCoreError::Phantom, invalid) {
        Ok(val) => Some(val),
        Err(_) => None,
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

pub fn get_account_hash(name: &str, missing: NFTCoreError, invalid: NFTCoreError) -> AccountHash {
    let key = get_key_with_user_errors(name, missing, invalid);
    key.into_account()
        .unwrap_or_revert_with(NFTCoreError::UnexpectedKeyVariant)
}

pub fn get_uref(name: &str, missing: NFTCoreError, invalid: NFTCoreError) -> URef {
    let key = get_key_with_user_errors(name, missing, invalid);
    key.into_uref()
        .unwrap_or_revert_with(NFTCoreError::UnexpectedKeyVariant)
}

pub fn named_uref_exists(name: &str) -> bool {
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

pub fn get_key_with_user_errors(name: &str, missing: NFTCoreError, invalid: NFTCoreError) -> Key {
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

pub fn read_with_user_errors<T: CLTyped + FromBytes>(
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

pub fn read_host_buffer_into(dest: &mut [u8]) -> Result<usize, ApiError> {
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

pub fn read_host_buffer(size: usize) -> Result<Vec<u8>, ApiError> {
    let mut dest: Vec<u8> = if size == 0 {
        Vec::new()
    } else {
        let bytes_non_null_ptr = contract_api::alloc_bytes(size);
        unsafe { Vec::from_raw_parts(bytes_non_null_ptr.as_ptr(), size, size) }
    };
    read_host_buffer_into(&mut dest)?;
    Ok(dest)
}

pub fn to_ptr<T: ToBytes>(t: T) -> (*const u8, usize, Vec<u8>) {
    let bytes = t.into_bytes().unwrap_or_revert();
    let ptr = bytes.as_ptr();
    let size = bytes.len();
    (ptr, size, bytes)
}

pub enum Caller {
    Session(AccountHash),
    StoredCaller(ContractHash, ContractPackageHash),
}

pub fn get_verified_caller() -> Result<Caller, NFTCoreError> {
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
            Ok(Caller::Session(calling_account_hash))
        }
        CallStackElement::StoredSession {
            contract_hash,
            contract_package_hash,
            ..
        }
        | CallStackElement::StoredContract {
            contract_hash,
            contract_package_hash,
        } => {
            if let NFTHolderMode::Accounts = holder_mode {
                return Err(NFTCoreError::InvalidHolderMode);
            }
            Ok(Caller::StoredCaller(contract_hash, contract_package_hash))
        }
    }
}

pub fn get_token_identifier_from_runtime_args(
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

pub fn get_token_identifiers_from_dictionary(
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

pub fn get_burn_mode() -> BurnMode {
    let burn_mode: BurnMode = get_stored_value_with_user_errors::<u8>(
        BURN_MODE,
        NFTCoreError::MissingBurnMode,
        NFTCoreError::InvalidBurnMode,
    )
    .try_into()
    .unwrap_or_revert();
    burn_mode
}

pub fn is_token_burned(token_identifier: &TokenIdentifier) -> bool {
    get_dictionary_value_from_key::<()>(BURNT_TOKENS, &token_identifier.get_dictionary_item_key())
        .is_some()
}

pub fn get_transfer_filter_contract() -> Option<ContractHash> {
    if !named_uref_exists(TRANSFER_FILTER_CONTRACT) {
        None
    } else {
        Some(get_stored_value_with_user_errors::<ContractHash>(
            TRANSFER_FILTER_CONTRACT,
            NFTCoreError::MissingTransferFilterContract,
            NFTCoreError::InvalidTransferFilterContract,
        ))
    }
}

pub fn max_number_of_pages(total_token_supply: u64) -> u64 {
    if total_token_supply < PAGE_SIZE {
        let dictionary_name = format!("{PREFIX_PAGE_DICTIONARY}_{}", 0);
        storage::new_dictionary(&dictionary_name)
            .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
        1
    } else {
        let max_number_of_pages = total_token_supply / PAGE_SIZE;
        let overflow = total_token_supply % PAGE_SIZE;
        for page_number in 0..max_number_of_pages {
            let dictionary_name = format!("{PREFIX_PAGE_DICTIONARY}_{page_number}");
            storage::new_dictionary(&dictionary_name)
                .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
        }
        // With a page size of say 1000 and a token supply of 1050
        // max_number_of_pages = 1, but we need an additional page
        // to track the overflow
        if overflow == 0 {
            max_number_of_pages
        } else {
            max_number_of_pages + 1
        }
    }
}

pub fn insert_hash_id_lookups(
    current_number_of_minted_tokens: u64,
    token_identifier: TokenIdentifier,
) {
    if token_identifier.get_index().is_some() {
        return;
    }
    let hash_by_index_uref = get_uref(
        HASH_BY_INDEX,
        NFTCoreError::MissingHashByIndex,
        NFTCoreError::InvalidHashByIndex,
    );
    let index_by_hash_uref = get_uref(
        INDEX_BY_HASH,
        NFTCoreError::MissingIndexByHash,
        NFTCoreError::InvalidIndexByHash,
    );
    if storage::dictionary_get::<u64>(
        index_by_hash_uref,
        &token_identifier.get_dictionary_item_key(),
    )
    .unwrap_or_revert()
    .is_some()
    {
        runtime::revert(NFTCoreError::DuplicateIdentifier)
    }
    if storage::dictionary_get::<String>(
        hash_by_index_uref,
        &current_number_of_minted_tokens.to_string(),
    )
    .unwrap_or_revert()
    .is_some()
    {
        runtime::revert(NFTCoreError::DuplicateIdentifier)
    }
    storage::dictionary_put(
        hash_by_index_uref,
        &current_number_of_minted_tokens.to_string(),
        token_identifier.clone().get_hash().unwrap_or_revert(),
    );
    storage::dictionary_put(
        index_by_hash_uref,
        &token_identifier.get_dictionary_item_key(),
        current_number_of_minted_tokens,
    );
}

pub fn get_token_index(token_identifier: &TokenIdentifier) -> u64 {
    match token_identifier {
        TokenIdentifier::Index(token_index) => *token_index,
        TokenIdentifier::Hash(_) => {
            let index_by_hash_uref = get_uref(
                INDEX_BY_HASH,
                NFTCoreError::MissingIndexByHash,
                NFTCoreError::InvalidIndexByHash,
            );
            storage::dictionary_get::<u64>(
                index_by_hash_uref,
                &token_identifier.get_dictionary_item_key(),
            )
            .unwrap_or_revert()
            .unwrap_or_revert_with(NFTCoreError::InvalidTokenIdentifier)
        }
    }
}

pub fn migrate_owned_tokens_in_ordinal_mode() {
    let current_number_of_minted_tokens = utils::get_stored_value_with_user_errors::<u64>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingTotalTokenSupply,
        NFTCoreError::InvalidTotalTokenSupply,
    );
    let page_table_uref = get_uref(
        PAGE_TABLE,
        NFTCoreError::MissingPageTableURef,
        NFTCoreError::InvalidPageTableURef,
    );
    let page_table_width = get_stored_value_with_user_errors::<u64>(
        PAGE_LIMIT,
        NFTCoreError::MissingPageLimit,
        NFTCoreError::InvalidPageLimit,
    );
    let mut searched_token_ids: Vec<u64> = vec![];
    for token_id in 0..current_number_of_minted_tokens {
        if !searched_token_ids.contains(&token_id) {
            let token_identifier = TokenIdentifier::new_index(token_id);
            let token_owner_key = get_dictionary_value_from_key::<Key>(
                TOKEN_OWNERS,
                &token_identifier.get_dictionary_item_key(),
            )
            .unwrap_or_revert_with(NFTCoreError::MissingNftKind);
            let token_owner_item_key = encode_dictionary_item_key(token_owner_key);
            let owned_tokens_list = get_token_identifiers_from_dictionary(
                &NFTIdentifierMode::Ordinal,
                &token_owner_item_key,
            )
            .unwrap_or_revert();
            for token_identifier in owned_tokens_list.into_iter() {
                let token_id = token_identifier.get_index().unwrap_or_revert();
                let page_number = token_id / PAGE_SIZE;
                let page_index = token_id % PAGE_SIZE;
                let mut page_record = match storage::dictionary_get::<Vec<bool>>(
                    page_table_uref,
                    &token_owner_item_key,
                )
                .unwrap_or_revert()
                {
                    Some(page_record) => page_record,
                    None => vec![false; page_table_width as usize],
                };
                let page_uref = get_uref(
                    &format!("{PREFIX_PAGE_DICTIONARY}_{page_number}"),
                    NFTCoreError::MissingStorageUref,
                    NFTCoreError::InvalidStorageUref,
                );
                let _ = core::mem::replace(&mut page_record[page_number as usize], true);
                storage::dictionary_put(page_table_uref, &token_owner_item_key, page_record);
                let mut page =
                    match storage::dictionary_get::<Vec<bool>>(page_uref, &token_owner_item_key)
                        .unwrap_or_revert()
                    {
                        None => vec![false; PAGE_SIZE as usize],
                        Some(single_page) => single_page,
                    };
                let is_already_marked_as_owned =
                    core::mem::replace(&mut page[page_index as usize], true);
                if is_already_marked_as_owned {
                    runtime::revert(NFTCoreError::InvalidPageIndex)
                }
                storage::dictionary_put(page_uref, &token_owner_item_key, page);
                searched_token_ids.push(token_id)
            }
        }
    }
}

pub fn should_migrate_token_hashes(token_owner: Key) -> bool {
    if get_token_identifiers_from_dictionary(
        &NFTIdentifierMode::Hash,
        &encode_dictionary_item_key(token_owner),
    )
    .is_none()
    {
        return false;
    }
    let page_table_uref = get_uref(
        PAGE_TABLE,
        NFTCoreError::MissingPageTableURef,
        NFTCoreError::InvalidPageTableURef,
    );
    // If the owner has registered, then they will have an page table entry
    // but it will contain no bits set.
    let page_table = storage::dictionary_get::<Vec<bool>>(
        page_table_uref,
        &encode_dictionary_item_key(token_owner),
    )
    .unwrap_or_revert()
    .unwrap_or_revert_with(NFTCoreError::UnregisteredOwnerFromMigration);
    if page_table.contains(&true) {
        return false;
    }
    true
}

pub fn migrate_token_hashes(token_owner: Key) {
    let mut unmatched_hash_count = get_stored_value_with_user_errors::<u64>(
        UNMATCHED_HASH_COUNT,
        NFTCoreError::MissingUnmatchedHashCount,
        NFTCoreError::InvalidUnmatchedHashCount,
    );

    if unmatched_hash_count == 0 {
        runtime::revert(NFTCoreError::InvalidNumberOfMintedTokens)
    }

    let token_owner_item_key = encode_dictionary_item_key(token_owner);
    let owned_tokens_list =
        get_token_identifiers_from_dictionary(&NFTIdentifierMode::Hash, &token_owner_item_key)
            .unwrap_or_revert_with(NFTCoreError::InvalidTokenOwner);

    let page_table_uref = get_uref(
        PAGE_TABLE,
        NFTCoreError::MissingPageTableURef,
        NFTCoreError::InvalidPageTableURef,
    );

    let page_table_width = get_stored_value_with_user_errors::<u64>(
        PAGE_LIMIT,
        NFTCoreError::MissingPageLimit,
        NFTCoreError::InvalidPageLimit,
    );

    for token_identifier in owned_tokens_list.into_iter() {
        let token_address = unmatched_hash_count - 1;
        let page_table_entry = token_address / PAGE_SIZE;
        let page_address = token_address % PAGE_SIZE;
        let mut page_table =
            match storage::dictionary_get::<Vec<bool>>(page_table_uref, &token_owner_item_key)
                .unwrap_or_revert()
            {
                Some(page_record) => page_record,
                None => vec![false; page_table_width as usize],
            };
        let _ = core::mem::replace(&mut page_table[page_table_entry as usize], true);
        storage::dictionary_put(page_table_uref, &token_owner_item_key, page_table);
        let page_uref = get_uref(
            &format!("{PREFIX_PAGE_DICTIONARY}_{page_table_entry}"),
            NFTCoreError::MissingStorageUref,
            NFTCoreError::InvalidStorageUref,
        );
        let mut page = match storage::dictionary_get::<Vec<bool>>(page_uref, &token_owner_item_key)
            .unwrap_or_revert()
        {
            Some(single_page) => single_page,
            None => vec![false; PAGE_SIZE as usize],
        };
        let _ = core::mem::replace(&mut page[page_address as usize], true);
        storage::dictionary_put(page_uref, &token_owner_item_key, page);
        insert_hash_id_lookups(unmatched_hash_count - 1, token_identifier);
        unmatched_hash_count -= 1;
    }

    let unmatched_hash_count_uref = get_uref(
        UNMATCHED_HASH_COUNT,
        NFTCoreError::MissingUnmatchedHashCount,
        NFTCoreError::InvalidUnmatchedHashCount,
    );

    storage::write(unmatched_hash_count_uref, unmatched_hash_count);
}

pub fn get_receipt_name(page_table_entry: u64) -> String {
    let receipt = utils::get_stored_value_with_user_errors::<String>(
        RECEIPT_NAME,
        NFTCoreError::MissingReceiptName,
        NFTCoreError::InvalidReceiptName,
    );
    format!("{receipt}_m_{PAGE_SIZE}_p_{page_table_entry}")
}

pub fn get_reporting_mode() -> OwnerReverseLookupMode {
    utils::get_stored_value_with_user_errors::<u8>(
        REPORTING_MODE,
        NFTCoreError::MissingReportingMode,
        NFTCoreError::InvalidReportingMode,
    )
    .try_into()
    .unwrap_or_revert()
}

pub fn add_page_entry_and_page_record(
    tokens_count: u64,
    item_key: &str,
    on_mint: bool,
) -> (u64, URef) {
    // there is an explicit page_table;
    // this is the entry in that overall page table which maps to the underlying page
    // upon which this mint's address will exist
    let page_table_entry = tokens_count / PAGE_SIZE;
    let page_address = tokens_count % PAGE_SIZE;

    // Update the page entry first
    let page_table_uref = utils::get_uref(
        PAGE_TABLE,
        NFTCoreError::MissingPageTableURef,
        NFTCoreError::InvalidPageTableURef,
    );

    // Update the individual page record.
    let page_uref = utils::get_uref(
        &format!("{PREFIX_PAGE_DICTIONARY}_{page_table_entry}"),
        NFTCoreError::MissingPageUref,
        NFTCoreError::InvalidPageUref,
    );

    let mut page_table =
        match storage::dictionary_get::<Vec<bool>>(page_table_uref, item_key).unwrap_or_revert() {
            Some(page_table) => page_table,
            None => runtime::revert(if on_mint {
                NFTCoreError::UnregisteredOwnerInMint
            } else {
                NFTCoreError::UnregisteredOwnerInTransfer
            }),
        };

    let mut page = if !page_table[page_table_entry as usize] {
        // We mark the page table entry to true to signal the allocation of a page.
        let _ = core::mem::replace(&mut page_table[page_table_entry as usize], true);
        storage::dictionary_put(page_table_uref, item_key, page_table);
        vec![false; PAGE_SIZE as usize]
    } else {
        storage::dictionary_get::<Vec<bool>>(page_uref, item_key)
            .unwrap_or_revert()
            .unwrap_or_revert_with(NFTCoreError::MissingPage)
    };

    let _ = core::mem::replace(&mut page[page_address as usize], true);

    storage::dictionary_put(page_uref, item_key, page);
    (page_table_entry, page_uref)
}

pub fn update_page_entry_and_page_record(
    tokens_count: u64,
    old_item_key: &str,
    new_item_key: &str,
) -> (u64, URef) {
    let page_table_entry = tokens_count / PAGE_SIZE;
    let page_address = tokens_count % PAGE_SIZE;

    let page_uref = utils::get_uref(
        &format!("{PREFIX_PAGE_DICTIONARY}_{page_table_entry}"),
        NFTCoreError::MissingStorageUref,
        NFTCoreError::InvalidStorageUref,
    );

    let mut source_page = storage::dictionary_get::<Vec<bool>>(page_uref, old_item_key)
        .unwrap_or_revert()
        .unwrap_or_revert_with(NFTCoreError::InvalidPageNumber);

    if !source_page[page_address as usize] {
        runtime::revert(NFTCoreError::InvalidTokenIdentifier)
    }

    let _ = core::mem::replace(&mut source_page[page_address as usize], false);

    storage::dictionary_put(page_uref, old_item_key, source_page);

    let page_table_uref = utils::get_uref(
        PAGE_TABLE,
        NFTCoreError::MissingPageTableURef,
        NFTCoreError::InvalidPageTableURef,
    );

    let mut target_page_table = storage::dictionary_get::<Vec<bool>>(page_table_uref, new_item_key)
        .unwrap_or_revert()
        .unwrap_or_revert_with(NFTCoreError::UnregisteredOwnerInTransfer);

    let mut target_page = if !target_page_table[page_table_entry as usize] {
        // Create a new page here
        let _ = core::mem::replace(&mut target_page_table[page_table_entry as usize], true);
        storage::dictionary_put(page_table_uref, new_item_key, target_page_table);
        vec![false; PAGE_SIZE as usize]
    } else {
        storage::dictionary_get::<Vec<bool>>(page_uref, new_item_key)
            .unwrap_or_revert()
            .unwrap_or_revert()
    };

    let _ = core::mem::replace(&mut target_page[page_address as usize], true);

    storage::dictionary_put(page_uref, new_item_key, target_page);
    (page_table_entry, page_uref)
}

pub fn create_metadata_requirements(
    base: NFTMetadataKind,
    req: Vec<u8>,
    opt: Vec<u8>,
) -> MetadataRequirement {
    let mut metadata_requirements = MetadataRequirement::new();
    for optional in opt {
        metadata_requirements.insert(
            optional.try_into().unwrap_or_revert(),
            Requirement::Optional,
        );
    }
    for required in req {
        metadata_requirements.insert(
            required.try_into().unwrap_or_revert(),
            Requirement::Required,
        );
    }
    metadata_requirements.insert(base, Requirement::Required);
    metadata_requirements
}

// Initializes events-releated named keys and records all event schemas.
pub fn init_events() {
    let schemas = Schemas::new()
        .with::<Mint>()
        .with::<Burn>()
        .with::<Approval>()
        .with::<ApprovalRevoked>()
        .with::<ApprovalForAll>()
        .with::<Transfer>()
        .with::<MetadataUpdated>()
        .with::<VariablesSet>()
        .with::<Migration>();
    casper_event_standard::init(schemas);
}

pub fn requires_rlo_migration() -> bool {
    match runtime::get_key(MIGRATION_FLAG) {
        Some(migration_flag_key) => {
            let migration_uref = migration_flag_key
                .into_uref()
                .unwrap_or_revert_with(NFTCoreError::InvalidKey);
            let has_rlo_migration = storage::read::<bool>(migration_uref)
                .unwrap_or_revert()
                .unwrap_or_revert();
            runtime::remove_key(MIGRATION_FLAG);
            !has_rlo_migration
        }
        None => match runtime::get_key(RLO_MFLAG) {
            Some(rlo_flag_key) => {
                let rlo_flag_uref = rlo_flag_key
                    .into_uref()
                    .unwrap_or_revert_with(NFTCoreError::InvalidKey);
                storage::read::<bool>(rlo_flag_uref)
                    .unwrap_or_revert()
                    .unwrap_or_revert()
            }
            None => true,
        },
    }
}

pub fn migrate_contract_whitelist_to_acl_whitelist() {
    // Add ACL whitelist dict and migrate old contract whitelist to new ACL dict
    if runtime::get_key(ACL_WHITELIST).is_none() {
        storage::new_dictionary(ACL_WHITELIST)
            .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
        let contract_whitelist = utils::get_stored_value_with_user_errors::<Vec<ContractHash>>(
            CONTRACT_WHITELIST,
            NFTCoreError::MissingWhitelistMode,
            NFTCoreError::InvalidWhitelistMode,
        );

        // If mining mode is Installer and contract whitelist is not empty then migrate to minting
        // mode ACL and fill ACL_WHITELIST dictionnary
        if !contract_whitelist.is_empty() {
            let minting_mode: MintingMode = utils::get_stored_value_with_user_errors::<u8>(
                MINTING_MODE,
                NFTCoreError::MissingMintingMode,
                NFTCoreError::InvalidMintingMode,
            )
            .try_into()
            .unwrap_or_revert();

            // Migrate to ACL
            if MintingMode::Installer == minting_mode {
                runtime::put_key(
                    MINTING_MODE,
                    storage::new_uref(MintingMode::Acl as u8).into(),
                );
            }

            // Update acl whitelist
            for contract_hash in contract_whitelist.iter() {
                utils::upsert_dictionary_value_from_key(
                    ACL_WHITELIST,
                    &contract_hash.to_string(),
                    true,
                );
            }
        }
    }
}
