# Contract Code for Minting, Transferring and Burning

Contract code that can serve as a callback hook for the "transfer filter" mechanism.

Please note, this contract is meant for testing purposes only and is not meant to be used for production 
purposes.

## Compiling contract code

The contract code can be compiled to Wasm by running the `make build-contract` command provided in the Makefile at the top level.
The Wasm will be found in the `client/transfer_filter/target/wasm32-unknown-unknown/release` as `transfer_filter.wasm`.
