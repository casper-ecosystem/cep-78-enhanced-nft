# NFT Transfers

This document describes how to transfer NFTs on a Casper network using the Casper client.

## Prerequisites

- Install the contract using the [Quickstart](./quickstart-guide.md) or the [Full Installation](./full-installation-tutorial.md) tutorials
- Learn to [Query NFT Contracts](./query.md) and save the various hashes and URefs required throughout this document

## Transferring NFTs to Another Account

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
<summary><b>Sample command without comments</b></summary>

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
<summary><b>Sample command without comments</b></summary>

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
<summary><b>Sample command without comments</b></summary>

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
<summary><b>Sample command without comments</b></summary>

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

## Minting Tokens

If the contract allows minting, the following command will mint a number of CEP-78 tokens to the account provided. This increases the total supply of the token in question.

```bash
casper-client put-deploy -n http://<HOST:PORT> \
--chain-name [CHAIN_NAME] \
--secret-key [PATH_TO_SECRET_KEY] \
--session-package-name "cep78_contract_package_CEP78" \
--session-entry-point "mint" \
// This account will receive the newly minted CEP-78 tokens.
--session-arg "owner:key='account-hash-[HASH]'" \
// The number of additional CEP-78 tokens to add to the total supply.
--session-arg "amount:U256='10'" \
--payment-amount 1000000000
```

<details>
<summary><b>Sample command without comments</b></summary>

```bash
casper-client put-deploy -n https://rpc.testnet.casperlabs.io/ \
--chain-name "casper-test" \
--secret-key ~/KEYS/secret_key.pem \
--session-package-name "cep78_contract_package_CEP78" \
--session-entry-point "mint" \
--session-arg "owner:key='account-hash-683f53f56926f54ef9584b07585b025c68415dc05f7b2e56749153574b83d5cd'" \
--session-arg "amount:U256='10'" \
--payment-amount 1000000000
```

</details>

## Burning Tokens

If the contract allows burning, the following command will burn a number of CEP-78 tokens from the account provided. This decreases the total supply of the token in question.

```bash
casper-client put-deploy -n http://<HOST:PORT> \
--chain-name [CHAIN_NAME] \
--secret-key [PATH_TO_SECRET_KEY] \
--session-package-name "cep78_contract_package_CEP78" \
--session-entry-point "burn" \
// The account from which the tokens will be burned.
--session-arg "owner:key='account-hash-[HASH]'" \
// The number of CEP-78 tokens to remove from the total supply.
--session-arg "amount:U256='10'" \
--payment-amount 1000000000
```

<details>
<summary><b>Sample command without comments</b></summary>

```bash
casper-client put-deploy -n https://rpc.testnet.casperlabs.io/ \
--chain-name "casper-test" \
--secret-key ~/KEYS/secret_key.pem \
--session-package-name "cep78_contract_package_CEP78" \
--session-entry-point "burn" \
--session-arg "owner:key='account-hash-683f53f56926f54ef9584b07585b025c68415dc05f7b2e56749153574b83d5cd'" \
--session-arg "amount:U256='10'" \
--payment-amount 1000000000
```

</details>

### Next Steps

- [Testing Framework for CEP-78](./tests.md)