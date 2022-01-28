prepare:
	rustup target add wasm32-unknown-unknown

build-contract:
	cd nft-installer && cargo build --release --target wasm32-unknown-unknown
	wasm-strip nft-installer/target/wasm32-unknown-unknown/release/contract.wasm 2>/dev/null | true

test: build-contract
	mkdir -p nft-tests/wasm
	cp nft-installer/target/wasm32-unknown-unknown/release/nft-installer.wasm nft-tests/wasm
	cd nft-tests && cargo test

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
	cd nft-installer && cargo clean
	cd nft-tests && cargo clean
	rm -rf nft-tests/wasm
