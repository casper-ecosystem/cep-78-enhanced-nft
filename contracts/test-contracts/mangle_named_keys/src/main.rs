#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casper_types::ApiError;

const ACCESS_KEY_NAME_1_0_0: &str = "nft_contract_package_access";
const HASH_KEY_NAME_1_0_0: &str = "nft_contract_package";

const MANGLED_ACCESS_KEY_NAME: &str = "mangled_access_key";
const MANGLED_HASH_KEY_NAME: &str = "mangled_hash_key";

#[no_mangle]
pub extern "C" fn call() {
    let access_key =
        runtime::get_key(ACCESS_KEY_NAME_1_0_0).unwrap_or_revert_with(ApiError::User(1801));
    runtime::put_key(MANGLED_ACCESS_KEY_NAME, access_key);
    runtime::remove_key(ACCESS_KEY_NAME_1_0_0);

    let package_key =
        runtime::get_key(HASH_KEY_NAME_1_0_0).unwrap_or_revert_with(ApiError::User(1802));
    runtime::put_key(MANGLED_HASH_KEY_NAME, package_key);
    runtime::remove_key(HASH_KEY_NAME_1_0_0);
}
