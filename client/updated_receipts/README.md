# Session code for migrating

Utility session code for migrating their receipts from the previous storage scheme 
to the new page based model scheme. This session code calls the `updated_receipts`
entry point on the contract inserting the necessary updated the reciepts into
the account's named keys.

## Compiling session code

The session code can be compiled to Wasm by running the `make build-contract` command provided in the Makefile at the top level.
The Wasm will be found in the `client/mint_session/target/wasm32-unknown-unknown/release` as `updated_receipts.wasm`.

## Usage

The `updated_receipts.wasm` takes the following runtime arguments.

* `nft_package_hash`: The NFT ContractPackageHash for a given instance of the NFT contract.
