pub mod events_ces;

// A feature to allow the contract to be used
// as a library and a binary.
#[cfg(feature = "contract-support")]
pub mod events_cep47;
