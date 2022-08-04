# Session code for minting

Utility session code for interacting with the `mint` entry point present on the enhanced NFT contract. The session code retrieves
the read only reference and inserts the reference under the executing `Account`s `NamedKeys`.

## Compiling session code

The session code can be compiled to Wasm by running the `make build-contract` command provided in the Makefile at the top level.
The Wasm will be found in the `client/mint_session/target/wasm32-unknown-unknown/release` as `mint_call.wasm`.

## Usage

The `mint_call` session code takes in the following required runtime arguments.

* `nft_contract_hash`: The hash of a given Enhanced NFT contract passed in as a `Key`.
* `token_owner`: The `Key` of the owner for the NFT to be minted. Note, this argument is ignored in the `Ownership::Minter` mode.
* `token_meta_data`: The metadata describing the NFT to be minted, passed in as a `String`.
