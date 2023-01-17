# Upgrading to v1.1.1 using a Standard NamedKey Convention

This tutorial uses the Casper command-line client to upgrade *and* migrate from an NFT contract installed using release [v1.0.0](https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/tag/v1.0.0) and standard NamedKeys. The outcome of the upgrade will be a [v1.1.1](https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/tag/v1.1.1) NFT contract with its storage structure migrated to the new format.

## Prerequisites

- You have previously installed a CEP-78 contract using release v1.0.0, in a standard way, on a Casper network. Thus, you have experience with the [Casper CEP-78 NFT Standard](https://github.com/casper-ecosystem/cep-78-enhanced-nft/), the Casper command-line client, and interacting with a Casper network.
- The v1.0.0 NFT contract uses the contract package hash and contract package access URef created during installation in a **standard way**, without using other NamedKeys to manage the contract package.
- You have the v1.0.0 contract package hash stored under the `nft_contract_package` NamedKey in the account that installed the contract.
- You have the v1.0.0 contract package access URef stored under the `nft_contract_package_access` NamedKey in the account that installed the contract.
- You understand what is [New in Version 1.1](https://github.com/casper-ecosystem/cep-78-enhanced-nft/#new-in-version-11) of the CEP-78 Enhanced NFT Standard.

## Upgrading and Migrating Terminology

An [upgrade](https://docs.casperlabs.io/dapp-dev-guide/writing-contracts/upgrading-contracts/) is the usual manner to release newer versions of a contract inside a contract package. When users install v1.1.* of a CEP-78 contract, they perform an upgrade and a data migration to a new [page system](https://github.com/casper-ecosystem/cep-78-enhanced-nft#the-cep-78-page-system) tracking token ownership. The [OwnerReverseLookupMode](https://github.com/casper-ecosystem/cep-78-enhanced-nft#ownerreverselookupmode) modality introduced in version 1.1.0 allows users to list NFTs by owner. The [README](../README.md) states:

```
If you are upgrading a contract from CEP-78 version 1.0 to 1.1, `OwnerReverseLookupMode` will be set to `Complete`, as this was the standard behavior of CEP-78 1.0. In addition to being set to `Complete`, existing records will be migrated into the CEP-78 1.1 format, which will impose a one-time gas cost to cover the migration.
```

Future upgrades may not involve data migration, but the data migration is necessary with release v1.1.*.

## Steps to Upgrade to Version 1.1.1

Navigate to the [v1.1.1 release](https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/tag/v1.1.1) and download the [cep-78-wasm.tar.gz](https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/download/v1.1.1/cep-78-wasm.tar.gz) file. Unarchive it to have access to the Wasm files provided. The Rust source code is also available if you would like to build and test the contract according to the Makefile provided. 

The `cep-78-wasm` folder contains the `contract.wasm` to send to the network to upgrade the NFT contract to version 1.1.1. In addition, you will see other useful Wasm files to interact with the contract once it is stored on-chain: `balance_of_call.wasm`, `get_approved_call.wasm`, `mint_call.wasm`, `owner_of_call.wasm`, `transfer_call.wasm`, and `updated_receipts.wasm`.

### Standard NamedKeys before Migration

The standard migration path assumes that the contract uses the NamedKey entries created during the v1.0.0 installation without any modifications. See the example below as well as the [NamedKeyConvention](https://github.com/casper-ecosystem/cep-78-enhanced-nft#namedkeyconventionmode) modality.

| NamedKey Pre-Migration | Explanation |
|-------------|-------------|
| nft_contract | The hash identifying the NFT contract |
| nft_contract_package | The hash identifying the contract package containing the NFT contract | 
| nft_contract_package_access | The URef used as an access token or reference to the contract package | 
| contract_version | The value tracking the latest contract version in the contract package | 
| nft-CEP-78-collection-contract-package-wasm... | A dictionary tracking the NFTs minted in v1.0.0 |


![Account Named Keys pre Migration](../assets/standard-namedkeys-pre-migration.png)  

### Initiating the Upgrade

When upgrading using the `casper-client`, you must provide two runtime arguments:

- `named_key_convention`: The [NamedKeyConvention](https://github.com/casper-ecosystem/cep-78-enhanced-nft#namedkeyconventionmode) runtime argument as a u8 value equal to 1: `--session-arg "named_key_convention:u8='1'"`. See the [ARG_NAMED_KEY_CONVENTION](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/408db77c3b9ca22752c7f877ea99a01dfca03a7b/contract/src/main.rs#L1991).
- `collection_name`: The collection name specified when the contract was [installed](https://github.com/casper-ecosystem/cep-78-enhanced-nft#installing-the-contract) using the `collection_name` option. See the [contract code](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/408db77c3b9ca22752c7f877ea99a01dfca03a7b/contract/src/main.rs#L93) for details. 

Here is the `casper-client` command to upgrade and migrate to version 1.1.1 of the NFT collection specified:

```bash
casper-client put-deploy \
--node-address [NODE_SERVER_ADDRESS] \
--chain-name [CHAIN_NAME] \
--secret-key [KEY_PATH]/secret_key.pem \
--payment-amount [PAYMENT_AMOUNT_IN_MOTES] \
--session-path [PATH]/contract.wasm \
--session-arg "named_key_convention:u8='1'" \
--session-arg "collection_name:string='[COLLECTION_NAME]'"
```

Here is the full list of required arguments:
- `node-address`: An IP address of a peer on the network. The default port for JSON-RPC servers on Mainnet and Testnet is 7777.
- `chain-name`: The chain name of the network where you wish to send the deploy. For Mainnet, use *casper*. For Testnet, use *casper-test*.
- `secret-key`: The file name containing the secret key of the account paying for the deploy.
- `payment-amount`: The payment for the deploy in motes.
- `session-path`: The path to the compiled Wasm on your computer. When using the [cep-78-wasm.tar.gz](https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/download/v1.1.1/cep-78-wasm.tar.gz) provided, this would be the path to the `contract.wasm` file.
- `named_key_convention`: Argument that specifies the use of the `V_1_0_standard` [NamedKeyConvention](../README.md#namedkeyconventionmode).
- `collection_name`: Argument that specifies the collection name as a String.

The command returns the deploy hash that you can use to verify whether or not the deploy succeeded.

**Important Notes**: 

- When upgrading by installing a new version of the CEP-78 contract, you do not need to specify all the runtime arguments needed during the initial installation of version 1.0.0, such as `total_token_supply`, `ownership_mode`, etc.

**Example command to upgrade to v1.1.1:**

The following is an example of upgrading and migrating to version 1.1.1 of a previously installed NFT collection using version 1.0.0 with standard NamedKeys.

```bash
casper-client put-deploy \
--node-addres http://65.21.235.219:7777 \
--chain-name "casper-test" \
--secret-key ~/KEYS/secret_key.pem \
--payment-amount 300000000000 \
--session-path contract.wasm \
--session-arg "named_key_convention:u8='1'" \
--session-arg "collection_name:string='CEP-78-collection'"
```

Here is the corresponding example [deploy](https://testnet.cspr.live/deploy/59a785471600e183718b790fb19b3dec7242fde105928b9f90f01347b3c65f46) on Testnet.

### Standard NamedKeys after Migration

As part of the migration, the contract's NamedKeys will be renamed using a new naming convention, which includes the collection name and a `cep78_contract_` prefix.

| Old NamedKey| New NamedKey | Explanation |
|-------------|--------------|-------------|
| nft_contract | cep78_contract_hash_[COLLECTION_NAME] | The hash identifying the NFT contract |
| nft_contract_package | cep78_contract_package_[COLLECTION_NAME] | The hash identifying the contract package containing the NFT contract | 
| nft_contract_package_access | cep78_contract_package_access_[COLLECTION_NAME] | The URef used as an access token or reference to the contract package | 
| contract_version | cep78_contract_version_[COLLECTION_NAME] | The value tracking the latest contract version in the contract package | 


![Account Named Keys](../assets/standard-namedkeys-post-migration.png)  

Notice that the original NamedKeys are still on the list, but the new contract will not use them. Also, starting with version 1.1.0, all named keys will follow the new naming convention containing the collection name. 

## Minting after Migration

After the upgrade and migration, you can use the `mint_call.wasm` to mint additional NFTs. You will need the contract hash of the upgraded contract, stored under the `cep78_contract_hash_CEP-78-collection` NamedKey.

```bash
casper-client put-deploy \
--node-address [NODE_SERVER_ADDRESS] \
--chain-name [CHAIN_NAME] \
--secret-key [KEY_PATH]/secret_key.pem \
--payment-amount [PAYMENT_AMOUNT_IN_MOTES] \
--session-path [PATH]/mint_call.wasm \
--session-arg "nft_contract_hash:key='[CONTRACT_HASH_HEX_STRING]'" \
--session-arg "collection_name:string='[COLLECTION_NAME]'" \
--session-arg "token_owner:key='[TOKEN_OWNER]'" \
--session-arg "token_meta_data:string='[TOKEN_METADATA]'"
```

The required arguments are:
- `node-address`: An IP address of a peer on the network. The default port for JSON-RPC servers on Mainnet and Testnet is 7777.
- `chain-name`: The chain name of the network where you wish to send the deploy. For Mainnet, use *casper*. For Testnet, use *casper-test*.
- `secret-key`: The file name containing the secret key of the account paying for the deploy.
- `payment-amount`: The payment for the deploy in motes.
- `session-path`- The path to the compiled Wasm on your computer. When using the [cep-78-wasm.tar.gz](https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/download/v1.1.1/cep-78-wasm.tar.gz) provided, this would be the path to the `mint_call.wasm` file.
- `nft_contract_hash`: The hash of a given Enhanced NFT contract passed in as a Key.
- `collection_name`: The name of the NFT collection to which the minted token belongs.
- `token_owner`: The Key of the owner for the NFT to be minted. Note, this argument is ignored in the Ownership::Minter mode.
- `token_meta_data`: The metadata describing the NFT to be minted, passed in as a String.

The command returns the deploy hash that you can use to verify whether or not the deploy succeeded. For more information, see the [mint_call](https://github.com/casper-ecosystem/cep-78-enhanced-nft/tree/dev/client/mint_session) usage.

**Example command to mint NFTs:**

The following is an example of minting v1.1.1 NFTs using the Rust `casper-client`. 

```bash
casper-client put-deploy \
--node-address http://65.21.235.219:7777 \
--chain-name "casper-test" \
--secret-key ~/KEYS/secret_key.pem \
--payment-amount 40000000000 \
--session-path mint_call.wasm \
--session-arg "nft_contract_hash:key='hash-42aace53c55fb3a386c36a66bebf3900169a402169b7b59fc7ef159dba28f516'" \
--session-arg "collection_name:string='CEP-78-collection'" \
--session-arg "token_owner:key='account-hash-5cb74580bcf97d0a7fa034e60b3d2952e0b170ea5162153b1570e8b1ee4ec3f5'" \
--session-arg "token_meta_data:string='{\"name\": \"NFT V1.1.1\",\"token_uri\": \"https:\/\/www.casperlabs.io\",\"checksum\": \"940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb\"}'"
```

## Retrieving the Balance of NFTs after Minting

The `balance_of_call.wasm` is available to retrieve and save the number of NFTs owned by either an Account or Contract to the NamedKeys of the Account executing the Wasm.

```bash
casper-client put-deploy \
--node-address [NODE_SERVER_ADDRESS] \
--chain-name [CHAIN_NAME] \
--secret-key [KEY_PATH]/secret_key.pem \
--payment-amount [PAYMENT_AMOUNT_IN_MOTES] \
--session-path [PATH]/balance_of_call.wasm \
--session-arg "nft_contract_hash:key='[CONTRACT_HASH_HEX_STRING]'" \
--session-arg "token_owner:key='[TOKEN_OWNER]'" \
--session-arg "key_name:string='[KEY_NAME]'"
```

The required arguments are:
- `node-address`: An IP address of a peer on the network. The default port for JSON-RPC servers on Mainnet and Testnet is 7777.
- `chain-name`: The chain name of the network where you wish to send the deploy. For Mainnet, use *casper*. For Testnet, use *casper-test*.
- `secret-key`: The file name containing the secret key of the account paying for the deploy.
- `payment-amount`: The payment for the deploy in motes.
- `session-path`- The path to the compiled Wasm on your computer. When using the [cep-78-wasm.tar.gz](https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/download/v1.1.1/cep-78-wasm.tar.gz) provided, this would be the path to the `balance_of_call.wasm` file.
- `nft_contract_hash`: The hash of a given Enhanced NFT contract passed in as a Key.
- `token_owner`: The Key of either the Account or Contract whose balance is being queried.
- `key_name`: The NamedKey under which the token amount will be stored, passed in as a String.

The command returns the deploy hash that you can use to verify whether or not the deploy succeeded. For more information, see the [balance_of_call](https://github.com/casper-ecosystem/cep-78-enhanced-nft/tree/dev/client/balance_of_session) usage.

**Example command to save the number of NFTs:**

The following is an example that saves the number of NFTs using the Rust `casper-client`.

```bash
casper-client put-deploy \
--node-address http://65.21.235.219:7777 \
--chain-name "casper-test" \
--payment-amount 2000000000 \
--secret-key ~/KEYS/secret_key.pem \
--session-path balance_of_call.wasm \
--session-arg "nft_contract_hash:key='hash-42aace53c55fb3a386c36a66bebf3900169a402169b7b59fc7ef159dba28f516'" \
--session-arg "token_owner:key='account-hash-5cb74580bcf97d0a7fa034e60b3d2952e0b170ea5162153b1570e8b1ee4ec3f5'" \
--session-arg "key_name:string='balance'"
```


