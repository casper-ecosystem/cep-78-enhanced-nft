use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

use serde::{Deserialize, Serialize};

use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{
    bytesrepr,
    bytesrepr::{FromBytes, ToBytes},
    CLType, CLTyped,
};

use crate::{
    modalities::NFTMetadataKind, utils, NFTCoreError, ARG_JSON_SCHEMA, METADATA_CEP78,
    METADATA_CUSTOM_VALIDATED, METADATA_NFT721, METADATA_RAW,
};

// Metadata mutability is different from schema mutability.
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct MetadataSchemaProperty {
    name: String,
    description: String,
    required: bool,
}

impl ToBytes for MetadataSchemaProperty {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
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
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
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
            let custom_schema_json = utils::get_stored_value_with_user_errors::<String>(
                ARG_JSON_SCHEMA,
                NFTCoreError::MissingJsonSchema,
                NFTCoreError::InvalidJsonSchema,
            );

            serde_json_wasm::from_str::<CustomMetadataSchema>(&custom_schema_json)
                .map_err(|_| NFTCoreError::InvalidJsonSchema)
                .unwrap_or_revert()
        }
    }
}

impl ToBytes for CustomMetadataSchema {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut result = bytesrepr::allocate_buffer(self)?;
        result.extend(self.properties.to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.properties.serialized_length()
    }
}

impl FromBytes for CustomMetadataSchema {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
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
            let metadata = serde_json_wasm::from_str::<MetadataCEP78>(&token_metadata)
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
            serde_json::to_string_pretty(&metadata)
                .map_err(|_| NFTCoreError::FailedToJsonifyCEP99Metadata)
        }
        NFTMetadataKind::NFT721 => {
            let metadata = serde_json_wasm::from_str::<MetadataNFT721>(&token_metadata)
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
            serde_json::to_string_pretty(&metadata)
                .map_err(|_| NFTCoreError::FailedToJsonifyNFT721Metadata)
        }
        NFTMetadataKind::Raw => Ok(token_metadata),
        NFTMetadataKind::CustomValidated => {
            let custom_metadata =
                serde_json_wasm::from_str::<BTreeMap<String, String>>(&token_metadata)
                    .map(|attributes| CustomMetadata { attributes })
                    .map_err(|_| NFTCoreError::FailedToParseCustomMetadata)?;

            for (property_name, property_type) in token_schema.properties.iter() {
                if property_type.required && custom_metadata.attributes.get(property_name).is_none()
                {
                    runtime::revert(NFTCoreError::InvalidCustomMetadata)
                }
            }
            serde_json::to_string_pretty(&custom_metadata.attributes)
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
