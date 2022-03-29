pub const ARG_COLLECTION_NAME: &str = "collection_name";
pub const ARG_COLLECTION_SYMBOL: &str = "collection_symbol";
pub const ARG_TOTAL_TOKEN_SUPPLY: &str = "total_token_supply"; // <-- Think about if mutable or not...

// pub const ARG_TOKEN_OWNER: &str = "token_owner";
// pub const ARG_TOKEN_NAME: &str = "token_name";

pub const ARG_TOKEN_ID: &str = "token_id";
pub const ARG_TOKEN_OWNER: &str = "token_owner";
pub const ARG_TO_ACCOUNT_HASH: &str = "to_account_hash";
pub const ARG_FROM_ACCOUNT_HASH: &str = "from_account_hash";
pub const ARG_ALLOW_MINTING: &str = "allow_minting";
pub const ARG_PUBLIC_MINTING: &str = "public_minting";
pub const ARG_ACCOUNT_HASH: &str = "account_hash";
pub const ARG_TOKEN_META_DATA: &str = "token_meta_data";
pub const ARG_APPROVE_TRANSFER_FOR_ACCOUNT_HASH: &str = "approve_transfer_for_account_hash"; //Change name?
pub const ARG_APPROVE_ALL: &str = "approve_all";
pub const ARG_OPERATOR: &str = "operator";
pub const ARG_OWNERSHIP_MODE: &str = "ownership_mode";

// STORAGE is the list of all NFTS
// Owners is a dictionary owner --> nfts
//pub const STORAGE: &str = "storage";
//pub const OWNERS: &str = "owners";
pub const APPROVED_FOR_TRANSFER: &str = "approved_for_transfer"; //Change name?
pub const NUMBER_OF_MINTED_TOKENS: &str = "number_of_minted_tokens";

pub const INSTALLER: &str = "installer";
pub const CONTRACT_NAME: &str = "nft_contract";
pub const HASH_KEY_NAME: &str = "nft_contract_package";
pub const ACCESS_KEY_NAME: &str = "nft_contract_package_access";
pub const CONTRACT_VERSION: &str = "contract_version";
pub const COLLECTION_NAME: &str = "collection_name";
pub const COLLECTION_SYMBOL: &str = "collection_symbol";
pub const TOTAL_TOKEN_SUPPLY: &str = "total_token_supply";
pub const OWNERSHIP_MODE: &str = "ownership_mode";
pub const ALLOW_MINTING: &str = "allow_minting";
pub const PUBLIC_MINTING: &str = "public_minting";
pub const TOKEN_OWNERS: &str = "token_owners";
pub const TOKEN_ISSUERS: &str = "token_issuers";
pub const TOKEN_META_DATA: &str = "token_meta_data";
pub const OWNED_TOKENS: &str = "owned_tokens";
pub const BURNT_TOKENS: &str = "burnt_tokens";
pub const TOKEN_COUNTS: &str = "balances";
//pub const IS_TRANSFERRABLE: &str = "is_transferrable";

pub const ENTRY_POINT_INIT: &str = "init";
pub const ENTRY_POINT_SET_VARIABLES: &str = "set_variables";
pub const ENTRY_POINT_MINT: &str = "mint";
pub const ENTRY_POINT_BURN: &str = "burn";
pub const ENTRY_POINT_TRANSFER: &str = "transfer";
pub const ENTRY_POINT_APPROVE: &str = "approve";
pub const ENTRY_POINT_BALANCE_OF: &str = "balance_of";
//pub const ENTRY_POINT_COLLECTION_NAME: &str = "collection_name";
//pub const ENTRY_POINT_SET_ALLOW_MINTING: &str = "set_allow_minting";
pub const ENTRY_POINT_OWNER_OF: &str = "owner_of";
pub const ENTRY_POINT_GET_APPROVED: &str = "get_approved";
pub const ENTRY_POINT_METADATA: &str = "metadata";
//pub const ENTRY_POINT_OWNED_TOKENS: &str = "owned_tokens";
