#!/bin/bash
rm -rf wasm
mkdir ./wasm/
cp ../tests/wasm/contract.wasm ./wasm
cp ../tests/wasm/mint_call.wasm ./wasm
cp ../tests/wasm/balance_of_call.wasm ./wasm
cp ../tests/wasm/owner_of_call.wasm ./wasm
cp ../tests/wasm/get_approved_call.wasm ./wasm
cp ../tests/wasm/transfer_call.wasm ./wasm
cp ../tests/wasm/updated_receipts.wasm ./wasm
