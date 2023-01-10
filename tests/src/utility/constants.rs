pub(crate) const NFT_CONTRACT_WASM: &str = "contract.wasm";
pub(crate) const MINT_SESSION_WASM: &str = "mint_call.wasm";
pub(crate) const MINTING_CONTRACT_WASM: &str = "minting_contract.wasm";
pub(crate) const TRANSFER_SESSION_WASM: &str = "transfer_call.wasm";
pub(crate) const BALANCE_OF_SESSION_WASM: &str = "balance_of_call.wasm";
pub(crate) const OWNER_OF_SESSION_WASM: &str = "owner_of_call.wasm";
pub(crate) const GET_APPROVED_WASM: &str = "get_approved_call.wasm";
pub(crate) const UPDATED_RECEIPTS_WASM: &str = "updated_receipts.wasm";
pub(crate) const MANGLE_NAMED_KEYS: &str = "mangle_named_keys.wasm";
pub(crate) const CONTRACT_NAME: &str = "cep78_contract_hash_nft-test";
pub(crate) const GET_TOKEN_EVENTS_WASM: &str = "get_events_call.wasm";
pub(crate) const MINTING_CONTRACT_NAME: &str = "minting_contract_hash";
pub(crate) const NFT_TEST_COLLECTION: &str = "nft-test";
pub(crate) const NFT_TEST_SYMBOL: &str = "TEST";
pub(crate) const ENTRY_POINT_INIT: &str = "init";
pub(crate) const ENTRY_POINT_SET_VARIABLES: &str = "set_variables";
pub(crate) const ENTRY_POINT_MINT: &str = "mint";
pub(crate) const ENTRY_POINT_BURN: &str = "burn";
pub(crate) const ENTRY_POINT_TRANSFER: &str = "transfer";
pub(crate) const ENTRY_POINT_APPROVE: &str = "approve";
pub(crate) const ENTRY_POINT_METADATA: &str = "metadata";
pub(crate) const ENTRY_POINT_SET_APPROVE_FOR_ALL: &str = "set_approval_for_all";
pub(crate) const ENTRY_POINT_SET_TOKEN_METADATA: &str = "set_token_metadata";
pub(crate) const ENTRY_POINT_REGISTER_OWNER: &str = "register_owner";
pub(crate) const ARG_COLLECTION_NAME: &str = "collection_name";
pub(crate) const ARG_COLLECTION_SYMBOL: &str = "collection_symbol";
pub(crate) const ARG_TOTAL_TOKEN_SUPPLY: &str = "total_token_supply";
pub(crate) const ARG_ALLOW_MINTING: &str = "allow_minting";
pub(crate) const ARG_MINTING_MODE: &str = "minting_mode";
pub(crate) const ARG_HOLDER_MODE: &str = "holder_mode";
pub(crate) const ARG_WHITELIST_MODE: &str = "whitelist_mode";
pub(crate) const ARG_CONTRACT_WHITELIST: &str = "contract_whitelist";
pub(crate) const NUMBER_OF_MINTED_TOKENS: &str = "number_of_minted_tokens";
pub(crate) const ARG_TOKEN_META_DATA: &str = "token_meta_data";
pub(crate) const METADATA_CUSTOM_VALIDATED: &str = "metadata_custom_validated";
pub(crate) const METADATA_CEP78: &str = "metadata_cep78";
pub(crate) const METADATA_NFT721: &str = "metadata_nft721";
pub(crate) const METADATA_RAW: &str = "metadata_raw";
pub(crate) const ARG_TOKEN_OWNER: &str = "token_owner";
pub(crate) const ARG_NFT_CONTRACT_HASH: &str = "nft_contract_hash";
pub(crate) const ARG_JSON_SCHEMA: &str = "json_schema";
pub(crate) const ARG_APPROVE_ALL: &str = "approve_all";
pub(crate) const ARG_NFT_METADATA_KIND: &str = "nft_metadata_kind";
pub(crate) const ARG_IDENTIFIER_MODE: &str = "identifier_mode";
pub(crate) const ARG_METADATA_MUTABILITY: &str = "metadata_mutability";
pub(crate) const ARG_BURN_MODE: &str = "burn_mode";
pub(crate) const ARG_OWNER_LOOKUP_MODE: &str = "owner_reverse_lookup_mode";
pub(crate) const TOKEN_ISSUERS: &str = "token_issuers";
pub(crate) const ARG_OWNERSHIP_MODE: &str = "ownership_mode";
pub(crate) const ARG_NFT_KIND: &str = "nft_kind";
pub(crate) const TOKEN_COUNTS: &str = "balances";
pub(crate) const TOKEN_OWNERS: &str = "token_owners";
pub(crate) const BURNT_TOKENS: &str = "burnt_tokens";
pub(crate) const OPERATOR: &str = "operator";
pub(crate) const BALANCES: &str = "balances";
pub(crate) const RECEIPT_NAME: &str = "receipt_name";
pub(crate) const EVENTS: &str = "events";
pub(crate) const EVENT_ID_TRACKER: &str = "id_tracker";
pub(crate) const ARG_OPERATOR: &str = "operator";
pub(crate) const ARG_TARGET_KEY: &str = "target_key";
pub(crate) const ARG_SOURCE_KEY: &str = "source_key";
pub(crate) const ARG_TOKEN_ID: &str = "token_id";
pub(crate) const ARG_TOKEN_HASH: &str = "token_hash";
pub(crate) const ARG_KEY_NAME: &str = "key_name";
pub(crate) const ARG_IS_HASH_IDENTIFIER_MODE: &str = "is_hash_identifier_mode";
pub(crate) const ARG_NAMED_KEY_CONVENTION: &str = "named_key_convention";
pub(crate) const ARG_ACCESS_KEY_NAME_1_0_0: &str = "access_key_name";
pub(crate) const ARG_HASH_KEY_NAME_1_0_0: &str = "hash_key_name";
pub(crate) const ARG_STARTING_EVENT_ID: &str = "starting_event_id";
pub(crate) const ARG_ALL_EVENTS: &str = "all_events";
pub(crate) const ARG_LAST_EVENT_ID: &str = "last_event_id";
pub(crate) const ACCOUNT_USER_1: [u8; 32] = [1u8; 32];
pub(crate) const ACCOUNT_USER_2: [u8; 32] = [2u8; 32];
pub(crate) const ACCOUNT_USER_3: [u8; 32] = [3u8; 32];
pub(crate) const TEST_PRETTY_721_META_DATA: &str = r#"{
  "name": "John Doe",
  "symbol": "abc",
  "token_uri": "https://www.barfoo.com"
}"#;
pub(crate) const TEST_PRETTY_UPDATED_721_META_DATA: &str = r#"{
  "name": "John Doe",
  "symbol": "abc",
  "token_uri": "https://www.foobar.com"
}"#;
pub(crate) const TEST_PRETTY_CEP78_METADATA: &str = r#"{
  "name": "John Doe",
  "token_uri": "https://www.barfoo.com",
  "checksum": "940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb"
}"#;
pub(crate) const TEST_PRETTY_UPDATED_CEP78_METADATA: &str = r#"{
  "name": "John Doe",
  "token_uri": "https://www.foobar.com",
  "checksum": "fda4feaa137e83972db628e521c92159f5dc253da1565c9da697b8ad845a0788"
}"#;
pub(crate) const TEST_COMPACT_META_DATA: &str =
    r#"{"name": "John Doe","symbol": "abc","token_uri": "https://www.barfoo.com"}"#;
pub(crate) const MALFORMED_META_DATA: &str = r#"{
  "name": "John Doe",
  "symbol": abc,
  "token_uri": "https://www.barfoo.com"
}"#;
pub(crate) const ACCESS_KEY_NAME_1_0_0: &str = "nft_contract_package_access";
pub(crate) const CONTRACT_1_0_0_WASM: &str = "1_0_0/contract.wasm";
pub(crate) const MINT_1_0_0_WASM: &str = "1_0_0/mint_call.wasm";
pub(crate) const PAGE_SIZE: u64 = 1000;
pub(crate) const UNMATCHED_HASH_COUNT: &str = "unmatched_hash_count";
pub(crate) const PAGE_DICTIONARY_PREFIX: &str = "page_";
pub(crate) const PAGE_LIMIT: &str = "page_limit";
pub(crate) const HASH_KEY_NAME: &str = "nft_contract_package";
pub(crate) const ARG_NFT_PACKAGE_HASH: &str = "nft_package_hash";
pub(crate) const INDEX_BY_HASH: &str = "index_by_hash";
pub(crate) const PAGE_TABLE: &str = "page_table";
pub(crate) const ARG_MINTING_CONTRACT_REVERSE_LOOKUP: &str = "reverse_lookup";
