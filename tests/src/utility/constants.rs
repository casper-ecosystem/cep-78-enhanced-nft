use casper_engine_test_support::DEFAULT_ACCOUNT_ADDR;
use casper_types::{account::AccountHash, Key, PublicKey, SecretKey};
use once_cell::sync::Lazy;

pub const BALANCE_OF_SESSION_WASM: &str = "balance_of_session.wasm";
pub const GET_APPROVED_WASM: &str = "get_approved_session.wasm";
pub const IS_APPROVED_FOR_ALL_WASM: &str = "is_approved_for_all_session.wasm";
pub const MINT_SESSION_WASM: &str = "mint_session.wasm";
pub const MINTING_CONTRACT_WASM: &str = "minting_contract.wasm";
pub const TRANSFER_FILTER_CONTRACT_WASM: &str = "transfer_filter_contract.wasm";
pub const NFT_CONTRACT_WASM: &str = "cep78.wasm";
pub const OWNER_OF_SESSION_WASM: &str = "owner_of_session.wasm";
pub const TRANSFER_SESSION_WASM: &str = "transfer_session.wasm";

pub const ARG_IS_HASH_IDENTIFIER_MODE: &str = "is_hash_identifier_mode";
pub const ARG_KEY_NAME: &str = "key_name";
pub const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
pub const ARG_NFT_CONTRACT_PACKAGE_HASH: &str = "nft_contract_package_hash";
pub const ARG_REVERSE_LOOKUP: &str = "reverse_lookup";
pub const ARG_FILTER_CONTRACT_RETURN_VALUE: &str = "return_value";

pub const CONTRACT_NAME: &str = "cep78_contract_hash_nft-test";
pub const CONTRACT_VERSION: &str = "cep78_contract_version_nft-test";
pub const CONTRACT_PACKAGE: &str = "cep78_contract_package_nft-test";
pub const MINTING_CONTRACT_NAME: &str = "minting_contract_hash";
pub const MINTING_CONTRACT_PACKAGE_NAME: &str = "minting_contract_package_hash";
pub const MINTING_CONTRACT_VERSION: &str = "minting_contract_version";
pub const TRANSFER_FILTER_CONTRACT_NAME: &str = "transfer_filter_contract_hash";
pub const NFT_TEST_COLLECTION: &str = "nft-test";
pub const NFT_TEST_SYMBOL: &str = "TEST";
pub const TOKEN_HASH: &str = "token_hash";

pub static ACCOUNT_1_SECRET_KEY: Lazy<SecretKey> =
    Lazy::new(|| SecretKey::secp256k1_from_bytes([1u8; 32]).unwrap());
pub static ACCOUNT_1_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| PublicKey::from(&*ACCOUNT_1_SECRET_KEY));
pub static ACCOUNT_1_ADDR: Lazy<AccountHash> = Lazy::new(|| ACCOUNT_1_PUBLIC_KEY.to_account_hash());
pub static ACCOUNT_1_KEY: Lazy<Key> = Lazy::new(|| Key::Account(*ACCOUNT_1_ADDR));

pub static ACCOUNT_2_SECRET_KEY: Lazy<SecretKey> =
    Lazy::new(|| SecretKey::secp256k1_from_bytes([2u8; 32]).unwrap());
pub static ACCOUNT_2_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| PublicKey::from(&*ACCOUNT_2_SECRET_KEY));
pub static ACCOUNT_2_ADDR: Lazy<AccountHash> = Lazy::new(|| ACCOUNT_2_PUBLIC_KEY.to_account_hash());
pub static ACCOUNT_2_KEY: Lazy<Key> = Lazy::new(|| Key::Account(*ACCOUNT_2_ADDR));

pub static ACCOUNT_3_SECRET_KEY: Lazy<SecretKey> =
    Lazy::new(|| SecretKey::secp256k1_from_bytes([3u8; 32]).unwrap());
pub static ACCOUNT_3_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| PublicKey::from(&*ACCOUNT_3_SECRET_KEY));
pub static ACCOUNT_3_ADDR: Lazy<AccountHash> = Lazy::new(|| ACCOUNT_3_PUBLIC_KEY.to_account_hash());
pub static ACCOUNT_3_KEY: Lazy<Key> = Lazy::new(|| Key::Account(*ACCOUNT_3_ADDR));

pub static DEFAULT_ACCOUNT_KEY: Lazy<Key> = Lazy::new(|| Key::Account(*DEFAULT_ACCOUNT_ADDR));

pub const PAGE_SIZE: u64 = 1000;

pub const TEST_PRETTY_721_META_DATA: &str = r#"{
  "name": "John Doe",
  "symbol": "abc",
  "token_uri": "https://www.barfoo.com"
}"#;
pub const TEST_PRETTY_UPDATED_721_META_DATA: &str = r#"{
  "name": "John Doe",
  "symbol": "abc",
  "token_uri": "https://www.foobar.com"
}"#;
pub const TEST_PRETTY_CEP78_METADATA: &str = r#"{
  "name": "John Doe",
  "token_uri": "https://www.barfoo.com",
  "checksum": "940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb"
}"#;
pub const TEST_PRETTY_UPDATED_CEP78_METADATA: &str = r#"{
  "name": "John Doe",
  "token_uri": "https://www.foobar.com",
  "checksum": "fda4feaa137e83972db628e521c92159f5dc253da1565c9da697b8ad845a0788"
}"#;
pub const TEST_COMPACT_META_DATA: &str =
    r#"{"name": "John Doe","symbol": "abc","token_uri": "https://www.barfoo.com"}"#;
pub const MALFORMED_META_DATA: &str = r#"{
  "name": "John Doe",
  "symbol": abc,
  "token_uri": "https://www.barfoo.com"
}"#;
