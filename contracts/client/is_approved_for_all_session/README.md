# Session Code for the is_approved_for_all Entry Point

Utility session code for interacting with the `is_approved_for_all` entry point present on the enhanced NFT contract. It returns
a `Boolean` if a given account is an operator that can approve another `Account` or `Contract` apart from the owner of the
NFT itself as an approved account. An operator can also transfer any token on behalf of an owner. It returns `true` if there is an operator for one owner, `None` if there is no spender.

## Compiling session code

The session code can be compiled to Wasm by running the `make build-contract` command provided in the Makefile at the top level.
The Wasm will be found in the `client/is_approved_for_all_session/target/wasm32-unknown-unknown/release` as `is_approved_for_all_session.wasm`.

## Usage

The `is_approved_for_all` session code takes in the following required runtime arguments.

- `nft_contract_hash`: The hash of a given Enhanced NFT contract passed in as a `Key`.
- `key_name`: The name for the entry within the `NamedKeys` under which `Option<Key>` value is stored, passed in as a `String`.

* `token_owner`: The `Key` of the owner for whose operator is being queried.
* `operator`: The `Key` of the operator account.
