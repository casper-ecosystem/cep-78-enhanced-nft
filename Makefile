prepare:
	rustup target add wasm32-unknown-unknown

build-contract:
	cd contract && cargo build --release --target wasm32-unknown-unknown
	cd mint_session && cargo build --release --target wasm32-unknown-unknown
	cd burn_session && cargo build --release --target wasm32-unknown-unknown
	cd balance_of_session && cargo build --release --target wasm32-unknown-unknown
	wasm-strip contract/target/wasm32-unknown-unknown/release/contract.wasm 2>/dev/null | true
	wasm-strip mint_session/target/wasm32-unknown-unknown/release/mint.wasm 2>/dev/null | true
	wasm-strip burn_session/target/wasm32-unknown-unknown/release/mint.wasm 2>/dev/null | true
	wasm-strip balance_of_session/target/wasm32-unknown-unknown/release/mint.wasm 2>/dev/null | true

test: build-contract
	mkdir -p tests/wasm
	cp contract/target/wasm32-unknown-unknown/release/contract.wasm tests/wasm
	cp mint_session/target/wasm32-unknown-unknown/release/mint.wasm tests/wasm
	cp balance_of_session/target/wasm32-unknown-unknown/release/balance_of.wasm tests/wasm
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
