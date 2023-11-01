#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

mod constants;
mod error;
mod events;
mod metadata;
mod modalities;
mod utils;

extern crate alloc;

use alloc::{
    boxed::Box,
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use casper_contract::{
    contract_api::{
        runtime::{self, call_contract, revert},
        storage::{self},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    contracts::NamedKeys, runtime_args, CLType, CLValue, ContractHash, ContractPackageHash,
    EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Key, KeyTag, Parameter, RuntimeArgs,
    Tagged,
};
use constants::{
    ACCESS_KEY_NAME_1_0_0, ACL_PACKAGE_MODE, ACL_WHITELIST, ALLOW_MINTING, APPROVED,
    ARG_ACCESS_KEY_NAME_1_0_0, ARG_ACL_PACKAGE_MODE, ARG_ACL_WHITELIST,
    ARG_ADDITIONAL_REQUIRED_METADATA, ARG_ALLOW_MINTING, ARG_APPROVE_ALL, ARG_BURN_MODE,
    ARG_COLLECTION_NAME, ARG_COLLECTION_SYMBOL, ARG_CONTRACT_WHITELIST, ARG_EVENTS_MODE,
    ARG_HASH_KEY_NAME_1_0_0, ARG_HOLDER_MODE, ARG_IDENTIFIER_MODE, ARG_JSON_SCHEMA,
    ARG_METADATA_MUTABILITY, ARG_MINTING_MODE, ARG_NAMED_KEY_CONVENTION, ARG_NFT_KIND,
    ARG_NFT_METADATA_KIND, ARG_NFT_PACKAGE_KEY, ARG_OPERATOR, ARG_OPERATOR_BURN_MODE,
    ARG_OPTIONAL_METADATA, ARG_OWNERSHIP_MODE, ARG_OWNER_LOOKUP_MODE, ARG_PACKAGE_OPERATOR_MODE,
    ARG_RECEIPT_NAME, ARG_SOURCE_KEY, ARG_SPENDER, ARG_TARGET_KEY, ARG_TOKEN_HASH, ARG_TOKEN_ID,
    ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, ARG_TOTAL_TOKEN_SUPPLY, ARG_TRANSFER_FILTER_CONTRACT,
    ARG_WHITELIST_MODE, BURNT_TOKENS, BURN_MODE, COLLECTION_NAME, COLLECTION_SYMBOL,
    ENTRY_POINT_APPROVE, ENTRY_POINT_BALANCE_OF, ENTRY_POINT_BURN, ENTRY_POINT_GET_APPROVED,
    ENTRY_POINT_INIT, ENTRY_POINT_IS_APPROVED_FOR_ALL, ENTRY_POINT_METADATA, ENTRY_POINT_MIGRATE,
    ENTRY_POINT_MINT, ENTRY_POINT_OWNER_OF, ENTRY_POINT_REGISTER_OWNER, ENTRY_POINT_REVOKE,
    ENTRY_POINT_SET_APPROVALL_FOR_ALL, ENTRY_POINT_SET_TOKEN_METADATA, ENTRY_POINT_SET_VARIABLES,
    ENTRY_POINT_TRANSFER, ENTRY_POINT_UPDATED_RECEIPTS, EVENTS_MODE, HASH_BY_INDEX,
    HASH_KEY_NAME_1_0_0, HOLDER_MODE, IDENTIFIER_MODE, INDEX_BY_HASH, INSTALLER, JSON_SCHEMA,
    MAX_TOTAL_TOKEN_SUPPLY, METADATA_CEP78, METADATA_CUSTOM_VALIDATED, METADATA_MUTABILITY,
    METADATA_NFT721, METADATA_RAW, MINTING_MODE, NFT_KIND, NFT_METADATA_KIND, NFT_METADATA_KINDS,
    NUMBER_OF_MINTED_TOKENS, OPERATOR, OPERATORS, OPERATOR_BURN_MODE, OWNED_TOKENS, OWNERSHIP_MODE,
    PACKAGE_OPERATOR_MODE, PAGE_LIMIT, PAGE_TABLE, PREFIX_ACCESS_KEY_NAME, PREFIX_CEP78,
    PREFIX_CONTRACT_NAME, PREFIX_CONTRACT_VERSION, PREFIX_HASH_KEY_NAME, PREFIX_PAGE_DICTIONARY,
    RECEIPT_NAME, REPORTING_MODE, RLO_MFLAG, TOKEN_COUNT, TOKEN_ISSUERS, TOKEN_OWNERS,
    TOTAL_TOKEN_SUPPLY, TRANSFER_FILTER_CONTRACT, TRANSFER_FILTER_CONTRACT_METHOD,
    UNMATCHED_HASH_COUNT, WHITELIST_MODE,
};
use core::convert::{TryFrom, TryInto};
use error::NFTCoreError;
use events::{
    events_cep47::{record_cep47_event_dictionary, CEP47Event},
    events_ces::{
        Approval, ApprovalForAll, ApprovalRevoked, Burn, MetadataUpdated, Migration, Mint,
        RevokedForAll, Transfer, VariablesSet,
    },
};
use metadata::CustomMetadataSchema;
use modalities::{
    BurnMode, EventsMode, MetadataMutability, MintingMode, NFTHolderMode, NFTIdentifierMode,
    NFTKind, NFTMetadataKind, NamedKeyConventionMode, OwnerReverseLookupMode, OwnershipMode,
    Requirement, TokenIdentifier, TransferFilterContractResult, WhitelistMode,
};
use utils::Caller;

#[no_mangle]
pub extern "C" fn init() {
    // We only allow the init() entrypoint to be called once.
    // If COLLECTION_NAME uref already exists we revert since this implies that
    // the init() entrypoint has already been called.
    if utils::named_uref_exists(COLLECTION_NAME) {
        runtime::revert(NFTCoreError::ContractAlreadyInitialized);
    }

    // Only the installing account may call this method. All other callers are erroneous.
    let installing_account = utils::get_account_hash(
        INSTALLER,
        NFTCoreError::MissingInstaller,
        NFTCoreError::InvalidInstaller,
    );

    // We revert if caller is not the managing installing account
    if installing_account != runtime::get_caller() {
        runtime::revert(NFTCoreError::InvalidAccount)
    }

    // Start collecting the runtime arguments.
    let collection_name: String = utils::get_named_arg_with_user_errors(
        ARG_COLLECTION_NAME,
        NFTCoreError::MissingCollectionName,
        NFTCoreError::InvalidCollectionName,
    )
    .unwrap_or_revert();

    let collection_symbol: String = utils::get_named_arg_with_user_errors(
        ARG_COLLECTION_SYMBOL,
        NFTCoreError::MissingCollectionSymbol,
        NFTCoreError::InvalidCollectionSymbol,
    )
    .unwrap_or_revert();

    let total_token_supply: u64 = utils::get_named_arg_with_user_errors(
        ARG_TOTAL_TOKEN_SUPPLY,
        NFTCoreError::MissingTotalTokenSupply,
        NFTCoreError::InvalidTotalTokenSupply,
    )
    .unwrap_or_revert();

    if total_token_supply > MAX_TOTAL_TOKEN_SUPPLY {
        runtime::revert(NFTCoreError::ExceededMaxTotalSupply)
    }

    let allow_minting: bool = utils::get_named_arg_with_user_errors(
        ARG_ALLOW_MINTING,
        NFTCoreError::MissingMintingStatus,
        NFTCoreError::InvalidMintingStatus,
    )
    .unwrap_or_revert();

    let minting_mode: MintingMode = utils::get_named_arg_with_user_errors::<u8>(
        ARG_MINTING_MODE,
        NFTCoreError::MissingMintingMode,
        NFTCoreError::InvalidMintingMode,
    )
    .unwrap_or_revert()
    .try_into()
    .unwrap_or_revert();

    let ownership_mode: OwnershipMode = utils::get_named_arg_with_user_errors::<u8>(
        ARG_OWNERSHIP_MODE,
        NFTCoreError::MissingOwnershipMode,
        NFTCoreError::InvalidOwnershipMode,
    )
    .unwrap_or_revert()
    .try_into()
    .unwrap_or_revert();

    let nft_kind: NFTKind = utils::get_named_arg_with_user_errors::<u8>(
        ARG_NFT_KIND,
        NFTCoreError::MissingNftKind,
        NFTCoreError::InvalidNftKind,
    )
    .unwrap_or_revert()
    .try_into()
    .unwrap_or_revert();

    let holder_mode: NFTHolderMode = utils::get_named_arg_with_user_errors::<u8>(
        ARG_HOLDER_MODE,
        NFTCoreError::MissingHolderMode,
        NFTCoreError::InvalidHolderMode,
    )
    .unwrap_or_revert()
    .try_into()
    .unwrap_or_revert();

    let whitelist_mode: WhitelistMode = utils::get_named_arg_with_user_errors::<u8>(
        ARG_WHITELIST_MODE,
        NFTCoreError::MissingWhitelistMode,
        NFTCoreError::InvalidWhitelistMode,
    )
    .unwrap_or_revert()
    .try_into()
    .unwrap_or_revert();

    let acl_whitelist = utils::get_named_arg_with_user_errors::<Vec<Key>>(
        ARG_ACL_WHITELIST,
        NFTCoreError::MissingACLWhiteList,
        NFTCoreError::InvalidACLWhitelist,
    )
    .unwrap_or_revert();

    // Revert if minting mode is not ACL and acl list is not empty
    if MintingMode::Acl != minting_mode && !acl_whitelist.is_empty() {
        runtime::revert(NFTCoreError::InvalidMintingMode)
    }

    // Revert if minting mode is ACL or holder_mode is contracts and acl list is locked and empty
    if MintingMode::Acl == minting_mode
        && acl_whitelist.is_empty()
        && WhitelistMode::Locked == whitelist_mode
    {
        runtime::revert(NFTCoreError::EmptyACLWhitelist)
    }

    let acl_package_mode: bool = utils::get_named_arg_with_user_errors(
        ARG_ACL_PACKAGE_MODE,
        NFTCoreError::MissingACLPackageMode,
        NFTCoreError::InvalidACLPackageMode,
    )
    .unwrap_or_revert();

    let package_operator_mode: bool = utils::get_optional_named_arg_with_user_errors::<bool>(
        ARG_PACKAGE_OPERATOR_MODE,
        NFTCoreError::InvalidPackageOperatorMode,
    )
    .unwrap_or_default();

    let receipt_name: String = utils::get_named_arg_with_user_errors(
        ARG_RECEIPT_NAME,
        NFTCoreError::MissingReceiptName,
        NFTCoreError::InvalidReceiptName,
    )
    .unwrap_or_revert();

    let package_hash = utils::get_named_arg_with_user_errors::<String>(
        ARG_NFT_PACKAGE_KEY,
        NFTCoreError::MissingCep78PackageHash,
        NFTCoreError::InvalidCep78InvalidHash,
    )
    .unwrap_or_revert();

    let base_metadata_kind: NFTMetadataKind = utils::get_named_arg_with_user_errors::<u8>(
        ARG_NFT_METADATA_KIND,
        NFTCoreError::MissingNFTMetadataKind,
        NFTCoreError::InvalidNFTMetadataKind,
    )
    .unwrap_or_revert()
    .try_into()
    .unwrap_or_revert();

    let optional_metadata = utils::get_named_arg_with_user_errors::<Vec<u8>>(
        ARG_OPTIONAL_METADATA,
        NFTCoreError::MissingOptionalNFTMetadataKind,
        NFTCoreError::InvalidOptionalNFTMetadataKind,
    )
    .unwrap_or_revert();

    let additional_required_metadata = utils::get_named_arg_with_user_errors::<Vec<u8>>(
        ARG_ADDITIONAL_REQUIRED_METADATA,
        NFTCoreError::MissingAdditionalNFTMetadataKind,
        NFTCoreError::InvalidAdditionalNFTMetadataKind,
    )
    .unwrap_or_revert();

    let nft_metadata_kinds = utils::create_metadata_requirements(
        base_metadata_kind,
        additional_required_metadata,
        optional_metadata,
    );

    let json_schema: String = utils::get_named_arg_with_user_errors(
        ARG_JSON_SCHEMA,
        NFTCoreError::MissingJsonSchema,
        NFTCoreError::InvalidJsonSchema,
    )
    .unwrap_or_revert();

    // Check if schema is missing before checking its validity
    if base_metadata_kind == NFTMetadataKind::CustomValidated && json_schema.is_empty() {
        runtime::revert(NFTCoreError::MissingJsonSchema)
    }

    // Attempt to parse the provided schema if the CustomValidated metadata kind is required or
    // optional and fail installation if the schema cannot be parsed.
    if let Some(required_or_optional) = nft_metadata_kinds.get(&NFTMetadataKind::CustomValidated) {
        if required_or_optional == &Requirement::Required
            || required_or_optional == &Requirement::Optional
        {
            serde_json_wasm::from_str::<CustomMetadataSchema>(&json_schema)
                .map_err(|_| NFTCoreError::InvalidJsonSchema)
                .unwrap_or_revert();
        }
    }

    let identifier_mode: NFTIdentifierMode = utils::get_named_arg_with_user_errors::<u8>(
        ARG_IDENTIFIER_MODE,
        NFTCoreError::MissingIdentifierMode,
        NFTCoreError::InvalidIdentifierMode,
    )
    .unwrap_or_revert()
    .try_into()
    .unwrap_or_revert();

    let metadata_mutability: MetadataMutability = utils::get_named_arg_with_user_errors::<u8>(
        ARG_METADATA_MUTABILITY,
        NFTCoreError::MissingMetadataMutability,
        NFTCoreError::InvalidMetadataMutability,
    )
    .unwrap_or_revert()
    .try_into()
    .unwrap_or_revert();

    let burn_mode: BurnMode = utils::get_named_arg_with_user_errors::<u8>(
        ARG_BURN_MODE,
        NFTCoreError::MissingBurnMode,
        NFTCoreError::InvalidBurnMode,
    )
    .unwrap_or_revert()
    .try_into()
    .unwrap_or_revert();

    let operator_burn_mode: bool = utils::get_optional_named_arg_with_user_errors::<bool>(
        ARG_OPERATOR_BURN_MODE,
        NFTCoreError::InvalidOperatorBurnMode,
    )
    .unwrap_or_default();

    let reporting_mode: OwnerReverseLookupMode = utils::get_named_arg_with_user_errors::<u8>(
        ARG_OWNER_LOOKUP_MODE,
        NFTCoreError::MissingReportingMode,
        NFTCoreError::InvalidReportingMode,
    )
    .unwrap_or_revert()
    .try_into()
    .unwrap_or_revert();

    // OwnerReverseLookup TransfersOnly mode should be Transferable
    if OwnerReverseLookupMode::TransfersOnly == reporting_mode
        && OwnershipMode::Transferable != ownership_mode
    {
        runtime::revert(NFTCoreError::OwnerReverseLookupModeNotTransferable)
    }

    let transfer_filter_contract_contract_key =
        utils::get_optional_named_arg_with_user_errors::<Option<Key>>(
            ARG_TRANSFER_FILTER_CONTRACT,
            NFTCoreError::InvalidTransferFilterContract,
        )
        .unwrap_or_default();

    let transfer_filter_contract_contract_hash: Option<ContractHash> =
        transfer_filter_contract_contract_key.map(|transfer_filter_contract_contract_key| {
            ContractHash::from(
                transfer_filter_contract_contract_key
                    .into_hash()
                    .unwrap_or_revert(),
            )
        });

    if ownership_mode != OwnershipMode::Transferable
        && transfer_filter_contract_contract_hash.is_some()
    {
        runtime::revert(NFTCoreError::TransferFilterContractNeedsTransferableMode)
    }

    // Put all created URefs into the contract's context (necessary to retain access rights,
    // for future use).
    //
    // Initialize contract with URefs for all invariant values, which can never be changed.
    runtime::put_key(
        COLLECTION_NAME,
        storage::new_uref(collection_name.clone()).into(),
    );
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
    runtime::put_key(NFT_KIND, storage::new_uref(nft_kind as u8).into());
    runtime::put_key(JSON_SCHEMA, storage::new_uref(json_schema).into());
    runtime::put_key(MINTING_MODE, storage::new_uref(minting_mode as u8).into());
    runtime::put_key(HOLDER_MODE, storage::new_uref(holder_mode as u8).into());
    runtime::put_key(
        WHITELIST_MODE,
        storage::new_uref(whitelist_mode as u8).into(),
    );
    runtime::put_key(RECEIPT_NAME, storage::new_uref(receipt_name).into());
    runtime::put_key(
        &format!("{PREFIX_CEP78}_{collection_name}"),
        storage::new_uref(package_hash).into(),
    );
    runtime::put_key(
        NFT_METADATA_KIND,
        storage::new_uref(base_metadata_kind).into(),
    );
    runtime::put_key(
        NFT_METADATA_KINDS,
        storage::new_uref(nft_metadata_kinds).into(),
    );
    runtime::put_key(
        IDENTIFIER_MODE,
        storage::new_uref(identifier_mode as u8).into(),
    );
    runtime::put_key(
        METADATA_MUTABILITY,
        storage::new_uref(metadata_mutability as u8).into(),
    );
    runtime::put_key(BURN_MODE, storage::new_uref(burn_mode as u8).into());
    runtime::put_key(
        OPERATOR_BURN_MODE,
        storage::new_uref(operator_burn_mode).into(),
    );

    let events_mode: EventsMode = utils::get_named_arg_with_user_errors::<u8>(
        ARG_EVENTS_MODE,
        NFTCoreError::MissingEventsMode,
        NFTCoreError::InvalidEventsMode,
    )
    .unwrap_or_revert()
    .try_into()
    .unwrap_or_revert();

    // Initialize events structures for CES.
    if let EventsMode::CES = events_mode {
        utils::init_events();
    }
    runtime::put_key(EVENTS_MODE, storage::new_uref(events_mode as u8).into());

    // Initialize contract with variables which must be present but maybe set to
    // different values after initialization.
    runtime::put_key(ALLOW_MINTING, storage::new_uref(allow_minting).into());
    // This is an internal variable that the installing account cannot change
    // but is incremented by the contract itself.
    runtime::put_key(NUMBER_OF_MINTED_TOKENS, storage::new_uref(0u64).into());

    // Create the data dictionaries to store essential values, topically.
    storage::new_dictionary(TOKEN_OWNERS)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(TOKEN_ISSUERS)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(APPROVED).unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(OPERATORS)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(BURNT_TOKENS)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(TOKEN_COUNT)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(METADATA_CUSTOM_VALIDATED)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(METADATA_CEP78)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(METADATA_NFT721)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(METADATA_RAW)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(HASH_BY_INDEX)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(INDEX_BY_HASH)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(PAGE_TABLE)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    storage::new_dictionary(ACL_WHITELIST)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);

    for key in acl_whitelist.iter() {
        utils::upsert_dictionary_value_from_key(
            ACL_WHITELIST,
            &utils::encode_dictionary_item_key(*key),
            true,
        );
    }

    runtime::put_key(ACL_PACKAGE_MODE, storage::new_uref(acl_package_mode).into());
    runtime::put_key(
        PACKAGE_OPERATOR_MODE,
        storage::new_uref(package_operator_mode).into(),
    );

    if vec![
        OwnerReverseLookupMode::Complete,
        OwnerReverseLookupMode::TransfersOnly,
    ]
    .contains(&reporting_mode)
    {
        let page_table_width = utils::max_number_of_pages(total_token_supply);
        runtime::put_key(PAGE_LIMIT, storage::new_uref(page_table_width).into());
    }
    runtime::put_key(
        REPORTING_MODE,
        storage::new_uref(reporting_mode as u8).into(),
    );
    runtime::put_key(RLO_MFLAG, storage::new_uref(false).into());

    if let Some(transfer_filter_contract) = transfer_filter_contract_contract_hash {
        runtime::put_key(
            TRANSFER_FILTER_CONTRACT,
            storage::new_uref(transfer_filter_contract).into(),
        );
    }
}

// set_variables allows the user to set any variable or any combination of variables simultaneously.
// set variables defines what variables are mutable and immutable.
#[no_mangle]
pub extern "C" fn set_variables() {
    let installer = utils::get_account_hash(
        INSTALLER,
        NFTCoreError::MissingInstaller,
        NFTCoreError::InvalidInstaller,
    );

    // Only the installing account can change the mutable variables.
    if installer != runtime::get_caller() {
        runtime::revert(NFTCoreError::InvalidAccount);
    }

    if let Some(allow_minting) = utils::get_optional_named_arg_with_user_errors::<bool>(
        ARG_ALLOW_MINTING,
        NFTCoreError::InvalidAllowMinting,
    ) {
        let allow_minting_uref = utils::get_uref(
            ALLOW_MINTING,
            NFTCoreError::MissingAllowMinting,
            NFTCoreError::InvalidAllowMinting,
        );
        storage::write(allow_minting_uref, allow_minting);
    }

    if let Some(acl_package_mode) = utils::get_optional_named_arg_with_user_errors::<bool>(
        ARG_ACL_PACKAGE_MODE,
        NFTCoreError::MissingACLPackageMode,
    ) {
        let acl_package_mode_uref = utils::get_uref(
            ACL_PACKAGE_MODE,
            NFTCoreError::MissingACLPackageMode,
            NFTCoreError::InvalidACLPackageMode,
        );
        storage::write(acl_package_mode_uref, acl_package_mode);
    }

    if let Some(package_operator_mode) = utils::get_optional_named_arg_with_user_errors::<bool>(
        ARG_PACKAGE_OPERATOR_MODE,
        NFTCoreError::MissingACLPackageMode,
    ) {
        let package_operator_mode_uref = utils::get_uref(
            PACKAGE_OPERATOR_MODE,
            NFTCoreError::MissingPackageOperatorMode,
            NFTCoreError::InvalidPackageOperatorMode,
        );
        storage::write(package_operator_mode_uref, package_operator_mode);
    }

    if let Some(operator_burn_mode) = utils::get_optional_named_arg_with_user_errors::<bool>(
        ARG_OPERATOR_BURN_MODE,
        NFTCoreError::MissingOperatorBurnMode,
    ) {
        let operator_burn_mode_uref = utils::get_uref(
            OPERATOR_BURN_MODE,
            NFTCoreError::MissingOperatorBurnMode,
            NFTCoreError::InvalidOperatorBurnMode,
        );
        storage::write(operator_burn_mode_uref, operator_burn_mode);
    }

    let mut new_acl_whitelist = utils::get_optional_named_arg_with_user_errors::<Vec<Key>>(
        ARG_ACL_WHITELIST,
        NFTCoreError::InvalidACLWhitelist,
    )
    .unwrap_or_default();

    // Deprecated in 1.4 in favor of above ARG_ACL_WHITELIST
    let new_contract_whitelist =
        utils::get_optional_named_arg_with_user_errors::<Vec<ContractHash>>(
            ARG_CONTRACT_WHITELIST,
            NFTCoreError::InvalidContractWhitelist,
        )
        .unwrap_or_default();

    for contract_hash in new_contract_whitelist.iter() {
        new_acl_whitelist.push(Key::from(*contract_hash));
    }

    if !new_acl_whitelist.is_empty() {
        let whitelist_mode: WhitelistMode = utils::get_stored_value_with_user_errors::<u8>(
            WHITELIST_MODE,
            NFTCoreError::MissingWhitelistMode,
            NFTCoreError::InvalidWhitelistMode,
        )
        .try_into()
        .unwrap_or_revert();
        match whitelist_mode {
            WhitelistMode::Unlocked => {
                // Clear acl whitelist
                runtime::remove_key(ACL_WHITELIST);
                storage::new_dictionary(ACL_WHITELIST)
                    .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
                // Update acl whitelist
                for key in new_acl_whitelist.iter() {
                    utils::upsert_dictionary_value_from_key(
                        ACL_WHITELIST,
                        &utils::encode_dictionary_item_key(*key),
                        true,
                    );
                }
            }
            WhitelistMode::Locked => runtime::revert(NFTCoreError::InvalidWhitelistMode),
        }
    }

    let events_mode: EventsMode = utils::get_stored_value_with_user_errors::<u8>(
        EVENTS_MODE,
        NFTCoreError::MissingEventsMode,
        NFTCoreError::InvalidEventsMode,
    )
    .try_into()
    .unwrap_or_revert();

    // Emit VariablesSet event.
    match events_mode {
        EventsMode::NoEvents => {}
        EventsMode::CEP47 => record_cep47_event_dictionary(CEP47Event::VariablesSet),
        EventsMode::CES => casper_event_standard::emit(VariablesSet::new()),
    }
}

// Mints a new token. Minting will fail if allow_minting is set to false.
#[no_mangle]
pub extern "C" fn mint() {
    // The contract owner can toggle the minting behavior on and off over time.
    // The contract is toggled on by default.
    let minting_status = utils::get_stored_value_with_user_errors::<bool>(
        ALLOW_MINTING,
        NFTCoreError::MissingAllowMinting,
        NFTCoreError::InvalidAllowMinting,
    );

    // If contract minting behavior is currently toggled off we revert.
    if !minting_status {
        runtime::revert(NFTCoreError::MintingIsPaused);
    }

    let total_token_supply = utils::get_stored_value_with_user_errors::<u64>(
        TOTAL_TOKEN_SUPPLY,
        NFTCoreError::MissingTotalTokenSupply,
        NFTCoreError::InvalidTotalTokenSupply,
    );

    // The minted_tokens_count is the number of minted tokens so far.
    let minted_tokens_count = utils::get_stored_value_with_user_errors::<u64>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    // Revert if the token supply has been exhausted.
    if minted_tokens_count >= total_token_supply {
        runtime::revert(NFTCoreError::TokenSupplyDepleted);
    }

    let minting_mode: MintingMode = utils::get_stored_value_with_user_errors::<u8>(
        MINTING_MODE,
        NFTCoreError::MissingMintingMode,
        NFTCoreError::InvalidMintingMode,
    )
    .try_into()
    .unwrap_or_revert();

    let (caller, contract_package): (Key, Option<Key>) =
        match utils::get_verified_caller().unwrap_or_revert() {
            Caller::Session(account_hash) => (account_hash.into(), None),
            Caller::StoredCaller(contract_hash, contract_package_hash) => {
                (contract_hash.into(), Some(contract_package_hash.into()))
            }
        };

    // Revert if minting is private and caller is not installer.
    if MintingMode::Installer == minting_mode {
        match caller.tag() {
            KeyTag::Account => {
                let installer_account = runtime::get_key(INSTALLER)
                    .unwrap_or_revert_with(NFTCoreError::MissingInstallerKey)
                    .into_account()
                    .unwrap_or_revert_with(NFTCoreError::FailedToConvertToAccountHash);

                // Revert if private minting is required and caller is not installer.
                if runtime::get_caller() != installer_account {
                    runtime::revert(NFTCoreError::InvalidMinter)
                }
            }
            _ => runtime::revert(NFTCoreError::InvalidKey),
        }
    }

    // Revert if minting is acl and caller is not whitelisted.
    if MintingMode::Acl == minting_mode {
        let acl_package_mode: bool = utils::get_stored_value_with_user_errors::<bool>(
            ACL_PACKAGE_MODE,
            NFTCoreError::MissingACLPackageMode,
            NFTCoreError::InvalidACLPackageMode,
        );
        let is_whitelisted = match (acl_package_mode, contract_package) {
            (true, Some(contract_package)) => utils::get_dictionary_value_from_key::<bool>(
                ACL_WHITELIST,
                &utils::encode_dictionary_item_key(contract_package),
            )
            .unwrap_or_default(),
            _ => utils::get_dictionary_value_from_key::<bool>(
                ACL_WHITELIST,
                &utils::encode_dictionary_item_key(caller),
            )
            .unwrap_or_default(),
        };

        match caller.tag() {
            KeyTag::Hash => {
                if !is_whitelisted {
                    runtime::revert(NFTCoreError::UnlistedContractHash)
                }
            }
            KeyTag::Account => {
                if !is_whitelisted {
                    runtime::revert(NFTCoreError::InvalidMinter);
                }
            }
            _ => runtime::revert(NFTCoreError::InvalidKey),
        }
    }

    let metadata_kinds: BTreeMap<NFTMetadataKind, Requirement> =
        utils::get_stored_value_with_user_errors(
            NFT_METADATA_KINDS,
            NFTCoreError::MissingNFTMetadataKind,
            NFTCoreError::InvalidNFTMetadataKind,
        );

    let token_metadata = utils::get_named_arg_with_user_errors::<String>(
        ARG_TOKEN_META_DATA,
        NFTCoreError::MissingTokenMetaData,
        NFTCoreError::InvalidTokenMetaData,
    )
    .unwrap_or_revert();

    let identifier_mode: NFTIdentifierMode = utils::get_stored_value_with_user_errors::<u8>(
        IDENTIFIER_MODE,
        NFTCoreError::MissingIdentifierMode,
        NFTCoreError::InvalidIdentifierMode,
    )
    .try_into()
    .unwrap_or_revert();

    // This is the token ID.
    let optional_token_hash: String = utils::get_optional_named_arg_with_user_errors::<String>(
        ARG_TOKEN_HASH,
        NFTCoreError::InvalidIdentifier,
    )
    .unwrap_or_default();
    let token_identifier: TokenIdentifier = match identifier_mode {
        NFTIdentifierMode::Ordinal => TokenIdentifier::Index(minted_tokens_count),
        NFTIdentifierMode::Hash => TokenIdentifier::Hash(if optional_token_hash.is_empty() {
            base16::encode_lower(&runtime::blake2b(token_metadata.clone()))
        } else {
            optional_token_hash
        }),
    };

    for (metadata_kind, required) in metadata_kinds {
        if required == Requirement::Unneeded {
            continue;
        }
        let token_metadata_validation =
            metadata::validate_metadata(&metadata_kind, token_metadata.clone());
        match token_metadata_validation {
            Ok(validated_token_metadata) => {
                utils::upsert_dictionary_value_from_key(
                    &metadata::get_metadata_dictionary_name(&metadata_kind),
                    &token_identifier.get_dictionary_item_key(),
                    validated_token_metadata,
                );
            }
            Err(err) => {
                if required == Requirement::Required {
                    runtime::revert(err);
                }
            }
        }
    }

    // The contract's ownership behavior (determined at installation) determines,
    // who owns the NFT we are about to mint.()
    let ownership_mode = utils::get_ownership_mode().unwrap_or_revert();
    let token_owner_key: Key =
        if let OwnershipMode::Assigned | OwnershipMode::Transferable = ownership_mode {
            runtime::get_named_arg(ARG_TOKEN_OWNER)
        } else {
            caller
        };

    utils::upsert_dictionary_value_from_key(
        TOKEN_OWNERS,
        &token_identifier.get_dictionary_item_key(),
        token_owner_key,
    );
    utils::upsert_dictionary_value_from_key(
        TOKEN_ISSUERS,
        &token_identifier.get_dictionary_item_key(),
        caller,
    );
    let owned_tokens_item_key = utils::encode_dictionary_item_key(token_owner_key);

    if let NFTIdentifierMode::Hash = identifier_mode {
        // Update the forward and reverse trackers
        utils::insert_hash_id_lookups(minted_tokens_count, token_identifier.clone());
    }

    //Increment the count of owned tokens.
    let updated_token_count =
        match utils::get_dictionary_value_from_key::<u64>(TOKEN_COUNT, &owned_tokens_item_key) {
            Some(balance) => balance + 1u64,
            None => 1u64,
        };
    utils::upsert_dictionary_value_from_key(
        TOKEN_COUNT,
        &owned_tokens_item_key,
        updated_token_count,
    );

    // Increment number_of_minted_tokens by one
    let number_of_minted_tokens_uref = utils::get_uref(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingTotalTokenSupply,
        NFTCoreError::InvalidTotalTokenSupply,
    );
    storage::write(number_of_minted_tokens_uref, minted_tokens_count + 1u64);

    // Emit Mint event.
    let events_mode: EventsMode =
        EventsMode::try_from(utils::get_stored_value_with_user_errors::<u8>(
            EVENTS_MODE,
            NFTCoreError::MissingEventsMode,
            NFTCoreError::InvalidEventsMode,
        ))
        .unwrap_or_revert();

    match events_mode {
        EventsMode::NoEvents => {}
        EventsMode::CES => casper_event_standard::emit(Mint::new(
            token_owner_key,
            token_identifier.clone(),
            token_metadata,
        )),
        EventsMode::CEP47 => record_cep47_event_dictionary(CEP47Event::Mint {
            recipient: token_owner_key,
            token_id: token_identifier.clone(),
        }),
    }

    if let OwnerReverseLookupMode::Complete = utils::get_reporting_mode() {
        if (NFTIdentifierMode::Hash == identifier_mode)
            && runtime::get_key(OWNED_TOKENS).is_some()
            && utils::should_migrate_token_hashes(token_owner_key)
        {
            utils::migrate_token_hashes(token_owner_key)
        }

        let (page_table_entry, page_uref) = utils::add_page_entry_and_page_record(
            minted_tokens_count,
            &owned_tokens_item_key,
            true,
        );

        let receipt_string = utils::get_receipt_name(page_table_entry);
        let receipt_address = Key::dictionary(page_uref, owned_tokens_item_key.as_bytes());
        let token_identifier_string = token_identifier.get_dictionary_item_key();

        let receipt = CLValue::from_t((receipt_string, receipt_address, token_identifier_string))
            .unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);
        runtime::ret(receipt)
    }
}

// Marks token as burnt. This blocks any future call to transfer token.
#[no_mangle]
pub extern "C" fn burn() {
    if let BurnMode::NonBurnable = utils::get_burn_mode() {
        runtime::revert(NFTCoreError::InvalidBurnMode)
    }

    let identifier_mode: NFTIdentifierMode = utils::get_stored_value_with_user_errors::<u8>(
        IDENTIFIER_MODE,
        NFTCoreError::MissingIdentifierMode,
        NFTCoreError::InvalidIdentifierMode,
    )
    .try_into()
    .unwrap_or_revert();

    let token_identifier = utils::get_token_identifier_from_runtime_args(&identifier_mode);

    let (caller, contract_package): (Key, Option<Key>) =
        match utils::get_verified_caller().unwrap_or_revert() {
            Caller::Session(account_hash) => (account_hash.into(), None),
            Caller::StoredCaller(contract_hash, contract_package_hash) => {
                (contract_hash.into(), Some(contract_package_hash.into()))
            }
        };

    let token_owner = match utils::get_dictionary_value_from_key::<Key>(
        TOKEN_OWNERS,
        &token_identifier.get_dictionary_item_key(),
    ) {
        Some(owner) => owner,
        None => runtime::revert(NFTCoreError::MissingOwnerTokenIdentifierKey),
    };

    // Check if caller is owner
    let is_owner = token_owner == caller;

    // Check if caller is operator to execute burn
    let is_operator = if !is_owner {
        let owner_operator_item_key = utils::encode_key_and_value(&token_owner, &caller);
        utils::get_dictionary_value_from_key::<bool>(OPERATORS, &owner_operator_item_key)
            .unwrap_or_default()
    } else {
        false
    };

    // With operator package mode check if caller's package is operator to let contract execute burn
    let is_package_operator = if !is_owner && !is_operator {
        match (
            utils::get_stored_value_with_user_errors::<bool>(
                PACKAGE_OPERATOR_MODE,
                NFTCoreError::MissingPackageOperatorMode,
                NFTCoreError::InvalidPackageOperatorMode,
            ),
            contract_package,
        ) {
            (true, Some(contract_package)) => {
                let owner_operator_item_key =
                    utils::encode_key_and_value(&token_owner, &contract_package);
                utils::get_dictionary_value_from_key::<bool>(OPERATORS, &owner_operator_item_key)
                    .unwrap_or_default()
            }
            _ => false,
        }
    } else {
        false
    };

    // Revert if caller is not token_owner nor operator for the owner
    if !is_owner && !is_operator && !is_package_operator {
        runtime::revert(NFTCoreError::InvalidTokenOwner)
    };

    // It makes sense to keep this token as owned by the caller. It just happens that the caller
    // owns a burnt token. That's all. Similarly, we should probably also not change the
    // owned_tokens dictionary.
    if utils::is_token_burned(&token_identifier) {
        runtime::revert(NFTCoreError::PreviouslyBurntToken)
    }

    // Mark the token as burnt by adding the token_id to the burnt tokens dictionary.
    utils::upsert_dictionary_value_from_key::<()>(
        BURNT_TOKENS,
        &token_identifier.get_dictionary_item_key(),
        (),
    );

    let owned_tokens_item_key = utils::encode_dictionary_item_key(token_owner);

    let updated_balance =
        match utils::get_dictionary_value_from_key::<u64>(TOKEN_COUNT, &owned_tokens_item_key) {
            Some(balance) => {
                if balance > 0u64 {
                    balance - 1u64
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

    utils::upsert_dictionary_value_from_key(TOKEN_COUNT, &owned_tokens_item_key, updated_balance);

    // Emit Burn event.
    let events_mode: EventsMode =
        EventsMode::try_from(utils::get_stored_value_with_user_errors::<u8>(
            EVENTS_MODE,
            NFTCoreError::MissingEventsMode,
            NFTCoreError::InvalidEventsMode,
        ))
        .unwrap_or_revert();

    match events_mode {
        EventsMode::NoEvents => {}
        EventsMode::CES => {
            casper_event_standard::emit(Burn::new(token_owner, token_identifier, caller))
        }
        EventsMode::CEP47 => record_cep47_event_dictionary(CEP47Event::Burn {
            owner: token_owner,
            token_id: token_identifier,
            burner: caller,
        }),
    }
}

// Marks an account as approved for an identified token transfer
#[no_mangle]
pub extern "C" fn approve() {
    // If we are in minter or assigned mode it makes no sense to approve an account. Hence we
    // revert.
    if let OwnershipMode::Minter | OwnershipMode::Assigned =
        utils::get_ownership_mode().unwrap_or_revert()
    {
        runtime::revert(NFTCoreError::InvalidOwnershipMode)
    }

    let (caller, contract_package): (Key, Option<Key>) =
        match utils::get_verified_caller().unwrap_or_revert() {
            Caller::Session(account_hash) => (account_hash.into(), None),
            Caller::StoredCaller(contract_hash, contract_package_hash) => {
                (contract_hash.into(), Some(contract_package_hash.into()))
            }
        };

    let identifier_mode: NFTIdentifierMode = utils::get_stored_value_with_user_errors::<u8>(
        IDENTIFIER_MODE,
        NFTCoreError::MissingIdentifierMode,
        NFTCoreError::InvalidIdentifierMode,
    )
    .try_into()
    .unwrap_or_revert();

    let token_id = utils::get_token_identifier_from_runtime_args(&identifier_mode);
    let token_identifier_dictionary_key = token_id.get_dictionary_item_key();

    let number_of_minted_tokens = utils::get_stored_value_with_user_errors::<u64>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    if let NFTIdentifierMode::Ordinal = identifier_mode {
        // Revert if token_id is out of bounds
        if let TokenIdentifier::Index(index) = &token_id {
            if *index >= number_of_minted_tokens {
                runtime::revert(NFTCoreError::InvalidTokenIdentifier);
            }
        }
    }

    let owner = match utils::get_dictionary_value_from_key::<Key>(
        TOKEN_OWNERS,
        &token_identifier_dictionary_key,
    ) {
        Some(owner) => owner,
        None => runtime::revert(NFTCoreError::MissingOwnerTokenIdentifierKey),
    };

    // Revert if caller is not token owner nor operator.
    // Only the token owner or an operator can approve an account
    let is_owner = caller == owner;
    let is_operator = !is_owner
        && utils::get_dictionary_value_from_key::<bool>(
            OPERATORS,
            &utils::encode_key_and_value(&owner, &caller),
        )
        .unwrap_or_default();

    let is_package_operator = if !is_owner && !is_operator {
        match (
            utils::get_stored_value_with_user_errors::<bool>(
                PACKAGE_OPERATOR_MODE,
                NFTCoreError::MissingPackageOperatorMode,
                NFTCoreError::InvalidPackageOperatorMode,
            ),
            contract_package,
        ) {
            (true, Some(contract_package)) => {
                let owner_operator_item_key =
                    utils::encode_key_and_value(&owner, &contract_package);
                utils::get_dictionary_value_from_key::<bool>(OPERATORS, &owner_operator_item_key)
                    .unwrap_or_default()
            }
            _ => false,
        }
    } else {
        false
    };

    if !is_owner && !is_operator && !is_package_operator {
        runtime::revert(NFTCoreError::InvalidTokenOwner);
    }

    // We assume a burnt token cannot be approved
    if utils::is_token_burned(&token_id) {
        runtime::revert(NFTCoreError::PreviouslyBurntToken)
    }

    let spender = match utils::get_optional_named_arg_with_user_errors::<Key>(
        ARG_OPERATOR, // Deprecated in favor of ARG_SPENDER
        NFTCoreError::InvalidApprovedAccountHash,
    ) {
        Some(deprecated_operator) => deprecated_operator,
        None => utils::get_named_arg_with_user_errors::<Key>(
            ARG_SPENDER,
            NFTCoreError::MissingSpenderAccountHash,
            NFTCoreError::InvalidSpenderAccountHash,
        )
        .unwrap_or_revert(),
    };

    // If token owner or operator tries to approve itself that's probably a mistake and we revert.
    if caller == spender {
        runtime::revert(NFTCoreError::InvalidAccount);
    }

    utils::upsert_dictionary_value_from_key(
        APPROVED,
        &token_identifier_dictionary_key,
        Some(spender),
    );

    let events_mode = EventsMode::try_from(utils::get_stored_value_with_user_errors::<u8>(
        crate::constants::EVENTS_MODE,
        NFTCoreError::MissingEventsMode,
        NFTCoreError::InvalidEventsMode,
    ))
    .unwrap_or_revert();

    // Emit Approval event.
    match events_mode {
        EventsMode::NoEvents => {}
        EventsMode::CES => casper_event_standard::emit(Approval::new(owner, spender, token_id)),
        EventsMode::CEP47 => record_cep47_event_dictionary(CEP47Event::ApprovalGranted {
            owner,
            spender,
            token_id,
        }),
    };
}

// Revokes an account as approved for an identified token transfer
#[no_mangle]
pub extern "C" fn revoke() {
    // If we are in minter or assigned mode it makes no sense to approve an account. Hence we
    // revert.
    if let OwnershipMode::Minter | OwnershipMode::Assigned =
        utils::get_ownership_mode().unwrap_or_revert()
    {
        runtime::revert(NFTCoreError::InvalidOwnershipMode)
    }

    let (caller, contract_package): (Key, Option<Key>) =
        match utils::get_verified_caller().unwrap_or_revert() {
            Caller::Session(account_hash) => (account_hash.into(), None),
            Caller::StoredCaller(contract_hash, contract_package_hash) => {
                (contract_hash.into(), Some(contract_package_hash.into()))
            }
        };

    let identifier_mode: NFTIdentifierMode = utils::get_stored_value_with_user_errors::<u8>(
        IDENTIFIER_MODE,
        NFTCoreError::MissingIdentifierMode,
        NFTCoreError::InvalidIdentifierMode,
    )
    .try_into()
    .unwrap_or_revert();

    let token_id = utils::get_token_identifier_from_runtime_args(&identifier_mode);
    let token_identifier_dictionary_key = token_id.get_dictionary_item_key();

    let number_of_minted_tokens = utils::get_stored_value_with_user_errors::<u64>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    if let NFTIdentifierMode::Ordinal = identifier_mode {
        // Revert if token_id is out of bounds
        if let TokenIdentifier::Index(index) = &token_id {
            if *index >= number_of_minted_tokens {
                runtime::revert(NFTCoreError::InvalidTokenIdentifier);
            }
        }
    }

    let owner = match utils::get_dictionary_value_from_key::<Key>(
        TOKEN_OWNERS,
        &token_identifier_dictionary_key,
    ) {
        Some(owner) => owner,
        None => runtime::revert(NFTCoreError::MissingOwnerTokenIdentifierKey),
    };

    // Revert if caller is not the token owner or an operator. Only the token owner / operators can
    // revoke an approved account
    let is_owner = caller == owner;
    let is_operator = !is_owner
        && utils::get_dictionary_value_from_key::<bool>(
            OPERATORS,
            &utils::encode_key_and_value(&owner, &caller),
        )
        .unwrap_or_default();

    let is_package_operator = if !is_owner && !is_operator {
        match (
            utils::get_stored_value_with_user_errors::<bool>(
                PACKAGE_OPERATOR_MODE,
                NFTCoreError::MissingPackageOperatorMode,
                NFTCoreError::InvalidPackageOperatorMode,
            ),
            contract_package,
        ) {
            (true, Some(contract_package)) => {
                let owner_operator_item_key =
                    utils::encode_key_and_value(&owner, &contract_package);
                utils::get_dictionary_value_from_key::<bool>(OPERATORS, &owner_operator_item_key)
                    .unwrap_or_default()
            }
            _ => false,
        }
    } else {
        false
    };

    if !is_owner && !is_operator && !is_package_operator {
        runtime::revert(NFTCoreError::InvalidTokenOwner);
    }

    // We assume a burnt token cannot be revoked
    if utils::is_token_burned(&token_id) {
        runtime::revert(NFTCoreError::PreviouslyBurntToken)
    }

    utils::upsert_dictionary_value_from_key(
        APPROVED,
        &token_identifier_dictionary_key,
        Option::<Key>::None,
    );

    let events_mode = EventsMode::try_from(utils::get_stored_value_with_user_errors::<u8>(
        crate::constants::EVENTS_MODE,
        NFTCoreError::MissingEventsMode,
        NFTCoreError::InvalidEventsMode,
    ))
    .unwrap_or_revert();

    // Emit ApprovalRevoked event.
    match events_mode {
        EventsMode::NoEvents => {}
        EventsMode::CES => casper_event_standard::emit(ApprovalRevoked::new(owner, token_id)),
        EventsMode::CEP47 => {
            record_cep47_event_dictionary(CEP47Event::ApprovalRevoked { owner, token_id })
        }
    };
}

// Approves the specified operator for transfer of owner's tokens.
#[no_mangle]
pub extern "C" fn set_approval_for_all() {
    // If we are in minter or assigned mode it makes no sense to approve an operator. Hence we
    // revert.
    if let OwnershipMode::Minter | OwnershipMode::Assigned =
        utils::get_ownership_mode().unwrap_or_revert()
    {
        runtime::revert(NFTCoreError::InvalidOwnershipMode)
    }

    // If approve_all is true we approve operator for all caller owned tokens.
    // If false we set operator to None for all caller owned tokens
    let approve_all = utils::get_named_arg_with_user_errors::<bool>(
        ARG_APPROVE_ALL,
        NFTCoreError::MissingApproveAll,
        NFTCoreError::InvalidApproveAll,
    )
    .unwrap_or_revert();

    let caller: Key = match utils::get_verified_caller().unwrap_or_revert() {
        Caller::Session(account_hash) => account_hash.into(),
        Caller::StoredCaller(contract_hash, _) => contract_hash.into(),
    };

    let operator = utils::get_named_arg_with_user_errors::<Key>(
        ARG_OPERATOR,
        NFTCoreError::MissingOperator,
        NFTCoreError::InvalidOperator,
    )
    .unwrap_or_revert();

    // If caller tries to approve itself as operator that's probably a mistake and we revert.
    if caller == operator {
        runtime::revert(NFTCoreError::InvalidAccount);
    }

    // Depending on approve_all we either approve all or disapprove all.
    let owner_operator_item_key = utils::encode_key_and_value(&caller, &operator);
    utils::upsert_dictionary_value_from_key(OPERATORS, &owner_operator_item_key, approve_all);

    let events_mode: EventsMode = utils::get_stored_value_with_user_errors::<u8>(
        EVENTS_MODE,
        NFTCoreError::MissingEventsMode,
        NFTCoreError::InvalidEventsMode,
    )
    .try_into()
    .unwrap_or_revert();

    match events_mode {
        EventsMode::NoEvents => {}
        EventsMode::CES => {
            if approve_all {
                casper_event_standard::emit(ApprovalForAll::new(caller, operator));
            } else {
                casper_event_standard::emit(RevokedForAll::new(caller, operator));
            }
        }
        EventsMode::CEP47 => {
            if approve_all {
                record_cep47_event_dictionary(CEP47Event::ApprovalForAll {
                    owner: caller,
                    operator,
                });
            } else {
                record_cep47_event_dictionary(CEP47Event::RevokedForAll {
                    owner: caller,
                    operator,
                });
            }
        }
    }
}

// Returns a boolean state if an account is operator for an owner
#[no_mangle]
pub extern "C" fn is_approved_for_all() {
    let owner_key = utils::get_named_arg_with_user_errors::<Key>(
        ARG_TOKEN_OWNER,
        NFTCoreError::MissingAccountHash,
        NFTCoreError::InvalidAccountHash,
    )
    .unwrap_or_revert();

    let operator = utils::get_named_arg_with_user_errors::<Key>(
        ARG_OPERATOR,
        NFTCoreError::MissingOperator,
        NFTCoreError::InvalidOperator,
    )
    .unwrap_or_revert();

    let owner_operator_item_key = utils::encode_key_and_value(&owner_key, &operator);

    let is_operator =
        utils::get_dictionary_value_from_key::<bool>(OPERATORS, &owner_operator_item_key)
            .unwrap_or_default();

    let operator_cl_value =
        CLValue::from_t(is_operator).unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);

    runtime::ret(operator_cl_value);
}

// Transfers token from token owner to specified account. Transfer will go through if caller is
// owner or an approved account or an operator. Transfer will fail if OwnershipMode is Minter or
// Assigned.
#[no_mangle]
pub extern "C" fn transfer() {
    // If we are in minter or assigned mode we are not allowed to transfer ownership of token, hence
    // we revert.
    if let OwnershipMode::Minter | OwnershipMode::Assigned =
        utils::get_ownership_mode().unwrap_or_revert()
    {
        runtime::revert(NFTCoreError::InvalidOwnershipMode)
    }

    let identifier_mode: NFTIdentifierMode = utils::get_stored_value_with_user_errors::<u8>(
        IDENTIFIER_MODE,
        NFTCoreError::MissingIdentifierMode,
        NFTCoreError::InvalidIdentifierMode,
    )
    .try_into()
    .unwrap_or_revert();

    let token_identifier = utils::get_token_identifier_from_runtime_args(&identifier_mode);

    // We assume we cannot transfer burnt tokens
    if utils::is_token_burned(&token_identifier) {
        runtime::revert(NFTCoreError::PreviouslyBurntToken)
    }

    let owner = match utils::get_dictionary_value_from_key::<Key>(
        TOKEN_OWNERS,
        &token_identifier.get_dictionary_item_key(),
    ) {
        Some(owner) => owner,
        None => runtime::revert(NFTCoreError::MissingOwnerTokenIdentifierKey),
    };

    let source_owner_key = utils::get_named_arg_with_user_errors::<Key>(
        ARG_SOURCE_KEY,
        NFTCoreError::MissingAccountHash,
        NFTCoreError::InvalidAccountHash,
    )
    .unwrap_or_revert();

    if source_owner_key != owner {
        runtime::revert(NFTCoreError::InvalidAccount);
    }

    let (caller, contract_package): (Key, Option<Key>) =
        match utils::get_verified_caller().unwrap_or_revert() {
            Caller::Session(account_hash) => (account_hash.into(), None),
            Caller::StoredCaller(contract_hash, contract_package_hash) => {
                (contract_hash.into(), Some(contract_package_hash.into()))
            }
        };

    // Check if caller is owner
    let is_owner = owner == caller;

    // Check if caller is approved to execute transfer
    let is_approved = !is_owner
        && match utils::get_dictionary_value_from_key::<Option<Key>>(
            APPROVED,
            &token_identifier.get_dictionary_item_key(),
        ) {
            Some(Some(maybe_approved)) => caller == maybe_approved,
            Some(None) | None => false,
        };

    // Check if caller is operator to execute transfer
    let is_operator = if !is_owner && !is_approved {
        let owner_operator_item_key = utils::encode_key_and_value(&source_owner_key, &caller);
        utils::get_dictionary_value_from_key::<bool>(OPERATORS, &owner_operator_item_key)
            .unwrap_or_default()
    } else {
        false
    };

    // With operator package mode check if caller's package is operator to let contract execute
    // transfer
    let is_package_operator = if !is_owner && !is_approved && !is_operator {
        match (
            utils::get_stored_value_with_user_errors::<bool>(
                PACKAGE_OPERATOR_MODE,
                NFTCoreError::MissingPackageOperatorMode,
                NFTCoreError::InvalidPackageOperatorMode,
            ),
            contract_package,
        ) {
            (true, Some(contract_package)) => {
                let owner_operator_item_key =
                    utils::encode_key_and_value(&source_owner_key, &contract_package);
                utils::get_dictionary_value_from_key::<bool>(OPERATORS, &owner_operator_item_key)
                    .unwrap_or_default()
            }
            _ => false,
        }
    } else {
        false
    };

    if let Some(filter_contract) = utils::get_transfer_filter_contract() {
        let mut args = RuntimeArgs::new();
        args.insert(ARG_SOURCE_KEY, source_owner_key).unwrap();
        args.insert(ARG_TARGET_KEY, owner).unwrap();

        match &token_identifier {
            TokenIdentifier::Index(idx) => {
                args.insert(ARG_TOKEN_ID, *idx).unwrap();
            }
            TokenIdentifier::Hash(hash) => {
                args.insert(ARG_TOKEN_ID, hash.clone()).unwrap();
            }
        }

        let result: TransferFilterContractResult =
            call_contract::<u8>(filter_contract, TRANSFER_FILTER_CONTRACT_METHOD, args).into();
        if TransferFilterContractResult::DenyTransfer == result {
            revert(NFTCoreError::TransferFilterContractDenied);
        }
    }

    // Revert if caller is not owner nor approved nor an operator.
    if !is_owner && !is_approved && !is_operator && !is_package_operator {
        runtime::revert(NFTCoreError::InvalidTokenOwner);
    }

    let target_owner_key = utils::get_named_arg_with_user_errors::<Key>(
        ARG_TARGET_KEY,
        NFTCoreError::MissingAccountHash,
        NFTCoreError::InvalidAccountHash,
    )
    .unwrap_or_revert();

    if NFTIdentifierMode::Hash == identifier_mode && runtime::get_key(OWNED_TOKENS).is_some() {
        if utils::should_migrate_token_hashes(source_owner_key) {
            utils::migrate_token_hashes(source_owner_key)
        }

        if utils::should_migrate_token_hashes(target_owner_key) {
            utils::migrate_token_hashes(target_owner_key)
        }
    }

    let target_owner_item_key = utils::encode_dictionary_item_key(target_owner_key);

    // Updated token_owners dictionary. Revert if token_owner not found.
    match utils::get_dictionary_value_from_key::<Key>(
        TOKEN_OWNERS,
        &token_identifier.get_dictionary_item_key(),
    ) {
        Some(token_actual_owner) => {
            if token_actual_owner != source_owner_key {
                runtime::revert(NFTCoreError::InvalidTokenOwner)
            }
            utils::upsert_dictionary_value_from_key(
                TOKEN_OWNERS,
                &token_identifier.get_dictionary_item_key(),
                target_owner_key,
            );
        }
        None => runtime::revert(NFTCoreError::MissingOwnerTokenIdentifierKey),
    }

    let source_owner_item_key = utils::encode_dictionary_item_key(source_owner_key);

    // Update the from_account balance
    let updated_from_account_balance =
        match utils::get_dictionary_value_from_key::<u64>(TOKEN_COUNT, &source_owner_item_key) {
            Some(balance) => {
                if balance > 0u64 {
                    balance - 1u64
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
    utils::upsert_dictionary_value_from_key(
        TOKEN_COUNT,
        &source_owner_item_key,
        updated_from_account_balance,
    );

    // Update the to_account balance
    let updated_to_account_balance =
        match utils::get_dictionary_value_from_key::<u64>(TOKEN_COUNT, &target_owner_item_key) {
            Some(balance) => balance + 1u64,
            None => 1u64,
        };

    utils::upsert_dictionary_value_from_key(
        TOKEN_COUNT,
        &target_owner_item_key,
        updated_to_account_balance,
    );

    utils::upsert_dictionary_value_from_key(
        APPROVED,
        &token_identifier.get_dictionary_item_key(),
        Option::<Key>::None,
    );

    let events_mode = EventsMode::try_from(utils::get_stored_value_with_user_errors::<u8>(
        EVENTS_MODE,
        NFTCoreError::MissingEventsMode,
        NFTCoreError::InvalidEventsMode,
    ))
    .unwrap_or_revert();

    match events_mode {
        EventsMode::NoEvents => {}
        EventsMode::CEP47 => record_cep47_event_dictionary(CEP47Event::Transfer {
            sender: caller,
            recipient: target_owner_key,
            token_id: token_identifier.clone(),
        }),
        EventsMode::CES => {
            // Emit Transfer event.
            let spender = if caller == owner { None } else { Some(caller) };
            casper_event_standard::emit(Transfer::new(
                owner,
                spender,
                target_owner_key,
                token_identifier.clone(),
            ));
        }
    }

    let reporting_mode = utils::get_reporting_mode();

    if let OwnerReverseLookupMode::Complete | OwnerReverseLookupMode::TransfersOnly = reporting_mode
    {
        // Update to_account owned_tokens. Revert if owned_tokens list is not found
        let tokens_count = utils::get_token_index(&token_identifier);
        if OwnerReverseLookupMode::TransfersOnly == reporting_mode {
            utils::add_page_entry_and_page_record(tokens_count, &source_owner_item_key, false);
        }

        let (page_table_entry, page_uref) = utils::update_page_entry_and_page_record(
            tokens_count,
            &source_owner_item_key,
            &target_owner_item_key,
        );

        let owned_tokens_actual_key = Key::dictionary(page_uref, source_owner_item_key.as_bytes());

        let receipt_string = utils::get_receipt_name(page_table_entry);

        let receipt = CLValue::from_t((receipt_string, owned_tokens_actual_key))
            .unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);
        runtime::ret(receipt)
    }
}

// Returns the length of the Vec<String> in OWNED_TOKENS dictionary. If key is not found
// it returns 0.
#[no_mangle]
pub extern "C" fn balance_of() {
    let owner_key = utils::get_named_arg_with_user_errors::<Key>(
        ARG_TOKEN_OWNER,
        NFTCoreError::MissingAccountHash,
        NFTCoreError::InvalidAccountHash,
    )
    .unwrap_or_revert();

    let owner_key_item_string = utils::encode_dictionary_item_key(owner_key);

    let balance = utils::get_dictionary_value_from_key::<u64>(TOKEN_COUNT, &owner_key_item_string)
        .unwrap_or(0u64);

    let balance_cl_value =
        CLValue::from_t(balance).unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);
    runtime::ret(balance_cl_value);
}

// Returns the owner for a specified token identifier, throws error if token id is not valid
#[no_mangle]
pub extern "C" fn owner_of() {
    let identifier_mode: NFTIdentifierMode = utils::get_stored_value_with_user_errors::<u8>(
        IDENTIFIER_MODE,
        NFTCoreError::MissingIdentifierMode,
        NFTCoreError::InvalidIdentifierMode,
    )
    .try_into()
    .unwrap_or_revert();

    let token_identifier = utils::get_token_identifier_from_runtime_args(&identifier_mode);

    let number_of_minted_tokens = utils::get_stored_value_with_user_errors::<u64>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    if let NFTIdentifierMode::Ordinal = identifier_mode {
        // Revert if token_id is out of bounds
        if token_identifier.get_index().unwrap_or_revert() >= number_of_minted_tokens {
            runtime::revert(NFTCoreError::InvalidTokenIdentifier);
        }
    }

    let token_owner = match utils::get_dictionary_value_from_key::<Key>(
        TOKEN_OWNERS,
        &token_identifier.get_dictionary_item_key(),
    ) {
        Some(token_owner) => token_owner,
        None => runtime::revert(NFTCoreError::MissingOwnerTokenIdentifierKey),
    };

    let token_owner_cl_value =
        CLValue::from_t(token_owner).unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);

    runtime::ret(token_owner_cl_value);
}

#[no_mangle]
pub extern "C" fn metadata() {
    let number_of_minted_tokens = utils::get_stored_value_with_user_errors::<u64>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    let identifier_mode: NFTIdentifierMode = utils::get_stored_value_with_user_errors::<u8>(
        IDENTIFIER_MODE,
        NFTCoreError::MissingIdentifierMode,
        NFTCoreError::InvalidIdentifierMode,
    )
    .try_into()
    .unwrap_or_revert();

    let token_identifier = utils::get_token_identifier_from_runtime_args(&identifier_mode);

    if let NFTIdentifierMode::Ordinal = identifier_mode {
        // Revert if token_id is out of bounds
        if token_identifier.get_index().unwrap_or_revert() >= number_of_minted_tokens {
            runtime::revert(NFTCoreError::InvalidTokenIdentifier);
        }
    }

    let metadata_kind_list: BTreeMap<NFTMetadataKind, Requirement> =
        utils::get_stored_value_with_user_errors(
            NFT_METADATA_KINDS,
            NFTCoreError::MissingNFTMetadataKind,
            NFTCoreError::InvalidNFTMetadataKind,
        );

    for (&metadata_kind, required) in metadata_kind_list.iter() {
        match required {
            &Requirement::Required => {
                let metadata = utils::get_dictionary_value_from_key::<String>(
                    &metadata::get_metadata_dictionary_name(&metadata_kind),
                    &token_identifier.get_dictionary_item_key(),
                )
                .unwrap_or_revert_with(NFTCoreError::InvalidTokenIdentifier);
                runtime::ret(
                    CLValue::from_t(metadata)
                        .unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue),
                );
            }
            _ => continue,
        }
    }
    runtime::revert(NFTCoreError::MissingTokenMetaData)
}

// Returns approved account hash for a specified token identifier, throws error if token id is not
// valid
#[no_mangle]
pub extern "C" fn get_approved() {
    let identifier_mode: NFTIdentifierMode = utils::get_stored_value_with_user_errors::<u8>(
        IDENTIFIER_MODE,
        NFTCoreError::MissingIdentifierMode,
        NFTCoreError::InvalidIdentifierMode,
    )
    .try_into()
    .unwrap_or_revert();

    let token_identifier = utils::get_token_identifier_from_runtime_args(&identifier_mode);

    // Revert if token_id is out of bounds.
    let number_of_minted_tokens = utils::get_stored_value_with_user_errors::<u64>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    if let NFTIdentifierMode::Ordinal = identifier_mode {
        // Revert if token_id is out of bounds
        if token_identifier.get_index().unwrap_or_revert() >= number_of_minted_tokens {
            runtime::revert(NFTCoreError::InvalidTokenIdentifier);
        }
    }

    // Revert if already burnt
    if utils::is_token_burned(&token_identifier) {
        runtime::revert(NFTCoreError::PreviouslyBurntToken)
    }

    let maybe_approved = match utils::get_dictionary_value_from_key::<Option<Key>>(
        APPROVED,
        &token_identifier.get_dictionary_item_key(),
    ) {
        Some(maybe_approved) => maybe_approved,
        None => None,
    };

    let approved_cl_value = CLValue::from_t(maybe_approved)
        .unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);

    runtime::ret(approved_cl_value);
}

#[no_mangle]
pub extern "C" fn set_token_metadata() {
    let metadata_mutability: MetadataMutability = utils::get_stored_value_with_user_errors::<u8>(
        METADATA_MUTABILITY,
        NFTCoreError::MissingMetadataMutability,
        NFTCoreError::InvalidMetadataMutability,
    )
    .try_into()
    .unwrap_or_revert();

    if let MetadataMutability::Immutable = metadata_mutability {
        runtime::revert(NFTCoreError::ForbiddenMetadataUpdate)
    }

    let identifier_mode: NFTIdentifierMode = utils::get_stored_value_with_user_errors::<u8>(
        IDENTIFIER_MODE,
        NFTCoreError::MissingIdentifierMode,
        NFTCoreError::InvalidIdentifierMode,
    )
    .try_into()
    .unwrap_or_revert();

    let token_identifier = utils::get_token_identifier_from_runtime_args(&identifier_mode);

    let token_owner = utils::get_dictionary_value_from_key::<Key>(
        TOKEN_OWNERS,
        &token_identifier.get_dictionary_item_key(),
    );

    if let Some(token_owner_key) = token_owner {
        let caller: Key = match utils::get_verified_caller().unwrap_or_revert() {
            Caller::Session(account_hash) => account_hash.into(),
            Caller::StoredCaller(contract_hash, _) => contract_hash.into(),
        };
        if caller != token_owner_key {
            runtime::revert(NFTCoreError::InvalidTokenOwner)
        }
    } else {
        runtime::revert(NFTCoreError::MissingOwnerTokenIdentifierKey)
    }

    let metadata_kinds: BTreeMap<NFTMetadataKind, Requirement> =
        utils::get_stored_value_with_user_errors(
            NFT_METADATA_KINDS,
            NFTCoreError::MissingNFTMetadataKind,
            NFTCoreError::InvalidNFTMetadataKind,
        );

    let updated_token_metadata: String = utils::get_named_arg_with_user_errors(
        ARG_TOKEN_META_DATA,
        NFTCoreError::MissingTokenMetaData,
        NFTCoreError::InvalidTokenMetaData,
    )
    .unwrap_or_revert();

    for (metadata_kind, required) in metadata_kinds {
        if required == Requirement::Unneeded {
            continue;
        }
        let token_metadata_validation =
            metadata::validate_metadata(&metadata_kind, updated_token_metadata.clone());
        match token_metadata_validation {
            Ok(validated_token_metadata) => {
                utils::upsert_dictionary_value_from_key(
                    &metadata::get_metadata_dictionary_name(&metadata_kind),
                    &token_identifier.get_dictionary_item_key(),
                    validated_token_metadata,
                );
            }
            Err(err) => {
                if required == Requirement::Required {
                    runtime::revert(err);
                }
            }
        }
    }

    let events_mode = EventsMode::try_from(utils::get_stored_value_with_user_errors::<u8>(
        EVENTS_MODE,
        NFTCoreError::MissingEventsMode,
        NFTCoreError::InvalidEventsMode,
    ))
    .unwrap_or_revert();

    // Emit MetadataUpdate event.
    match events_mode {
        EventsMode::NoEvents => {}
        EventsMode::CES => {
            casper_event_standard::emit(MetadataUpdated::new(
                token_identifier,
                updated_token_metadata,
            ));
        }
        EventsMode::CEP47 => record_cep47_event_dictionary(CEP47Event::MetadataUpdate {
            token_id: token_identifier,
        }),
    }
}

fn update_token_supply() -> (u64, u64) {
    let total_token_supply: u64 = match utils::get_optional_named_arg_with_user_errors(
        ARG_TOTAL_TOKEN_SUPPLY,
        NFTCoreError::InvalidTotalTokenSupply,
    ) {
        Some(total_token_supply_arg) => {
            if total_token_supply_arg
                > utils::get_stored_value_with_user_errors::<u64>(
                    TOTAL_TOKEN_SUPPLY,
                    NFTCoreError::MissingTotalTokenSupply,
                    NFTCoreError::InvalidTotalTokenSupply,
                )
            {
                runtime::revert(NFTCoreError::CannotUpgradeToMoreSupply)
            }

            let total_token_supply_uref = utils::get_uref(
                ARG_TOTAL_TOKEN_SUPPLY,
                NFTCoreError::MissingTotalTokenSupply,
                NFTCoreError::InvalidTotalTokenSupply,
            );
            storage::write(total_token_supply_uref, total_token_supply_arg);
            total_token_supply_arg
        }
        None => utils::get_stored_value_with_user_errors::<u64>(
            TOTAL_TOKEN_SUPPLY,
            NFTCoreError::MissingTotalTokenSupply,
            NFTCoreError::InvalidTotalTokenSupply,
        ),
    };

    if total_token_supply == 0 {
        runtime::revert(NFTCoreError::CannotUpgradeWithZeroSupply)
    }

    let current_number_of_minted_tokens = utils::get_stored_value_with_user_errors::<u64>(
        NUMBER_OF_MINTED_TOKENS,
        NFTCoreError::MissingNumberOfMintedTokens,
        NFTCoreError::InvalidNumberOfMintedTokens,
    );

    if total_token_supply < current_number_of_minted_tokens {
        runtime::revert(NFTCoreError::ExceededMaxTotalSupply)
    }
    (total_token_supply, current_number_of_minted_tokens)
}

fn do_migration() {
    let (total_token_supply, current_number_of_minted_tokens) = update_token_supply();
    storage::new_dictionary(PAGE_TABLE)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    let page_table_width = utils::max_number_of_pages(total_token_supply);
    runtime::put_key(PAGE_LIMIT, storage::new_uref(page_table_width).into());
    runtime::put_key(
        REPORTING_MODE,
        storage::new_uref(OwnerReverseLookupMode::Complete as u8).into(),
    );

    let collection_name = utils::get_stored_value_with_user_errors::<String>(
        COLLECTION_NAME,
        NFTCoreError::MissingCollectionName,
        NFTCoreError::InvalidCollectionName,
    );

    let new_contract_package_hash_representation =
        runtime::get_named_arg::<ContractPackageHash>(ARG_NFT_PACKAGE_KEY);

    let receipt_uref = utils::get_uref(
        RECEIPT_NAME,
        NFTCoreError::MissingReceiptName,
        NFTCoreError::InvalidReceiptName,
    );

    let new_receipt_string_representation = format!("{PREFIX_CEP78}_{collection_name}");
    runtime::put_key(
        &new_receipt_string_representation,
        storage::new_uref(new_contract_package_hash_representation.to_formatted_string()).into(),
    );
    storage::write(receipt_uref, new_receipt_string_representation);

    let identifier: NFTIdentifierMode = utils::get_stored_value_with_user_errors::<u8>(
        IDENTIFIER_MODE,
        NFTCoreError::MissingIdentifierMode,
        NFTCoreError::InvalidIdentifierMode,
    )
    .try_into()
    .unwrap_or_revert();

    match identifier {
        NFTIdentifierMode::Ordinal => utils::migrate_owned_tokens_in_ordinal_mode(),
        NFTIdentifierMode::Hash => {
            storage::new_dictionary(HASH_BY_INDEX)
                .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
            storage::new_dictionary(INDEX_BY_HASH)
                .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
            runtime::put_key(
                UNMATCHED_HASH_COUNT,
                storage::new_uref(current_number_of_minted_tokens).into(),
            );
        }
    }
}

#[no_mangle]
pub extern "C" fn migrate() {
    let reporting_mode = if runtime::get_key(REPORTING_MODE).is_some() {
        Some(utils::get_reporting_mode())
    } else {
        None
    };

    if [None, Some(OwnerReverseLookupMode::NoLookUp)].contains(&reporting_mode) {
        if utils::requires_rlo_migration() && runtime::get_key(RLO_MFLAG).is_none() {
            do_migration();
        } else {
            update_token_supply();
        }
    }

    let metadata_kind: NFTMetadataKind = utils::get_stored_value_with_user_errors(
        NFT_METADATA_KIND,
        NFTCoreError::MissingNFTMetadataKind,
        NFTCoreError::InvalidNFTMetadataKind,
    );
    let mut nft_metadata_kind_list: BTreeMap<NFTMetadataKind, Requirement> = BTreeMap::new();
    nft_metadata_kind_list.insert(metadata_kind, Requirement::Required);
    runtime::put_key(
        NFT_METADATA_KINDS,
        storage::new_uref(nft_metadata_kind_list).into(),
    );

    runtime::put_key(RLO_MFLAG, storage::new_uref(false).into());

    let events_mode: EventsMode = utils::get_optional_named_arg_with_user_errors::<u8>(
        ARG_EVENTS_MODE,
        NFTCoreError::InvalidEventsMode,
    )
    .unwrap_or(EventsMode::NoEvents as u8)
    .try_into()
    .unwrap_or_revert();

    match events_mode {
        EventsMode::NoEvents => {}
        EventsMode::CES => {
            // Initialize events structures.
            utils::init_events();
            // Emit Migration event.
            casper_event_standard::emit(Migration::new());
        }
        EventsMode::CEP47 => record_cep47_event_dictionary(CEP47Event::Migrate),
    }

    runtime::put_key(EVENTS_MODE, storage::new_uref(events_mode as u8).into());

    let acl_package_mode: bool = utils::get_optional_named_arg_with_user_errors::<bool>(
        ARG_ACL_PACKAGE_MODE,
        NFTCoreError::InvalidACLPackageMode,
    )
    .unwrap_or_default();

    // Don't overwrite stored value on migration if acl package mode is missing param or false, use
    // set_variables to disable mode
    if acl_package_mode {
        runtime::put_key(ACL_PACKAGE_MODE, storage::new_uref(acl_package_mode).into());
    }

    let package_operator_mode: bool = utils::get_optional_named_arg_with_user_errors::<bool>(
        ARG_PACKAGE_OPERATOR_MODE,
        NFTCoreError::InvalidPackageOperatorMode,
    )
    .unwrap_or_default();

    // Don't overwrite stored value on migration if package operator mode is missing param or false,
    // use set_variables to disable mode
    if package_operator_mode {
        runtime::put_key(
            PACKAGE_OPERATOR_MODE,
            storage::new_uref(package_operator_mode).into(),
        );
    }

    let operator_burn_mode: bool = utils::get_optional_named_arg_with_user_errors::<bool>(
        ARG_OPERATOR_BURN_MODE,
        NFTCoreError::InvalidPackageOperatorMode,
    )
    .unwrap_or_default();

    // Don't overwrite stored value on migration if package operator mode is missing param or false,
    // use set_variables to disable mode
    if operator_burn_mode {
        runtime::put_key(
            OPERATOR_BURN_MODE,
            storage::new_uref(operator_burn_mode).into(),
        );
    }

    // Duplicate old dict OPERATOR named key to new dict APPROVED named key
    if runtime::get_key(APPROVED).is_none() && runtime::get_key(OPERATOR).is_some() {
        runtime::put_key(
            APPROVED,
            runtime::get_key(OPERATOR).unwrap_or_revert_with(NFTCoreError::MissingOperatorDict),
        );
    }
    // Reverts if APPROVED named key still missing
    if runtime::get_key(APPROVED).is_none() {
        runtime::revert(NFTCoreError::MissingApprovedDict)
    }
    // Add OPERATORS dict
    if runtime::get_key(OPERATORS).is_none() {
        storage::new_dictionary(OPERATORS)
            .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
    }

    utils::migrate_contract_whitelist_to_acl_whitelist();
}

#[no_mangle]
pub extern "C" fn updated_receipts() {
    if let OwnerReverseLookupMode::Complete = utils::get_reporting_mode() {
        let caller: Key = match utils::get_verified_caller().unwrap_or_revert() {
            Caller::Session(account_hash) => account_hash.into(),
            Caller::StoredCaller(contract_hash, _) => contract_hash.into(),
        };

        let identifier_mode: NFTIdentifierMode = utils::get_stored_value_with_user_errors::<u8>(
            IDENTIFIER_MODE,
            NFTCoreError::MissingIdentifierMode,
            NFTCoreError::InvalidIdentifierMode,
        )
        .try_into()
        .unwrap_or_revert();

        if identifier_mode == NFTIdentifierMode::Hash && utils::should_migrate_token_hashes(caller)
        {
            utils::migrate_token_hashes(caller);
        }

        let token_owner_item_key = utils::encode_dictionary_item_key(caller);

        let page_table =
            utils::get_dictionary_value_from_key::<Vec<bool>>(PAGE_TABLE, &token_owner_item_key)
                .unwrap_or_default();

        let mut updated_receipts: Vec<(String, Key)> = vec![];

        for (page_table_entry, allocated) in page_table.into_iter().enumerate() {
            if !allocated {
                continue;
            }
            let page_uref = utils::get_uref(
                &format!("{PREFIX_PAGE_DICTIONARY}_{page_table_entry}"),
                NFTCoreError::MissingPageUref,
                NFTCoreError::InvalidPageUref,
            );
            let page_dictionary_address =
                Key::dictionary(page_uref, token_owner_item_key.as_bytes());
            let receipt_name = utils::get_receipt_name(page_table_entry as u64);
            updated_receipts.push((receipt_name, page_dictionary_address))
        }

        runtime::ret(CLValue::from_t(updated_receipts).unwrap_or_revert())
    }
}

#[no_mangle]
pub extern "C" fn register_owner() {
    if vec![
        OwnerReverseLookupMode::Complete,
        OwnerReverseLookupMode::TransfersOnly,
    ]
    .contains(&utils::get_reporting_mode())
    {
        let owner_key = match utils::get_ownership_mode().unwrap_or_revert() {
            OwnershipMode::Minter => match utils::get_verified_caller().unwrap_or_revert() {
                Caller::Session(account_hash) => account_hash.into(),
                Caller::StoredCaller(contract_hash, _) => contract_hash.into(),
            },
            OwnershipMode::Assigned | OwnershipMode::Transferable => {
                utils::get_named_arg_with_user_errors::<Key>(
                    ARG_TOKEN_OWNER,
                    NFTCoreError::MissingTokenOwner,
                    NFTCoreError::InvalidTokenOwner,
                )
                .unwrap_or_revert()
            }
        };

        let page_table_uref = utils::get_uref(
            PAGE_TABLE,
            NFTCoreError::MissingPageTableURef,
            NFTCoreError::InvalidPageTableURef,
        );

        let owner_item_key = utils::encode_dictionary_item_key(owner_key);

        if storage::dictionary_get::<Vec<bool>>(page_table_uref, &owner_item_key)
            .unwrap_or_revert()
            .is_none()
        {
            let page_table_width = utils::get_stored_value_with_user_errors::<u64>(
                PAGE_LIMIT,
                NFTCoreError::MissingPageLimit,
                NFTCoreError::InvalidPageLimit,
            );
            storage::dictionary_put(
                page_table_uref,
                &owner_item_key,
                vec![false; page_table_width as usize],
            );
        }
        let collection_name = utils::get_stored_value_with_user_errors::<String>(
            COLLECTION_NAME,
            NFTCoreError::MissingCollectionName,
            NFTCoreError::InvalidCollectionName,
        );
        let package_uref = storage::new_uref(utils::get_stored_value_with_user_errors::<String>(
            &format!("{PREFIX_CEP78}_{collection_name}"),
            NFTCoreError::MissingCep78PackageHash,
            NFTCoreError::InvalidCep78InvalidHash,
        ));
        runtime::ret(CLValue::from_t((collection_name, package_uref)).unwrap_or_revert())
    }
}

fn generate_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();

    // This entrypoint initializes the contract and is required to be called during the session
    // where the contract is installed; immediately after the contract has been installed but
    // before exiting session. All parameters are required.
    // This entrypoint is intended to be called exactly once and will error if called more than
    // once.
    let init_contract = EntryPoint::new(
        ENTRY_POINT_INIT,
        vec![
            Parameter::new(ARG_COLLECTION_NAME, CLType::String),
            Parameter::new(ARG_COLLECTION_SYMBOL, CLType::String),
            Parameter::new(ARG_TOTAL_TOKEN_SUPPLY, CLType::U64),
            Parameter::new(ARG_ALLOW_MINTING, CLType::Bool),
            Parameter::new(ARG_MINTING_MODE, CLType::U8),
            Parameter::new(ARG_OWNERSHIP_MODE, CLType::U8),
            Parameter::new(ARG_NFT_KIND, CLType::U8),
            Parameter::new(ARG_HOLDER_MODE, CLType::U8),
            Parameter::new(ARG_WHITELIST_MODE, CLType::U8),
            Parameter::new(ARG_ACL_WHITELIST, CLType::List(Box::new(CLType::Key))),
            Parameter::new(ARG_ACL_PACKAGE_MODE, CLType::Bool),
            Parameter::new(ARG_PACKAGE_OPERATOR_MODE, CLType::Bool),
            Parameter::new(ARG_JSON_SCHEMA, CLType::String),
            Parameter::new(ARG_RECEIPT_NAME, CLType::String),
            Parameter::new(ARG_IDENTIFIER_MODE, CLType::U8),
            Parameter::new(ARG_BURN_MODE, CLType::U8),
            Parameter::new(ARG_OPERATOR_BURN_MODE, CLType::Bool),
            Parameter::new(ARG_NFT_METADATA_KIND, CLType::U8),
            Parameter::new(ARG_METADATA_MUTABILITY, CLType::U8),
            Parameter::new(ARG_OWNER_LOOKUP_MODE, CLType::U8),
            Parameter::new(ARG_EVENTS_MODE, CLType::U8),
            Parameter::new(
                ARG_TRANSFER_FILTER_CONTRACT,
                CLType::Option(Box::new(CLType::Key)),
            ),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    // This entrypoint exposes all variables that can be changed by managing account post
    // installation. Meant to be called by the managing account (INSTALLER) post
    // installation if a variable needs to be changed. Each parameter of the entrypoint
    // should only be passed if that variable is changed.
    // For instance if the allow_minting variable is being changed and nothing else
    // the managing account would send the new allow_minting value as the only argument.
    // If no arguments are provided it is essentially a no-operation, however there
    // is still a gas cost.
    // By switching allow_minting to false we pause minting.
    let set_variables = EntryPoint::new(
        ENTRY_POINT_SET_VARIABLES,
        vec![
            Parameter::new(ARG_ALLOW_MINTING, CLType::Bool),
            // Deprecated in 1.4 in favor of ACL_WHITELIST
            Parameter::new(
                ARG_CONTRACT_WHITELIST,
                CLType::List(Box::new(CLType::ByteArray(32u32))),
            ),
            Parameter::new(ARG_ACL_WHITELIST, CLType::List(Box::new(CLType::Key))),
            Parameter::new(ARG_ACL_PACKAGE_MODE, CLType::Bool),
            Parameter::new(ARG_PACKAGE_OPERATOR_MODE, CLType::Bool),
            Parameter::new(ARG_OPERATOR_BURN_MODE, CLType::Bool),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    // This entrypoint mints a new token with provided metadata.
    // Meant to be called post installation.
    // Reverts with MintingIsPaused error if allow_minting is false.
    // When a token is minted the calling account is listed as its owner and the token is
    // automatically assigned an U64 ID equal to the current number_of_minted_tokens.
    // Before minting the token the entrypoint checks if number_of_minted_tokens
    // exceed the total_token_supply. If so, it reverts the minting with an error
    // TokenSupplyDepleted. The mint entrypoint also checks whether the calling account
    // is the managing account (the installer) If not, and if public_minting is set to
    // false, it reverts with the error InvalidAccount. The newly minted token is
    // automatically assigned a U64 ID equal to the current number_of_minted_tokens. The
    // account is listed as the token owner, as well as added to the accounts list of owned
    // tokens. After minting is successful the number_of_minted_tokens is incremented by
    // one.
    let mint = EntryPoint::new(
        ENTRY_POINT_MINT,
        vec![
            Parameter::new(ARG_TOKEN_OWNER, CLType::Key),
            Parameter::new(ARG_TOKEN_META_DATA, CLType::String),
        ],
        CLType::Tuple3([
            Box::new(CLType::String),
            Box::new(CLType::Key),
            Box::new(CLType::String),
        ]),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    // This entrypoint burns the token with provided token_id argument, after which it is no
    // longer possible to transfer it.
    // Looks up the owner of the supplied token_id arg. If caller is not owner we revert with
    // error InvalidTokenOwner. If token id is invalid (e.g. out of bounds) it reverts
    // with error  InvalidTokenID. If token is listed as already burnt we revert with
    // error PreviouslyBurntTOken. If not the token is then registered as burnt.
    let burn = EntryPoint::new(
        ENTRY_POINT_BURN,
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    // This entrypoint transfers ownership of token from one account to another.
    // It looks up the owner of the supplied token_id arg. Revert if token is already burnt,
    // token_id is invalid, or if caller is not owner nor an approved account nor operator.
    // If token id is invalid it reverts with error InvalidTokenID.
    let transfer = EntryPoint::new(
        ENTRY_POINT_TRANSFER,
        vec![
            Parameter::new(ARG_SOURCE_KEY, CLType::Key),
            Parameter::new(ARG_TARGET_KEY, CLType::Key),
        ],
        CLType::Tuple2([Box::new(CLType::String), Box::new(CLType::Key)]),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    // This entrypoint approves another token holder (an approved account) to transfer tokens. It
    // reverts if token_id is invalid, if caller is not the owner nor operator, if token has already
    // been burnt, or if caller tries to approve themselves as an approved account.
    let approve = EntryPoint::new(
        ENTRY_POINT_APPROVE,
        vec![Parameter::new(ARG_SPENDER, CLType::Key)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    // This entrypoint revokes an approved account to transfer tokens. It reverts
    // if token_id is invalid, if caller is not the owner, if token has already
    // been burnt, if caller tries to approve itself.
    let revoke = EntryPoint::new(
        ENTRY_POINT_REVOKE,
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    // This entrypoint approves all tokens owned by the caller and future to another token holder
    // (an operator) to transfer tokens. It reverts if token_id is invalid, if caller is not the
    // owner, if caller tries to approve itself as an operator.
    let set_approval_for_all = EntryPoint::new(
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        vec![
            Parameter::new(ARG_APPROVE_ALL, CLType::Bool),
            Parameter::new(ARG_OPERATOR, CLType::Key),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    // This entrypoint returns if an account is operator for a token owner
    let is_approved_for_all = EntryPoint::new(
        ENTRY_POINT_IS_APPROVED_FOR_ALL,
        vec![
            Parameter::new(ARG_TOKEN_OWNER, CLType::Key),
            Parameter::new(ARG_OPERATOR, CLType::Key),
        ],
        CLType::Bool,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    // This entrypoint returns the token owner given a token_id. It reverts if token_id
    // is invalid. A burnt token still has an associated owner.
    let owner_of = EntryPoint::new(
        ENTRY_POINT_OWNER_OF,
        vec![], // <- either HASH or INDEX
        CLType::Key,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    // This entrypoint returns the approved account (if any) associated with the provided token_id
    // Reverts if token has been burnt.
    let get_approved = EntryPoint::new(
        ENTRY_POINT_GET_APPROVED,
        vec![], // <- either HASH or INDEX
        CLType::Option(Box::new(CLType::Key)),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    // This entrypoint returns number of owned tokens associated with the provided token holder
    let balance_of = EntryPoint::new(
        ENTRY_POINT_BALANCE_OF,
        vec![Parameter::new(ARG_TOKEN_OWNER, CLType::Key)],
        CLType::U64,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    // This entrypoint returns the metadata associated with the provided token_id
    let metadata = EntryPoint::new(
        ENTRY_POINT_METADATA,
        vec![], // <- either HASH or INDEX
        CLType::String,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    // This entrypoint updates the metadata if valid.
    let set_token_metadata = EntryPoint::new(
        ENTRY_POINT_SET_TOKEN_METADATA,
        vec![Parameter::new(ARG_TOKEN_META_DATA, CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    // This entrypoint will upgrade the contract from the 1_0 version to the
    // 1_1 version. The contract will insert any addition dictionaries and
    // sentinel values that were absent in the previous version of the contract.
    // It will also perform the necessary data transformations of historical
    // data if needed
    let migrate = EntryPoint::new(
        ENTRY_POINT_MIGRATE,
        vec![Parameter::new(ARG_NFT_PACKAGE_KEY, CLType::Any)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    // This entrypoint will allow NFT owners to update their receipts from
    // the previous owned_tokens list model to the current pagination model
    // scheme. Calling the entrypoint will return a list of receipt names
    // alongside the dictionary addressed to the relevant pages.
    let updated_receipts = EntryPoint::new(
        ENTRY_POINT_UPDATED_RECEIPTS,
        vec![],
        CLType::List(Box::new(CLType::Tuple2([
            Box::new(CLType::String),
            Box::new(CLType::Key),
        ]))),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    // This entrypoint allows users to register with a give CEP-78 instance,
    // allocating the necessary page table to enable the reverse lookup
    // functionality and allowing users to pay the upfront cost of allocation
    // resulting in more stable gas costs when minting and transferring
    // Note: This entrypoint MUST be invoked if the reverse lookup is enabled
    // in order to own NFTs.
    let register_owner = EntryPoint::new(
        ENTRY_POINT_REGISTER_OWNER,
        vec![],
        CLType::Tuple2([Box::new(CLType::String), Box::new(CLType::URef)]),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    entry_points.add_entry_point(init_contract);
    entry_points.add_entry_point(set_variables);
    entry_points.add_entry_point(mint);
    entry_points.add_entry_point(burn);
    entry_points.add_entry_point(transfer);
    entry_points.add_entry_point(approve);
    entry_points.add_entry_point(revoke);
    entry_points.add_entry_point(owner_of);
    entry_points.add_entry_point(balance_of);
    entry_points.add_entry_point(get_approved);
    entry_points.add_entry_point(metadata);
    entry_points.add_entry_point(set_approval_for_all);
    entry_points.add_entry_point(is_approved_for_all);
    entry_points.add_entry_point(set_token_metadata);
    entry_points.add_entry_point(migrate);
    entry_points.add_entry_point(updated_receipts);
    entry_points.add_entry_point(register_owner);
    entry_points
}

fn install_contract() {
    // Represents the name of the NFT collection
    // This value cannot be changed after installation.
    let collection_name: String = utils::get_named_arg_with_user_errors(
        ARG_COLLECTION_NAME,
        NFTCoreError::MissingCollectionName,
        NFTCoreError::InvalidCollectionName,
    )
    .unwrap_or_revert();

    // TODO: figure out examples of collection_symbol
    // The symbol for the NFT collection.
    // This value cannot be changed after installation.
    let collection_symbol: String = utils::get_named_arg_with_user_errors(
        ARG_COLLECTION_SYMBOL,
        NFTCoreError::MissingCollectionSymbol,
        NFTCoreError::InvalidCollectionSymbol,
    )
    .unwrap_or_revert();

    // This represents the total number of NFTs that will
    // be minted by a specific instance of a contract.
    // This value cannot be changed after installation.
    let total_token_supply: u64 = utils::get_named_arg_with_user_errors(
        ARG_TOTAL_TOKEN_SUPPLY,
        NFTCoreError::MissingTotalTokenSupply,
        NFTCoreError::InvalidTotalTokenSupply,
    )
    .unwrap_or_revert();

    if total_token_supply == 0 {
        runtime::revert(NFTCoreError::CannotInstallWithZeroSupply)
    }

    if total_token_supply > MAX_TOTAL_TOKEN_SUPPLY {
        runtime::revert(NFTCoreError::ExceededMaxTotalSupply)
    }

    let allow_minting: bool = utils::get_optional_named_arg_with_user_errors(
        ARG_ALLOW_MINTING,
        NFTCoreError::InvalidMintingStatus,
    )
    .unwrap_or(true);

    // Represents the modes in which NFTs can be minted, i.e whether a singular known
    // entity v. users interacting with the contract. Refer to the `MintingMode`
    // enum in the `src/modalities.rs` file for details.
    // This value cannot be changed after installation.
    let minting_mode: u8 = utils::get_optional_named_arg_with_user_errors(
        ARG_MINTING_MODE,
        NFTCoreError::InvalidMintingMode,
    )
    .unwrap_or(0);

    // Represents the ownership model of the NFTs that will be minted
    // over the lifetime of the contract. Refer to the enum `OwnershipMode`
    // in the `src/modalities.rs` file for details.
    // This value cannot be changed after installation.
    let ownership_mode: u8 = utils::get_named_arg_with_user_errors(
        ARG_OWNERSHIP_MODE,
        NFTCoreError::MissingOwnershipMode,
        NFTCoreError::InvalidOwnershipMode,
    )
    .unwrap_or_revert();

    // Represents the type of NFT (i.e something physical/digital)
    // which will be minted over the lifetime of the contract.
    // Refer to the enum `NFTKind`
    // in the `src/modalities.rs` file for details.
    // This value cannot be changed after installation.
    let nft_kind: u8 = utils::get_named_arg_with_user_errors(
        ARG_NFT_KIND,
        NFTCoreError::MissingNftKind,
        NFTCoreError::InvalidNftKind,
    )
    .unwrap_or_revert();

    // Represents whether Accounts or Contracts, or both can hold NFTs for
    // a given contract instance. Refer to the enum `NFTHolderMode`
    // in the `src/modalities.rs` file for details.
    // This value cannot be changed after installation
    let holder_mode: u8 = utils::get_optional_named_arg_with_user_errors(
        ARG_HOLDER_MODE,
        NFTCoreError::InvalidHolderMode,
    )
    .unwrap_or(2u8);

    // Represents whether a given contract whitelist can be modified
    // for a given NFT contract instance. If not provided as an argument
    // it will default to unlocked.
    // This value cannot be changed after installation
    let whitelist_lock: u8 = utils::get_optional_named_arg_with_user_errors(
        ARG_WHITELIST_MODE,
        NFTCoreError::InvalidWhitelistMode,
    )
    .unwrap_or(0u8);

    // Deprecated in 1.4 in favor of following acl whitelist
    // A whitelist of keys specifying which entity can mint
    // NFTs in the contract holder mode with restricted minting.
    // This value can only be modified if the whitelist lock is
    // set to be unlocked.
    let contract_white_list: Vec<ContractHash> = utils::get_optional_named_arg_with_user_errors(
        ARG_CONTRACT_WHITELIST,
        NFTCoreError::InvalidContractWhitelist,
    )
    .unwrap_or_default();

    let mut acl_white_list: Vec<Key> = utils::get_optional_named_arg_with_user_errors(
        ARG_ACL_WHITELIST,
        NFTCoreError::InvalidACLWhitelist,
    )
    .unwrap_or_default();

    for contract_hash in contract_white_list.iter() {
        acl_white_list.push(Key::from(*contract_hash));
    }

    let acl_package_mode: bool = utils::get_optional_named_arg_with_user_errors(
        ARG_ACL_PACKAGE_MODE,
        NFTCoreError::InvalidACLPackageMode,
    )
    .unwrap_or_default();

    let package_operator_mode: bool = utils::get_optional_named_arg_with_user_errors::<bool>(
        ARG_PACKAGE_OPERATOR_MODE,
        NFTCoreError::InvalidPackageOperatorMode,
    )
    .unwrap_or_default();

    // Represents the schema for the metadata for a given NFT contract instance.
    // Refer to the `NFTMetadataKind` enum in src/utils for details.
    // This value cannot be changed after installation.
    let base_metadata_kind: u8 = utils::get_named_arg_with_user_errors(
        ARG_NFT_METADATA_KIND,
        NFTCoreError::MissingNFTMetadataKind,
        NFTCoreError::InvalidNFTMetadataKind,
    )
    .unwrap_or_revert();

    let additional_required_metadata: Vec<u8> = utils::get_optional_named_arg_with_user_errors(
        ARG_ADDITIONAL_REQUIRED_METADATA,
        NFTCoreError::InvalidAdditionalRequiredMetadata,
    )
    .unwrap_or_default();

    let optional_metadata: Vec<u8> = utils::get_optional_named_arg_with_user_errors(
        ARG_OPTIONAL_METADATA,
        NFTCoreError::InvalidOptionalMetadata,
    )
    .unwrap_or_default();

    // The JSON schema representation of the NFT which will be minted.
    // This value cannot be changed after installation.
    let json_schema: String = utils::get_optional_named_arg_with_user_errors(
        ARG_JSON_SCHEMA,
        NFTCoreError::InvalidJsonSchema,
    )
    .unwrap_or_default();

    // Represents whether NFTs minted by a given contract will be identified
    // by an ordinal u64 index or a base16 encoded SHA256 hash of an NFTs metadata.
    // This value cannot be changed after installation. Refer to `NFTIdentifierMode` in
    // `src/modalities.rs` for further details.
    let identifier_mode: u8 = utils::get_named_arg_with_user_errors(
        ARG_IDENTIFIER_MODE,
        NFTCoreError::MissingIdentifierMode,
        NFTCoreError::InvalidIdentifierMode,
    )
    .unwrap_or_revert();

    // Represents whether the metadata related to NFTs can be updated.
    // This value cannot be changed after installation. Refer to `MetadataMutability` in
    // `src/modalities.rs` for further details.
    let metadata_mutability: u8 = utils::get_named_arg_with_user_errors(
        ARG_METADATA_MUTABILITY,
        NFTCoreError::MissingMetadataMutability,
        NFTCoreError::InvalidMetadataMutability,
    )
    .unwrap_or_revert();

    if identifier_mode == 1 && metadata_mutability == 1 {
        runtime::revert(NFTCoreError::InvalidMetadataMutability)
    }

    // Represents whether the minted tokens can be burnt.
    // This value cannot be changed post installation. Refer to `BurnMode` in
    // `src/modalities.rs` for further details.
    let burn_mode: u8 = utils::get_optional_named_arg_with_user_errors(
        ARG_BURN_MODE,
        NFTCoreError::InvalidBurnMode,
    )
    .unwrap_or(0u8);

    // Represents whether the minted tokens can be burnt by an operator
    let operator_burn_mode: bool = utils::get_optional_named_arg_with_user_errors::<bool>(
        ARG_OPERATOR_BURN_MODE,
        NFTCoreError::InvalidOperatorBurnMode,
    )
    .unwrap_or_default();

    // Represents whether the lookup of owner => identifiers (ordinal/hash)
    // is supported. Additionally, it also represents if receipts are returned after
    // invoking either the mint or transfer entrypoints.
    // This value cannot be changed post installation.
    // Refer to `src/modalities.rs` for further details.
    let reporting_mode: u8 = utils::get_optional_named_arg_with_user_errors(
        ARG_OWNER_LOOKUP_MODE,
        NFTCoreError::InvalidReportingMode,
    )
    .unwrap_or(0u8);

    let transfer_filter_contract_contract_key: Option<Key> =
        utils::get_optional_named_arg_with_user_errors(
            ARG_TRANSFER_FILTER_CONTRACT,
            NFTCoreError::InvalidTransferFilterContract,
        );

    if ownership_mode == 0 && minting_mode == 0 && reporting_mode == 1 {
        runtime::revert(NFTCoreError::InvalidReportingMode)
    }

    let entry_points = generate_entry_points();

    let named_keys = {
        let mut named_keys = NamedKeys::new();
        named_keys.insert(INSTALLER.to_string(), runtime::get_caller().into());

        named_keys
    };

    let hash_key_name = format!("{PREFIX_HASH_KEY_NAME}_{collection_name}");

    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(hash_key_name.clone()),
        Some(format!("{PREFIX_ACCESS_KEY_NAME}_{collection_name}")),
    );

    // Store contract_hash and contract_version under the keys CONTRACT_NAME and CONTRACT_VERSION
    runtime::put_key(
        &format!("{PREFIX_CONTRACT_NAME}_{collection_name}"),
        contract_hash.into(),
    );
    runtime::put_key(
        &format!("{PREFIX_CONTRACT_VERSION}_{collection_name}"),
        storage::new_uref(contract_version).into(),
    );

    let nft_contract_package_hash: ContractPackageHash = runtime::get_key(&hash_key_name)
        .unwrap_or_revert()
        .into_hash()
        .map(ContractPackageHash::new)
        .unwrap();

    let events_mode: u8 = utils::get_optional_named_arg_with_user_errors(
        ARG_EVENTS_MODE,
        NFTCoreError::InvalidEventsMode,
    )
    .unwrap_or(0u8);

    // A sentinel string value which represents the entry for the addition
    // of a read only reference to the NFTs owned by the calling `Account` or `Contract`
    // This allows for users to look up a set of named keys and correctly identify
    // the contract package from which the NFTs were obtained.
    let receipt_name = format!("{PREFIX_CEP78}_{collection_name}");

    let args = runtime_args! {
        ARG_COLLECTION_NAME => collection_name,
        ARG_COLLECTION_SYMBOL => collection_symbol,
        ARG_TOTAL_TOKEN_SUPPLY => total_token_supply,
        ARG_ALLOW_MINTING => allow_minting,
        ARG_OWNERSHIP_MODE => ownership_mode,
        ARG_NFT_KIND => nft_kind,
        ARG_MINTING_MODE => minting_mode,
        ARG_HOLDER_MODE => holder_mode,
        ARG_WHITELIST_MODE => whitelist_lock,
        ARG_ACL_WHITELIST => acl_white_list,
        ARG_JSON_SCHEMA => json_schema,
        ARG_RECEIPT_NAME => receipt_name,
        ARG_NFT_METADATA_KIND => base_metadata_kind,
        ARG_ADDITIONAL_REQUIRED_METADATA => additional_required_metadata,
        ARG_OPTIONAL_METADATA => optional_metadata,
        ARG_IDENTIFIER_MODE => identifier_mode,
        ARG_METADATA_MUTABILITY => metadata_mutability,
        ARG_BURN_MODE => burn_mode,
        ARG_OPERATOR_BURN_MODE => operator_burn_mode,
        ARG_OWNER_LOOKUP_MODE => reporting_mode,
        ARG_NFT_PACKAGE_KEY => nft_contract_package_hash.to_formatted_string(),
        ARG_EVENTS_MODE => events_mode,
        ARG_ACL_PACKAGE_MODE => acl_package_mode,
        ARG_PACKAGE_OPERATOR_MODE => package_operator_mode,
        ARG_TRANSFER_FILTER_CONTRACT =>
        transfer_filter_contract_contract_key,
    };

    // Call contract to initialize it
    runtime::call_contract::<()>(contract_hash, ENTRY_POINT_INIT, args);
}

fn migrate_contract(access_key_name: String, package_key_name: String) {
    let nft_contract_package_hash = runtime::get_key(&package_key_name)
        .unwrap_or_revert()
        .into_hash()
        .map(ContractPackageHash::new)
        .unwrap_or_revert_with(NFTCoreError::MissingPackageHashForUpgrade);

    let collection_name: String = utils::get_named_arg_with_user_errors(
        ARG_COLLECTION_NAME,
        NFTCoreError::MissingCollectionName,
        NFTCoreError::InvalidCollectionName,
    )
    .unwrap_or_revert();

    runtime::put_key(
        &format!("{PREFIX_HASH_KEY_NAME}_{collection_name}"),
        nft_contract_package_hash.into(),
    );

    if let Some(access_key) = runtime::get_key(&access_key_name) {
        runtime::put_key(
            &format!("{PREFIX_ACCESS_KEY_NAME}_{collection_name}"),
            access_key,
        )
    }

    let (contract_hash, contract_version) = storage::add_contract_version(
        nft_contract_package_hash,
        generate_entry_points(),
        NamedKeys::new(),
    );

    // Store contract_hash and contract_version under the keys CONTRACT_NAME and CONTRACT_VERSION
    runtime::put_key(
        &format!("{PREFIX_CONTRACT_NAME}_{collection_name}"),
        contract_hash.into(),
    );
    runtime::put_key(
        &format!("{PREFIX_CONTRACT_VERSION}_{collection_name}"),
        storage::new_uref(contract_version).into(),
    );

    let events_mode = utils::get_optional_named_arg_with_user_errors::<u8>(
        ARG_EVENTS_MODE,
        NFTCoreError::InvalidEventsMode,
    )
    .unwrap_or(0u8);

    let acl_package_mode: bool = utils::get_optional_named_arg_with_user_errors(
        ARG_ACL_PACKAGE_MODE,
        NFTCoreError::InvalidACLPackageMode,
    )
    .unwrap_or_default();

    let package_operator_mode: bool = utils::get_optional_named_arg_with_user_errors::<bool>(
        ARG_PACKAGE_OPERATOR_MODE,
        NFTCoreError::InvalidPackageOperatorMode,
    )
    .unwrap_or_default();

    let operator_burn_mode: bool = utils::get_optional_named_arg_with_user_errors::<bool>(
        ARG_OPERATOR_BURN_MODE,
        NFTCoreError::InvalidOperatorBurnMode,
    )
    .unwrap_or_default();

    let mut runtime_args = runtime_args! {
        ARG_NFT_PACKAGE_KEY => nft_contract_package_hash,
        ARG_EVENTS_MODE => events_mode,
        ARG_ACL_PACKAGE_MODE => acl_package_mode,
        ARG_PACKAGE_OPERATOR_MODE => package_operator_mode,
        ARG_OPERATOR_BURN_MODE => operator_burn_mode,
    };

    if let Some(new_token_supply) = utils::get_optional_named_arg_with_user_errors::<u64>(
        ARG_TOTAL_TOKEN_SUPPLY,
        NFTCoreError::InvalidTotalTokenSupply,
    ) {
        runtime_args
            .insert(ARG_TOTAL_TOKEN_SUPPLY, new_token_supply)
            .unwrap_or_revert();
    }

    runtime::call_contract::<()>(contract_hash, ENTRY_POINT_MIGRATE, runtime_args);
}

#[no_mangle]
pub extern "C" fn call() {
    let convention_mode: NamedKeyConventionMode =
        utils::get_optional_named_arg_with_user_errors::<u8>(
            ARG_NAMED_KEY_CONVENTION,
            NFTCoreError::InvalidNamedKeyConvention,
        )
        .unwrap_or(NamedKeyConventionMode::DerivedFromCollectionName as u8)
        .try_into()
        .unwrap_or_revert();

    match convention_mode {
        NamedKeyConventionMode::DerivedFromCollectionName => install_contract(),
        NamedKeyConventionMode::V1_0Standard => migrate_contract(
            ACCESS_KEY_NAME_1_0_0.to_string(),
            HASH_KEY_NAME_1_0_0.to_string(),
        ),
        NamedKeyConventionMode::V1_0Custom => migrate_contract(
            runtime::get_named_arg(ARG_ACCESS_KEY_NAME_1_0_0),
            runtime::get_named_arg(ARG_HASH_KEY_NAME_1_0_0),
        ),
    }
}
