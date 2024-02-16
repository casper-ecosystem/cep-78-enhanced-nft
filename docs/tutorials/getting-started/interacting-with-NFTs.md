# Interacting with the NFT Contract using the Rust Casper Client

This document describes how to transfer NFTs on a Casper network using the Casper client.

## Prerequisites

- Install the contract using the [Quickstart](./quickstart-guide.md) or the [Full Installation](./full-installation-tutorial.md) tutorials
- Learn to [Query NFT Contracts](./querying-NFTs.md) and save the various hashes and URefs required throughout this document

## Table of Contents

1. [Directly Invoking Entrypoints](#directly-invoking-entrypoints)

2. [Transferring NFTs](#transferring-nfts)

3. [Approving Another Account](#approving-another-account)

4. [Minting NFTs](#minting-nfts)

5. [Burning NFTs](#burning-nfts)


## Directly Invoking Entrypoints

With the release of CEP-78 version 1.1, users that are interacting with a CEP-78 contract that does not use `ReverseLookupMode` should opt out of using the client Wasm files provided as part of the release. Opting out in this situation is recommended, as directly invoking the entrypoints incurs a lower gas cost compared against using the provided client Wasm to invoke the entrypoint.

You may invoke the `mint`, `transfer` or `burn` entrypoints directly through either the contract package hash or the contract hash directly.

Specifically in the case of `mint`, there are fewer runtime arguments that must be provided, thereby reducing the total gas cost of minting an NFT.

<details>
<summary><b>Example Mint using StoredVersionByHash</b></summary>

```bash

casper-client put-deploy -n https://rpc.testnet.casperlabs.io/ --chain-name "casper-test" \ --payment-amount 7500000000 \ -k ~/secret_key.pem \
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

casper-client put-deploy -n https://rpc.testnet.casperlabs.io/ --chain-name "casper-test" \ --payment-amount 7500000000 \ -k ~/secret_key.pem \
--session-hash hash-b3b7a74ae9ef2ea8afc06d6a0830961259605e417e95a53c0cb1ca9737bb0ec7 \
--session-entry-point "transfer" \
--session-arg "source_key:key='account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb'" \
--session-arg "target_key:key='account-hash-b4782e7c47e4deca5bd90b7adb2d6e884f2d331825d5419d6cbfb59e17642aab'" \
--session-arg "token_id:u64='0'"

```

</details>

## Transferring NFTs

The following command invokes the `transfer` entrypoint on your instance of CEP-78, directing it to transfer 10 of the associated NFT tokens to another account.

```bash
casper-client put-deploy -n http://<HOST:PORT> \
// The chain name of the Casper network on which your CEP-78 instance was installed.
--chain-name [CHAIN_NAME] \
// The local path to your account's secret key.
--secret-key [PATH_TO_SECRET_KEY] \
// The contract hash of your CEP-78 contract instance.
--session-hash [CEP-78_CONTRACT_HASH] \
// The name of the entry point you are invoking.
--session-entry-point "transfer" \
// The account hash of the account to which you are sending CEP-78 tokens.
--session-arg "recipient:key='account-hash-[HASH]" \
// The amount of CEP-78 tokens you are sending to the receiving account.
--session-arg "amount:u256='10'" \
// The gas payment you are allotting, in motes.
--payment-amount "10000000000"
```

<details>
<summary><b>Casper client command without comments</b></summary>

```bash
casper-client put-deploy -n https://rpc.testnet.casperlabs.io/ \
--chain-name "casper-test" \
--secret-key ~/KEYS/secret_key.pem \
--session-hash hash-b568f50a64acc8bbe43462ffe243849a88111060b228dacb8f08d42e26985180 \
--session-entry-point "transfer" \
--session-arg "recipient:key='account-hash-9f81014b9c7406c531ebf0477132283f4eb59143d7903a2fae54358b26cea44b" \
--session-arg "amount:u256='50'" \
--payment-amount "10000000000"
```

</details>

This command will return a deploy hash that you can query using `casper-client get-deploy`. Querying the deploy allows you to verify execution success, but you will need to use the `balance_of` entry point to verify the account's balance.

### Transferring NFTs using Wasm

Below is an example of a `casper-client` command that uses the `transfer` Wasm to transfer ownership of an NFT from one user to another.

- `casper-client put-deploy -n https://rpc.testnet.casperlabs.io/ --chain-name "casper-test" --payment-amount 5000000000 -k ~/secret_key.pem --session-path ~/casper/enhanced-nft/client/transfer_session/target/wasm32-unknown-unknown/release/transfer_call.wasm`

1. `--session-arg "nft_contract_hash:key='hash-52e78ae3f6c485d036a74f65ebbb8c75fcc7c33fb42eb667fb32aeba72c63fb5'"`

   The contract hash of the CEP-78 NFT Contract associated with the NFT to be transferred.

2. `--session-arg "source_key:key='account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb'"`

   The account hash of the user that currently owns the NFT and wishes to transfer it.

3. `--session-arg "target_key:key='account-hash-b4772e7c47e4deca5bd90b7adb2d6e884f2d331825d5419d6cbfb59e17642aab'"`

   The account hash of the user that will receive the NFT.

4. `--session-arg "is_hash_identifier_mode:bool='false'"`

   Argument that the hash identifier mode is ordinal, thereby requiring a `token_id` rather than a `token_hash`.

5. `--session-arg "token_id:u64='0'"`

   The `token_id` of the NFT to be transferred.

<details>
<summary><b>Casper client command without comments</b></summary>

```bash
casper-client put-deploy -n https://rpc.testnet.casperlabs.io/ --chain-name "casper-test" \
--payment-amount 5000000000 \
-k ~/secret_key.pem \
--session-path ~/casper/enhanced-nft/client/transfer_session/target/wasm32-unknown-unknown/release/transfer_call.wasm \
--session-arg "nft_contract_hash:key='hash-52e78ae3f6c485d036a74f65ebbb8c75fcc7c33fb42eb667fb32aeba72c63fb5'" \
--session-arg "source_key:key='account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb'" \
--session-arg "target_key:key='account-hash-b4772e7c47e4deca5bd90b7adb2d6e884f2d331825d5419d6cbfb59e17642aab'" \
--session-arg "is_hash_identifier_mode:bool='false'" \
--session-arg "token_id:u64='0'"
```

</details>

### Invoking the `balance_of` Entry Point

The following Casper client command invokes the `balance_of` entry point on the `cep78_test_contract`.

```bash
casper-client put-deploy -n http://<HOST:PORT> \
--chain-name [CHAIN_NAME] \
--secret-key [PATH_TO_SECRET_KEY] \
--session-package-name "cep78_test_contract" \
--session-entry-point "balance_of" \
// The contract hash of your CEP-78 contract instance, passed in as an `account-hash-`.
--session-arg "token_contract:account_hash='account-hash-[HASH]'" \
// The account hash of the account whose balance you are checking.
--session-arg "address:key='account-hash-[HASH]'" \
--payment-amount 1000000000
```

<details>
<summary><b>Casper client command without comments</b></summary>

```bash
casper-client put-deploy -n https://rpc.testnet.casperlabs.io/ \
--chain-name "casper-test" \
--secret-key ~/KEYS/secret_key.pem \
--session-package-name "cep78_test_contract" \
--session-entry-point "balance_of" \
--session-arg "token_contract:account_hash='account-hash-b568f50a64acc8bbe43462ffe243849a88111060b228dacb8f08d42e26985180'" \
--session-arg "address:key='account-hash-303c0f8208220fe9a4de40e1ada1d35fdd6c678877908f01fddb2a56502d67fd'" \
--payment-amount 1000000000
```

</details>

## Approving Another Account

The Casper NFT contract features an `approve` entry point that allows an account to delegate another account to spend a preset number of CEP-78 tokens from their balance.

The following command approves another account to spend 15 tokens from the balance of the account that installed and owns the CEP-78 contract instance.

```bash
casper-client put-deploy -n http://<HOST:PORT> \
--chain-name [CHAIN_NAME] \
--secret-key [PATH_TO_SECRET_KEY] \
// The contract hash of the CEP-78 token contract.
--session-hash [CEP-78_CONTRACT_HASH] \
--session-entry-point "approve" \
// The account hash of the account that will receive an allowance from the balance of the account that sent the Deploy.
--session-arg "spender:key='account-hash-[HASH]'" \
// The number of CEP-78 tokens included in the allowance.
--session-arg "amount:u256='15'" \
--payment-amount "10000000000"
```

<details>
<summary><b>Casper client command without comments</b></summary>

```bash
casper-client put-deploy -n https://rpc.testnet.casperlabs.io/ \
--chain-name "casper-test" \
--secret-key ~/KEYS/secret_key.pem \
--session-hash hash-05d893e76c731729fc26339e5a970bd79fbf4a6adf743c8385431fb494bff45e \
--session-entry-point "approve" \
--session-arg "spender:key='account-hash-17192017d32db5dc9f598bf8ac6ac35ee4b64748669b00572d88335941479513'" \
--session-arg "amount:u256='15'" \
--payment-amount "10000000000"
```

</details>

### Transferring Tokens from an Approved Account

The following command allows an account to transfer CEP-78 tokens held by another account up to their approved allowance.

```bash
casper-client put-deploy -n http://<HOST:PORT> \
--chain-name [CHAIN_NAME] \
// The secret key for the account that is spending their allowance from another account's balance.
--secret-key [PATH_TO_SECRET_KEY] \
// The CEP-78 token contract.
--session-hash [CEP-78_CONTRACT_HASH] \
--session-entry-point "transfer" \
// The account hash of the account that holds the CEP-78 in their balance.
--session-arg "owner:key='account-hash-[HASH]'" \
// The account hash of the account that will receive the transferred CEP-78 tokens.
--session-arg "recipient:key='account-hash-[HASH]'" \
// The amount of tokens to be transferred. If this amount exceeds the allowance of the account sending the deploy, it will fail.
--session-arg "amount:u256='10'" \
--payment-amount "10000000000"
```

<details>
<summary><b>Casper client command without comments</b></summary>

```bash
casper-client put-deploy -n https://rpc.testnet.casperlabs.io/\
--chain-name "casper-test" \
--secret-key ~/KEYS/secret_key.pem \
--session-hash hash-05d893e76c731729fc26339e5a970bd79fbf4a6adf743c8385431fb494bff45e \
--session-entry-point "transfer" \
--session-arg "owner:key='account-hash-39f15c23df9be1244572bb499fac62cbcad3cab2dc1438609842f602f943d7d2'" \
--session-arg "recipient:key='account-hash-17192017d32db5dc9f598bf8ac6ac35ee4b64748669b00572d88335941479513'" \
--session-arg "amount:u256='10'" \
--payment-amount "10000000000"
```
</details>

## Minting NFTs

Below is an example of a `casper-client` command that uses the `mint` function of the contract to mint an NFT for the user associated with `node-1` in an [NCTL environment](https://docs.casper.network/developers/dapps/nctl-test/).

- `casper-client put-deploy -n https://rpc.testnet.casperlabs.io/ --chain-name "casper-test" --payment-amount 5000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem --session-path ~/casper/enhanced-nft/client/mint_session/target/wasm32-unknown-unknown/release/mint_call.wasm`

1. `--session-arg "nft_contract_hash:key='hash-206339c3deb8e6146974125bb271eb510795be6f250c21b1bd4b698956669f95'"`

   The contract hash of the previously installed CEP-78 NFT contract from which we will be minting.

2. `--session-arg "collection_name:string='cep78_<collection_name>'"`

   The collection name of the previously installed CEP-78 NFT contract from which we will be minting.

3. `--session-arg "token_owner:key='account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb'"`

   The collection name of the NFT to be minted.

4. `--session-arg "token_meta_data:string='{\"name\": \"John Doe\",\"token_uri\": \"https:\/\/www.barfoo.com\",\"checksum\": \"940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb\"}'"`

   Metadata describing the NFT to be minted, passed in as a `string`.


<details>
<summary><b>Casper client command without comments</b></summary>

```bash
casper-client put-deploy -n https://rpc.testnet.casperlabs.io/ --chain-name "casper-test" \
--payment-amount 5000000000 \
-k ~/KEYS/secret_key.pem \
--session-entry-point "mint" \
--session-arg "nft_contract_hash:key='hash-206339c3deb8e6146974125bb271eb510795be6f250c21b1bd4b698956669f95'" \
--session-arg "collection_name:string='CEP-78-collection'"` \
--session-arg "token_owner:key='account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb'"  \
--session-arg "token_meta_data:string='{\"name\": \"John Doe\",\"token_uri\": \"https:\/\/www.barfoo.com\",\"checksum\": \"940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb\"}'"
```

</details>


## Burning NFTs

Below is an example of a `casper-client` command that uses the `burn` entrypoint to burn an NFT within a CEP-78 collection. If this command is used, the NFT in question will no longer be accessible by anyone.

- `casper-client put-deploy -n https://rpc.testnet.casperlabs.io/ --chain-name "casper-test" --payment-amount 5000000000 -k ~/KEYS/secret_key.pem \`

1. `--session-hash hash-52e78ae3f6c485d036a74f65ebbb8c75fcc7c33fb42eb667fb32aeba72c63fb5`

   The session hash corresponding to the NFT's contract hash.

2. `--session-entry-point "burn"`

   The entrypoint corresponding to the `burn` function.

3. `--session-arg "token_id:u64='1'"`

   The token ID for the NFT to be burned. If the `identifier_mode` is not set to `Ordinal`, you must provide the `token_hash` instead.

<details>
<summary><b>Casper client command without comments</b></summary>

```bash
casper-client put-deploy -n https://rpc.testnet.casperlabs.io/ --chain-name "casper-test" \
--payment-amount 5000000000 \
-k ~/KEYS/secret_key.pem \
--session-hash hash-52e78ae3f6c485d036a74f65ebbb8c75fcc7c33fb42eb667fb32aeba72c63fb5 \
--session-entry-point "burn" \
--session-arg "token_id:u64='1'"
```

</details>

### Next Steps

- [Testing Framework for CEP-78](./testing-NFTs.md)