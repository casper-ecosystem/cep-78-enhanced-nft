# Session code for the Owner_of Entry Point

Utility session code for calling the `owner_of` entrypoint on the enhanced NFT contract. It returns the `Key` of the owner 
for a given NFT.

Please be aware that users may query dictionary items directly, off-chain, without incurring network fees by using the [`casper-client`](https://crates.io/crates/casper-client) command [`casper-client get-dictionary-item`](https://docs.rs/casper-client/1.5.0/casper_client/fn.get_dictionary_item.html). Sending a deploy to interact with the `owner_of` entry point will incur transaction costs.

## Compiling session code

The session code can be compiled to Wasm by running the `make build-contract` command provided in the Makefile at the top level.
The Wasm will be found in the `client/owner_of_session/target/wasm32-unknown-unknown/release` as `owner_of_call.wasm`.

## Usage

The `owner_of` session code takes in the following required runtime arguments.

* `nft_contract_hash`: The hash of a given Enhanced NFT contract passed in as a `Key`.
* `token_id`: The `id` of the NFT, passed in as a `u64`.
* `key_name`: The name for the entry within the `NamedKeys` under which `Option<Key>` value is stored, passed in as a `String`.
* `is_hash_identifier_mode`: A boolean argument that should be set to `true` if using the `Hash` NFT Identifier Mode and `false` if using the `Ordinal` mode.

If the contract in question uses the `Hash` NFT Identifier Mode, the following runtime argument is required.

* `token_hash`: The base16 encoded representation of the `blake2b` hash of the token's metadata.
