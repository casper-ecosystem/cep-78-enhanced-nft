# A Casper NFT Tutorial

This tutorial introduces an implementation of the CEP-78 standard for the Casper blockchain, known as the Casper Enhanced NFT standard. The code for this tutorial is available in [GitHub](https://github.com/casper-ecosystem/cep-78-enhanced-nft/).

The following functions implement the rules defined for Casper NFTs: `totalSupply`, `transfer`, `transferFrom`, `approve`, `balanceOf`, and `allowance`. A portion of this tutorial reviews the [contract](../../contract/src/main.rs).

The [Writing Rust Contracts on Casper](https://docs.casper.network/developers/writing-onchain-code/simple-contract/) document outlines many aspects of this tutorial and should be read first.

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
    - [Querying Contract Entry Points](#querying-contract-entry-points)


## Environment Setup

### Prerequisites

Before using this guide, ensure you meet the following requirements:

- Set up the [development prerequisites](https://docs.casper.network/developers/prerequisites/), including the [Casper client](https://docs.casper.network/developers/prerequisites/#install-casper-client)
- Get a valid [node address](https://docs.casper.network/developers/prerequisites/#acquire-node-address-from-network-peers), or use the following Testnet node: [https://rpc.testnet.casperlabs.io/](https://rpc.testnet.casperlabs.io/)
- Know how to install a [smart contract](https://docs.casper.network/developers/cli/sending-deploys/) on a Casper network
- Hold enough CSPR tokens to pay for transactions

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

Next, compile your contract and run the contract unit tests.

```bash
make test
```

## Reviewing the Contract Implementation

In this repository, you will find a library and an [example NFT implementation](../../contract/src/main.rs) for Casper networks. This section explains the example contract in more detail.

There are four steps to follow when you intend to create your own implementation of the NFT contract, as follows:

1. Fork the code from the example repository listed above.
2. Perform any customization changes necessary on your personal fork of the example contract.
3. Compile the customized code to Wasm.
4. Send the customized Wasm as a deploy to a Casper network.

### Required Crates

This tutorial applies to the Rust implementation of the Casper NFT standard, which requires the following Casper crates:

- [casper_contract](https://docs.rs/casper-contract/latest/casper_contract/index.html) - A Rust library for writing smart contracts on Casper networks
- [casper_types](https://docs.rs/casper-types/latest/casper_types/) - Types used to allow creation of Wasm contracts and tests for use on Casper networks

Here is the code snippet which imports those crates:

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

Initializing the contract happens through the `call() -> install_contract() -> init()` functions inside the [main.rs](../../contract/src/main.rs) contract file. The `init()` function reads the runtime arguments and defines parameters such as `collection_name`, `collection_symbol`, and `total_token_supply`, among the other required and optional arguments described in the [README](../../README.md#required-runtime-arguments).

### Contract Entrypoints

This section briefly explains the essential entrypoints used in the Casper NFT contract. To see their full implementation, refer to the [main.rs](../../contract/src/main.rs) contract file. For further questions, contact the Casper support team via the [Discord channel](https://discord.com/invite/casperblockchain). The following entrypoints are listed as they are found in the code.

- [**approve**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1002) - Allows a spender to transfer up to an amount of the owners’s tokens
- [**balance_of**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1616) - Returns the token balance of the owner
- [**burn**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L874) - Burns tokens, reducing the total supply
- [**get_approved**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1728) - Returns the hash of the approved account for a specified token identifier
- [**init**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L81) - Sets the collection name, symbol, and total token supply; initializes the allow minting setting, minting mode, ownership mode, NFT kind, holder mode, whitelist mode and contract whitelist, JSON schema, receipt name, identifier mode, and burn mode. This entrypoint can only be called once when the contract is installed on the network
- [**metadata**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1675) - Returns the metadata associated with a token identifier
- [**mint**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L619) - Mints additional tokens if mintin is allowed, increasing the total supply
- [**owner_of**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1636) - Returns the owner for a specified token identifier
- [**set_approval_for_all**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1254) - Allows a spender to transfer all of the owner's tokens
- [**set_token_metadata**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1773) - Sets the metadata associated with a token identifier
- [**set_variables**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L496) - Allows the user to set any combination of variables simultaneously, defining which variables are mutable or immutable
- [**transfer**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1359) - Transfers tokens from the token owner to a specified account.  The transfer will succeed if the caller is the token owner or an approved operators. The transfer will fail if the OwnershipMode is set to Minter or Assigned

There is also the [**migrate**](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/440bff44277ab5fd295f37229fe92278339d3753/contract/src/main.rs#L1975) entrypoint, which was needed only for migrating a 1.0 version of the NFT contract to version 1.1.

## Installing the Contract

After customizing your instance of the  NFT contract, install it on the network, just like any other Casper contract. The following sections briefly cover the commands you need. Refer to [Sending Deploys to a Casper network using the Rust Client](https://docs.casper.network/developers/dapps/sending-deploys/) for more details.

### Querying Global State

This step queries information about the network state given the latest state root hash. You will also need the [IP address](https://docs.casper.network/developers/prerequisites/#acquire-node-address-from-network-peers) from a Testnet peer node.

```bash
casper-client get-state-root-hash --node-address https://rpc.testnet.casperlabs.io/
```

### Querying the Account State

Run the following command and supply the path to your public key in hexadecimal format to get the account hash, if you don't have it already.

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
casper-client query-global-state --node-address https://rpc.testnet.casperlabs.io/ \
--state-root-hash e45cab47e15615cfe27c889b0a6446986077a1d6fb5b6a2be49d230273bc8d5b \
--key account-hash-5cb74580bcf97d0a7fa034e60b3d2952e0b170ea5162153b1570e8b1ee4ec3f5
```

</details>

<!-- TODO add a sample response -->

### Sending the Installation Deploy

Next, install the contract on the network. Use the Testnet to understand the exact gas amount required for installation. Refer to the [note about gas prices](https://docs.casper.network/developers/cli/sending-deploys/#a-note-about-gas-price) to understand payment amounts and gas price adjustments.

```bash
casper-client put-deploy --node-address http://<HOST:PORT> \
--chain-name [NETWORK_NAME] \
--payment-amount [AMOUNT] \
--secret-key [PATH_TO_SECRET_KEY] \
--session-path [WASM_FILE_PATH] \
--session-arg <"NAME:TYPE='VALUE'">
```

- `NETWORK_NAME`: Use the relevant network name
- `PATH_TO_SECRET_KEY`: The path to your secret key
- `AMOUNT`: Gas amount in motes needed for deploy execution
- `WASM_FILE_PATH`: The location of the compiled NFT Wasm file
- `NAME:TYPE='VALUE'`: The required and optional arguments for installing the contract

<details>
<summary><b>Expand for a sample query and response</b></summary>

```bash
casper-client put-deploy --node-address https://rpc.testnet.casperlabs.io/ \
--chain-name "casper-test" \
--payment-amount 200000000000 \
--secret-key ~/KEYS/secret_key.pem \
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
casper-client get-deploy --node-address https://rpc.testnet.casperlabs.io/ [DEPLOY_HASH]
```

<!-- TODO add a sample response -->

### Querying Contract Entry Points

This step will narrow down the context and check the status of a specific entry point using arguments.

```bash
casper-client query-global-state --node-address http://<HOST:PORT> \
--state-root-hash [STATE_ROOT_HASH] \
--key [ACCOUNT_HASH] \
-q "[CONTRACT_NAME/ARGUMENT]"
```

<!-- TODO add a correct query -->

<details>
<summary><b>Expand querying the contract name</b></summary>

```bash
casper-client query-global-state --node-address https://rpc.testnet.casperlabs.io/ \
--state-root-hash e45cab47e15615cfe27c889b0a6446986077a1d6fb5b6a2be49d230273bc8d5b \
--key account-hash-5cb74580bcf97d0a7fa034e60b3d2952e0b170ea5162153b1570e8b1ee4ec3f5 \
-q "nft_collection/name"
```

</details>

<!-- TODO add a sample response -->
