# Token Ownership in Casper NFT Contracts (Release v1.1.1)

This tutorial demonstrates how to check token ownership in CEP-78 NFT contracts, starting with version [v1.1.1](https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/tag/v1.1.1). For this tutorial, the `OwnerReverseLookupMode` modality must be set to `Complete` as described [here](../README.md#ownerreverselookupmode).

As someone interacting with an NFT contract, you might want to answer the following questions:

1. Which NFTs do I own?
2. Which NFTs does someone else own?

You might be an account user or owner of a contract that interacts with the NFT contract.

The first method to answer these questions is an [account-centric approach](#querying-the-account), in which you trust the account owner and the information stored in the account's NamedKeys. This account could be an account you own or someone else owns. This method is less secure and needs to be based on trust. To apply this method, proceed according to the following steps:

- Look for NamedKeys in this format: "cep78_*_m_1000_p_#".
- Query each "cep78_*_m_1000_p_#" dictionary using the `casper-client get-dictionary-item` and the `dictionary-address`.

The second method is a [contract-centric approach](#querying-the-contract), in which you query the NFT contract. This method is more secure than the first approach and can be used when you need to verify or cannot trust an account's NamedKeys. To apply this method, follow these steps:

- Query the "page_table" dictionary from the CEP-78 contract using its seed URef and the account hash (without the "account-hash-" prefix).
- Then, query each page dictionary given its seed URef and the account hash (again, without the "account-hash-" prefix).

> **Note: Please choose the most secure method that serves your use case based on the level of trust you require.**

![Methods of Querying Token Ownership](../assets/methods-of-querying.png)

The tutorial presents sample accounts, contracts, and NamedKeys to explain, by example, the two methods of querying token ownership.

## Prerequisites

- You have installed or upgraded to a CEP-78 contract that uses release v1.1.1, and the `OwnerReverseLookupMode` modality is set to `Complete` as described [here](../README.md#ownerreverselookupmode).
- The contract has minted one or more tokens, and you have access to the account or the contract that owns these tokens.
- You have experience with the [Casper CEP-78 NFT Standard](https://github.com/casper-ecosystem/cep-78-enhanced-nft/) and the Casper command-line client and know how to interact with a Casper network.
- You understand the [Owner Reverse Lookup Functionality](https://github.com/casper-ecosystem/cep-78-enhanced-nft/#owner-reverse-lookup-functionality) and [CEP-78 Page System](../README.md#the-cep-78-page-system) introduced in [Version 1.1](https://github.com/casper-ecosystem/cep-78-enhanced-nft/#new-in-version-11) of the CEP-78 Enhanced NFT Standard.

## Method 1 - Querying the Account 

In this method of checking token ownership, examine the account or the calling contract's NamedKeys. Look for NamedKeys that use this format: "cep78_*_m_1000_p_#". For more information on this format, read about the [CEP-78 Page System](../README.md#the-cep-78-page-system). This way, you can access the dictionary storing the NFTs directly and retrieve ownership information.

In the following example, the contract minted a small number of NFTs and has the following NamedKey: `cep78_CEP-78-collection_m_1000_p_0`. 

<div align="center">
<img src="../assets/highlighted-collection-named-key.png" alt="Highlighted Collection NamedKey" width="500"/>
</div>

To query the dictionary referenced by this NamedKey and retrieve the tokens that this account owns, use the `casper-client` and the `get-dictionary-item` option. Given the dictionary's address and the latest state root hash, the command would look like this:

**Sample query into the "cep78_*_m_1000_p_0" dictionary:**

```bash
casper-client get-dictionary-item  \
--node-address http://65.21.235.219:7777  \
--state-root-hash a77af17080112066caeb73d8133752584ddd11407f1fae94be0849a8abe1d1f9 \
--dictionary-address dictionary-eb837c4c92199e66619e163271a7e487704b5be7b103e785ed5b262f36ab6f50
```

Here is the sample output that typically contains a list of 1,000 boolean values. In this example, most rows were omitted because only the first two values were "true".

**Sample response from the "cep78_*_m_1000_p_0" dictionary:**

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

To interpret the output, you need to know how the token identifier mode was set at the time of contract installation. 

### Tokens Identified by Token ID

If the token identifier mode was set to "Ordinal", the token number is the token ID. In this case, the output above tells us that this account owns the first two tokens in the list. Also, the NamedKey `cep78_CEP-78-collection_m_1000_p_0` indicates that the tokens owned are on "page_0" from the "page_table" dictionary. By doing the math explained [here](../README.md#the-cep-78-page-system) and considering that the token number is the token ID, this account owns tokens 0 and 1.

> **Note**: What if the named key was `cep78_CEP-78-collection_m_1000_p_11` for the same sample output above? In that case, the account would own tokens on page 11, at index 0 and 1, which would be tokens 11,000 and 11,001.

### Tokens Identified by Hash

Suppose the token identifier mode was set to "Hash" when the NFT contract was installed. You need to query the "hash_by_index" dictionary and map all the token numbers to their corresponding hash values. 

<div align="center">
<img src="../assets/hash-by-index-dictionary.png" alt="Hash by Index Dictionary" width="500"/>
</div>

Specify the "hash_by_index" dictionary URef and the index value to retrieve the corresponding token hash in this example query.

**Sample query into the "hash_by_index" dictionary:**

This query retrieves the token's hash given the "hash_by_index" dictionary and index 0.

```bash
casper-client get-dictionary-item \
--node-address http://65.21.235.219:7777 \
--state-root-hash 5aa0cda415ef9f422e51e65b1fe69c60e6cac8bc370fd6ff416fe2101ae3242a \
--seed-uref "uref-dd3041bf2fc5f7ec3937b97b0dd51d01634437680accfe1c3f9b921753afe2f4-007" \
--dictionary-item-key "0"
```

**Sample response from the "hash_by_index" dictionary:**

The sample response shows that the hash of the NFT token at index 0 is "2b66bf103522470b75a4dae645b03db974cdf0061c4ca7b9e5b812e85d7a7737". In other words, the account whose NamedKeys you are consulting owns the token with hash "2b66bf103522470b75a4dae645b03db974cdf0061c4ca7b9e5b812e85d7a7737".

```json
{
  "id": -5671053351359407927,
  "jsonrpc": "2.0",
  "result": {
    "api_version": "1.4.10",
    "dictionary_key": "dictionary-472606f90b87a81395a4e71662033ff3148b570d59f0a46f0954aad4f2ea608f",
    "merkle_proof": "[37226 hex chars]",
    "stored_value": {
      "CLValue": {
        "bytes": "4000000032623636626631303335323234373062373561346461653634356230336462393734636466303036316334636137623965356238313265383564376137373337",
        "cl_type": "String",
        "parsed": "2b66bf103522470b75a4dae645b03db974cdf0061c4ca7b9e5b812e85d7a7737"
      }
    }
  }
}
```

## Method 2 - Querying the Contract

The second way to check token ownership is to examine the NFT contract. To proceed, start with the contract hash.

<div align="center">
<img src="../assets/the-nft-contract-hash.png" alt="Accessing the NFT Contract Hash" width="500"/>
</div>

The NFT contract should have a "page_table" NamedKey, which is the seed URef for the "page_table" dictionary. Use this dictionary and the account hash (without the "account-hash-" prefix) to find the pages tracking tokens owned by the account specified. 

<div align="center">
<img src="../assets/page-table-dictionary.png" alt="The page_table Dictionary" width="500"/>
</div>

**Sample query into the "page_table" dictionary:**

```bash
casper-client get-dictionary-item \
--node-address http://65.21.235.219:7777 \
--state-root-hash a77af17080112066caeb73d8133752584ddd11407f1fae94be0849a8abe1d1f9 \
--seed-uref "uref-38b21f84ce85d22ee8cfba27744c44d6b393edfb3f56a4474897cb3969039324-007" \
--dictionary-item-key "e861226c153eefc0ca48bf29c76bc305235151aebde76257bf9bbacb4fa041f7"
```

**Sample response from the "page_table" dictionary:**

```json
{
  "id": 3795029107185218759,
  "jsonrpc": "2.0",
  "result": {
    "api_version": "1.4.10",
    "dictionary_key": "dictionary-615dfe771cda776c65c233b90f5c5752774124e6f498ba32b28ab9e853ee203f",
    "merkle_proof": "[37500 hex chars]",
    "stored_value": {
      "CLValue": {
        "bytes": "0100000001",
        "cl_type": {
          "List": "Bool"
        },
        "parsed": [
          true
        ]
      }
    }
  }
}
```

The sample response includes only one "parsed" value equal to "true", meaning that one page was allocated at index 0 to track tokens owned by the account specified. In other words, the account with hash "e861226c153eefc0ca48bf29c76bc305235151aebde76257bf9bbacb4fa041f7" owns tokens tracked in the "page_0" dictionary. To understand the page structure further, review how the contract manages storage and token ownership [here](../README.md#the-cep-78-page-system).

Since the NFT contract allocated the "page_0" dictionary to track tokens for this account, expect to see a NamedKey called "page_0". Using the URef of the "page_0" dictionary and the account hash, retrieve the token IDs that the account owns.

<div align="center">
<img src="../assets/page-0-dictionary.png" alt="The page_0 Dictionary" width="500"/>
</div>

**Sample query into the "page_0" dictionary:**

```bash
casper-client get-dictionary-item \
--node-address http://65.21.235.219:7777 \
--state-root-hash a77af17080112066caeb73d8133752584ddd11407f1fae94be0849a8abe1d1f9 \
--seed-uref "uref-f0079428fd7cd358c981d3eb1dd751f7cf744bb850d67e0506c76fd69f0bb3bc-007" \
--dictionary-item-key "e861226c153eefc0ca48bf29c76bc305235151aebde76257bf9bbacb4fa041f7" 
```

**Sample response from the "page_0" dictionary:**

```json
{
  "id": -769907090820463941,
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

Notice that the output is the same as what was displayed when using the first method; the dictionary address is "dictionary-eb837c4c92199e66619e163271a7e487704b5be7b103e785ed5b262f36ab6f50". Therefore, to understand the output, follow the [Tokens Identified by Token ID](tokens-identified-by-token-id) and [Tokens Identified by Hash](tokens-identified-by-hash) sections.

## Frequently Asked Questions

### What if the value parsed from the "page_table" dictionary would look like this?

```bash
        "parsed": [
          false,
          true
```

In that case, the account specified in the request would own tokens in the "page_1" dictionary because the value at index 1 is "true", and the value at index 0 is "false".

### What is the "page_limit" NamedKey?

The "page_limit" NamedKey saves the maximum number of pages allocated for tracking tokens minted by an NFT contract. If the page_limit is 1, the contract has set the maximum token supply to be less than or equal to 1,000 tokens.

<div align="center">
<img src="../assets/page-limit-dictionary.png" alt="The page_limit Dictionary" width="500"/>
</div>

### How would the "page_*" NamedKeys look for a larger collection?

If an NFT collection has a total token supply of 10,000 NFTs, the NamedKeys for the NFT contract would be split into 10 pages. 

<div align="center">
<img src="../assets/larger-collection-pages.png" alt="Larger Collection Pages" width="500"/>
</div>

Also, the page_limit value would be set to "10".

<div align="center">
<img src="../assets/larger-collection-limit.png" alt="Larger Collection Limit" width="500"/>
</div>

### When is the "balance" dictionary useful?

The "balance" dictionary tracks how many tokens an account owns, not knowing which tokens it owns.

### When is the "token_owners" dictionary useful?

The "token_owners" dictionary maps token IDs to token owners, but you cannot use this dictionary to answer which tokens a specific account owns. So if you want to know who owns a specific token, use the "token_owners" dictionary.


## Conclusions

To answer the question "which NFTs does this account own" you need to consult the "page_table" dictionary and find which pages are allocated to the account in question. Each page has another page-specific dictionary containing token ownership information in the format "page_#" (for example, "page_0").

If you trust an account and want to see which NFT it owns, look at its NamedKeys. See how many NamedKeys the account has for a given collection (in the format "m_1000_p_#"). These NamedKeys point to the "page_#" dictionaries that contain token ownership information. Using the state root hash and the dictionary address for each page (in the format "dictionary-...", you can retrieve token ownership details.

Suppose you want additional security or cannot trust the account's NamedKeys. In that case, you need to query the NFT contract using the "page_table" dictionary to determine which pages are allocated to the account. Then, you can use the seed URef of the page-specific dictionaries and the account hash (without the "account-hash-" prefix) to retrieve the tokens assigned to the account specified.

