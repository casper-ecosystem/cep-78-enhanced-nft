# Installing and Interacting with the CEP-78: Enhanced NFT Standard Contract using the Casper Client

This contract code installs an instance of the CEP-78 enhanced NFT standard as per session arguments provided at the time of installation.

## Installing the Contract using Casper Client

Installing the enhanced NFT contract to global state requires the use of a [Deploy](https://docs.casperlabs.io/dapp-dev-guide/building-dapps/sending-deploys/). In this case, the session code can be compiled to Wasm by running the `make build-contract` command provided in the Makefile at the top level. The Wasm will be found in the `contract/target/wasm32-unknown-unknown/release` directory as `contract.wasm`.

Below is an example of a `casper-client` command that provides all required session arguments to install a valid instance of the CEP-78 contract on global state.

* `casper-client --put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem --session-path ~/casper/enhanced-nft/contract/target/wasm32-unknown-unknown/release/contract.wasm`

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

7) `--session-args-json '[{"name":"token_meta_data","type":"String","value":"{\"name\": \"John Doe\",\"token_uri\": \"https:\/\/www.barfoo.com\",\"checksum\": \"940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb\"}"},{"name":"token_owner", "type":"Key","value":"account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb"},{"name":"nft_contract_hash","type":"Key","value":"hash-206339c3deb8e6146974125bb271eb510795be6f250c21b1bd4b698956669f95"}]'`

    Metadata information passed through, in this instance conforming to the CEP-78 standard as shown in the 'NFTMetadataKind' argument above.

8) `--session-arg "identifier_mode:u8='0'"`

    The mode used to identify individual NFTs. For 0, this means an ordinal identification sequence rather than by hash.

9) `--session-arg "metadata_mutability:u8='0'"`

    A setting allowing for mutability of metadata. This is only available when using the ordinal identification mode, as the hash mode depends on immutability for identification. In this instance, despite ordinal identification, the 0 represents immutable metadata.


The session arguments match the available modalities as listed in the main [README](https://github.com/casper-ecosystem/cep-78-enhanced-nft).

<details>
<summary><b>Casper client command without comments</b></summary>

```bash

casper-client --put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem --session-path ~/casper/enhanced-nft/contract/target/wasm32-unknown-unknown/release/contract.wasm 
--session-arg "collection_name:string='CEP-78-collection'" 
--session-arg "collection_symbol:string='CEP78'" 
--session-arg "total_token_supply:u64='100'" 
--session-arg "ownership_mode:u8='2'" 
--session-arg "nft_kind:u8='1'" 
--session-arg "nft_metadata_kind:u8='0'" 
--session-args-json '[{"name":"token_meta_data","type":"String","value":"{\"name\": \"John Doe\",\"token_uri\": \"https:\/\/www.barfoo.com\",\"checksum\": \"940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb\"}"},{"name":"token_owner", "type":"Key","value":"account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb"},{"name":"nft_contract_hash","type":"Key","value":"hash-206339c3deb8e6146974125bb271eb510795be6f250c21b1bd4b698956669f95"}]' 
--session-arg "identifier_mode:u8='0'" 
--session-arg "metadata_mutability:u8='0'"

```

</details>

## Minting an NFT

Below is an example of a `casper-client` command that uses the `mint` function of hte contract to mint an NFT for the user associated with `node-1` in an [NCTL environment](https://docs.casperlabs.io/dapp-dev-guide/building-dapps/nctl-test/).

* `casper-client --put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem --session-path ~/casper/enhanced-nft/client/mint_session/target/wasm32-unknown-unknown/release/mint_call.wasm`

1) `--session-arg "nft_contract_hash:key='hash-206339c3deb8e6146974125bb271eb510795be6f250c21b1bd4b698956669f95'"`

    The contract hash of the previously installed CEP-78 NFT contract from which we will be minting.

2) `--session-arg "token_owner:key='account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb'"`

    The account hash of the account receiving the minted token.

3) `--session-arg "token_meta_data:string='{\"name\": \"John Doe\",\"token_uri\": \"https:\/\/www.barfoo.com\",\"checksum\": \"940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb\"}'"`

    Metadata describing the NFT to be minted, passed in as a `string`.



<details>
<summary><b>Casper client command without comments</b></summary>

```bash

casper-client --put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem --session-path ~/casper/enhanced-nft/client/mint_session/target/wasm32-unknown-unknown/release/mint_call.wasm
--session-arg "nft_contract_hash:key='hash-206339c3deb8e6146974125bb271eb510795be6f250c21b1bd4b698956669f95'"
--session-arg "token_owner:key='account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb'"
--session-arg "token_meta_data:string='{\"name\": \"John Doe\",\"token_uri\": \"https:\/\/www.barfoo.com\",\"checksum\": \"940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb\"}'"

```

</details>

## Transferring NFTs Between Users

Below is an example of a `casper-client` command that uses the `transfer` function to transfer ownership of an NFT from one user to another. In this case, we are transferring the previously minted NFT from the user associated with `node-2` to the user associated with `node-3`.

* `casper-client --put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-2/keys/secret_key.pem --session-path ~/casper/enhanced-nft/client/transfer_session/target/wasm32-unknown-unknown/release/transfer_call.wasm`

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

casper-client --put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-2/keys/secret_key.pem --session-path ~/casper/enhanced-nft/client/transfer_session/target/wasm32-unknown-unknown/release/transfer_call.wasm 
--session-arg "nft_contract_hash:key='hash-52e78ae3f6c485d036a74f65ebbb8c75fcc7c33fb42eb667fb32aeba72c63fb5'" 
--session-arg "source_key:key='account-hash-e9ff87766a1d2bab2565bfd5799054946200b51b20c3ca7e54a9269e00fe7cfb'" 
--session-arg "target_key:key='account-hash-b4772e7c47e4deca5bd90b7adb2d6e884f2d331825d5419d6cbfb59e17642aab'" 
--session-arg "is_hash_identifier_mode:bool='false'" 
--session-arg "token_id:u64='0'"  

```

</details>

## Burning an NFT

Below is an example of a `casper-client` command that uses the `burn` function to burn an NFT within a CEP-78 collection. If this command is used, the NFT in question will no longer be accessibly by anyone.

* `casper-client --put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem`

1) `--session-hash hash-52e78ae3f6c485d036a74f65ebbb8c75fcc7c33fb42eb667fb32aeba72c63fb5`

    The session hash corresponding to the NFT's contract package.

2) `--session-entry-point "burn"`

    The entry point corresponding to the `burn` function

3) `--session-arg "token_id:u64='1'"`

    The token ID for the NFT to be burned. If the `identifier_mode` is not set to `Ordinal`, you must provide the `token_hash` instead.

<details>
<summary><b>Casper client command without comments</b></summary>

casper-client --put-deploy -n http://localhost:11101/rpc --chain-name "casper-net-1" --payment-amount 500000000000 -k ~/casper/casper-node/utils/nctl/assets/net-1/nodes/node-1/keys/secret_key.pem
--session-hash hash-52e78ae3f6c485d036a74f65ebbb8c75fcc7c33fb42eb667fb32aeba72c63fb5
--session-entry-point "burn"
--session-arg "token_id:u64='1'"

</details>
