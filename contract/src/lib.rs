#![no_std]

extern crate alloc;

pub mod constants;
pub mod error;
pub mod events;
pub mod modalities;

// A feature to allow the contract to be used
// as a library and a binary.
#[cfg(feature = "contract-support")]
pub mod utils;
