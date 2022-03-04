prepare:
	rustup target add wasm32-unknown-unknown

build-contract:
	cd nft-installer && cargo build --release --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/nft-installer.wasm 2>/dev/null | true

test: build-contract
	mkdir -p nft-tests/wasm
	cp target/wasm32-unknown-unknown/release/nft-installer.wasm nft-tests/wasm
	cd nft-tests && cargo test

clippy:
	cd nft-core/contract && cargo clippy --all-targets -- -D warnings
	cd nft-installer && cargo clippy --all-targets -- -D warnings
	cd nft-tests && cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cd nft-core/contract && cargo fmt -- --check
	cd nft-installer && cargo fmt -- --check
	cd nft-tests && cargo fmt -- --check

lint: clippy
	cd nft-core/contract && cargo fmt
	cd nft-installer && cargo fmt
	cd nft-tests && cargo fmt

clean:
	cd nft-core/contract && cargo clean
	cd nft-installer && cargo clean
	cd nft-tests && cargo clean
	rm -rf nft-tests/wasm
