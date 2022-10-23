# Session Code for Transferring

Utility session code for interacting with the `transfer` entry point present on the enhanced NFT contract. The session code transfers an NFT from the `Source` to the `Target`. Note that this code depends on the contract being set to `Ownership::Transferable` mode.

## Compiling session code

The session code can be compiled to Wasm by running the `make build-contract` command provided in the Makefile at the top level.
The Wasm will be found in the `client/transfer_session/target/wasm32-unknown-unknown/release` as `transfer_call.wasm`.

## Usage

The `transfer_call` session code takes in the following required runtime arguments.

* `nft_contract_hash`: The hash of a given Enhanced NFT contract passed in as a `Key`.
* `token_id`: The `id` of the NFT, passed in as a `u64`.
* `target_key`: The `Key` of the account receiving the NFT. 
* `source_key`: The `Key` of the account sending the NFT.
* `is_hash_identifier_mode`: A boolean argument that should be set to `true` if using the `Hash` NFT Identifier Mode and `false` if using the `Ordinal` mode.

If the contract in question uses the `Hash` NFT Identifier Mode, the following runtime argument is required.

* `token_hash`: The base16 encoded representation of the `blake2b` hash of the token's metadata.