#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

use alloc::{string::String, vec::Vec};

use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{self, runtime_args, ApiError, Key, PackageHash};

const ENTRY_POINT_UPDATE_RECEIPTS: &str = "updated_receipts";
pub(crate) const ARG_NFT_CONTRACT_PACKAGE_HASH: &str = "nft_contract_package_hash";

#[no_mangle]
pub extern "C" fn call() {
    let nft_package_hash: PackageHash =
        runtime::get_named_arg::<Key>(ARG_NFT_CONTRACT_PACKAGE_HASH)
            .into_package_addr()
            .unwrap_or_revert_with(ApiError::User(1701))
            .into();

    // This value represents a list of receipt names and addresses corresponding
    // to the pages marking ownership of NFTs owned by the account.
    let updated_receipt_data = runtime::call_versioned_contract::<Vec<(String, Key)>>(
        nft_package_hash.into(),
        None,
        ENTRY_POINT_UPDATE_RECEIPTS,
        runtime_args! {},
    );
    for (receipt_name, dictionary_address) in updated_receipt_data.into_iter() {
        runtime::put_key(&receipt_name, dictionary_address)
    }
}
