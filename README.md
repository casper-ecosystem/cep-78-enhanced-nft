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

## Table of Contents

1) [Modalities](#modalities)

2) [Usage](#usage)

3) [Installing and Interacting with the Contract using the Rust Casper Client](#installing-and-interacting-with-the-contract-using-the-rust-casper-client)

4) [Test Suite and Specification](#test-suite-and-specification)

5) [Error Codes](#error-codes)

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
3. `Raw`: This mode specifies that metadata validation will not occur and raw strings can be passed to `token_metadata` runtime argument as part of the call to `mint` entry point.
4. `CustomValidated`: This mode specifies that a custom schema provided at the time of install will be used when validating the metadata as part of the call to `mint` entry point.

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
2. `Mutable`: Metadata for NFTs minted in this mode can update the metadata via the `set_token_metadata` entry point.

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

#### Modality Conflicts

The `MetadataMutability` option of `Mutable` cannot be used in conjunction with `NFTIdentifierMode` modality of `Hash`.


### Usage

#### Installing the contract.

The `main.rs` file within the contract provides the installer for the NFT contract. Users can compile the contract to Wasm using the `make build-contract` with the provided Makefile.

The pre-built Wasm for the contract and all other utility session code can be found as part of the most current release. Users wishing to build the Wasm themselves can pull the code and the `make build-contract` provided in the included Makefile. Please note, however, that as part of building the contract, you will need to install `wasm-strip`.

The `call` method will install the contract with the necessary entry points and call `init()` entry point to allow the contract to self initialize and setup the necessary state to allow for operation,
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
* `"allow_minting"`: The `"allow_minting"` flag allows the installer of the contract to pause the minting of new NFTs. The `allow_minting` is a boolean toggle that allows minting when `true`. If not provided at install the toggle will default to `true`. This value can be changed by the installer by calling the `set_variables()` entry point.
* `"whitelist_mode"`: The [`WhitelistMode`](#whitelistmode) modality dictates whether the contract whitelist can be updated. This optional parameter will default to an unlocked whitelist that can be updated post installation. This parameter cannot be changed once the contract has been installed.
* `"holder_mode"`: The [`NFTHolderMode`](#nftholdermode) modality dictates which entities can hold NFTs. This is an optional parameter and will default to a mixed mode allowing either `Accounts` or `Contracts` to hold NFTs. This parameter cannot be changed once the contract has been installed.
* `"contract_whitelist"`: The contract whitelist is a list of contract hashes that specifies which contracts can call the `mint()` entrypoint to mint NFTs. This is an optional parameter which will default to an empty whitelist. This value can be changed via the `set_variables` post installation. If the whitelist mode is set to locked, a non-empty whitelist must be passed; else, installation of the contract will fail.
* `"burn_mode"`: The [`BurnMode`](#burnmode) modality dictates whether minted NFTs can be burnt. This is an optional parameter and will allow tokens to be burnt by default. This parameter cannot be changed once the contract has been installed.

##### Example deploy

The following is an example of deploying the installation of the NFT contract via the Rust Casper command client.

```rust
casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem --session-path ~/casper/enhanced-nft/contract/target/wasm32-unknown-unknown/release/contract.wasm --session-arg "collection_name:string='enhanced-nft-1'" --session-arg "collection_symbol:string='ENFT-1'" --session-arg "total_token_supply:u256='10'" --session-arg "ownership_mode:u8='0'" --session-arg "nft_kind:u8='1'" --session-arg "json_schema:string='nft-schema'" --session-arg "allow_minting:bool='true'" 
```

#### Utility session code

Specific entry points in use by the current implementation of the NFT contract require session code to accept return values passed by the contract over the Wasm boundary.
In order to help with the installation and use of the NFT contract, session code for such entry points has been provided. It is recommended that
users and DApp developers attempting to engage with the NFT contract do so with the help of the provided utility session code. The session code can be found in the `client`
folder within the project folder.

| Entry point name | Session code                  |
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

casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem --session-path ~/casper/enhanced-nft/contract/target/wasm32-unknown-unknown/release/contract.wasm 
--session-arg "collection_name:string='CEP-78-collection'" 
--session-arg "collection_symbol:string='CEP78'" 
--session-arg "total_token_supply:u64='100'" 
--session-arg "ownership_mode:u8='2'" 
--session-arg "nft_kind:u8='1'" 
--session-arg "nft_metadata_kind:u8='0'" 
--session-arg "json_schema:string=''"
--session-arg "identifier_mode:u8='0'" 
--session-arg "metadata_mutability:u8='0'"

```

</details>

#### Minting an NFT

Below is an example of a `casper-client` command that uses the `mint` function of the contract to mint an NFT for the user associated with `node-1` in an [NCTL environment](https://docs.casperlabs.io/dapp-dev-guide/building-dapps/nctl-test/).

* `casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem --session-path ~/casper/enhanced-nft/client/mint_session/target/wasm32-unknown-unknown/release/mint_call.wasm`

1) `--session-arg "nft_contract_hash:key='hash-206339c3deb8e6146974125bb271eb510795be6f250c21b1bd4b698956669f95'"`

    The contract hash of the previously installed CEP-78 NFT contract from which we will be minting.

2) `--session-arg "token_owner:key='account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb'"`

    The account hash of the account receiving the minted token.

3) `--session-arg "token_meta_data:string='{\"name\": \"John Doe\",\"token_uri\": \"https:\/\/www.barfoo.com\",\"checksum\": \"940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb\"}'"`

    Metadata describing the NFT to be minted, passed in as a `string`.



<details>
<summary><b>Casper client command without comments</b></summary>

```bash

casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem --session-path ~/casper/enhanced-nft/client/mint_session/target/wasm32-unknown-unknown/release/mint_call.wasm
--session-arg "nft_contract_hash:key='hash-206339c3deb8e6146974125bb271eb510795be6f250c21b1bd4b698956669f95'"
--session-arg "token_owner:key='account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb'"
--session-arg "token_meta_data:string='{\"name\": \"John Doe\",\"token_uri\": \"https:\/\/www.barfoo.com\",\"checksum\": \"940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb\"}'"

```

</details>

#### Transferring NFTs Between Users

Below is an example of a `casper-client` command that uses the `transfer` function to transfer ownership of an NFT from one user to another. In this case, we are transferring the previously minted NFT from the user associated with `node-2` to the user associated with `node-3`.

* `casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-2/keys/secret_key.pem --session-path ~/casper/enhanced-nft/client/transfer_session/target/wasm32-unknown-unknown/release/transfer_call.wasm`

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

casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-2/keys/secret_key.pem --session-path ~/casper/enhanced-nft/client/transfer_session/target/wasm32-unknown-unknown/release/transfer_call.wasm 
--session-arg "nft_contract_hash:key='hash-52e78ae3f6c485d036a74f65ebbb8c75fcc7c33fb42eb667fb32aeba72c63fb5'" 
--session-arg "source_key:key='account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb'" 
--session-arg "target_key:key='account-hash-b4772e7c47e4deca5bd90b7adb2d6e884f2d331825d5419d6cbfb59e17642aab'" 
--session-arg "is_hash_identifier_mode:bool='false'" 
--session-arg "token_id:u64='0'"  

```

</details>

#### Burning an NFT

Below is an example of a `casper-client` command that uses the `burn` function to burn an NFT within a CEP-78 collection. If this command is used, the NFT in question will no longer be accessible by anyone.

* `casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem`

1) `--session-hash hash-52e78ae3f6c485d036a74f65ebbb8c75fcc7c33fb42eb667fb32aeba72c63fb5`

    The session hash corresponding to the NFT's contract hash.

2) `--session-entry-point "burn"`

    The entry point corresponding to the `burn` function.

3) `--session-arg "token_id:u64='1'"`

    The token ID for the NFT to be burned. If the `identifier_mode` is not set to `Ordinal`, you must provide the `token_hash` instead.

<details>
<summary><b>Casper client command without comments</b></summary>

```bash

casper-client put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem
--session-hash hash-52e78ae3f6c485d036a74f65ebbb8c75fcc7c33fb42eb667fb32aeba72c63fb5
--session-entry-point "burn"
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
