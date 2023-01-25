# Token Ownership in Casper NFT Contracts (Release v1.1.1)

This tutorial demonstrates how to check token ownership in CEP-78 NFT contracts, starting with version [v1.1.1](https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/tag/v1.1.1). As an account user or owner of a contract interacting with the NFT contract, I want to be able to answer the following questions:

1. Which NFTs do I own?
2. Which NFTs does someone else own?

The first method to answer these questions is an [account-centric approach](#querying-the-account), in which we trust the account owner and the information stored in the account's NamedKeys. This account could be an account we own, or someone else owns. This method is less secure and needs to be based on trust.

The second method is a [contract-centric approach](#querying-the-contract), in which we interrogate the NFT contract directly to retrieve token ownership information. This method is more secure than the first approach and can be used when we need to verify or cannot trust an account's NamedKeys. 

> **Note**: Please choose the most secure method that serves your use case based on the level of trust required.

## Prerequisites

- You have installed or upgraded to a CEP-78 contract that uses release v1.1.1. 
- The contract has minted one or more tokens, and you have access to the account or the contract that owns these tokens.
- You have experience with the [Casper CEP-78 NFT Standard](https://github.com/casper-ecosystem/cep-78-enhanced-nft/) and the Casper command-line client and know how to interact with a Casper network.
- You understand the [The CEP-78 Page System](../README.md#the-cep-78-page-system) introduced in [Version 1.1](https://github.com/casper-ecosystem/cep-78-enhanced-nft/#new-in-version-11) of the CEP-78 Enhanced NFT Standard.

## Querying the Account

To use this approach, we first examine the account's NamedKeys or the calling contract's NamedKeys. We look for NamedKeys that use this format: `cep78_[COLLECTION_NAME]_m_1000_p_X. For more information on this format, read about the [CEP-78 Page System](../README.md#the-cep-78-page-system). If we have such a key in the account, then we can access the dictionary storing the NFTs and retrieve ownership information.

Let's consider an example where the contract minted a small number of NFTs and has the following NamedKey: `cep78_CEP-78-collection_m_1000_p_0`. We can query the dictionary stored under this NamedKey to retrieve the tokens that this account owns.

![Highlighted Collection NamedKey](../assets/highlighted-collection-named-key.png)

We will use the `casper-client` and its `get-dictionary-item` option to query this dictionary. Given the dictionary's address and the latest state root hash, the command would look like this:

```bash
casper-client get-dictionary-item  \
   --node-address http://65.21.235.219:7777  \
   --state-root-hash a77af17080112066caeb73d8133752584ddd11407f1fae94be0849a8abe1d1f9 \
   --dictionary-address dictionary-eb837c4c92199e66619e163271a7e487704b5be7b103e785ed5b262f36ab6f50
```

Here is some sample output that we will discuss in more detail below. The list would have 1,000 rows, but we have omitted most rows, because only the first two values in this example are set to "true".

```json
{
  "id": -5901481722405843563,
  "jsonrpc": "2.0",
  "result": {
    "api_version": "1.4.10",
    "dictionary_key": "dictionary-eb837c4c92199e66619e163271a7e487704b5be7b103e785ed5b262f36ab6f50",
    "merkle_proof": "[39498 hex chars]",
    "stored_value": {
      "CLValue": {
        "bytes": "[2008 hex chars]",
        "cl_type": {
          "List": "Bool"
        },
        "parsed": [
          true,
          true,
          false,
          false,
          false,
           ...
          false,
          false
        ]
      }
    }
  }
}
```

To interpret the output, we need to know how the token identifier mode was set at the time of contract installation. 

### Tokens Identified by Token ID

If the token identifier mode was set to "Ordinal", the token number is the token ID. In this case, the output above tells us that this account owns the first two tokens in the list. Also, the NamedKey `cep78_CEP-78-collection_m_1000_p_0` tells us that we are on page 0. By doing the math explained [here](../README.md#the-cep-78-page-system) and considering that the token number is the token ID, we conclude that the account owns tokens number 0 and 1.

#### Another example

What if the named key was `cep78_CEP-78-collection_m_1000_p_11` for the same sample output? In that case, the account would own tokens on page 11, at index 0 and 1, which would be tokens 11,000 and 11,001.

### Tokens Identified by Hash

If the token identifier mode was set to "Hash", we need to do some additional work and query the "index by hash" dictionary, mapping all the token numbers to each token's hash. 

In this example query, we need to specify the dictionary name (index by hash) and index values 0 and 1 to retrieve the corresponding token hashes.

<!-- add a screenshot, example query, and example output -->

## Querying the Contract

