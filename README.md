# NFT Redesign

## Design goals
- Externally observables association between `Accounts` and `NFT`s they "own".
- Reference implementation must be straightforward/clear and obvious.
- User attempting to make their own NFT contract should be able to install the contract 
  with any differentiating arguments easily. Must work out of the box.
- Should be well documented with sufficient/exhaustive tests.
- Must be entirely self-contained within a singular repo, this includes the contract, tests 
  and any utilities such as clients and/or links to documentation. 
- Must support mainstream expectations about common NFT, additional features beyond the norms 
  as long as they don't interfere with the core functionality.
- Metadata and Payload should be conformant with the community standards and need not be
  constrained to CLType. 
