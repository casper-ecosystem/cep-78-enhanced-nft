# Contract Code for Minting, Transferring and Burning

Contract code that handles the `Mint`, `Transfer` and `Burn` entry points. 

Please note, this contract is meant for testing purposes only and is not meant to be used for production 
purposes.

## Compiling contract code

The contract code can be compiled to Wasm by running the `make build-contract` command provided in the Makefile at the top level.
The Wasm will be found in the `client/minting_contract/target/wasm32-unknown-unknown/release` as `minting_contract.wasm`.

## Usage

The `minting_contract` contract code contains the following entry points.

* `mint` - This entry point allows for the minting of NFTS.
* `transfer` - This entry point allows for the transfer of NFTs between accounts.
* `burn` - This entry point allows for the burning of NFTs.

It also recognizes the following runtime arguments.

* `nft_contract_hash`: The hash of a given Enhanced NFT contract passed in as a `Key`.
* `token_owner`: The `Key` of the owner for the NFT to be minted.
* `token_metadata`: The metadata describing the NFT to be minted, passed in as a `String`.
* `target_key`: The `Key` of the account receiving the NFT. 
* `source_key`: The `Key` of the account sending the NFT.
* `token_id`: The `id` of the NFT, passed in as a `u64`.