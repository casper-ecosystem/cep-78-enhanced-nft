# Session Code for the Get_approved Entry Point

Utility session code for interacting with the `get_approved` entry point present on the enhanced NFT contract. It returns
a `Key` if a given NFT is approved to be spent by another `Account` or `Contract` apart from the owner of the 
NFT itself. It returns `Some(Key)` if there is an approved spender, `None` if there is no spender.

## Compiling session code

The session code can be compiled to Wasm by running the `make build-contract` command provided in the Makefile at the top level.
The Wasm will be found in the `client/get_approved_session/target/wasm32-unknown-unknown/release` as `get_approved.wasm`.

## Usage

The `get_approved` session code takes in the following required runtime arguments.

* `nft_contract_hash`: The hash of a given Enhanced NFT contract passed in as a `Key`.
* `token_id`: The `id` of the NFT, passed in as a `u64`.
* `key_name`: The name for the entry within the `NamedKeys` under which `Option<Key>` value is stored, passed in as a `String`.
* `is_hash_identifier_mode`: A boolean argument that should be set to `true` if using the `Hash` NFT Identifier Mode and `false` if using the `Ordinal` mode.

If the contract in question uses the `Hash` NFT Identifier Mode, the following runtime argument is required.

* `token_hash`: The base16 encoded representation of the `blake2b` hash of the token's metadata.