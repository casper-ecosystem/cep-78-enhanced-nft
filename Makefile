PINNED_TOOLCHAIN := $(shell cat rust-toolchain)

prepare:
	rustup target add wasm32-unknown-unknown
	rustup component add clippy --toolchain ${PINNED_TOOLCHAIN}
	rustup component add rustfmt --toolchain ${PINNED_TOOLCHAIN}

build-contract:
	cd contract && cargo build --release --target wasm32-unknown-unknown
	cd client/mint_session && cargo build --release --target wasm32-unknown-unknown
	cd client/balance_of_session && cargo build --release --target wasm32-unknown-unknown
	cd client/owner_of_session && cargo build --release --target wasm32-unknown-unknown
	cd client/get_approved_session && cargo build --release --target wasm32-unknown-unknown
	cd client/is_approved_for_all_session && cargo build --release --target wasm32-unknown-unknown
	cd client/transfer_session && cargo build --release --target wasm32-unknown-unknown
	cd client/updated_receipts && cargo build --release --target wasm32-unknown-unknown
	cd test-contracts/minting_contract && cargo build --release --target wasm32-unknown-unknown
	cd test-contracts/mangle_named_keys && cargo build --release --target wasm32-unknown-unknown
	cd test-contracts/transfer_filter_contract && cargo build --release --target wasm32-unknown-unknown
	wasm-strip contract/target/wasm32-unknown-unknown/release/contract.wasm
	wasm-strip client/mint_session/target/wasm32-unknown-unknown/release/mint_call.wasm
	wasm-strip client/balance_of_session/target/wasm32-unknown-unknown/release/balance_of_call.wasm
	wasm-strip client/owner_of_session/target/wasm32-unknown-unknown/release/owner_of_call.wasm
	wasm-strip client/get_approved_session/target/wasm32-unknown-unknown/release/get_approved_call.wasm
	wasm-strip client/is_approved_for_all_session/target/wasm32-unknown-unknown/release/is_approved_for_all_call.wasm
	wasm-strip client/transfer_session/target/wasm32-unknown-unknown/release/transfer_call.wasm
	wasm-strip client/updated_receipts/target/wasm32-unknown-unknown/release/updated_receipts.wasm
	wasm-strip test-contracts/minting_contract/target/wasm32-unknown-unknown/release/minting_contract.wasm
	wasm-strip test-contracts/transfer_filter_contract/target/wasm32-unknown-unknown/release/transfer_filter_contract.wasm

setup-test: build-contract
	mkdir -p tests/wasm
	mkdir -p tests/wasm/1_0_0; curl -L https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/download/v1.0.0/cep-78-wasm.tar.gz | tar zxv -C tests/wasm/1_0_0/
	mkdir -p tests/wasm/1_1_0; curl -L https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/download/v1.1.0/cep-78-wasm.tar.gz | tar zxv -C tests/wasm/1_1_0/
	mkdir -p tests/wasm/1_2_0; curl -L https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/download/v1.2.0/cep-78-wasm.tar.gz | tar zxv -C tests/wasm/1_2_0/
	mkdir -p tests/wasm/1_3_0; curl -L https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/download/v1.3.0/cep-78-wasm.tar.gz | tar zxv -C tests/wasm/1_3_0/
	mkdir -p tests/wasm/1_4_0; curl -L https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/download/v1.4.0/cep-78-wasm.tar.gz | tar zxv -C tests/wasm/1_4_0/

	cp contract/target/wasm32-unknown-unknown/release/contract.wasm tests/wasm
	cp client/mint_session/target/wasm32-unknown-unknown/release/mint_call.wasm tests/wasm
	cp client/balance_of_session/target/wasm32-unknown-unknown/release/balance_of_call.wasm tests/wasm
	cp client/owner_of_session/target/wasm32-unknown-unknown/release/owner_of_call.wasm tests/wasm
	cp client/get_approved_session/target/wasm32-unknown-unknown/release/get_approved_call.wasm tests/wasm
	cp client/is_approved_for_all_session/target/wasm32-unknown-unknown/release/is_approved_for_all_call.wasm tests/wasm
	cp client/transfer_session/target/wasm32-unknown-unknown/release/transfer_call.wasm tests/wasm
	cp client/updated_receipts/target/wasm32-unknown-unknown/release/updated_receipts.wasm tests/wasm
	cp test-contracts/minting_contract/target/wasm32-unknown-unknown/release/minting_contract.wasm tests/wasm
	cp test-contracts/mangle_named_keys/target/wasm32-unknown-unknown/release/mangle_named_keys.wasm tests/wasm
	cp test-contracts/transfer_filter_contract/target/wasm32-unknown-unknown/release/transfer_filter_contract.wasm tests/wasm

test: setup-test
	cd tests && cargo test

clippy:
	cd contract && cargo clippy --target wasm32-unknown-unknown --bins -- -D warnings
	cd contract && cargo clippy --no-default-features --lib -- -D warnings
	cd client/mint_session && cargo clippy --release --target wasm32-unknown-unknown -- -D warnings
	cd client/balance_of_session && cargo clippy --release --target wasm32-unknown-unknown -- -D warnings
	cd client/owner_of_session && cargo clippy --release --target wasm32-unknown-unknown -- -D warnings
	cd client/get_approved_session && cargo clippy --release --target wasm32-unknown-unknown -- -D warnings
	cd client/transfer_session && cargo clippy --release --target wasm32-unknown-unknown -- -D warnings
	cd client/updated_receipts && cargo clippy --release --target wasm32-unknown-unknown -- -D warnings
	cd test-contracts/minting_contract && cargo clippy --release --target wasm32-unknown-unknown -- -D warnings
	cd test-contracts/mangle_named_keys && cargo clippy --release --target wasm32-unknown-unknown -- -D warnings
	cd test-contracts/transfer_filter_contract && cargo clippy --release --target wasm32-unknown-unknown -- -D warnings
	cd tests && cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cd contract && cargo fmt -- --check
	cd client/mint_session && cargo fmt -- --check
	cd client/balance_of_session && cargo fmt -- --check
	cd client/owner_of_session && cargo fmt -- --check
	cd client/get_approved_session && cargo fmt -- --check
	cd client/transfer_session && cargo fmt -- --check
	cd client/updated_receipts && cargo fmt -- --check
	cd test-contracts/minting_contract && cargo fmt -- --check
	cd test-contracts/mangle_named_keys && cargo fmt -- --check
	cd test-contracts/transfer_filter_contract && cargo fmt -- --check
	cd tests && cargo fmt -- --check

lint: clippy
	cd contract && cargo fmt
	cd client/mint_session && cargo fmt
	cd client/balance_of_session && cargo fmt
	cd client/owner_of_session && cargo fmt
	cd client/get_approved_session && cargo fmt
	cd client/transfer_session && cargo fmt
	cd client/updated_receipts && cargo fmt
	cd test-contracts/minting_contract
	cd test-contracts/mangle_named_keys
	cd test-contracts/transfer_filter_contract
	cd tests && cargo fmt

clean:
	cd contract && cargo clean
	cd client/mint_session && cargo clean
	cd client/balance_of_session && cargo clean
	cd client/owner_of_session && cargo clean
	cd client/get_approved_session && cargo clean
	cd client/transfer_session && cargo clean
	cd client/updated_receipts && cargo clean
	cd test-contracts/minting_contract && cargo clean
	cd test-contracts/mangle_named_keys && cargo clean
	cd test-contracts/transfer_filter_contract && cargo clean
	cd tests && cargo clean
	rm -rf tests/wasm
