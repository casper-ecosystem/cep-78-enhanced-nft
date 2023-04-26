# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.0] - 2023-04-03

### Added

- Support for browsers (as WASM are now converted to JS modules and bundled that way).

### Changed

- Added `casper-js-sdk` and `@make-software/ces-js-parser` as peer dependencies. This will prevent conflicts between multiple versions of dependencies.

## [1.2.0] - 2023-03-16

### Added

- Added support for CEP47 Events
- Added support for CES events basing on ces-js-parser
- Some small code cleanups (added typings etc)
- Some changes in `examples/`
- Added `OwnerReverseLookupMode.TransfersOnly` modality
- Added `revoke` entrypoint support

### Fixed

- Added missing `collectionName` argument in mint when using a sessionCode: true
- `contract_whitelist` is now build with Hashes rather then Keys
- Fixed inconsistency in `getBurnModeConfig()` (now it returns Number as other similar methods)

## [1.1.0] - 2023-01-10

### Added

- Added support for NamedKeyConventionMode

### Fixed

- Fixed how the `migrate` deploy is constructed.
