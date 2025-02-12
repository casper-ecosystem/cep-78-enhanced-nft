# Variables
PINNED_TOOLCHAIN := $(shell cat contracts/rust-toolchain)
WASM_TARGET_DIR := ./target/wasm32-unknown-unknown/release
WASM_OUTPUT_DIR := tests/wasm
RUSTFLAGS := -C target-cpu=mvp
CARGO_BUILD_FLAGS := -Z build-std=std,panic_abort

# List of session and contract crates
SESSION_CRATES := balance_of_session get_approved_session is_approved_for_all_session \
                  mint_session owner_of_session transfer_session updated_receipts
CONTRACT_CRATES := mangle_named_keys minting_contract transfer_filter_contract
ALL_CRATES := cep78 $(SESSION_CRATES) $(CONTRACT_CRATES)
VERSIONS :=

# Helper macros
define build_and_strip
	RUSTFLAGS="$(RUSTFLAGS)" cargo +$(PINNED_TOOLCHAIN) build --release --target wasm32-unknown-unknown $(CARGO_BUILD_FLAGS) -p $1 ;
	wasm-strip $(WASM_TARGET_DIR)/$1.wasm ;
endef

# Targets
prepare:
	rustup install $(PINNED_TOOLCHAIN)
	rustup target add wasm32-unknown-unknown
	rustup component add clippy --toolchain $(PINNED_TOOLCHAIN)
	rustup component add rustfmt --toolchain $(PINNED_TOOLCHAIN)
	rustup component add rust-src --toolchain $(PINNED_TOOLCHAIN)

.PHONY: build-contract
build-contract:
	$(call build_and_strip,cep78)

.PHONY: build-all-contracts
build-all-contracts: build-contract
	$(foreach crate, $(SESSION_CRATES), $(call build_and_strip,$(crate)))
	$(foreach crate, $(CONTRACT_CRATES), $(call build_and_strip,$(crate)))

.PHONY: setup-test
setup-test: build-all-contracts
	mkdir -p $(WASM_OUTPUT_DIR)

	$(foreach version,$(VERSIONS), \
		if [ ! -d "tests/wasm/$(version)" ]; then \
			mkdir -p tests/wasm/$(version); \
			curl -L https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/download/v$(subst _,.,$(version))/cep-78-wasm.tar.gz | tar zxv -C tests/wasm/$(version)/; \
		fi; \
	)

	cp $(WASM_TARGET_DIR)/*.wasm $(WASM_OUTPUT_DIR)

.PHONY: test
test: setup-test
	cargo test -p tests --lib

.PHONY: clippy
clippy:
	cargo +$(PINNED_TOOLCHAIN) clippy --release -p cep78 --lib --target wasm32-unknown-unknown $(CARGO_BUILD_FLAGS) -- -D warnings
	$(foreach crate, $(ALL_CRATES), \
		cargo +$(PINNED_TOOLCHAIN) clippy --release -p $(crate) --bins --target wasm32-unknown-unknown $(CARGO_BUILD_FLAGS) -- -D warnings; \
	)
	cargo clippy --release -p tests --all-targets -- -D warnings

.PHONY: check-lint
check-lint: clippy
	$(foreach crate, $(ALL_CRATES), cargo +$(PINNED_TOOLCHAIN) fmt -p $(crate) -- --check;)
	cargo fmt -p tests -- --check

.PHONY: lint
lint: clippy format

.PHONY: format
format:
	$(foreach crate, $(ALL_CRATES), cargo +$(PINNED_TOOLCHAIN) fmt -p $(crate);)
	cargo fmt -p tests

.PHONY: clean
clean:
	$(foreach crate, $(ALL_CRATES), cargo clean -p $(crate);)
	cargo clean -p tests
	rm -rf $(WASM_OUTPUT_DIR)
	rm -rf ./*/Cargo.lock

.PHONY: cargo-update
cargo-update:
	$(foreach crate, $(ALL_CRATES), cargo update -p $(crate);)
	cargo update -p tests
