// A collection of tests that are focused
// around burning tokens.
#[cfg(test)]
mod burn;
// A collection of tests that are focused
// around installing the contract.
#[cfg(test)]
mod installer;
// A collection of tests that are focused
// around minting NFT tokens.
#[cfg(test)]
mod mint;
// A collection of tests that are focused
// around toggling control variables in the contract.
#[cfg(test)]
mod set_variables;
// A collection of tests that are focused
// around transfer token ownership
#[cfg(test)]
mod transfer;
// A collection of tests that are focused
// around updating metadata.
#[cfg(test)]
mod metadata;

// A collection of helper methods and constants.
#[cfg(test)]
mod utility;
