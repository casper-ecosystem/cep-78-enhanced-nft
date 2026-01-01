# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0] - 2025-08-05

### Changed

- Updated JavaScript client for 2.0 network support and added tests ([#161](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/161))

## [1.5.1] - 2023-11-20

### Fixed

- Added ACL types in `js_client` ([#256](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/256))

## [1.5.0] - 2023-10-05

### Fixed

- Updated `@make-software/ces-js-parser` version to support future releases ([#246](https://github.com/casper-ecosystem/cep-78-enhanced-nft/pull/246))

## [1.4.0] - 2023-05-26

### Fixed

- Fixed invalid paths in `package.json`

## [1.3.0] - 2023-04-03

### Added

- Browser support: WASM modules are now bundled as JS
- Added `casper-js-sdk` and `@make-software/ces-js-parser` as peer dependencies
- Enabled support for peer dependency resolution

### Changed

- Restructured project to prevent dependency version conflicts

## [1.2.0] - 2023-03-16

### Added

- Support for CEP-47 events
- Support for CES events using `ces-js-parser`
- `OwnerReverseLookupMode.TransfersOnly` modality
- `revoke` entrypoint support
- Typing improvements and various small cleanups
- Examples updated

### Fixed

- Added missing `collectionName` argument when using `sessionCode: true` in `mint`
- `contract_whitelist` now built using hashes instead of keys
- `getBurnModeConfig()` now returns a `Number` for consistency

## [1.1.0] - 2023-01-10

### Added

- Support for `NamedKeyConventionMode`

### Fixed

- Corrected construction of the `migrate` deploy
