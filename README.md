# Enhanced NFT standard

## Design goals
- User attempting to use an NFT contract should be able to install the contract
  with any differentiating arguments easily. Must work out of the box.
- Reference implementation must be straightforward/clear and obvious.
- Externally observable association between `Accounts` and `NFT`s they "own".
- Should be well documented with sufficient/exhaustive tests.
- Must be entirely self-contained within a singular repo, this includes the contract, tests 
  and any utilities such as clients and/or links to documentation. 
- Must support mainstream expectations about common NFT, additional features beyond the norms 
  as long as they don't interfere with the core functionality.
- Metadata and Payload should be conformant with the community standards and need not be
  constrained to CLType.


## Update
- Third parties mint token on platform and get paid, when they receive to native transfers; an on-chain marketplace.
- Modality for immutability/mutability for metadata (Default: Immutable and cannot be toggled)  
- Extend ownership model to include contracts as well.