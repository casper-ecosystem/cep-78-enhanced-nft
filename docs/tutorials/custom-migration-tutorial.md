# Upgrading to v1.1.1 using a Custom NamedKey Convention

This tutorial uses the Casper command-line client to upgrade *and* migrate from an NFT contract installed using release [v1.0.0](https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/tag/v1.0.0) and customized NamedKeys. The outcome of the upgrade will be a [v1.1.1](https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/tag/v1.1.1) NFT contract with its storage structure migrated to the new format.

## Prerequisites

- You have previously installed a CEP-78 contract using release v1.0.0 on a Casper network. Thus, you have experience with the [Casper CEP-78 NFT Standard](https://github.com/casper-ecosystem/cep-78-enhanced-nft/), the Casper command-line client, and interacting with a Casper network.
- Your v1.0.0 NFT contract instance uses custom NamedKeys for the contract package hash and contract package access URef.
- You have the v1.0.0 contract package hash stored under a custom NamedKey in the account that installed the contract.
- You have the v1.0.0 contract package access URef stored under a custom NamedKey in the account that installed the contract.
- You understand what is new in [Version 1.1.0](https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/tag/v1.1.0) of the CEP-78 Enhanced NFT Standard.

## Upgrading and Migrating Terminology

The upgrade to version 1.1.1 involves a data migration to a new [page system](../reverse-lookup.md#the-cep-78-page-system) tracking token ownership. The usual [upgrade](https://docs.casperlabs.io/dapp-dev-guide/writing-contracts/upgrading-contracts/) process triggers the data migration. For more information, see [Standard Migration Tutorial](standard-migration-tutorial.md#upgrading-and-migrating-terminology).

## Steps to Upgrade to Version 1.1.1

Navigate to the [v1.1.1 release](https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/tag/v1.1.1) and download the [cep-78-wasm.tar.gz](https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/download/v1.1.1/cep-78-wasm.tar.gz) file. Unarchive it to have access to the Wasm files provided. The Rust source code is also available if you would like to build and test the contract according to the Makefile provided. 

The `cep-78-wasm` folder contains the `contract.wasm` to send to the network to upgrade the NFT contract to version 1.1.1. In addition, you will see other useful Wasm files to interact with the contract once it is stored on-chain.

### Custom NamedKeys before Migration

The custom migration path assumes that the contract has modified the NamedKey entries created during the v1.0.0. See the example below as well as the [NamedKeyConvention](../modalities.md#namedkeyconventionmode) modality. 

| NamedKey Pre-Migration | Explanation |
|-------------|-------------|
| nft_contract | The hash identifying the NFT contract |
| A user-specified String | The hash identifying the contract package containing the NFT contract. In this example, it is "mangled_hash_key" | 
| A user-specified String | The URef used as an access token to the contract package. In this example, it is "mangled_access_key" | 
| contract_version | The value tracking the latest contract version in the contract package | 
| nft-CEP-78-collection-contract-package-wasm... | A dictionary tracking the NFTs minted in v1.0.0 |

![Account Named Keys pre Migration](../assets/custom-namedkeys-pre-migration.png)  

### Initiating the Upgrade

When upgrading using the `casper-client`, you must provide four runtime arguments:

- `named_key_convention`: The [NamedKeyConvention](../modalities.md#namedkeyconventionmode) runtime argument as a u8 value equal to 2: `--session-arg "named_key_convention:u8='2'"`. See the [ARG_NAMED_KEY_CONVENTION](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/408db77c3b9ca22752c7f877ea99a01dfca03a7b/contract/src/main.rs#L1991).
- `collection_name`: The collection name specified when the contract was [installed](./getting-started/full-installation-tutorial.md) using the `collection_name` option. See the [contract code](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/408db77c3b9ca22752c7f877ea99a01dfca03a7b/contract/src/main.rs#L93) for details. 
- `hash_key_name`: The custom contract package hash NamedKey as a String. See the [ARG_HASH_KEY_NAME_1_0_0](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/408db77c3b9ca22752c7f877ea99a01dfca03a7b/contract/src/main.rs#L2006).
- `access_key_name`: The custom contract package access NamedKey as a String. See the [ARG_ACCESS_KEY_NAME_1_0_0](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/408db77c3b9ca22752c7f877ea99a01dfca03a7b/contract/src/main.rs#L2005).

Here is the `casper-client` command to upgrade and migrate to version 1.1.1 when custom NamedKeys are used:

```bash
casper-client put-deploy \
--node-address [NODE_SERVER_ADDRESS] \
--chain-name [CHAIN_NAME] \
--secret-key [KEY_PATH]/secret_key.pem \
--payment-amount [PAYMENT_AMOUNT_IN_MOTES] \
--session-path [PATH]/contract.wasm \
--session-arg "named_key_convention:u8='1'" \
--session-arg "collection_name:string='[COLLECTION_NAME]'" \
--session-arg "hash_key_name:string='[ARG_HASH_KEY_NAME_1_0_0]'" \
--session-arg "access_key_name:string='[ARG_ACCESS_KEY_NAME_1_0_0]'" 
```

Here is the full list of required arguments:
- `node-address`: An IP address of a peer on the network. The default port for JSON-RPC servers on Mainnet and Testnet is 7777.
- `chain-name`: The chain name of the network where you wish to send the deploy. For Mainnet, use *casper*. For Testnet, use *casper-test*.
- `secret-key`: The file name containing the secret key of the account paying for the deploy.
- `payment-amount`: The payment for the deploy in motes.
- `session-path`: The path to the compiled Wasm on your computer. When using the [cep-78-wasm.tar.gz](https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/download/v1.1.1/cep-78-wasm.tar.gz) provided, this would be the path to the `contract.wasm` file.
- `named_key_convention`: Argument that specifies the use of the `V_1_0_standard` [NamedKeyConvention](../modalities.md#namedkeyconventionmode).
- `collection_name`: Argument that specifies the collection name as a String.
- `hash_key_name`: The custom contract package hash NamedKey as a String.
- `access_key_name`: The custom contract package access NamedKey as a String.

The command returns the deploy hash that you can use to verify whether or not the deploy succeeded.

**Important Notes**: 

- When upgrading by installing a new version of the CEP-78 contract, you do not need to specify all the runtime arguments needed during the initial installation of version 1.0.0, such as `total_token_supply`, `ownership_mode`, etc.

- However, `total_token_supply` may be provided as an optional runtime argument if you wish to decrease the total supply during the upgrade. The provided argument cannot be larger than the previous total, but must be larger than the total number of minted tokens. It cannot be zero.

- Therefore, if the previous contract instance had a total token supply of 1,000 and 750 minted tokens, the `total_token_supply` optional argument must be between 751 and 1,000.

**Example command to upgrade to v1.1.1:**

The following is an example of upgrading and migrating to version 1.1.1 of a previously installed NFT collection using version 1.0.0 and with custom NamedKeys.

```bash
casper-client put-deploy \
--node-addres https://rpc.testnet.casperlabs.io/ \
--chain-name "casper-test" \
--secret-key ~/KEYS/secret_key.pem \
--payment-amount 300000000000 \
--session-path contract.wasm \
--session-arg "named_key_convention:u8='2'" \
--session-arg "collection_name:string='CEP-78-collection'" \
--session-arg "hash_key_name:string='mangled_hash_key'" \
--session-arg "access_key_name:string='mangled_access_key'"
```

**Other Examples:**

- A [Testnet account](https://testnet.cspr.live/account/013060d19fa5d5e471c2bbe88f35871454d2e162c444100eaca34671339c78ced4) that uses the custom NamedKeys "mangled_hash_key" and "mangled_access_key".
- An example [Testnet deploy](https://testnet.cspr.live/deploy/55cb135c9b600263baedf72b124b02ff6dd74dd27d2d9be444b1bed6ee5e3301) that specified the custom NamedKey convention.

### Custom NamedKeys after Migration

As part of the migration, the contract's NamedKeys will be renamed using a new naming convention, which includes the collection name and a `cep78_contract_` prefix.

| Deprecated NamedKey| New NamedKey | Explanation |
|-------------|--------------|-------------|
| nft_contract | cep78_contract_hash_[COLLECTION_NAME] | The hash identifying the NFT contract |
| A user-specified String | cep78_contract_package_[COLLECTION_NAME] | The hash identifying the contract package containing the NFT contract. In this example, it is "mangled_hash_key" | 
| A user-specified String | cep78_contract_package_access_[COLLECTION_NAME] | The URef used as an access token or reference to the contract package. In this example, it is "mangled_access_key" | 
| contract_version | cep78_contract_version_[COLLECTION_NAME] | The value tracking the latest contract version in the contract package | 

![Account Custom Named Keys](../assets/custom-namedkeys-post-migration.png)  

> **Note**: Notice that the deprecated NamedKeys are still on the list, but the new contract will not use them. Also, starting with version 1.1.0, all named keys will follow the new naming convention containing the collection name. 
