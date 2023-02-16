# CEP-78: Enhanced NFT standard

## Design Goals
- DApp developers attempting to create an NFT contract should be able to install the contract as is configured for the specific built-in behavior they want their NFT contract instance to have. Must work out of the box.
- Reference implementation must be straightforward, clear, and obvious.
- Externally observable association between `Accounts` and/or `Contracts` and `NFT`s they "own".
- Should be well documented with exhaustive tests that prove all possible combinations of defined behavior work as intended.
- Must be entirely self-contained within a singular repo; this includes all code, all tests, all relevant CasperLabs provided SDKs and all relevant documentation. 
- Must support mainstream expectations about common NFT conventions.
- A given NFT contract instance must be able to choose when created if it is using a Metadata schema conformant with existing community standards or a specific custom schema that they provide.
- A NFT contract instance must validate provided metadata against the specified metadata schema for that contract.
- Standardized session code to interact with an NFT contract instance must be usable as is, so that a given DApp developer doesn't have to write any Wasm producing logic for normal usage of NFT contract instances produced by this contract.

## New in Version 1.1

The release of version 1.1 for the CEP-78 Enhanced NFT Standard includes the following:

* Ownership of NFTs issued by a given CEP-78 contract instance are tracked by token id; thus the current owner of any given token by id is available using the owned_by entrypoint. However, some use cases benefit from the ability to ask the reverse question: list all the NFTs from a given contract instance owned by a specific owner.

    * To be able to support this reverse lookup option requires a contract instance to keep track of additional data, which causes higher gas costs for all mints and transfers. The gas cost to ask for any given owner is also unpredictable; as the asker is charged for the appropriate gas cost to read and return a collection that may be empty or may be quite large.

    * The pros and cons of either approach should be considered when installing a new CEP-78 contract. By default, new CEP-78 contract instances will default to the `OwnerReverseLookupMode::NoLookup` option, which has the lowest operating costs and optimal scaling characteristics. However, the `OwnerReverseLookupMode::Complete` option can be chosen upon install, which will allow the contract to write the necessary additional data to allow a full lookup by owner.

    * To allow isolation of the additional costs, or tracking individual owners, the reverse lookup mode supports a register entrypoint which is used to register owners prior to minting or receiving a transferred token. In either `Assigned` or `Transferable` mode, this register entrypoint can be called by any party on behalf of another party.

* A single instance of CEP-78 is limited to 1,000,000 tokens maximum.

    * The naming convention for the default named key prefix of a given CEP-78 contract instance has been changed to `cep78_<collection_name>` with spaces and dashes within the collection name converted to underscores.

    * **Version 1.1.1** Added a new modality named `NamedKeyConventionMode` that dictates the upgrading and installation process, with further information [here](#namedkeyconventionmode)

    * **If an account attempts to install a second CEP-78 contract instance with the same collection name, it will overwrite the `NamedKey` entry under which the access URef is written. Losing the access URef will prevent the account from adding newer versions, i.e., upgrading that particular instance of CEP-78.**

## Table of Contents

1) [Modalities](#modalities)

2) [Usage](#usage)

3) [Installing and Interacting with the Contract using the Rust Casper Client](#installing-and-interacting-with-the-contract-using-the-rust-casper-client)

4) [Test Suite and Specification](#test-suite-and-specification)

5) [Data Storage and Gas Stabilization](#gas-stabilization)

6) [Error Codes](#error-codes)

## Modalities

The enhanced NFT implementation supports various 'modalities' that dictate the behavior of a specific contract instance. Modalities represent the common expectations around contract usage and behavior.
The following section discusses the currently implemented modalities and illustrates the significance of each.

#### Ownership

This modality specifies the behavior regarding ownership of NFTs and whether the owner of the NFT can change over the contract's lifetime. There are three modes:

1. `Minter`: `Minter` mode is where the ownership of the newly minted NFT is attributed to the minter of the NFT and cannot be specified by the minter. In the `Minter` mode the owner of the NFT will not change and thus cannot be transferred to another entity.
2. `Assigned`: `Assigned` mode is where the owner of the newly minted NFT must be specified by the minter of the NFT. In this mode, the assigned entity can be either minter themselves or a separate entity. However, similar to the `Minter` mode, the ownership in this mode cannot be changed, and NFTs minted in this mode cannot be transferred from one entity to another.
3. `Transferable`: In the `Transferable` mode the owner of the newly minted NFT must be specified by the minter. However, in the `Transferable` mode, NFTs can be transferred from the owner to another entity.

In all the three mentioned modes, the owner entity is currently restricted to `Accounts` on the Casper network. 

**Note**: In the `Transferable` mode, it is possible to transfer the NFT to an `Account` that does not exist.

This `Ownership` mode is a required installation parameter and cannot be changed once the contract has been installed.
The mode is passed in as `u8` value to the `"ownership_mode"` runtime argument. 

| Ownership    | u8  |
|--------------|-----|
| Minter       | 0   |
| Assigned     | 1   |
| Transferable | 2   |

The ownership mode of a contract can be determined by querying the `ownership_mode` entry within the contract's `NamedKeys`.

#### NFTKind 

The `NFTKind` modality specifies the commodity that NFTs minted by a particular contract will represent. Currently, the `NFTKind` modality does not alter or govern the behavior of the contract itself
and only exists to specify the correlation between on-chain data and off-chain items. There are three different variations of the `NFTKind` mode.

1. `Physical`: The NFT represents a real-world physical item e.g., a house.
2. `Digital`: The NFT represents a digital item, e.g., a unique JPEG or digital art.
3. `Virtual`: The NFT is the virtual representation of a physical notion, e.g., a patent or copyright.

The `NFTKind` mode is a required installation parameter and cannot be changed once the contract has been installed.
The mode is passed in as a `u8` value to `nft_kind` runtime argument.

| NFTKind  | u8  |
|----------|-----|
| Physical | 0   |
| Digital  | 1   |
| Virtual  | 2   |

#### NFTHolderMode

The `NFTHolderMode` dictates which entities on a Casper network can own and mint NFTs. There are three different options currently available:

1. `Accounts`: In this mode, only `Accounts` can own and mint NFTs.
2. `Contracts`: In this mode, only `Contracts` can own and mint NFTs.
3. `Mixed`: In this mode both `Accounts` and `Contracts` can own and mint NFTs.

If the `NFTHolderMode` is set to `Contracts` a `ContractHash` whitelist must be provided. This whitelist dictates which
`Contracts` are allowed to mint NFTs in the restricted `Installer` minting mode.

| NFTHolderMode | u8  |
|---------------|-----|
| Accounts      | 0   |
| Contracts     | 1   |
| Mixed         | 2   |

This modality is an optional installation parameter and will default to the `Mixed` mode if not provided. However, this
mode cannot be changed once the contract has been installed.
The mode is passed in as a `u8` value to `nft_holder_mode` runtime argument.

#### WhitelistMode

The `WhitelistMode` dictates if the contract whitelist restricting access to the mint entrypoint can be updated. There are currently 
two options:

1. `Unlocked`: The contract whitelist is unlocked and can be updated via the set variables endpoint.
2. `Locked`: The contract whitelist is locked and cannot be updated further.

This `WhitelistMode` is an optional installation parameter and will be set to unlocked if not passed. However, the whitelist mode itself
cannot be changed once the contract has been installed. The mode is passed in as a `u8` value to `whitelist_mode` runtime argument.

| WhitelistMode | u8  |
|---------------|-----|
| Unlocked      | 0   |
| Locked        | 1   |

#### Minting

The minting mode governs the behavior of contract when minting new tokens. The minting modality provides two options:

1. `Installer`: This mode restricts the ability to mint new NFT tokens only to the installing account of the NFT contract. 
2. `Public`: This mode allows any account to mint NFT tokens.

This modality is an optional installation parameter and will default to the `Installer` mode if not provided. However, this
mode cannot be changed once the contract has been installed. The mode is set by passing a `u8` value to the `minting_mode` runtime argument.

| MintingMode | u8  |
|-------------|-----|
| Installer   | 0   |
| Public      | 1   |

#### NFTMetadataKind

This modality dictates the schema for the metadata for NFTs minted by a given instance of an NFT contract. There are four supported modalities:

1. `CEP78`: This mode specifies that NFTs minted must have valid metadata conforming to the CEP-78 schema.
2. `NFT721`: This mode specifies that NFTs minted must have valid metadata conforming to the NFT-721 metadata schema.
3. `Raw`: This mode specifies that metadata validation will not occur and raw strings can be passed to `token_metadata` runtime argument as part of the call to `mint` entrypoint.
4. `CustomValidated`: This mode specifies that a custom schema provided at the time of install will be used when validating the metadata as part of the call to `mint` entrypoint.

##### CEP-78 metadata example
```json
{
  "name": "John Doe",
  "token_uri": "https://www.barfoo.com",
  "checksum": "940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb"
}
```

##### NFT-721 metadata example
```json
{
  "name": "John Doe",
  "symbol": "abc",
  "token_uri": "https://www.barfoo.com"
}
```

##### Custom Validated

The CEP-78 implementation allows installers of the contract to provide their custom schema at the time of installation. 
The schema is passed as a String value to `json_schema` runtime argument at the time of installation. Once provided, the schema
for a given instance of the contract cannot be changed.

The custom JSON schema must contain a top-level `properties` field. An example of a [`valid JSON schema`](#example-custom-validated-schema) is provided. In this example, each property has a name, the description of the property itself, and whether the property is required to be present in the metadata.
If the metadata kind is not set to custom validated, then the value passed to the `json_schema` runtime argument will be ignored.

###### Example Custom Validated schema
```json
{
   "properties":{
      "deity_name":{
         "name":"deity_name",
         "description":"The name of deity from a particular pantheon.",
         "required":true
      },
      "mythology":{
         "name":"mythology",
         "description":"The mythology the deity belongs to.",
         "required":true
      }
   }
}
```

###### Example Custom Metadata
```json
{
  "deity_name": "Baldur",
  "mythology": "Nordic"
}
```

| NFTMetadataKind | u8  |
|-----------------|-----|
| CEP78           | 0   |
| NFT721          | 1   |
| Raw             | 2   |
| CustomValidated | 3   |

#### NFTIdentifierMode

The identifier mode governs the primary identifier for NFTs minted for a given instance on an installed contract. This modality provides two options:

1. `Ordinal`: NFTs minted in this modality are identified by a `u64` value. This value is determined by the number of NFTs minted by the contract at the time the NFT is minted.
2. `Hash`: NFTs minted in this modality are identified by a base16 encoded representation of the blake2b hash of the metadata provided at the time of mint.

Since the primary identifier in the `Hash` mode is derived by hashing over the metadata, making it a content-addressed identifier, the metadata for the minted NFT cannot be updated after the mint.
Attempting to install the contract with the `MetadataMutability` modality set to `Mutable` in the `Hash` identifier mode will raise an error.
This modality is a required installation parameter and cannot be changed once the contract has been installed.
It is passed in as a `u8` value to the `identifier_mode` runtime argument.

| NFTIdentifierMode | u8  |
|-------------------|-----|
| Ordinal           | 0   |
| Hash              | 1   |

#### Metadata Mutability

The metadata mutability mode governs the behavior around updates to a given NFTs metadata. This modality provides two options:

1. `Immutable`: Metadata for NFTs minted in this mode cannot be updated once the NFT has been minted.
2. `Mutable`: Metadata for NFTs minted in this mode can update the metadata via the `set_token_metadata` entrypoint.

The `Mutable` option cannot be used in conjunction with the `Hash` modality for the NFT identifier; attempting to install the contract with this configuration raises `InvalidMetadataMutability` error. 
This modality is a required installation parameter and cannot be changed once the contract has been installed.
It is passed in as a `u8` value to the `metadata_mutability` runtime argument. 

| MetadataMutability | u8  |
|--------------------|-----|
| Immutable          | 0   |
| Mutable            | 1   |

#### BurnMode

The `BurnMode` modality dictates whether tokens minted by a given instance of an NFT contract can be burnt. This modality
provides two options:

1. `Burnable`: Minted tokens can be burnt.
2. `NonBurnable`: Minted tokens cannot be burnt.

| BurnMode    | u8  |
|-------------|-----|
| Burnable    | 0   |
| NonBurnable | 1   |

This modality is an optional installation parameter and will default to the `Burnable` mode if not provided. However, this
mode cannot be changed once the contract has been installed. The mode is set by passing a `u8` value to the `burn_mode` runtime argument.

#### OwnerReverseLookupMode

The `OwnerReverseLookupMode` modality is set at install and determines if a given contract instance writes necessary data to allow reverse lookup by owner in addition to by ID.

This modality provides the following options:

1. `NoLookup`: The reporting and receipt functionality is not supported. In this option, the contract instance does not maintain a reverse lookup database of ownership and therefore has more predictable gas costs and greater scaling.
2. `Complete`: The reporting and receipt functionality is supported. Token ownership will be tracked by the contract instance using the system described [here](#owner-reverse-lookup-functionality).

Additionally, when set to `Complete`, causes a receipt to be returned by the `mint` or `transfer` entrypoints, which the caller can store in their account or contract context for later reference.

Further, two special entrypoints are enabled in `Complete` mode. First, `register_owner` which when called will allocate the necessary tracking record for the imputed entity. This allows isolation of the one time gas cost to do this per owner, which is convenient for accounting purposes. Second, `updated_receipts`, which allows an owner of one or more NFTs held by the contract instance to attain up to date receipt information for the NFTs they currently own.

| OwnerReverseLookupMode | u8  |
|------------------------|-----|
| NoLookup               | 0   |
| Complete               | 1   |

This modality is an optional installation parameter and will default to the `NoLookup` mode if not provided. The mode is set by passing a `u8` value to the `owner_reverse_lookup_mode` runtime argument. This mode cannot be changed once the contract has been installed.

Note, if `ownership_mode` is set to `Minter` and the `minting_mode` is set to `Installer` only, `OwnerReverseLookupMode` will be set to `NoLookup`. This is because the minter, by definition, owns all of the tokens forever. Therefore, there is no reason to do a reverse lookup for that owner. This rule applies only to newly installed contract instances.

If you are upgrading a contract from CEP-78 version 1.0 to 1.1, `OwnerReverseLookupMode` will be set to `Complete`, as this was the standard behavior of CEP-78 1.0. In addition to being set to `Complete`, existing records will be migrated into the CEP-78 1.1 format, which will impose a one-time gas cost to cover the migration.

If you have an existing CEP-78 version 1.0 contract instance, and would prefer the newer functionality with no lookup, the only option is to install a separate, new contract instance and mint all of the NFTs anew in that instance and then burn the corresponding NFTs from the old instance. If you do not own all the NFTs held by the old contract instance, you do not have this option.

#### NamedKeyConventionMode

The `NamedKeyConvention` modality dictates whether the Wasm passed will attempt to install a version 1.1.1 instance of CEP-78 or attempt to migrate a version 1.0 CEP-78 instance to version 1.1.1.

This modality provides three options:

1. `DerivedFromCollectionName`: This modality will signal the contract to attempt to install a new version 1.1.1 instance of the CEP-78 contract. The contract package hash and the access URef will be saved in the installing account's `NamedKeys` as `cep78_contract_package_<collection_name>` and `cep78_contract_package_access_<collection_name>`.
2. `V_1_0_standard`: This modality will signal the contract to attempt to upgrade from version 1.0 to version 1.1.1. In this scenario, the contract will retrieve the package hash and the access URef from the `NamedKey` entries originally created during the 1.0 installation.
3. `V_1_0_custom`: This modality will signal the contract to attempt to upgrade from version 1.0 to version 1.1.1. In this scenario, the calling account must provide the `NamedKey` entries under which the package hash and the access URef are saved. Additionally, this requires the passing of the runtime arguments `access_key_name` and `hash_key_name` for the access URef and package hash, respectively. In this modality, these arguments are required and must be passed in.

| NamedKeyConvention               | u8  |
|----------------------------------|-----|
| DerivedFromCollectionName        | 0   |
| V_1_0_standard                   | 1   |
| V_1_0_custom                     | 2   |

#### Modality Conflicts

The `MetadataMutability` option of `Mutable` cannot be used in conjunction with `NFTIdentifierMode` modality of `Hash`.

#### EventsMode

The `EventsMode` modality allows the deployers of the contract to decide on schemas for recording events during the operation of the contract where changes to the tokens happen.

0. `NoEvents`: No events will be recorded during the operation, this is the default mode.
1. `CEP47`: The event schema from the CEP47 contract has been implemented as a possibility.
2. `CES` : Events will be recorded during the operation of the contract using an event schema in compliance with the Casper Event Schema. Refer to  section [Casper Event Standard](#casper-event-standard) for more information.

### Usage

#### Installing the contract.

The `main.rs` file within the contract provides the installer for the NFT contract. Users can compile the contract to Wasm using the `make build-contract` with the provided Makefile.

The pre-built Wasm for the contract and all other utility session code can be found as part of the most current release. Users wishing to build the Wasm themselves can pull the code and the `make build-contract` provided in the included Makefile. Please note, however, that as part of building the contract, you will need to install `wasm-strip`.

The `call` method will install the contract with the necessary entrypoints and call the `init()` entrypoint to allow the contract to self initialize and setup the necessary state to allow for operation,
The following are the required runtime arguments that must be passed to the installer session code to correctly install the NFT contract.

* `"collection_name":` The name of the NFT collection, passed in as a `String`. This parameter is required and cannot be changed post installation.
* `"collection_symbol"`: The symbol representing a given NFT collection, passed in as a `String`. This parameter is required and cannot be changed post installation.
* `"total_token_supply"`: The total number of NFTs that a specific instance of a contract will mint passed in as a `U64` value. This parameter is required and cannot be changed post installation. 
* `"ownership_mode"`: The [`OwnershipMode`](#ownership) modality that dictates the ownership behavior of the NFT contract. This argument is passed in as a `u8` value and is required at the time of installation.
* `"nft_kind"`: The [`NFTKind`](#nftkind) modality that specifies the off-chain items represented by the on-chain NFT data. This argument is passed in as a `u8` value and is required at the time of installation.
* `"json_schema"`: The JSON schema for the NFT tokens that will be minted by the NFT contract passed in as a `String`. This parameter is required if the metadata kind is set to `CustomValidated(4)` and cannot be changed post installation.
* `"nft_metadata_kind"`: The metadata schema for the NFTs to be minted by the NFT contract. This argument is passed in as a `u8` value and is required at the time of installation.
* `"identifier_mode"`: The [`NFTIdentifierMode`](#nftidentifiermode) modality dictates the primary identifier for NFTs minted by the contract. This argument is passed in as a `u8` value and is required at the time of installation.
* `"metadata_mutability"`: The [`MetadataMutability`](#metadata-mutability) modality dictates whether the metadata of minted NFTs can be updated. This argument is passed in as a `u8` value and is required at the time of installation.


The following are the optional parameters that can be passed in at the time of installation.

* `"minting_mode"`: The [`MintingMode`](#minting) modality that dictates the access to the `mint()` entry-point in the NFT contract. This is an optional parameter that will default to restricting access to the installer of the contract. This parameter cannot be changed once the contract has been installed.
* `"allow_minting"`: The `"allow_minting"` flag allows the installer of the contract to pause the minting of new NFTs. The `allow_minting` is a boolean toggle that allows minting when `true`. If not provided at install the toggle will default to `true`. This value can be changed by the installer by calling the `set_variables()` entrypoint.
* `"whitelist_mode"`: The [`WhitelistMode`](#whitelistmode) modality dictates whether the contract whitelist can be updated. This optional parameter will default to an unlocked whitelist that can be updated post installation. This parameter cannot be changed once the contract has been installed.
* `"holder_mode"`: The [`NFTHolderMode`](#nftholdermode) modality dictates which entities can hold NFTs. This is an optional parameter and will default to a mixed mode allowing either `Accounts` or `Contracts` to hold NFTs. This parameter cannot be changed once the contract has been installed.
* `"contract_whitelist"`: The contract whitelist is a list of contract hashes that specifies which contracts can call the `mint()` entrypoint to mint NFTs. This is an optional parameter which will default to an empty whitelist. This value can be changed via the `set_variables` post installation. If the whitelist mode is set to locked, a non-empty whitelist must be passed; else, installation of the contract will fail.
* `"burn_mode"`: The [`BurnMode`](#burnmode) modality dictates whether minted NFTs can be burnt. This is an optional parameter and will allow tokens to be burnt by default. This parameter cannot be changed once the contract has been installed.
* `"owner_reverse_lookup_mode"`: The [`OwnerReverseLookupMode`](#reportingmode) modality dictates whether the lookup for owners to token identifiers is available. This is an optional parameter and will not provide the lookup by default. This parameter cannot be changed once the contract has been installed.

##### Example deploy

The following is an example of deploying the installation of the NFT contract via the Rust Casper command client.

```bash
casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem --session-path ~/casper/enhanced-nft/contract/target/wasm32-unknown-unknown/release/contract.wasm \
--session-arg "collection_name:string='enhanced-nft-1'" \
--session-arg "collection_symbol:string='ENFT-1'" \
--session-arg "total_token_supply:u256='10'" \
--session-arg "ownership_mode:u8='0'" \
--session-arg "nft_kind:u8='1'" \
--session-arg "json_schema:string='nft-schema'" \
--session-arg "allow_minting:bool='true'" 
```

#### Utility session code

Specific entrypoints in use by the current implementation of the NFT contract require session code to accept return values passed by the contract over the Wasm boundary.
In order to help with the installation and use of the NFT contract, session code for such entrypoints has been provided. It is recommended that
users and DApp developers attempting to engage with the NFT contract do so with the help of the provided utility session code. The session code can be found in the `client`
folder within the project folder.

| Entrypoint name | Session code                  |
|------------------|-------------------------------|
| `"mint"`         | `client/mint_session`         |
| `"balance_of"`   | `client/balance_of_session`   |
| `"get_approved`  | `client/get_approved_session` |
| `"owner_of"`     | `client/owner_of_session`     |
| `"transfer"`     | `client/transfer_session`     |

### Installing and Interacting with the Contract using the Rust Casper Client

This contract code installs an instance of the CEP-78 enhanced NFT standard as per session arguments provided at the time of installation.

This contract requires a minimum Rust version of `1.63.0`.

#### Installing the Contract

Installing the enhanced NFT contract to global state requires the use of a [Deploy](https://docs.casperlabs.io/dapp-dev-guide/building-dapps/sending-deploys/). In this case, the session code can be compiled to Wasm by running the `make build-contract` command provided in the Makefile at the top level. The Wasm will be found in the `contract/target/wasm32-unknown-unknown/release` directory as `contract.wasm`.

Below is an example of a `casper-client` command that provides all required session arguments to install a valid instance of the CEP-78 contract on global state.

* `casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem --session-path ~/casper/enhanced-nft/contract/target/wasm32-unknown-unknown/release/contract.wasm`

1) `--session-arg "collection_name:string='CEP-78-collection'"` 

    The name of the NFT collection as a string. In this instance, "CEP-78-collection".


2) `--session-arg "collection_symbol:string='CEP78'"`

    The symbol representing the NFT collection as a string. In this instance, "CEP78".

3) `--session-arg "total_token_supply:u64='100'"`

    The total supply of tokens to be minted. In this instance, 100. If the contract owner is unsure of the total number of NFTs they will require, they should err on the side of caution. This value cannot be changed at a later date.
4) `--session-arg "ownership_mode:u8='2'"`

    The ownership mode for this contract. In this instance the 2 represents "Transferable" mode. Under these conditions, users can freely transfer their NFTs between one another.

5) `--session-arg "nft_kind:u8='1'"`

    The type of commodity represented by these NFTs. In this instance, the 1 represents a digital collection.

6) `--session-arg "nft_metadata_kind:u8='0'"`

    The type of metadata used by this contract. In this instance, the 0 represents CEP-78 standard for metadata.

7) `--session-arg "json_schema:string=''"`

    An empty JSON string, as the contract has awareness of the CEP-78 JSON schema. Using the custom validated modality would require passing through a valid JSON schema for your custom metadata.

8) `--session-arg "identifier_mode:u8='0'"`

    The mode used to identify individual NFTs. For 0, this means an ordinal identification sequence rather than by hash.

9) `--session-arg "metadata_mutability:u8='0'"`

    A setting allowing for mutability of metadata. This is only available when using the ordinal identification mode, as the hash mode depends on immutability for identification. In this instance, despite ordinal identification, the 0 represents immutable metadata.


The session arguments match the available modalities as listed in this [README](https://github.com/casper-ecosystem/cep-78-enhanced-nft).

<details>
<summary><b>Casper client command without comments</b></summary>

```bash
casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem --session-path ~/casper/enhanced-nft/contract/target/wasm32-unknown-unknown/release/contract.wasm \
--session-arg "collection_name:string='CEP-78-collection'" \
--session-arg "collection_symbol:string='CEP78'" \
--session-arg "total_token_supply:u64='100'" \
--session-arg "ownership_mode:u8='2'" \
--session-arg "nft_kind:u8='1'" \
--session-arg "nft_metadata_kind:u8='0'" \
--session-arg "json_schema:string=''" \
--session-arg "identifier_mode:u8='0'" \
--session-arg "metadata_mutability:u8='0'" 
```

</details>

#### Directly Invoking Entrypoints

With the release of CEP-78 version 1.1, users that are interacting with a CEP-78 contract that does not use `ReverseLookupMode` should opt out of using the client Wasms provided as part of the release. Opting out in this situation is recommended, as directly invoking the entrypoints incurs a lower gas cost compared against using the provided client Wasm to invoke the entrypoint.

You may invoke the `mint`, `transfer` or `burn` entrypoints directly through either the contract package hash or the contract hash directly.

Specifically in the case of `mint`, there are fewer runtime arguments that must be provided, thereby reducing the total gas cost of minting an NFT.

<details>
<summary><b>Example Mint using StoredVersionByHash</b></summary>

```bash

casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" \ --payment-amount 7500000000 \ -k ~/secret_key.pem \
--session-package-hash hash-b3b7a74ae9ef2ea8afc06d6a0830961259605e417e95a53c0cb1ca9737bb0ec7 \
--session-entry-point "mint" \
--session-arg "token_owner:key='account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb'" \
--session-arg "token_meta_data:string='{\"name\": \"John Doe\",\"token_uri\": \"https:\/\/www.barfoo.com\",\"checksum\": \"940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb\"}'"

```

</details>

<details>
<summary><b>Example Transfer using StoredContractByHash</b></summary>

Based on the identifier mode for the given contract instance, either the `token_id` runtime argument must be passed in or in the case of the hash identifier mode, the `token_hash` runtime argument.

```bash

casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" \ --payment-amount 7500000000 \ -k ~/secret_key.pem \
--session-hash hash-b3b7a74ae9ef2ea8afc06d6a0830961259605e417e95a53c0cb1ca9737bb0ec7 \
--session-entry-point "transfer" \
--session-arg "source_key:key='account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb'" \
--session-arg "target_key:key='account-hash-b4782e7c47e4deca5bd90b7adb2d6e884f2d331825d5419d6cbfb59e17642aab'" \
--session-arg "token_id:u64='0'" 

```

</details>

#### Minting an NFT

Below is an example of a `casper-client` command that uses the `mint` function of the contract to mint an NFT for the user associated with `node-1` in an [NCTL environment](https://docs.casperlabs.io/dapp-dev-guide/building-dapps/nctl-test/).

* `casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 5000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem --session-path ~/casper/enhanced-nft/client/mint_session/target/wasm32-unknown-unknown/release/mint_call.wasm`

1) `--session-arg "nft_contract_hash:key='hash-206339c3deb8e6146974125bb271eb510795be6f250c21b1bd4b698956669f95'"`

    The contract hash of the previously installed CEP-78 NFT contract from which we will be minting.

2) `--session-arg "collection_name:string='cep78_<collection_name>'"`

    The collection name of the previously installed CEP-78 NFT contract from which we will be minting.

3) `--session-arg "token_owner:key='account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb'"`

    The collection name of the NFT to be minted.

4) `--session-arg "token_meta_data:string='{\"name\": \"John Doe\",\"token_uri\": \"https:\/\/www.barfoo.com\",\"checksum\": \"940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb\"}'"`

    Metadata describing the NFT to be minted, passed in as a `string`.



<details>
<summary><b>Casper client command without comments</b></summary>

```bash

casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 5000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem --session-path ~/casper/enhanced-nft/client/mint_session/target/wasm32-unknown-unknown/release/mint_call.wasm \
--session-arg "nft_contract_hash:key='hash-206339c3deb8e6146974125bb271eb510795be6f250c21b1bd4b698956669f95'" \
`--session-arg "collection_name:string='cep78_<collection_name>'"` \
--session-arg "token_owner:key='account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb'"  \
--session-arg "token_meta_data:string='{\"name\": \"John Doe\",\"token_uri\": \"https:\/\/www.barfoo.com\",\"checksum\": \"940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb\"}'"

```

</details>

#### Transferring NFTs Between Users

Below is an example of a `casper-client` command that uses the `transfer` function to transfer ownership of an NFT from one user to another. In this case, we are transferring the previously minted NFT from the user associated with `node-2` to the user associated with `node-3`.

* `casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 5000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-2/keys/secret_key.pem --session-path ~/casper/enhanced-nft/client/transfer_session/target/wasm32-unknown-unknown/release/transfer_call.wasm`

1) `--session-arg "nft_contract_hash:key='hash-52e78ae3f6c485d036a74f65ebbb8c75fcc7c33fb42eb667fb32aeba72c63fb5'"`

    The contract hash of the CEP-78 NFT Contract associated with the NFT to be transferred.

2) `--session-arg "source_key:key='account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb'"`

    The account hash of the user that currently owns the NFT and wishes to transfer it.

3) `--session-arg "target_key:key='account-hash-b4772e7c47e4deca5bd90b7adb2d6e884f2d331825d5419d6cbfb59e17642aab'"`

    The account hash of the user that will receive the NFT.

4) `--session-arg "is_hash_identifier_mode:bool='false'"`

    Argument that the hash identifier mode is ordinal, thereby requiring a `token_id` rather than a `token_hash`.

5) `--session-arg "token_id:u64='0'"` 

    The `token_id` of the NFT to be transferred.

<details>
<summary><b>Casper client command without comments</b></summary>

```bash

casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 5000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-2/keys/secret_key.pem --session-path ~/casper/enhanced-nft/client/transfer_session/target/wasm32-unknown-unknown/release/transfer_call.wasm \
--session-arg "nft_contract_hash:key='hash-52e78ae3f6c485d036a74f65ebbb8c75fcc7c33fb42eb667fb32aeba72c63fb5'" \
--session-arg "source_key:key='account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb'" \
--session-arg "target_key:key='account-hash-b4772e7c47e4deca5bd90b7adb2d6e884f2d331825d5419d6cbfb59e17642aab'" \
--session-arg "is_hash_identifier_mode:bool='false'" \
--session-arg "token_id:u64='0'"  

```

</details>

#### Burning an NFT

Below is an example of a `casper-client` command that uses the `burn` function to burn an NFT within a CEP-78 collection. If this command is used, the NFT in question will no longer be accessible by anyone.

* `casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 5000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem`

1) `--session-hash hash-52e78ae3f6c485d036a74f65ebbb8c75fcc7c33fb42eb667fb32aeba72c63fb5`

    The session hash corresponding to the NFT's contract hash.

2) `--session-entry-point "burn"`

    The entrypoint corresponding to the `burn` function.

3) `--session-arg "token_id:u64='1'"`

    The token ID for the NFT to be burned. If the `identifier_mode` is not set to `Ordinal`, you must provide the `token_hash` instead.

<details>
<summary><b>Casper client command without comments</b></summary>

```bash

casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 5000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem \
--session-hash hash-52e78ae3f6c485d036a74f65ebbb8c75fcc7c33fb42eb667fb32aeba72c63fb5 \
--session-entry-point "burn" \
--session-arg "token_id:u64='1'"

```

</details>

## Test Suite and Specification

The expected behavior of the NFT contract implementation is asserted by its test suite found in the `tests` folder.
The test suite and the corresponding unit tests comprise the specification around the contract and outline the expected behaviors
of the NFT contract across the entire range of possible configurations (i.e modalities and toggles like allow minting). The test suite 
ensures that as new modalities are added, and current modalities are extended, no regressions and conflicting behaviors are introduced.
The test suite also asserts the correct working behavior of the utility session code provided in the client folder. The tests can be run 
by using the provided `Makefile` and running the `make test` command.

## Owner Reverse Lookup Functionality

In version 1.0 of the CEP-78 Enhanced NFT Standard contract, tracking minted tokens consisted of a single, unbounded list that would grow in size with each additional token. As a result, gas costs would increase over time as the list must be overwritten with each new minting.

In an effort to stabilize the gas costs of larger NFT collections, version 1.1 of CEP-78 includes the use of a pre-allocated page system to track ownership of NFTs within the contract.

This system stabilizes the cost for interacting with the contract, but not the mint price itself. The size of metadata for a collection, and any differences in that metadata, will still result in some fluctuation in the price for the NFT itself. However, the cost of engaging the system itself will remain stable. Users can expect to pay a higher upfront price for page allocation, but will not need to pay this cost again for any NFTs minted within that given page.

### The CEP-78 Page System

Ownership of NFTs within a CEP-78 contract is now tracked with a series of `pages`, with each page tracking a range of 1,000 tokens each. When installing an instance of the CEP-78 contract, the user determines the total token supply. This, in turn, determines the maximum number of pages, i.e., for a 10,000 token collection, each account could have up to 10 pages numbering from 0-9 tracking ownership of NFTs.

A `page_table` tracks which pages within a range have been allocated and set for a certain user. The size of the page table directly correlates to the total token supply, i.e. for a CEP-78 instance tracking 10,000 tokens, the page table would be 10 bits wide. For a total of 20,000 it would be 20 bits wide. The cost of the initial page table allocation depends on the overall total size of a collection, with larger collections possessing correspondingly greater gas costs. To make initial minting costs more stable across contracts, the process of allocating a page table has been shifted to the `register_owner` entrypoint.

After registering as an owner, the contract creates an entry within the `page_table` dictionary for the minting account or contract. This dictionary entry consists of a series of `boolean` values amounting to the total number of pages in the collection. In our 10,000 token example, this would be 10 bits set to false.

Upon minting the token, the user will pay for a page allocation. This adds them to the `page` dictionary, in which each entry corresponds to a specific account or contract that owns tokens within that page. That account or contract's entry in the `page` dictionary will consist of 1,000 `page_address` bits set to `False` upon allocation, and the minting of any given token in that page will set the `page_address` bit to `True`.

In addition, that account or contract's `page_table` will be updated by marking the corresponding page number's bit as `True`.

As an example, consider a new user minting their first NFT with a given CEP-78 contract set to a maximum number of 10,000 tokens. They are minting the 2,350th token within that collection. The following sequence of events would occur:

1) The contract registers their account as an owner.

2) The contract creates a `page_table` dictionary for that account, with 10 boolean values. As the numbering system begins with `0`, the third boolean value corresponding with page `2` is set to `True`.

3) The account pays for allocation of page 2, creating an entry in the `Page 2` dictionary for that account. Within that entry, there are 1,000 boolean values set to false. Minting the 2,350th token in the collection sets the corresponding `page_address` boolean for 350 as `True`.

4) Any further tokens minted by this account prior to the 3,000th token being minted will not have to pay for additional page allocations. If the account mints a token at or beyond 3,000, they must pay for the corresponding page allocation. For example, if they decided to mint the 5,125th token in the collection, they would need to pay for `page 5` to be allocated to them. They would then be added to the `page 5` dictionary with the `page_address` boolean for 125 set as `True`.

This system binds the data writing costs to a maximum size of any given page dictionary.

### Updated Receipts

If the contract enables `OwnerReverseLookupMode`, calling the `updated_receipts` entrypoint will return a list of receipt names alongside the dictionary for the relevant pages.

Updated receipts come in the format of "{<collection name>}_m{modulo}_p{<page number>}". Once again using the 2,350th token as an example, the receipt would read:

```
cep78_collection_m_350_p_2
```

You can determine the token number by multiplying the `page_number` by the `page_size`(1,000) and adding the `modulo`.

If the `NFTIdentifierMode` is set to `Ordinal`, this number corresponds directly to the token ID.

If it is set to `Hash`, you will need to reference the `HASH_BY_INDEX` dictionary to determine the mapping of token numbers to token hashes.

## Casper Event Standard

When the `CES` events mode is enabled during contract installation, the operations that make changes on the tokens are recorded in the `__events` dictionary. Such event entries can be observed via a node's Server Side Events stream or querying the dictionary at any time using the RPC interface.

The emitted events are encoded according to the [Casper Event Standard](https://github.com/make-software/casper-event-standard), and the schema can be known by an observer reading the `__events_schema` contract named key.

For this CEP-78 reference implementation in particular, the events schema is the following:

| Event name      | Included values and type                                   |
|-----------------|----------------------------------------------------------------------|
| Mint            | recipient (Key), token_id (Any), data (String)                       |
| Transfer        | owner (Key), operator (Option<Key>), recipient (Key), token_id (Any) |
| Burn            | owner (Key), token_id (Any)                                          |
| Approval        | owner (Key), operator (Option<Key>), token_id (Any)                  |
| ApprovalForAll  | owner (Key), operator (Option<Key>), token_ids (List<Any>)           |
| MetadataUpdated | token_id (Any), data (String)                                        |
| Migration       | -                                                                    |
| VariablesSet    | -                                                                    |

Token identifiers are stored under the `CLType` `Any` and the encoding depends on `NFTIdentifierMode`:.

* `NFTIdentifierMode::Ordinal`: the `token id` is encoded as a byte `0x00` followed by a `u64` number.
* `NFTIdentifierMode::Hash`: the `token_id` is encoded as a byte `0x01` followed by a `String`.

## Error Codes

|Code   |Error                              |
|-------|-----------------------------------|
|   1   | InvalidAccount                    |
|   2   | MissingInstaller                  |
|   3   | InvalidInstaller                  |
|   4   | UnexpectedKeyVariant              |
|   5   | MissingTokenOwner                 |
|   6   | InvalidTokenOwner                 |
|   7   | FailedToGetArgBytes               |
|   8   | FailedToCreateDictionary          |
|   9   | MissingStorageUref                |
|  10   | InvalidStorageUref                |
|  11   | MissingOwnerUref                  |
|  12   | InvalidOwnersUref                 |
|  13   | FailedToAccessStorageDictionary   |
|  14   | FailedToAccessOwnershipDictionary |
|  15   | DuplicateMinted                   |
|  16   | FailedToConvertCLValue            |
|  17   | MissingCollectionName             |
|  18   | InvalidCollectionName             |
|  19   | FailedToSerializeMetaData         |
|  20   | MissingAccount                    |
|  21   | MissingMintingStatus              |
|  22   | InvalidMintingStatus              |
|  23   | MissingCollectionSymbol           |
|  24   | InvalidCollectionSymbol           |
|  25   | MissingTotalTokenSupply           |
|  26   | InvalidTotalTokenSupply           |
|  27   | MissingTokenID                    |
|  28   | InvalidTokenIdentifier            |
|  29   | MissingTokenOwners                |
|  30   | MissingAccountHash                |
|  31   | InvalidAccountHash                |
|  32   | TokenSupplyDepleted               |
|  33   | MissingOwnedTokensDictionary      |
|  34   | TokenAlreadyBelongsToMinterFatal  |
|  35   | FatalTokenIdDuplication           |
|  36   | InvalidMinter                     |
|  37   | MissingMintingMode                |
|  38   | InvalidMintingMode                |
|  39   | MissingInstallerKey               |
|  40   | FailedToConvertToAccountHash      |
|  41   | InvalidBurner                     |
|  42   | PreviouslyBurntToken              |
|  43   | MissingAllowMinting               |
|  44   | InvalidAllowMinting               |
|  45   | MissingNumberOfMintedTokens       |
|  46   | InvalidNumberOfMintedTokens       |
|  47   | MissingTokenMetaData              |
|  48   | InvalidTokenMetaData              |
|  49   | MissingApprovedAccountHash        |
|  50   | InvalidApprovedAccountHash        |
|  51   | MissingApprovedTokensDictionary   |
|  52   | TokenAlreadyApproved              |
|  53   | MissingApproveAll                 |
|  54   | InvalidApproveAll                 |
|  55   | MissingOperator                   |
|  56   | InvalidOperator                   |
|  57   | Phantom                           |
|  58   | ContractAlreadyInitialized        |
|  59   | MintingIsPaused                   |
|  60   | FailureToParseAccountHash         |
|  61   | VacantValueInDictionary           |
|  62   | MissingOwnershipMode              |
|  63   | InvalidOwnershipMode              |
|  64   | InvalidTokenMinter                |
|  65   | MissingOwnedTokens                |
|  66   | InvalidAccountKeyInDictionary     |
|  67   | MissingJsonSchema                 |
|  68   | InvalidJsonSchema                 |
|  69   | InvalidKey                        |
|  70   | InvalidOwnedTokens                |
|  71   | MissingTokenURI                   |
|  72   | InvalidTokenURI                   |
|  73   | MissingNftKind                    |
|  74   | InvalidNftKind                    |
|  75   | MissingHolderMode                 |
|  76   | InvalidHolderMode                 |
|  77   | MissingWhitelistMode              |
|  78   | InvalidWhitelistMode              |
|  79   | MissingContractWhiteList          |
|  80   | InvalidContractWhitelist          |
|  81   | UnlistedContractHash              |
|  82   | InvalidContract                   |
|  83   | EmptyContractWhitelist            |
|  84   | MissingReceiptName                |
|  85   | InvalidReceiptName                |
|  86   | InvalidJsonMetadata               |
|  87   | InvalidJsonFormat                 |
|  88   | FailedToParseCep78Metadata        |
|  89   | FailedToParse721Metadata          |
|  90   | FailedToParseCustomMetadata       |
|  91   | InvalidCEP78Metadata              |
|  92   | FailedToJsonifyCEP78Metadata      |
|  93   | InvalidNFT721Metadata             |
|  94   | FailedToJsonifyNFT721Metadata     |
|  95   | InvalidCustomMetadata             |
|  96   | MissingNFTMetadataKind            |
|  97   | InvalidNFTMetadataKind            |
|  98   | MissingIdentifierMode             |
|  99   | InvalidIdentifierMode             |
|  100  | FailedToParseTokenId              |
|  101  | MissingMetadataMutability         |
|  102  | InvalidMetadataMutability         |
|  103  | FailedToJsonifyCustomMetadata     |
|  104  | ForbiddenMetadataUpdate           |
|  105  | MissingBurnMode                   |
|  106  | InvalidBurnMode                   |
|  107  | MissingHashByIndex                |
|  108  | InvalidHashByIndex                |
|  109  | MissingIndexByHash                |
|  110  | InvalidIndexByHash                |
|  111  | MissingPageTableURef              |
|  112  | InvalidPageTableURef              |
|  113  | MissingPageLimit                  |
|  114  | InvalidPageLimit                  |
|  115  | InvalidPageNumber                 |
|  116  | InvalidPageIndex                  |
|  117  | MissingUnmatchedHashCount         |
|  118  | InvalidUnmatchedHashCount         |
|  119  | MissingPackageHashForUpgrade      |
|  120  | MissingPageUref                   |
|  121  | InvalidPageUref                   |
|  122  | CannotUpgradeWithZeroSupply       |
|  123  | CannotInstallWithZeroSupply       |
|  124  | MissingMigrationFlag              |
|  125  | InvalidMigrationFlag              |
|  126  | ContractAlreadyMigrated           |
|  127  | UnregisteredOwnerInMint           |
|  128  | UnregisteredOwnerInTransfer       |
|  129  | MissingReportingMode              |
|  130  | InvalidReportingMode              |
|  131  | MissingPage                       |
|  132  | UnregisteredOwnerFromMigration    |
|  133  | ExceededMaxTotalSupply            |
|  134  | MissingCep78PackageHash           |
|  135  | InvalidCep78InvalidHash           |
|  136  | InvalidPackageHashName            |
|  137  | InvalidAccessKeyName              |
|  138  | InvalidCheckForUpgrade            |
|  139  | InvalidNamedKeyConvention         |