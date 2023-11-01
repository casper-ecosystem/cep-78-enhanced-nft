pub const BALANCE_OF_SESSION_WASM: &str = "balance_of_call.wasm";
pub const CONTRACT_1_0_0_WASM: &str = "1_0_0/contract.wasm";
pub const CONTRACT_1_1_0_WASM: &str = "1_1_0/contract.wasm";
pub const CONTRACT_1_2_0_WASM: &str = "1_2_0/contract.wasm";
pub const CONTRACT_1_3_0_WASM: &str = "1_3_0/contract.wasm";
pub const CONTRACT_1_4_0_WASM: &str = "1_4_0/contract.wasm";
pub const GET_APPROVED_WASM: &str = "get_approved_call.wasm";
pub const IS_APPROVED_FOR_ALL_WASM: &str = "is_approved_for_all_call.wasm";
pub const MANGLE_NAMED_KEYS: &str = "mangle_named_keys.wasm";
pub const MINT_1_0_0_WASM: &str = "1_0_0/mint_call.wasm";
pub const MINT_SESSION_WASM: &str = "mint_call.wasm";
pub const MINTING_CONTRACT_WASM: &str = "minting_contract.wasm";
pub const TRANSFER_FILTER_CONTRACT_WASM: &str = "transfer_filter_contract.wasm";
pub const NFT_CONTRACT_WASM: &str = "contract.wasm";
pub const OWNER_OF_SESSION_WASM: &str = "owner_of_call.wasm";
pub const TRANSFER_SESSION_WASM: &str = "transfer_call.wasm";
pub const UPDATED_RECEIPTS_WASM: &str = "updated_receipts.wasm";

pub const ARG_IS_HASH_IDENTIFIER_MODE: &str = "is_hash_identifier_mode";
pub const ARG_KEY_NAME: &str = "key_name";
pub const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
pub const ARG_NFT_CONTRACT_PACKAGE_HASH: &str = "nft_contract_package_hash";
pub const ARG_REVERSE_LOOKUP: &str = "reverse_lookup";
pub const ARG_FILTER_CONTRACT_RETURN_VALUE: &str = "return_value";

pub const CONTRACT_NAME: &str = "cep78_contract_hash_nft-test";
pub const MINTING_CONTRACT_NAME: &str = "minting_contract_hash";
pub const MINTING_CONTRACT_PACKAGE_NAME: &str = "minting_contract_package_hash";
pub const MINTING_CONTRACT_VERSION: &str = "minting_contract_version";
pub const TRANSFER_FILTER_CONTRACT_NAME: &str = "transfer_filter_contract_hash";
pub const NFT_TEST_COLLECTION: &str = "nft-test";
pub const NFT_TEST_SYMBOL: &str = "TEST";
pub const TOKEN_HASH: &str = "token_hash";

pub const ACCOUNT_USER_1: [u8; 32] = [1u8; 32];
pub const ACCOUNT_USER_2: [u8; 32] = [2u8; 32];
pub const ACCOUNT_USER_3: [u8; 32] = [3u8; 32];

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
