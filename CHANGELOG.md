# Changelog

All notable changes to this project will be documented in this file.  The format is based on [Keep a Changelog].

[comment]: <> (Added:      new features)
[comment]: <> (Changed:    changes in existing functionality)
[comment]: <> (Deprecated: soon-to-be removed features)
[comment]: <> (Removed:    now removed features)
[comment]: <> (Fixed:      any bug fixes)
[comment]: <> (Security:   in case of vulnerabilities)

## Release 1.1.0 

### Added

* Ownership of NFTs issued by a given CEP-78 contract instance are tracked by token id; thus the current owner of any given token by id is available using the owned_by entrypoint. However, some use cases benefit from the ability to ask the reverse question: list all the NFTs from a given contract instance owned by a specific owner.

    * To be able to support this reverse lookup option requires a contract instance to keep track of additional data, which causes higher gas costs for all mints and transfers. The gas cost to ask for any given owner is also unpredictable; as the asker is charged for the appropriate gas cost to read and return a collection that may be empty or may be quite large.

    * The pros and cons of either approach should be considered when installing a new CEP-78 contract. By default, new CEP-78 contract instances will default to the `OwnerReverseLookupMode::NoLookup` option, which has the lowest operating costs and optimal scaling characteristics. However, the `OwnerReverseLookupMode::Complete` option can be chosen upon install, which will allow the contract to write the necessary additional data to allow a full lookup by owner.

    * To allow isolation of the additional costs, or tracking individual owners, the reverse lookup mode supports a register entrypoint which is used to register owners prior to minting or receiving a transferred token. In either `Assigned` or `Transferable` mode, this register entrypoint can be called by any party on behalf of another party.

    
### Changed 

* A single instance of CEP-78 is limited to 1,000,000 tokens maximum.
* The naming convention for the default named key prefix of a given CEP-78 contract instance has been changed to `cep78_<collection_name>` with spaces and dashes within the collection name converted to underscores.

    * If an account installs more than one contract instance with the same collection name, it can lead to collision. It is recommended that all collection names be distinct or differentiated from each other via a suffix or sequence number as you prefer. Alternately, you may choose to not use the provided installation session logic and solve such a collision as you see fit.

    * **If an account attempts to install a second CEP-78 contract with the same name, it will overwrite the access rights and render the first instance unusable.**
    
[Keep a Changelog]: https://keepachangelog.com/en/1.0.0