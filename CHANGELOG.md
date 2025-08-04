# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

[comment]: <> (Added: new features)
[comment]: <> (Changed: changes in existing functionality)
[comment]: <> (Deprecated: soon-to-be removed features)
[comment]: <> (Removed: now removed features)
[comment]: <> (Fixed: any bug fixes)
[comment]: <> (Security: in case of vulnerabilities)

## [Unreleased]

## [2.0.0] - 2025-08-05

### Added

- Added full workflow in the docs ([#269](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/269)) — Adds Quickstart, full installation guide, and updated tutorials.
- Native eventing ([#286](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/286)) — Introduces native events feature, enables migration to 2.0 network, and adds test fixtures.

### Changed

- Disabled integer sign extensions ([#274](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/274), [#279](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/279)) — Modifies the WASM build to omit unsupported opcodes (fixes opcode-192 issue).
- Migration support for 2.0 ([#291](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/291)) — Updates internals for 2.0 network compatibility.
- JavaScript client updated for 2.0 ([#301](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/301)) — Updates JS SDK and adds tests.

### Deprecated

- Deprecated `EntryPoint` in favor of `EntityEntryPoint` ([#294](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/294)).

### Removed

- Removed legacy CI workflows ([#307](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/307)).
- Removed `entry_point`, `condor`, and `casper_2` named keys ([#316](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/316), [#317](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/317)) — Cleans up 2.0-specific keys and logic.

### Fixed

- Fixed outdated links in documentation ([#268](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/268)).
- Made `nft_kind` optional on install ([#267](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/267)).
- Ensured proper error handling in `get_optional_named_arg_with_user_errors` ([#270](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/270)) — Backports fix from CEP-18.
- Various events-related fixes ([#289](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/289)).
- Native events no longer commit state on failure ([#295](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/295)).
- Fixed broken Discord link in full installation guide ([#308](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/308)).
- Updated outdated URLs for documentation and RPC endpoints ([#309](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/309)).
- Bumped versions and updated JS SDK ([#310](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/310)).

### Security

_(No entries)_

## Release 1.5.1

### Changed

- Add ACL in types of js_client (#256)
- Fix Wrong error on insertion of duplicated token identifier (#258)
- Fix Failure on migration when EventsMode::CES to EventsMode::CES (#263)

### Added

- Optional custom string identifier in mint entrypoint (#255)

## Release 1.5.0

### Changed

- Update to README (#248)
- Update CES parser version (#246)
- Update modalities.md (#245)
- Command fix (#244)
- JS Client: Omit dev dependencies when running audit (#241)
- Remove potential revert in init and remove cep47 events dict creation on install (#240)
- Reformatting documentation (#239)

### Added

- ACL package mode - Including a contract package in the `acl_whitelist` will result in automatic whitelisting for any future versions of that contract. (#249)

- Package operator mode - Approving a package in `operators` allows any future version of that contract to act as an operator for transfer/approve/revoke entrypoints. (#249)

- Operator burn mode. This modes adds the possibility for operators to burn NFTs. (#250)

## Release 1.4.0

### Changed

- Contracts whitelist migrated to an ACL Whitelist. The ACL whitelist is a list of account and/or contract hashes that specifies which entity can call the `mint()` entrypoint to mint NFTs.

- This change results in the `contract_whitelist` dictionary being deprecated in favor of the new `acl_whitelist` dictionary.

### Added

- Transfer Filter Hook. The transfer filter modality, if enabled, specifies a contract package hash pointing to a contract that will be called when the `transfer` method is invoked on the contract.

- Added the `ACL` option to the `minting_mode` modality. This option allows only whitelisted accounts or contracts to mint tokens. More information can be found [here](./docs/modalities.md#minting).

## Release 1.3.0

### Changed

- Modified the `json-schema` runtime argument to be an optional installation parameter.

## Release 1.3.0

### Changed

- Modified the `json-schema` runtime argument to be an optional installation parameter.

## Release 1.2.0

### Added

- Added a new modality named `EventsMode` that dictates how the installed instance of CEP-78 will handle the recording of events. Refer to the [README](./docs/modalities.md#eventsmode) for further details

- Added the ability for the contract to specify one or more metadata schemas, with the option to further specify optional metadata schemas. Additional required metadatas are specified by `additional_required_metadata`, while additional optional metadata schemas can be specified using `optional_metadata`. Refer to the [`Installing the Contract`](./docs/using-casper-client.md#installing-the-contract) section of the README for more information on using these arguments.

- When upgrading from a contract instance, you may now change the `total_token_supply` to a number higher than the current number of minted tokens, but lower than your previous total. The number cannot be zero. More information is available in the upgrade tutorials.

- Added a new entrypoint called `revoke` that allows token owners to revoke a single approval.

- Added a new entrypoint `is_approval_for_all` that allows a caller to check if they are considered an `operator` for a token owner.

- For js-client changes, please view the respective [change log](./client-js/CHANGELOG.md).

### Changed

- `OwnerReverseLookupMode` now contains an additional option, `TransfersOnly`, which begins tracking ownership upon transfer. More information can be found [here](./docs/modalities.md#ownerreverselookupmode).

- Optimized the `set_approval_for_all` entrypoint implementation to reduce gas costs.

## Release 1.1.1

### Added

- Added a new modality named `NamedKeyConventionMode` that dictates the upgrading and installation process. Refer to the README for further details.

## Release 1.1.0

### Added

- Ownership of NFTs issued by a given CEP-78 contract instance are tracked by token id; thus the current owner of any given token by id is available using the owned_by entrypoint. However, some use cases benefit from the ability to ask the reverse question: list all the NFTs from a given contract instance owned by a specific owner.

  - To be able to support this reverse lookup option requires a contract instance to keep track of additional data, which causes higher gas costs for all mints and transfers. The gas cost to ask for any given owner is also unpredictable; as the asker is charged for the appropriate gas cost to read and return a collection that may be empty or may be quite large.

  - The pros and cons of either approach should be considered when installing a new CEP-78 contract. By default, new CEP-78 contract instances will default to the `OwnerReverseLookupMode::NoLookup` option, which has the lowest operating costs and optimal scaling characteristics. However, the `OwnerReverseLookupMode::Complete` option can be chosen upon install, which will allow the contract to write the necessary additional data to allow a full lookup by owner.

  - To allow isolation of the additional costs, or tracking individual owners, the reverse lookup mode supports a register entrypoint which is used to register owners prior to minting or receiving a transferred token. In either `Assigned` or `Transferable` mode, this register entrypoint can be called by any party on behalf of another party.

  - As a result of this change, the previously used `owned_tokens` dictionary is now deprecated. Moving forward, token ownership will be tracked using the `OwnerReverseLookupMode` modality and [CEP-78 Page System](./docs/reverse-lookup.md#the-cep-78-page-system).

### Changed

- A single instance of CEP-78 is limited to 1,000,000 tokens maximum.
- The naming convention for the default named key prefix of a given CEP-78 contract instance has been changed to `cep78_<collection_name>` with spaces and dashes within the collection name converted to underscores.

  - If an account installs more than one contract instance with the same collection name, it can lead to collision. It is recommended that all collection names be distinct or differentiated from each other via a suffix or sequence number as you prefer. Alternately, you may choose to not use the provided installation session logic and solve such a collision as you see fit.

  - **If an account attempts to install a second CEP-78 contract with the same name, it will overwrite the access rights and render the first instance unusable.**

[Keep a Changelog]: https://keepachangelog.com/en/1.0.0
