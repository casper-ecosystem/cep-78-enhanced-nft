prepare:
	rustup target add wasm32-unknown-unknown

build-contract:
	cd contract && cargo build --release --target wasm32-unknown-unknown
	cd client/mint_session && cargo build --release --target wasm32-unknown-unknown
	cd client/balance_of_session && cargo build --release --target wasm32-unknown-unknown
	cd client/owner_of_session && cargo build --release --target wasm32-unknown-unknown
	cd client/get_approved_session && cargo build --release --target wasm32-unknown-unknown
	cd client/minting_contract && cargo build --release --target wasm32-unknown-unknown
	cd client/transfer_session && cargo build --release --target wasm32-unknown-unknown
	wasm-strip contract/target/wasm32-unknown-unknown/release/contract.wasm 2>/dev/null | true
	wasm-strip entrypoint_session/target/wasm32-unknown-unknown/release/entrypoint_call.wasm 2>/dev/null | true
	wasm-strip client/mint_session/target/wasm32-unknown-unknown/release/mint_call.wasm 2>/dev/null | true
	wasm-strip client/balance_of_session/target/wasm32-unknown-unknown/release/balance_of_call.wasm 2>/dev/null | true
	wasm-strip client/owner_of_session/target/wasm32-unknown-unknown/release/owner_of_call.wasm 2>/dev/null | true
	wasm-strip client/get_approved_session/target/wasm32-unknown-unknown/release/get_approved_call.wasm 2>/dev/null | true
	wasm-strip client/minting_contract/target/wasm32-unknown-unknown/release/minting_contract.wasm 2>/dev/null | true
	wasm-strip client/transfer_session/target/wasm32-unknown-unknown/release/transfer_call.wasm 2>/dev/null | true

test: build-contract
	mkdir -p tests/wasm
	cp contract/target/wasm32-unknown-unknown/release/contract.wasm tests/wasm
	cp client/mint_session/target/wasm32-unknown-unknown/release/mint_call.wasm tests/wasm
	cp client/balance_of_session/target/wasm32-unknown-unknown/release/balance_of_call.wasm tests/wasm
	cp client/owner_of_session/target/wasm32-unknown-unknown/release/owner_of_call.wasm tests/wasm
	cp client/get_approved_session/target/wasm32-unknown-unknown/release/get_approved_call.wasm tests/wasm
	cp client/minting_contract/target/wasm32-unknown-unknown/release/minting_contract.wasm tests/wasm
	cp client/transfer_session/target/wasm32-unknown-unknown/release/transfer_call.wasm tests/wasm
	cd tests && cargo test

clippy:
	cd contract && cargo clippy --all-targets -- -D warnings
	cd tests && cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cd contract && cargo fmt -- --check
	cd tests && cargo fmt -- --check

lint: clippy
	cd contract && cargo fmt
	cd tests && cargo fmt

clean:
	cd contract && cargo clean
	cd tests && cargo clean
	rm -rf tests/wasm