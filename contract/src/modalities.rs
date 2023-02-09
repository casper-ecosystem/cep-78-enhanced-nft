use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec,
};
use casper_types::{
    bytesrepr::{FromBytes, ToBytes, U8_SERIALIZED_LENGTH},
    CLType, CLTyped,
};

use core::convert::TryFrom;

use crate::NFTCoreError;

#[repr(u8)]
#[derive(PartialEq, Eq)]
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
#[derive(PartialEq, Eq, Clone, Copy)]
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
            _ => Err(NFTCoreError::InvalidNftKind),
        }
    }
}

pub type MetadataRequirement = BTreeMap<NFTMetadataKind, Requirement>;

#[repr(u8)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Requirement {
    Required = 0,
    Optional = 1,
    Unneeded = 2,
}

impl TryFrom<u8> for Requirement {
    type Error = NFTCoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Requirement::Required),
            1 => Ok(Requirement::Optional),
            2 => Ok(Requirement::Unneeded),
            _ => Err(NFTCoreError::InvalidRequirement),
        }
    }
}

impl CLTyped for Requirement {
    fn cl_type() -> casper_types::CLType {
        CLType::U8
    }
}

impl FromBytes for Requirement {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        match bytes.split_first() {
            None => Err(casper_types::bytesrepr::Error::EarlyEndOfStream),
            Some((byte, rem)) => match Requirement::try_from(*byte) {
                Ok(kind) => Ok((kind, rem)),
                Err(_) => Err(casper_types::bytesrepr::Error::EarlyEndOfStream),
            },
        }
    }
}

impl ToBytes for Requirement {
    fn to_bytes(&self) -> Result<alloc::vec::Vec<u8>, casper_types::bytesrepr::Error> {
        Ok(vec![*self as u8])
    }

    fn serialized_length(&self) -> usize {
        U8_SERIALIZED_LENGTH
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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

impl CLTyped for NFTMetadataKind {
    fn cl_type() -> casper_types::CLType {
        CLType::U8
    }
}

impl FromBytes for NFTMetadataKind {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        match bytes.split_first() {
            None => Err(casper_types::bytesrepr::Error::EarlyEndOfStream),
            Some((byte, rem)) => match NFTMetadataKind::try_from(*byte) {
                Ok(kind) => Ok((kind, rem)),
                Err(_) => Err(casper_types::bytesrepr::Error::EarlyEndOfStream),
            },
        }
    }
}

impl ToBytes for NFTMetadataKind {
    fn to_bytes(&self) -> Result<alloc::vec::Vec<u8>, casper_types::bytesrepr::Error> {
        Ok(vec![*self as u8])
    }

    fn serialized_length(&self) -> usize {
        U8_SERIALIZED_LENGTH
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq)]
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
#[derive(PartialEq, Eq)]
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

#[derive(PartialEq, Eq, Clone)]
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

#[repr(u8)]
pub enum BurnMode {
    Burnable = 0,
    NonBurnable = 1,
}

impl TryFrom<u8> for BurnMode {
    type Error = NFTCoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(BurnMode::Burnable),
            1 => Ok(BurnMode::NonBurnable),
            _ => Err(NFTCoreError::InvalidBurnMode),
        }
    }
}

#[repr(u8)]
#[derive(Clone, PartialEq, Eq)]
pub enum OwnerReverseLookupMode {
    NoLookUp = 0,
    Complete = 1,
    TransfersOnly = 2,
}

impl TryFrom<u8> for OwnerReverseLookupMode {
    type Error = NFTCoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OwnerReverseLookupMode::NoLookUp),
            1 => Ok(OwnerReverseLookupMode::Complete),
            2 => Ok(OwnerReverseLookupMode::TransfersOnly),
            _ => Err(NFTCoreError::InvalidReportingMode),
        }
    }
}

#[repr(u8)]
pub enum NamedKeyConventionMode {
    DerivedFromCollectionName = 0,
    V1_0Standard = 1,
    V1_0Custom = 2,
}

impl TryFrom<u8> for NamedKeyConventionMode {
    type Error = NFTCoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NamedKeyConventionMode::DerivedFromCollectionName),
            1 => Ok(NamedKeyConventionMode::V1_0Standard),
            2 => Ok(NamedKeyConventionMode::V1_0Custom),
            _ => Err(NFTCoreError::InvalidNamedKeyConvention),
        }
    }
}
