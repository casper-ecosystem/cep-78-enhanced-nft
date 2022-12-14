#!/bin/bash
rm -rf wasm/*
cp ../contract/target/wasm32-unknown-unknown/release/contract.wasm ./wasm
cp ../client/mint_session/target/wasm32-unknown-unknown/release/mint_call.wasm ./wasm
cp ../client/balance_of_session/target/wasm32-unknown-unknown/release/balance_of_call.wasm ./wasm
cp ../client/owner_of_session/target/wasm32-unknown-unknown/release/owner_of_call.wasm ./wasm
cp ../client/get_approved_session/target/wasm32-unknown-unknown/release/get_approved_call.wasm ./wasm
cp ../client/transfer_session/target/wasm32-unknown-unknown/release/transfer_call.wasm ./wasm
cp ../client/updated_receipts/target/wasm32-unknown-unknown/release/updated_receipts.wasm ./wasm
cp ../test-contracts/minting_contract/target/wasm32-unknown-unknown/release/minting_contract.wasm ./wasm
