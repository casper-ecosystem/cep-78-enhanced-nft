#![no_main]
#![no_std]

extern crate alloc;
use alloc::{string::String, string::ToString, vec};
use casper_contract::contract_api::{runtime, storage};

use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::contracts::NamedKeys;
use casper_types::{
    runtime_args, CLType, ContractHash, ContractVersion, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, Parameter, RuntimeArgs, U256,
};

use nft_contract::*;

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

        // Given the token_id this entrypoint returns the operator, if any.
        // Reverts if token has been burnt.
        let get_approved = EntryPoint::new(
            ENTRY_POINT_GET_APPROVED,
            vec![Parameter::new(ARG_TOKEN_ID, CLType::U256)],
            CLType::String,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        // Given the owner account_hash this entrypoint returns number of owned tokens
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
        //     EntryPointAccess::Public,
        //     EntryPointType::Contract,
        // );

        // Given the owner account_hash this entrypoint returns number of owned tokens
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
