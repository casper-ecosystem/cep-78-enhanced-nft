use alloc::{
    borrow::ToOwned,
    collections::BTreeMap,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::{
    convert::{TryFrom, TryInto},
    mem::MaybeUninit,
};

use serde::{Deserialize, Serialize};

use casper_contract::{
    contract_api::{self, runtime, storage},
    ext_ffi,
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    account::AccountHash,
    api_error,
    bytesrepr::{self, Error, FromBytes, ToBytes},
    system::CallStackElement,
    ApiError, CLType, CLTyped, ContractHash, Key, URef,
};

use crate::{
    constants::OWNERSHIP_MODE, error::NFTCoreError, ARG_JSON_SCHEMA, ARG_TOKEN_HASH, ARG_TOKEN_ID,
    HOLDER_MODE, METADATA_CEP78, METADATA_CUSTOM_VALIDATED, METADATA_NFT721, METADATA_RAW,
    OWNED_TOKENS,
};

pub(crate) fn upsert_dictionary_value_from_key<T: CLTyped + FromBytes + ToBytes>(
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

#[repr(u8)]
#[derive(PartialEq)]
pub enum WhitelistMode {
    Unlocked = 0,
    Locked = 1,
}

impl TryFrom<u8> for WhitelistMode {
    type Error = NFTCoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(WhitelistMode::Unlocked),
            1 => Ok(WhitelistMode::Locked),
            _ => Err(NFTCoreError::InvalidWhitelistMode),
        }
    }
}

#[repr(u8)]
#[derive(PartialEq, Clone, Copy)]
pub enum NFTHolderMode {
    Accounts = 0,
    Contracts = 1,
    Mixed = 2,
}

impl TryFrom<u8> for NFTHolderMode {
    type Error = NFTCoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NFTHolderMode::Accounts),
            1 => Ok(NFTHolderMode::Contracts),
            2 => Ok(NFTHolderMode::Mixed),
            _ => Err(NFTCoreError::InvalidHolderMode),
        }
    }
}

#[repr(u8)]
pub enum MintingMode {
    /// The ability to mint NFTs is restricted to the installing account only.
    Installer = 0,
    /// The ability to mint NFTs is not restricted.
    Public = 1,
}

impl TryFrom<u8> for MintingMode {
    type Error = NFTCoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MintingMode::Installer),
            1 => Ok(MintingMode::Public),
            _ => Err(NFTCoreError::InvalidMintingMode),
        }
    }
}

#[repr(u8)]
pub enum NFTKind {
    /// The NFT represents a real-world physical
    /// like a house.
    Physical = 0,
    /// The NFT represents a digital asset like a unique
    /// JPEG or digital art.
    Digital = 1,
    /// The NFT is the virtual representation
    /// of a physical notion, e.g a patent
    /// or copyright.
    Virtual = 2,
}

impl TryFrom<u8> for NFTKind {
    type Error = NFTCoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NFTKind::Physical),
            1 => Ok(NFTKind::Digital),
            2 => Ok(NFTKind::Virtual),
            _ => Err(NFTCoreError::InvalidOwnershipMode),
        }
    }
}

#[repr(u8)]
pub enum NFTMetadataKind {
    CEP78 = 0,
    NFT721 = 1,
    Raw = 2,
    CustomValidated = 3,
}

impl TryFrom<u8> for NFTMetadataKind {
    type Error = NFTCoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NFTMetadataKind::CEP78),
            1 => Ok(NFTMetadataKind::NFT721),
            2 => Ok(NFTMetadataKind::Raw),
            3 => Ok(NFTMetadataKind::CustomValidated),
            _ => Err(NFTCoreError::InvalidNFTMetadataKind),
        }
    }
}

#[repr(u8)]
pub enum OwnershipMode {
    /// The minter owns it and can never transfer it.
    Minter = 0,
    /// The minter assigns it to an address and can never be transferred.
    Assigned = 1,
    /// The NFT can be transferred even to an recipient that does not exist.
    Transferable = 2,
}

impl TryFrom<u8> for OwnershipMode {
    type Error = NFTCoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OwnershipMode::Minter),
            1 => Ok(OwnershipMode::Assigned),
            2 => Ok(OwnershipMode::Transferable),
            _ => Err(NFTCoreError::InvalidOwnershipMode),
        }
    }
}

#[repr(u8)]
pub enum NFTIdentifierMode {
    Ordinal = 0,
    Hash = 1,
}

impl TryFrom<u8> for NFTIdentifierMode {
    type Error = NFTCoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NFTIdentifierMode::Ordinal),
            1 => Ok(NFTIdentifierMode::Hash),
            _ => Err(NFTCoreError::InvalidIdentifierMode),
        }
    }
}

#[repr(u8)]
pub enum MetadataMutability {
    Immutable = 0,
    Mutable = 1,
}

impl TryFrom<u8> for MetadataMutability {
    type Error = NFTCoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MetadataMutability::Immutable),
            1 => Ok(MetadataMutability::Mutable),
            _ => Err(NFTCoreError::InvalidMetadataMutability),
        }
    }
}

pub(crate) fn get_ownership_mode() -> Result<OwnershipMode, NFTCoreError> {
    get_stored_value_with_user_errors::<u8>(
        OWNERSHIP_MODE,
        NFTCoreError::MissingOwnershipMode,
        NFTCoreError::InvalidOwnershipMode,
    )
    .try_into()
}

pub(crate) fn get_holder_mode() -> Result<NFTHolderMode, NFTCoreError> {
    get_stored_value_with_user_errors::<u8>(
        HOLDER_MODE,
        NFTCoreError::MissingHolderMode,
        NFTCoreError::InvalidHolderMode,
    )
    .try_into()
}

pub(crate) fn get_owned_tokens_dictionary_item_key(token_owner_key: Key) -> String {
    match token_owner_key {
        Key::Account(token_owner_account_hash) => token_owner_account_hash.to_string(),
        Key::Hash(token_owner_hash_addr) => ContractHash::new(token_owner_hash_addr).to_string(),
        _ => runtime::revert(NFTCoreError::InvalidKey),
    }
}

pub(crate) fn get_dictionary_value_from_key<T: CLTyped + FromBytes>(
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

pub(crate) fn get_stored_value_with_user_errors<T: CLTyped + FromBytes>(
    name: &str,
    missing: NFTCoreError,
    invalid: NFTCoreError,
) -> T {
    let uref = get_uref(name, missing, invalid);
    read_with_user_errors(uref, missing, invalid)
}

pub(crate) fn get_named_arg_size(name: &str) -> Option<usize> {
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

pub(crate) fn get_optional_named_arg_with_user_errors<T: FromBytes>(
    name: &str,
    invalid: NFTCoreError,
) -> Option<T> {
    match get_named_arg_with_user_errors(name, NFTCoreError::Phantom, invalid) {
        Ok(val) => val,
        Err(_) => None,
    }
}

pub(crate) fn get_named_arg_with_user_errors<T: FromBytes>(
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

pub(crate) fn get_account_hash(
    name: &str,
    missing: NFTCoreError,
    invalid: NFTCoreError,
) -> AccountHash {
    let key = get_key_with_user_errors(name, missing, invalid);
    key.into_account()
        .unwrap_or_revert_with(NFTCoreError::UnexpectedKeyVariant)
}

pub(crate) fn get_uref(name: &str, missing: NFTCoreError, invalid: NFTCoreError) -> URef {
    let key = get_key_with_user_errors(name, missing, invalid);
    key.into_uref()
        .unwrap_or_revert_with(NFTCoreError::UnexpectedKeyVariant)
}

pub(crate) fn named_uref_exists(name: &str) -> bool {
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

pub(crate) fn get_key_with_user_errors(
    name: &str,
    missing: NFTCoreError,
    invalid: NFTCoreError,
) -> Key {
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

pub(crate) fn read_with_user_errors<T: CLTyped + FromBytes>(
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

pub(crate) fn read_host_buffer_into(dest: &mut [u8]) -> Result<usize, ApiError> {
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

pub(crate) fn read_host_buffer(size: usize) -> Result<Vec<u8>, ApiError> {
    let mut dest: Vec<u8> = if size == 0 {
        Vec::new()
    } else {
        let bytes_non_null_ptr = contract_api::alloc_bytes(size);
        unsafe { Vec::from_raw_parts(bytes_non_null_ptr.as_ptr(), size, size) }
    };
    read_host_buffer_into(&mut dest)?;
    Ok(dest)
}

pub(crate) fn to_ptr<T: ToBytes>(t: T) -> (*const u8, usize, Vec<u8>) {
    let bytes = t.into_bytes().unwrap_or_revert();
    let ptr = bytes.as_ptr();
    let size = bytes.len();
    (ptr, size, bytes)
}

pub(crate) fn get_verified_caller() -> Result<Key, NFTCoreError> {
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
            Ok(Key::Account(calling_account_hash))
        }
        CallStackElement::StoredSession { contract_hash, .. }
        | CallStackElement::StoredContract { contract_hash, .. } => {
            if let NFTHolderMode::Accounts = holder_mode {
                return Err(NFTCoreError::InvalidHolderMode);
            }
            Ok(contract_hash.into())
        }
    }
}

#[derive(PartialEq, Clone)]
pub(crate) enum TokenIdentifier {
    Index(u64),
    Hash(String),
}

impl TokenIdentifier {
    pub(crate) fn new_index(index: u64) -> Self {
        TokenIdentifier::Index(index)
    }

    pub(crate) fn new_hash(hash: String) -> Self {
        TokenIdentifier::Hash(hash)
    }

    pub(crate) fn get_index(&self) -> Option<u64> {
        if let Self::Index(index) = self {
            return Some(*index);
        }
        None
    }

    pub(crate) fn get_hash(self) -> Option<String> {
        if let Self::Hash(hash) = self {
            return Some(hash);
        }
        None
    }

    pub(crate) fn get_dictionary_item_key(&self) -> String {
        match self {
            TokenIdentifier::Index(token_index) => token_index.to_string(),
            TokenIdentifier::Hash(hash) => hash.clone(),
        }
    }
}

pub(crate) fn get_token_identifier_from_runtime_args(
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

pub(crate) fn get_token_identifiers_from_dictionary(
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

pub(crate) fn upsert_token_identifiers(
    identifier_mode: &NFTIdentifierMode,
    owners_item_key: &str,
    token_identifiers: Vec<TokenIdentifier>,
) -> Result<(), NFTCoreError> {
    match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            let token_indices: Vec<u64> = token_identifiers
                .into_iter()
                .map(|token_identifier| {
                    token_identifier
                        .get_index()
                        .unwrap_or_revert_with(NFTCoreError::InvalidIdentifierMode)
                })
                .collect();
            upsert_dictionary_value_from_key(OWNED_TOKENS, owners_item_key, token_indices);
            Ok(())
        }
        NFTIdentifierMode::Hash => {
            let token_hashes: Vec<String> = token_identifiers
                .into_iter()
                .map(|token_identifier| {
                    token_identifier
                        .get_hash()
                        .unwrap_or_revert_with(NFTCoreError::InvalidIdentifierMode)
                })
                .collect();
            upsert_dictionary_value_from_key(OWNED_TOKENS, owners_item_key, token_hashes);
            Ok(())
        }
    }
}

// Metadata mutability is different from schema mutability.
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct MetadataSchemaProperty {
    name: String,
    description: String,
    required: bool,
}

impl ToBytes for MetadataSchemaProperty {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut result = bytesrepr::allocate_buffer(self)?;
        result.extend(self.name.to_bytes()?);
        result.extend(self.description.to_bytes()?);
        result.extend(self.required.to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.name.serialized_length()
            + self.description.serialized_length()
            + self.required.serialized_length()
    }
}

impl FromBytes for MetadataSchemaProperty {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (name, remainder) = String::from_bytes(bytes)?;
        let (description, remainder) = String::from_bytes(remainder)?;
        let (required, remainder) = bool::from_bytes(remainder)?;
        let metadata_schema_property = MetadataSchemaProperty {
            name,
            description,
            required,
        };
        Ok((metadata_schema_property, remainder))
    }
}

impl CLTyped for MetadataSchemaProperty {
    fn cl_type() -> CLType {
        CLType::Any
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct CustomMetadataSchema {
    properties: BTreeMap<String, MetadataSchemaProperty>,
}

pub(crate) fn get_metadata_schema(kind: &NFTMetadataKind) -> CustomMetadataSchema {
    match kind {
        NFTMetadataKind::Raw => CustomMetadataSchema {
            properties: BTreeMap::new(),
        },
        NFTMetadataKind::NFT721 => {
            let mut properties = BTreeMap::new();
            properties.insert(
                "name".to_string(),
                MetadataSchemaProperty {
                    name: "name".to_string(),
                    description: "The name of the NFT".to_string(),
                    required: true,
                },
            );
            properties.insert(
                "symbol".to_string(),
                MetadataSchemaProperty {
                    name: "symbol".to_string(),
                    description: "The symbol of the NFT collection".to_string(),
                    required: true,
                },
            );
            properties.insert(
                "token_uri".to_string(),
                MetadataSchemaProperty {
                    name: "token_uri".to_string(),
                    description: "The URI pointing to an off chain resource".to_string(),
                    required: true,
                },
            );
            CustomMetadataSchema { properties }
        }
        NFTMetadataKind::CEP78 => {
            let mut properties = BTreeMap::new();
            properties.insert(
                "name".to_string(),
                MetadataSchemaProperty {
                    name: "name".to_string(),
                    description: "The name of the NFT".to_string(),
                    required: true,
                },
            );
            properties.insert(
                "token_uri".to_string(),
                MetadataSchemaProperty {
                    name: "token_uri".to_string(),
                    description: "The URI pointing to an off chain resource".to_string(),
                    required: true,
                },
            );
            properties.insert(
                "checksum".to_string(),
                MetadataSchemaProperty {
                    name: "checksum".to_string(),
                    description: "A SHA256 hash of the content at the token_uri".to_string(),
                    required: true,
                },
            );
            CustomMetadataSchema { properties }
        }
        NFTMetadataKind::CustomValidated => {
            let custom_schema_json = get_stored_value_with_user_errors::<String>(
                ARG_JSON_SCHEMA,
                NFTCoreError::MissingJsonSchema,
                NFTCoreError::InvalidJsonSchema,
            );

            casper_serde_json_wasm::from_str::<CustomMetadataSchema>(&custom_schema_json)
                .map_err(|_| NFTCoreError::InvalidJsonSchema)
                .unwrap_or_revert()
        }
    }
}

impl ToBytes for CustomMetadataSchema {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut result = bytesrepr::allocate_buffer(self)?;
        result.extend(self.properties.to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.properties.serialized_length()
    }
}

impl FromBytes for CustomMetadataSchema {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (properties, remainder) =
            BTreeMap::<String, MetadataSchemaProperty>::from_bytes(bytes)?;
        let metadata_schema = CustomMetadataSchema { properties };
        Ok((metadata_schema, remainder))
    }
}

impl CLTyped for CustomMetadataSchema {
    fn cl_type() -> CLType {
        CLType::Any
    }
}

// Using a structure for the purposes of serialization formatting.
#[derive(Serialize, Deserialize)]
pub(crate) struct MetadataNFT721 {
    name: String,
    symbol: String,
    token_uri: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct MetadataCEP78 {
    name: String,
    token_uri: String,
    checksum: String,
}

// Using a structure for the purposes of serialization formatting.
#[derive(Serialize, Deserialize)]
pub(crate) struct CustomMetadata {
    attributes: BTreeMap<String, String>,
}

pub(crate) fn validate_metadata(
    metadata_kind: &NFTMetadataKind,
    token_metadata: String,
) -> Result<String, NFTCoreError> {
    let token_schema = get_metadata_schema(metadata_kind);
    match metadata_kind {
        NFTMetadataKind::CEP78 => {
            let metadata = casper_serde_json_wasm::from_str::<MetadataCEP78>(&token_metadata)
                .map_err(|_| NFTCoreError::FailedToParseCep99Metadata)?;

            if let Some(name_property) = token_schema.properties.get("name") {
                if name_property.required && metadata.name.is_empty() {
                    runtime::revert(NFTCoreError::InvalidCEP99Metadata)
                }
            }
            if let Some(token_uri_property) = token_schema.properties.get("token_uri") {
                if token_uri_property.required && metadata.token_uri.is_empty() {
                    runtime::revert(NFTCoreError::InvalidCEP99Metadata)
                }
            }
            if let Some(checksum_property) = token_schema.properties.get("checksum") {
                if checksum_property.required && metadata.checksum.is_empty() {
                    runtime::revert(NFTCoreError::InvalidCEP99Metadata)
                }
            }
            casper_serde_json_wasm::to_string_pretty(&metadata)
                .map_err(|_| NFTCoreError::FailedToJsonifyCEP99Metadata)
        }
        NFTMetadataKind::NFT721 => {
            let metadata = casper_serde_json_wasm::from_str::<MetadataNFT721>(&token_metadata)
                .map_err(|_| NFTCoreError::FailedToParse721Metadata)?;

            if let Some(name_property) = token_schema.properties.get("name") {
                if name_property.required && metadata.name.is_empty() {
                    runtime::revert(NFTCoreError::InvalidNFT721Metadata)
                }
            }
            if let Some(token_uri_property) = token_schema.properties.get("token_uri") {
                if token_uri_property.required && metadata.token_uri.is_empty() {
                    runtime::revert(NFTCoreError::InvalidNFT721Metadata)
                }
            }
            if let Some(symbol_property) = token_schema.properties.get("symbol") {
                if symbol_property.required && metadata.symbol.is_empty() {
                    runtime::revert(NFTCoreError::InvalidNFT721Metadata)
                }
            }
            casper_serde_json_wasm::to_string_pretty(&metadata)
                .map_err(|_| NFTCoreError::FailedToJsonifyNFT721Metadata)
        }
        NFTMetadataKind::Raw => Ok(token_metadata),
        NFTMetadataKind::CustomValidated => {
            let custom_metadata =
                casper_serde_json_wasm::from_str::<BTreeMap<String, String>>(&token_metadata)
                    .map(|attributes| CustomMetadata { attributes })
                    .map_err(|_| NFTCoreError::FailedToParseCustomMetadata)?;

            for (property_name, property_type) in token_schema.properties.iter() {
                if property_type.required && custom_metadata.attributes.get(property_name).is_none()
                {
                    runtime::revert(NFTCoreError::InvalidCustomMetadata)
                }
            }
            casper_serde_json_wasm::to_string_pretty(&custom_metadata.attributes)
                .map_err(|_| NFTCoreError::FailedToJsonifyCustomMetadata)
        }
    }
}

pub(crate) fn get_metadata_dictionary_name(metadata_kind: &NFTMetadataKind) -> String {
    let name = match metadata_kind {
        NFTMetadataKind::CEP78 => METADATA_CEP78,
        NFTMetadataKind::NFT721 => METADATA_NFT721,
        NFTMetadataKind::Raw => METADATA_RAW,
        NFTMetadataKind::CustomValidated => METADATA_CUSTOM_VALIDATED,
    };
    name.to_string()
}
