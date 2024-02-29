# Installing an NFT Contract using the Rust Casper Client

This documentation will guide you through the process of installing and interacting with an instance of the CEP-78 enhanced NFT standard contract through Casper's Rust CLI client. The contract code installs an instance of CEP-78 as per session arguments provided at the time of installation. It requires a minimum Rust version of `1.63.0`. The code for this tutorial is available in [GitHub](https://github.com/casper-ecosystem/cep-78-enhanced-nft/). A portion of this tutorial reviews the [contract](../../../contract/src/main.rs).

Information on the modalities used throughout this installation process can be found in the [modalities documentation](modalities.md).

## Table of Contents

1. [Environment Setup](#environment-setup)
    - [Prerequisites](#prerequisites)
    - [Building the Contract and Tests](#building-the-contract-and-tests)
2. [Reviewing the Contract Implementation](#reviewing-the-contract-implementation)
    - [Required Crates](#required-crates)
    - [Initialization Flow](#Initialization-flow)
    - [Contract Entrypoints](#contract-entrypoints)
3. [Installing the Contract](#installing-the-contract)
    - [Querying Global State](#querying-global-state)
    - [Sending the Installation Deploy](#sending-the-installation-deploy)
    - [Verifying the Installation](#verifying-the-installation)
4. [Next Steps](#next-steps)

## Environment Setup

### Prerequisites

Before using this guide, ensure you meet the following requirements:

- Set up the [development prerequisites](https://docs.casper.network/developers/prerequisites/), including the [Casper client](https://docs.casper.network/developers/prerequisites/#install-casper-client)
- Get a valid [node address](https://docs.casper.network/developers/prerequisites/#acquire-node-address-from-network-peers) from the network
- Know how to install a [smart contract](https://docs.casper.network/developers/cli/sending-deploys/) on a Casper network
- Hold enough CSPR tokens to pay for transactions

The [Writing Rust Contracts on Casper](https://docs.casper.network/developers/writing-onchain-code/simple-contract/) document outlines many aspects of this tutorial and should be read as a prerequisite.

### Building the Contract and Tests

First clone the contract from GitHub:

```bash
git clone https://github.com/casper-ecosystem/cep-78-enhanced-nft/ && cd cep-78-enhanced-nft
```

Prepare your environment with the following command:

```bash
make prepare
```

If your environment is set up correctly, you will see this output:

```bash
rustup target add wasm32-unknown-unknown
info: component 'rust-std' for target 'wasm32-unknown-unknown' is up to date
```

If you do not see this message, check the [Getting Started Guide](https://docs.casper.network/developers/writing-onchain-code/getting-started/).

The contract code can be compiled to Wasm by running the `make build-contract` command provided in the Makefile at the top level. The Wasm will be found in the `contract/target/wasm32-unknown-unknown/release` directory as `contract.wasm`.

You can also compile your contract and run the contract unit tests with this command:

```bash
make test
```

## Reviewing the Contract Implementation

In this repository, you will find a library and an [example NFT implementation](../../../contract/src/main.rs) for Casper networks. This section explains the example contract in more detail.

There are four steps to follow when you intend to create your implementation of the NFT contract, as follows:

1. Fork the code from the example repository listed above.
2. Perform any necessary customization changes on your fork of the example contract.
3. Compile the customized code to Wasm.
4. Send the customized Wasm as a deploy to a Casper network.

### Required Crates

This tutorial applies to the Rust implementation of the Casper NFT standard, which requires the following Casper crates:

- [casper_contract](https://docs.rs/casper-contract/latest/casper_contract/index.html) - A Rust library for writing smart contracts on Casper networks
- [casper_types](https://docs.rs/casper-types/latest/casper_types/) - Types used to allow the creation of Wasm contracts and tests for use on Casper networks

Here is the code snippet that imports those crates:

```rust
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
```

**Note**: In Rust, the keyword `use` is like an include statement in C/C++.

The contract code defines additional modules in the `contract/src` folder:

```rust
mod constants;
mod error;
mod events;
mod metadata;
mod modalities;
mod utils;
```

- `constants` - Constant values required to run the contract code
- `error` - Errors related to the NFT contract
- `events` - A library for contract-emitted events
- `metadata` - Module handling the contract's metadata and corresponding dictionary
- `modalities` - Common expectations around contract usage and behavior
- `utils` - Utility and helper functions to run the contract code

### Initialization Flow

Initializing the contract happens through the `call() -> install_contract() -> init()` functions inside the [main.rs](../../../contract/src/main.rs) contract file. The `init()` function reads the runtime arguments and defines parameters such as `collection_name`, `collection_symbol`, and `total_token_supply`, among the other required and optional arguments described in the [README](../../../README.md#required-runtime-arguments).

### Contract Entrypoints

This section briefly explains the essential entrypoints used in the Casper NFT contract. To see their full implementation, refer to the [main.rs](../../../contract/src/main.rs) contract file. For further questions, contact the Casper support team via the [Discord channel](https://discord.com/invite/casperblockchain). The following entrypoints are listed as they are found in the code.

- [**approve**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1002) - Allows a spender to transfer up to an amount of the owners’s tokens
- [**balance_of**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1616) - Returns the token balance of the owner
- [**burn**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L874) - Burns tokens, reducing the total supply
- [**get_approved**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1728) - Returns the hash of the approved account for a specified token identifier
- [**init**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L81) - Sets the collection name, symbol, and total token supply; initializes the allow minting setting, minting mode, ownership mode, NFT kind, holder mode, whitelist mode and contract whitelist, JSON schema, receipt name, identifier mode, and burn mode. This entrypoint can only be called once when the contract is installed on the network
- [**metadata**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1675) - Returns the metadata associated with a token identifier
- [**mint**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L619) - Mints additional tokens if minting is allowed, increasing the total supply
- [**owner_of**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1636) - Returns the owner for a specified token identifier
- [**set_approval_for_all**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1254) - Allows a spender to transfer all of the owner's tokens
- [**set_token_metadata**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1773) - Sets the metadata associated with a token identifier
- [**set_variables**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L496) - Allows the user to set any combination of variables simultaneously, defining which variables are mutable or immutable
- [**transfer**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1359) - Transfers tokens from the token owner to a specified account.  The transfer will succeed if the caller is the token owner or an approved operator. The transfer will fail if the OwnershipMode is set to Minter or Assigned

There is also the [**migrate**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1975) entrypoint, which was needed only for migrating a 1.0 version of the NFT contract to version 1.1.

## Installing the Contract

Installing the enhanced NFT contract to global state requires the use of a [Deploy](https://docs.casper.network/developers/dapps/sending-deploys/). But before proceeding with the installation, verify the network state and the status of the account that will send the installation deploy.

### Querying Global State

This step queries information about the network state given the latest state root hash. You will also need the [IP address](https://docs.casper.network/developers/prerequisites/#acquire-node-address-from-network-peers) from a Testnet peer node.

```bash
casper-client get-state-root-hash --node-address http://localhost:11101/rpc/
```

### Querying the Account State

Run the following command and supply the path to your public key in hexadecimal format to get the account hash if you don't have it already.

```bash
casper-client account-address --public-key "[PATH_TO_PUBLIC_KEY_HEX]"
```

Use the command below to query the state of your account.

```bash
casper-client query-global-state --node-address http://<HOST:PORT> \
--state-root-hash [STATE_ROOT_HASH] \
--key [ACCOUNT_HASH]
```

<details>
<summary><b>Expand for a sample command</b></summary>

```bash
casper-client query-global-state --node-address http://localhost:11101/rpc/ \
--state-root-hash 376b18e95312328f212f9966200fa40734e66118cbd34ace0a1ec14eacaea6e6 \
--key account-hash-82729ae3b368bb2c45d23c05c872c446cbcf32b694f1d9efd3d1ea46cf227a11
```

</details>

<!-- TODO add a sample response -->

### Sending the Installation Deploy

Below is an example of a `casper-client` command that provides all required session arguments to install a valid instance of the CEP-78 contract on global state. 

Use the Testnet to understand the exact gas amount required for installation. Refer to the [note about gas prices](https://docs.casper.network/developers/cli/sending-deploys/#a-note-about-gas-price) to understand payment amounts and gas price adjustments.

- `casper-client put-deploy -n http://localhost:11101/rpc/ --chain-name "casper-net-1" --payment-amount 5000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-2/keys/secret_key.pem --session-path contract/target/wasm32-unknown-unknown/release/contract.wasm`

1. `--session-arg "collection_name:string='CEP-78-collection'"`

   The name of the NFT collection as a string. In this instance, "CEP-78-collection".

2. `--session-arg "collection_symbol:string='CEP78'"`

   The symbol representing the NFT collection as a string. In this instance, "CEP78".

3. `--session-arg "total_token_supply:u64='100'"`

   The total supply of tokens to be minted. In this instance, 100. If the contract owner is unsure of the total number of NFTs they will require, they should err on the side of caution.

4. `--session-arg "ownership_mode:u8='2'"`

   The ownership mode for this contract. In this instance the 2 represents "Transferable" mode. Under these conditions, users can freely transfer their NFTs between one another.

5. `--session-arg "nft_kind:u8='1'"`

   The type of commodity represented by these NFTs. In this instance, the 1 represents a digital collection.

6. `--session-arg "nft_metadata_kind:u8='0'"`

   The type of metadata used by this contract. In this instance, the 0 represents CEP-78 standard for metadata.

7. `--session-arg "json_schema:string=''"`

   An empty JSON string, as the contract has awareness of the CEP-78 JSON schema. Using the custom validated modality would require passing through a valid JSON schema for your custom metadata.

8. `--session-arg "identifier_mode:u8='0'"`

   The mode used to identify individual NFTs. For 0, this means an ordinal identification sequence rather than by hash.

9. `--session-arg "metadata_mutability:u8='0'"`

   A setting allowing for mutability of metadata. This is only available when using the ordinal identification mode, as the hash mode depends on immutability for identification. In this instance, despite ordinal identification, the 0 represents immutable metadata.

The session arguments match the available [modalities](../../modalities.md).


<details>
<summary><b>Expand for a sample query and response</b></summary>

```bash
casper-client put-deploy --node-address http://localhost:11101/rpc/ \
--chain-name "casper-net-1" \
--payment-amount 5000000000 \
--secret-key ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem \
--session-path contract/target/wasm32-unknown-unknown/release/contract.wasm \
--session-arg "collection_name:string='CEP-78-collection'" \
--session-arg "collection_symbol:string='CEP78'" \
--session-arg "total_token_supply:u64='100'" \
--session-arg "ownership_mode:u8='2'" \
--session-arg "nft_kind:u8='1'" \
--session-arg "nft_metadata_kind:u8='0'" \
--session-arg "json_schema:string='nft-schema'" \
--session-arg "identifier_mode:u8='0'" \
--session-arg "metadata_mutability:u8='0'"
```

This command will output the `deploy_hash`, which can be used in the next step to verify the installation.

```bash
{
  "id": 931694842944790108,
  "jsonrpc": "2.0",
  "result": {
    "api_version": "1.4.3",
    "deploy_hash": "b00E59f8aBA5c7aB9...."
  }
}
```

</details>


### Verifying the Installation

Verify the sent deploy using the `get-deploy` command.

```bash
casper-client get-deploy --node-address http://localhost:11101/rpc/ [DEPLOY_HASH]
```

<!-- TODO add a sample response -->

## Next Steps

- Learn to [Query](./querying-NFTs.md) the NFT contract
- Learn to [Mint, Transfer, and Burn](./interacting-with-NFTs.md) NFT tokens

