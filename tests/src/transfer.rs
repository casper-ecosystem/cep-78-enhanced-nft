use casper_engine_test_support::{
    ExecuteRequestBuilder, LmdbWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
};

use casper_types::{account::AccountHash, contracts::ContractHash, runtime_args, Key};
use cep78::{
    constants::{
        ACL_WHITELIST, APPROVED, ARG_APPROVE_ALL, ARG_COLLECTION_NAME, ARG_OPERATOR,
        ARG_SOURCE_KEY, ARG_SPENDER, ARG_TARGET_KEY, ARG_TOKEN_HASH, ARG_TOKEN_ID,
        ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, BALANCES, ENTRY_POINT_APPROVE, ENTRY_POINT_MINT,
        ENTRY_POINT_REGISTER_OWNER, ENTRY_POINT_REVOKE, ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        ENTRY_POINT_TRANSFER, PAGE_TABLE, TOKEN_OWNERS,
    },
    events::events_ces::{ApprovalRevoked, Transfer},
    modalities::{TokenIdentifier, TransferFilterContractResult}, // events::events_ces::{Approval, ApprovalRevoked, Transfer},
};

use crate::utility::{
    constants::{
        ACCOUNT_1_ADDR, ACCOUNT_1_KEY, ACCOUNT_2_ADDR, ACCOUNT_2_KEY, ACCOUNT_3_ADDR,
        ACCOUNT_3_KEY, ARG_FILTER_CONTRACT_RETURN_VALUE, ARG_NFT_CONTRACT_HASH, ARG_REVERSE_LOOKUP,
        DEFAULT_ACCOUNT_KEY, MINTING_CONTRACT_WASM, MINT_SESSION_WASM, NFT_CONTRACT_WASM,
        NFT_TEST_COLLECTION, NFT_TEST_SYMBOL, TEST_PRETTY_721_META_DATA,
        TRANSFER_FILTER_CONTRACT_WASM, TRANSFER_SESSION_WASM,
    },
    installer_request_builder::{
        InstallerRequestBuilder, MetadataMutability, MintingMode, NFTHolderMode, NFTIdentifierMode,
        NFTMetadataKind, OwnerReverseLookupMode, OwnershipMode, WhitelistMode,
    },
    support::{
        self, assert_expected_error, genesis, get_dictionary_value_from_key,
        get_minting_contract_hash, get_minting_contract_hash_key,
        get_minting_contract_package_hash, get_nft_contract_hash, get_nft_contract_hash_key,
        get_transfer_filter_contract_hash,
    },
};

#[test]
fn should_dissallow_transfer_with_minter_or_assigned_ownership_mode() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_ownership_mode(OwnershipMode::Assigned)
        .with_minting_mode(MintingMode::Installer)
        .with_reporting_mode(OwnerReverseLookupMode::Complete)
        .build();

    builder.exec(install_request).expect_success().commit();

    let token_owner = *DEFAULT_ACCOUNT_ADDR;
    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let actual_owner_balance: u64 = support::get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        BALANCES,
        &token_owner.to_string(),
    );
    let expected_owner_balance = 1u64;
    assert_eq!(actual_owner_balance, expected_owner_balance);

    let token_receiver_key = *ACCOUNT_1_KEY;

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_receiver_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let token_id = 0u64;

    let transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_TRANSFER,
        runtime_args! {
            ARG_SOURCE_KEY => *DEFAULT_ACCOUNT_KEY,
            ARG_TARGET_KEY =>  token_receiver_key,
            ARG_TOKEN_ID => token_id,
        },
    )
    .build();
    builder.exec(transfer_request).expect_failure();

    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(
        actual_error,
        63u16,
        "should not allow transfer when ownership mode is Assigned or Minter",
    );
}

#[test]
fn should_transfer_token_from_sender_to_receiver() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let token_owner = *DEFAULT_ACCOUNT_ADDR;
    let token_owner_key = *DEFAULT_ACCOUNT_KEY;

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => token_owner_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let actual_owner_balance: u64 = support::get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        BALANCES,
        &token_owner.to_string(),
    );
    let expected_owner_balance = 1u64;
    assert_eq!(actual_owner_balance, expected_owner_balance);

    let token_receiver = ACCOUNT_1_ADDR.to_owned();
    let token_receiver_key = *ACCOUNT_1_KEY;

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_receiver_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let token_id = 0u64;

    let transfer_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_ID => token_id,
            ARG_SOURCE_KEY => token_owner_key,
            ARG_TARGET_KEY => token_receiver_key,
        },
    )
    .build();
    builder.exec(transfer_request).expect_success().commit();

    let actual_token_owner = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    );

    assert_eq!(actual_token_owner, token_receiver_key); // Change  token_receiver to token_owner for red test

    let token_receiver_page =
        support::get_token_page_by_id(&builder, &nft_contract_key, &token_receiver_key, token_id);

    assert!(token_receiver_page[0]);

    let actual_sender_balance: u64 = support::get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        BALANCES,
        &token_owner.to_string(),
    );

    let expected_sender_balance = 0u64;
    assert_eq!(actual_sender_balance, expected_sender_balance);

    let actual_receiver_balance: u64 = support::get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        BALANCES,
        &token_receiver.to_string(),
    );
    let expected_receiver_balance = 1u64;
    assert_eq!(actual_receiver_balance, expected_receiver_balance);

    // Expect Transfer event.
    let expected_event = Transfer::new(
        token_owner_key,
        None,
        token_receiver_key,
        &TokenIdentifier::Index(token_id),
    );
    let actual_event: Transfer = support::get_event(&builder, &nft_contract_key, 1).unwrap();
    assert_eq!(actual_event, expected_event, "Expected Transfer event.");
}

fn approve_token_for_transfer_should_add_entry_to_approved_dictionary(
    mut builder: LmdbWasmTestBuilder,
    operator: Option<AccountHash>,
) {
    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let owner_key = *DEFAULT_ACCOUNT_KEY;

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => owner_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let spender_key = *ACCOUNT_1_KEY;
    let token_id = 0u64;

    if let Some(operator) = operator {
        let approval_all_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash,
            ENTRY_POINT_SET_APPROVALL_FOR_ALL,
            runtime_args! {
                ARG_APPROVE_ALL => true,
                ARG_OPERATOR => Key::Account(AccountHash::new(operator.value())),
            },
        )
        .build();
        builder.exec(approval_all_request).expect_success().commit();
    }

    let approving_account = match operator {
        Some(operator) => operator,
        None => *DEFAULT_ACCOUNT_ADDR,
    };

    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        approving_account,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_SPENDER => spender_key
        },
    )
    .build();
    builder.exec(approve_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let actual_approved_key: Option<Key> = support::get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        APPROVED,
        &token_id.to_string(),
    );

    assert_eq!(actual_approved_key, Some(spender_key));

    // Expect Approval event.
    // let expected_event = Approval::new(owner_key, spender_key, TokenIdentifier::Index(token_id));
    // let expected_event_index = if operator.is_some() { 2 } else { 1 };
    // let actual_event: Approval =
    //     support::get_event(&builder, &nft_contract_key, expected_event_index).unwrap();
    // assert_eq!(actual_event, expected_event, "Expected Approval event.");
}

#[test]
fn approve_token_for_transfer_from_an_account_should_add_entry_to_approved_dictionary() {
    let builder = genesis();
    approve_token_for_transfer_should_add_entry_to_approved_dictionary(builder, None)
}

#[test]
fn approve_token_for_transfer_from_an_operator_should_add_entry_to_approved_dictionary() {
    let builder = genesis();
    let operator = ACCOUNT_3_ADDR.to_owned();
    approve_token_for_transfer_should_add_entry_to_approved_dictionary(builder, Some(operator))
}

fn revoke_token_for_transfer_should_remove_entry_to_approved_dictionary(
    mut builder: LmdbWasmTestBuilder,
    operator: Option<AccountHash>,
) {
    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let owner_key = *DEFAULT_ACCOUNT_KEY;

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => owner_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let spender_key = *ACCOUNT_1_KEY;
    let token_id = 0u64;

    if let Some(operator) = operator {
        let approval_all_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash,
            ENTRY_POINT_SET_APPROVALL_FOR_ALL,
            runtime_args! {
                ARG_APPROVE_ALL => true,
                ARG_OPERATOR => Key::Account(AccountHash::new(operator.value())),
            },
        )
        .build();
        builder.exec(approval_all_request).expect_success().commit();
    }

    let approving_account = match operator {
        Some(operator) => operator,
        None => *DEFAULT_ACCOUNT_ADDR,
    };

    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        approving_account,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_SPENDER => spender_key
        },
    )
    .build();
    builder.exec(approve_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let actual_approved_key: Option<Key> = support::get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        APPROVED,
        &token_id.to_string(),
    );

    assert_eq!(actual_approved_key, Some(spender_key));

    let revoke_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REVOKE,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
        },
    )
    .build();
    builder.exec(revoke_request).expect_success().commit();

    let actual_approved_key: Option<Key> = support::get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        APPROVED,
        &token_id.to_string(),
    );

    assert_eq!(actual_approved_key, None);

    // Expect ApprovalRevoked event.
    let expected_event = ApprovalRevoked::new(owner_key, &TokenIdentifier::Index(token_id));
    let expected_event_index = if operator.is_some() { 3 } else { 2 };
    let actual_event: ApprovalRevoked =
        support::get_event(&builder, &nft_contract_key, expected_event_index).unwrap();
    assert_eq!(
        actual_event, expected_event,
        "Expected ApprovalRevoked event."
    );
}

#[test]
fn revoke_token_for_transfer_from_account_should_remove_entry_to_approved_dictionary() {
    let builder = genesis();
    revoke_token_for_transfer_should_remove_entry_to_approved_dictionary(builder, None)
}

#[test]
fn revoke_token_for_transfer_from_operator_should_remove_entry_to_approved_dictionary() {
    let builder = genesis();
    let operator = ACCOUNT_3_ADDR.to_owned();
    revoke_token_for_transfer_should_remove_entry_to_approved_dictionary(builder, Some(operator))
}

#[test]
fn should_dissallow_approving_when_ownership_mode_is_minter_or_assigned() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_ownership_mode(OwnershipMode::Assigned)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let spender_key = *ACCOUNT_1_KEY;
    let token_id = 0u64;

    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_SPENDER => spender_key
        },
    )
    .build();
    builder.exec(approve_request).expect_failure();

    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(
        actual_error,
        63u16,
        "should not allow transfer when ownership mode is Assigned or Minter",
    );
}

fn should_be_able_to_transfer_token(
    mut builder: LmdbWasmTestBuilder,
    operator: Option<AccountHash>,
) {
    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    // mint token for DEFAULT_ACCOUNT_ADDR
    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    // Create a "to approve" spender account account and transfer funds
    let spender_key = *ACCOUNT_1_KEY;
    let token_id = 0u64;

    if let Some(operator) = operator {
        let approval_all_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash,
            ENTRY_POINT_SET_APPROVALL_FOR_ALL,
            runtime_args! {
                ARG_APPROVE_ALL => true,
                ARG_OPERATOR => Key::Account(AccountHash::new(operator.value())),
            },
        )
        .build();
        builder.exec(approval_all_request).expect_success().commit();
    }

    let approving_account = match operator {
        Some(operator) => operator,
        None => *DEFAULT_ACCOUNT_ADDR,
    };

    // Approve spender
    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        approving_account,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_SPENDER => spender_key
        },
    )
    .build();
    builder.exec(approve_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let actual_approved_account: Option<Key> =
        get_dictionary_value_from_key(&builder, &nft_contract_key, APPROVED, &token_id.to_string());

    let expected_approved_account = Some(spender_key);
    assert_eq!(
        actual_approved_account, expected_approved_account,
        "approved account should have been set in dictionary when approved"
    );

    // Create to_account and transfer minted token using spender
    let to_account = *ACCOUNT_2_KEY;

    let register_owner = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => to_account
        },
    )
    .build();

    builder.exec(register_owner).expect_success().commit();

    let token_id = 0u64;
    let token_owner = *DEFAULT_ACCOUNT_KEY;

    let transfer_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_SOURCE_KEY =>  token_owner,
            ARG_TARGET_KEY => to_account,
            ARG_TOKEN_ID => token_id,
        },
    )
    .build();
    builder.exec(transfer_request).expect_success().commit();

    let actual_approved_account_hash: Option<Key> =
        get_dictionary_value_from_key(&builder, &nft_contract_key, APPROVED, &token_id.to_string());

    assert_eq!(
        actual_approved_account_hash, None,
        "approved account should be set to none after a transfer"
    );
}

#[test]
fn should_be_able_to_transfer_token_using_approved_account() {
    let builder = genesis();
    should_be_able_to_transfer_token(builder, None)
}

#[test]
fn should_be_able_to_transfer_token_using_operator() {
    let builder = genesis();
    let operator = ACCOUNT_3_ADDR.to_owned();
    should_be_able_to_transfer_token(builder, Some(operator))
}

#[test]
fn should_dissallow_same_approved_account_to_transfer_token_twice() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    // mint token for DEFAULT_ACCOUNT_ADDR
    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    // Create a "to approve" spender account and transfer funds
    let spender = ACCOUNT_1_ADDR.to_owned();
    let spender_key = *ACCOUNT_1_KEY;
    let token_id = 0u64;

    // Approve spender
    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_SPENDER => spender_key
        },
    )
    .build();
    builder.exec(approve_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let actual_approved_account: Option<Key> =
        get_dictionary_value_from_key(&builder, &nft_contract_key, APPROVED, &token_id.to_string());

    let expected_approved_account = Some(spender_key);

    assert_eq!(
        actual_approved_account, expected_approved_account,
        "approved account should have been set in dictionary when approved"
    );

    // Create to_account and transfer minted token using spender
    let to_account = *ACCOUNT_2_KEY;

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => to_account
        },
    )
    .build();
    builder.exec(register_request).expect_success().commit();

    let token_owner = *DEFAULT_ACCOUNT_KEY;

    let transfer_request = ExecuteRequestBuilder::standard(
        spender,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_ID => token_id,

            ARG_SOURCE_KEY =>  token_owner,
            ARG_TARGET_KEY => to_account,
        },
    )
    .build();
    builder.exec(transfer_request).expect_success().commit();

    // Create to_other_account and transfer minted token using spender
    let to_other_account = *ACCOUNT_3_KEY;

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => to_other_account
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let transfer_request = ExecuteRequestBuilder::standard(
        spender,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_ID => token_id,
            ARG_SOURCE_KEY =>  to_account, // token owner is now ACCOUNT_USER_2
            ARG_TARGET_KEY => to_other_account,
        },
    )
    .build();
    builder.exec(transfer_request).expect_failure();

    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(
        actual_error,
        6u16,
        "should not allow transfer twice for same spender",
    );
}

fn should_disallow_to_transfer_token_using_revoked_hash(
    mut builder: LmdbWasmTestBuilder,
    operator: Option<AccountHash>,
) {
    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    // mint token for DEFAULT_ACCOUNT_ADDR
    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    // Create a "to approve" spender and transfer funds
    let spender = ACCOUNT_1_ADDR.to_owned();
    let spender_key = *ACCOUNT_1_KEY;
    let token_id = 0u64;

    if let Some(operator) = operator {
        let approval_all_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash,
            ENTRY_POINT_SET_APPROVALL_FOR_ALL,
            runtime_args! {
                ARG_APPROVE_ALL => true,
                ARG_OPERATOR => Key::Account(operator),
            },
        )
        .build();
        builder.exec(approval_all_request).expect_success().commit();
    }

    let approving_account = match operator {
        Some(operator) => operator,
        None => *DEFAULT_ACCOUNT_ADDR,
    };

    // Approve spender
    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        approving_account,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_SPENDER => spender_key
        },
    )
    .build();
    builder.exec(approve_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let actual_approved_account: Option<Key> =
        get_dictionary_value_from_key(&builder, &nft_contract_key, APPROVED, &token_id.to_string());

    let expected_approved_account = Some(spender_key);
    assert_eq!(
        actual_approved_account, expected_approved_account,
        "approved account should have been set in dictionary when approved"
    );

    // Create to_account and transfer minted token using account
    let register_owner = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => *ACCOUNT_2_KEY
        },
    )
    .build();

    builder.exec(register_owner).expect_success().commit();

    // Revoke approval
    let revoke_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REVOKE,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
        },
    )
    .build();
    builder.exec(revoke_request).expect_success().commit();

    let token_owner = *DEFAULT_ACCOUNT_KEY;

    let unauthorized_transfer = ExecuteRequestBuilder::standard(
        spender,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_SOURCE_KEY =>  token_owner,
            ARG_TARGET_KEY => *ACCOUNT_2_KEY,
            ARG_TOKEN_ID => token_id,
        },
    )
    .build();
    builder.exec(unauthorized_transfer).expect_failure();

    let error = builder
        .get_error()
        .expect("previous execution must have failed");

    assert_expected_error(
        error,
        6u16,
        "transfer from revoked account must raise InvalidTokenOwner",
    );

    let actual_approved_account_hash: Option<Key> =
        get_dictionary_value_from_key(&builder, &nft_contract_key, APPROVED, &token_id.to_string());

    assert_eq!(
        actual_approved_account_hash, None,
        "approved account should be unset after revoke and a failed transfer"
    );
}

#[test]
fn should_disallow_to_transfer_token_using_revoked_account() {
    let builder = genesis();
    should_disallow_to_transfer_token_using_revoked_hash(builder, None)
}

#[test]
fn should_disallow_to_transfer_token_using_revoked_operator() {
    let builder = genesis();
    let operator = ACCOUNT_3_ADDR.to_owned();
    should_disallow_to_transfer_token_using_revoked_hash(builder, Some(operator))
}

// This test intends to test that the approve entry_point still behaves correctly with "operator"
// deprecated argument (now "spender")
#[test]
fn should_be_able_to_approve_with_deprecated_operator_argument() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    // mint token for DEFAULT_ACCOUNT_ADDR
    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    // Create a "to approve" spender account account and transfer funds
    let spender_key = *ACCOUNT_1_KEY;
    let token_id = 0u64;

    // Approve spender
    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_OPERATOR => spender_key // ARG_OPERATOR argument is deprecated and replaced by ARG_SPENDER
        },
    )
    .build();
    builder.exec(approve_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);
    let actual_approved_account: Option<Key> =
        get_dictionary_value_from_key(&builder, &nft_contract_key, APPROVED, &token_id.to_string());

    let expected_approved_account = Some(spender_key);
    assert_eq!(
        actual_approved_account, expected_approved_account,
        "approved account should have been set in dictionary when approved"
    );
}

#[test]
fn should_transfer_between_contract_to_account() {
    let mut builder = genesis();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    // TODO check
    let minting_contract_hash: ContractHash = get_minting_contract_hash(&builder).into();
    let contract_whitelist = vec![Key::from(minting_contract_hash)];

    let minting_contract_key: Key = get_minting_contract_hash_key(&builder);

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_minting_mode(MintingMode::Acl)
        .with_acl_whitelist(contract_whitelist)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let is_whitelisted_account = support::get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &minting_contract_hash.to_string(),
    );

    assert!(is_whitelisted_account, "acl whitelist is incorrectly set");

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => minting_contract_key,
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        ARG_REVERSE_LOOKUP => true
    };

    let minting_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash.into(),
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder.exec(minting_request).expect_success().commit();

    let token_id = 0u64;

    let actual_token_owner: Key = get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    );

    assert_eq!(minting_contract_key, actual_token_owner);

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let transfer_runtime_arguments = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_ID => token_id,
        ARG_SOURCE_KEY => minting_contract_key,
        ARG_TARGET_KEY => *DEFAULT_ACCOUNT_KEY,
    };

    let transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash.into(),
        ENTRY_POINT_TRANSFER,
        transfer_runtime_arguments,
    )
    .build();

    builder.exec(transfer_request).expect_success().commit();

    let updated_token_owner: Key = get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    );

    assert_eq!(*DEFAULT_ACCOUNT_KEY, updated_token_owner);
}

#[test]
fn should_prevent_transfer_when_caller_is_not_owner() {
    let mut builder = genesis();

    // Create an account that is not the owner of the NFT to transfer the token itself.
    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_holder_mode(NFTHolderMode::Accounts)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_id = 0u64;

    let actual_token_owner: Key = get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    );

    assert_eq!(*DEFAULT_ACCOUNT_KEY, actual_token_owner);

    let unauthorized_transfer = ExecuteRequestBuilder::standard(
        ACCOUNT_3_ADDR.to_owned(),
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_ID => token_id,
            ARG_SOURCE_KEY => *DEFAULT_ACCOUNT_KEY,
            ARG_TARGET_KEY => *ACCOUNT_3_KEY
        },
    )
    .build();

    builder.exec(unauthorized_transfer).expect_failure();

    let error = builder
        .get_error()
        .expect("previous execution must have failed");

    assert_expected_error(
        error,
        6u16,
        "transfer from another account must raise InvalidTokenOwner",
    );
}

#[test]
fn should_transfer_token_in_hash_identifier_mode() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_total_token_supply(10u64)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA ,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_hash: String =
        base16::encode_lower(&support::create_blake2b_hash(TEST_PRETTY_721_META_DATA));

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => Key::Account(AccountHash::new([3u8;32]))
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let transfer_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_HASH => token_hash,
            ARG_SOURCE_KEY => *DEFAULT_ACCOUNT_KEY,
            ARG_TARGET_KEY =>  Key::Account(AccountHash::new([3u8;32])),
        },
    )
    .build();

    builder.exec(transfer_request).expect_success().commit();
}

#[test]
fn should_not_allow_non_approved_contract_to_transfer() {
    let mut builder = genesis();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let minting_contract_hash = get_minting_contract_hash(&builder);
    let minting_contract_key: Key = Key::Hash(minting_contract_hash.value());

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Mixed)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA ,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_receiver_key = *ACCOUNT_1_KEY;

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_receiver_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let token_id = 0u64;

    let transfer_runtime_arguments = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_ID => token_id,
        ARG_SOURCE_KEY => *DEFAULT_ACCOUNT_KEY,
        ARG_TARGET_KEY => token_receiver_key,
    };

    let non_approved_transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_TRANSFER,
        transfer_runtime_arguments.clone(),
    )
    .build();

    builder.exec(non_approved_transfer_request).expect_failure();

    let error = builder
        .get_error()
        .expect("non approved transfer must have failed");

    assert_expected_error(error, 6u16, "InvalidTokenOwner(6) must be raised");

    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_SPENDER => minting_contract_key
        },
    )
    .build();

    builder.exec(approve_request).expect_success().commit();

    let approved_transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_TRANSFER,
        transfer_runtime_arguments,
    )
    .build();

    builder
        .exec(approved_transfer_request)
        .expect_success()
        .commit();
}

#[test]
fn transfer_should_correctly_track_page_table_entries() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let number_of_tokens_pre_migration = 20usize;
    for _ in 0..number_of_tokens_pre_migration {
        let mint_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            MINT_SESSION_WASM,
            runtime_args! {
                ARG_NFT_CONTRACT_HASH => nft_contract_key,
                ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
                ARG_TOKEN_META_DATA => "",
                ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
            },
        )
        .build();

        builder.exec(mint_request).expect_success().commit();
    }

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => *ACCOUNT_1_KEY
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let transfer_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_ID => 11u64,
            ARG_SOURCE_KEY => *DEFAULT_ACCOUNT_KEY,
            ARG_TARGET_KEY =>  *ACCOUNT_1_KEY,
        },
    )
    .build();

    builder.exec(transfer_request).expect_success().commit();

    let account_user_1_page_table = support::get_dictionary_value_from_key::<Vec<bool>>(
        &builder,
        &nft_contract_key,
        PAGE_TABLE,
        &ACCOUNT_1_ADDR.to_string(),
    );

    assert!(account_user_1_page_table[0])
}

#[test]
fn should_prevent_transfer_to_unregistered_owner() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(1000u64)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_reporting_mode(OwnerReverseLookupMode::Complete)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => "",
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_id = 0u64;
    let token_receiver_key = *ACCOUNT_1_KEY;

    let transfer_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_ID => token_id,
            ARG_SOURCE_KEY => *DEFAULT_ACCOUNT_KEY,
            ARG_TARGET_KEY => token_receiver_key,
        },
    )
    .build();

    builder.exec(transfer_request).expect_failure();

    let error = builder.get_error().expect("must have error");

    assert_expected_error(error, 128u16, "must raise unregistered owner in transfer");
}

#[test]
fn should_transfer_token_from_sender_to_receiver_with_transfer_only_reporting() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_reporting_mode(OwnerReverseLookupMode::TransfersOnly)
        .build();

    builder.exec(install_request).expect_success().commit();

    let token_owner = *DEFAULT_ACCOUNT_ADDR;
    let token_owner_key = *DEFAULT_ACCOUNT_KEY;

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_owner_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let mint_runtime_args = runtime_args! {
        ARG_TOKEN_OWNER => token_owner_key,
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
    };

    let minting_request = ExecuteRequestBuilder::contract_call_by_hash(
        token_owner,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder.exec(minting_request).expect_success().commit();

    let actual_owner_balance: u64 = support::get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        BALANCES,
        &token_owner.to_string(),
    );
    let expected_owner_balance = 1u64;
    assert_eq!(actual_owner_balance, expected_owner_balance);

    let token_receiver_key = ACCOUNT_1_KEY.to_owned();

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_receiver_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let transfer_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_ID => 0u64,
            ARG_SOURCE_KEY => token_owner_key,
            ARG_TARGET_KEY =>  token_receiver_key,
        },
    )
    .build();
    builder.exec(transfer_request).expect_success().commit();

    let actual_token_owner = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &0u64.to_string(),
    );

    assert_eq!(actual_token_owner, token_receiver_key);

    let token_receiver_page =
        support::get_token_page_by_id(&builder, &nft_contract_key, &token_receiver_key, 0u64);

    assert!(token_receiver_page[0]);

    let actual_sender_balance: u64 = support::get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        BALANCES,
        &token_owner.to_string(),
    );
    let expected_sender_balance = 0u64;
    assert_eq!(actual_sender_balance, expected_sender_balance);

    let actual_receiver_balance: u64 = support::get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        BALANCES,
        &ACCOUNT_1_ADDR.to_string(),
    );
    let expected_receiver_balance = 1u64;
    assert_eq!(actual_receiver_balance, expected_receiver_balance);
}

#[test]
fn disallow_owner_to_approve_itself() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let owner_key = *DEFAULT_ACCOUNT_KEY;

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => owner_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_id = 0u64;

    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_SPENDER => *DEFAULT_ACCOUNT_KEY
        },
    )
    .build();
    builder.exec(approve_request).expect_failure();

    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(
        actual_error,
        1u16,
        "should not allow an owner to approve itself",
    );
}

#[test]
fn disallow_operator_to_approve_itself() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(2u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let owner_key = *DEFAULT_ACCOUNT_KEY;

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => owner_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_id = 0u64;
    let operator = ACCOUNT_3_ADDR.to_owned();
    let operator_key = *ACCOUNT_3_KEY;

    let approval_all_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => operator_key,
        },
    )
    .build();
    builder.exec(approval_all_request).expect_success().commit();

    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        operator,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_SPENDER => operator_key
        },
    )
    .build();
    builder.exec(approve_request).expect_failure();

    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(
        actual_error,
        1u16,
        "should not allow an operator to approve itself",
    );
}

#[test]
fn disallow_owner_to_approve_for_all_itself() {
    let mut builder = genesis();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(2u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let owner_key = *DEFAULT_ACCOUNT_KEY;

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => owner_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let approval_all_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => *DEFAULT_ACCOUNT_KEY,
        },
    )
    .build();
    builder.exec(approval_all_request).expect_failure();

    let actual_error = builder.get_error().expect("must have error");
    support::assert_expected_error(
        actual_error,
        1u16,
        "should not allow an owner to approve_for_all itself",
    );
}

#[test]
fn check_transfers_with_transfer_filter_contract_modes() {
    let mut builder = genesis();

    let transfer_filter_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_FILTER_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(transfer_filter_contract_install_request)
        .expect_success()
        .commit();

    let transfer_filter_contract_hash = get_transfer_filter_contract_hash(&builder);

    let transfer_filter_contract_set_return_value_request =
        ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            transfer_filter_contract_hash,
            "set_return_value",
            runtime_args! {
                ARG_FILTER_CONTRACT_RETURN_VALUE => TransferFilterContractResult::DenyTransfer as u8
            },
        )
        .build();

    builder
        .exec(transfer_filter_contract_set_return_value_request)
        .expect_success()
        .commit();

    let token_supply = 2u64;

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(token_supply)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_transfer_filter_contract(Key::Hash(transfer_filter_contract_hash.value()))
        .build();

    builder.exec(install_request).expect_success().commit();

    let token_owner_key = *DEFAULT_ACCOUNT_KEY;

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    for _i in 0..token_supply {
        let mint_session_call = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            MINT_SESSION_WASM,
            runtime_args! {
                ARG_NFT_CONTRACT_HASH => nft_contract_key,
                ARG_TOKEN_OWNER => token_owner_key,
                ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
                ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
            },
        )
        .build();

        builder.exec(mint_session_call).expect_success().commit();
    }

    let token_receiver_key = *ACCOUNT_1_KEY;

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_receiver_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let token_id = 0u64;

    let transfer_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_ID => token_id,
            ARG_SOURCE_KEY => token_owner_key,
            ARG_TARGET_KEY => token_receiver_key,
        },
    )
    .build();

    builder.exec(transfer_request).expect_failure().commit();

    let error = builder.get_error().expect("must have error");
    support::assert_expected_error(
        error,
        159u16,
        "should not allow transfer when transfer filter returns TransferFilterContractResult::DenyTransfer",
    );

    let transfer_filter_contract_set_return_value_request =
        ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            transfer_filter_contract_hash,
            "set_return_value",
            runtime_args! {
                ARG_FILTER_CONTRACT_RETURN_VALUE => TransferFilterContractResult::ProceedTransfer as u8
            },
        )
        .build();

    builder
        .exec(transfer_filter_contract_set_return_value_request)
        .expect_success()
        .commit();

    let transfer_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_ID => token_id,
            ARG_SOURCE_KEY => token_owner_key,
            ARG_TARGET_KEY => token_receiver_key,
        },
    )
    .build();

    builder.exec(transfer_request).expect_success().commit();

    let transfer_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_ID => token_id,
            // NB: token_receiver and token_owner are swapped
            ARG_SOURCE_KEY => token_receiver_key,
            ARG_TARGET_KEY => token_owner_key,
        },
    )
    .build();

    builder.exec(transfer_request).expect_failure().commit();

    let error = builder.get_error().expect("must have error");
    assert_expected_error(error, 6u16, "InvalidTokenOwner(6) must be raised");

    let token_id = 1u64;

    let transfer_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_ID => token_id,

            ARG_SOURCE_KEY => token_owner_key,
            ARG_TARGET_KEY => token_receiver_key,
        },
    )
    .build();

    builder.exec(transfer_request).expect_success().commit();
}

#[test]
fn should_disallow_transfer_from_contract_with_package_operator_mode_without_operator() {
    let mut builder = genesis();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let package_operator_mode = true;

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Mixed)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_package_operator_mode(package_operator_mode)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA ,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_receiver_key = *ACCOUNT_1_KEY;

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_receiver_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let token_id = 0u64;

    let transfer_runtime_arguments = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_ID => token_id,
        ARG_SOURCE_KEY => *DEFAULT_ACCOUNT_KEY,
        ARG_TARGET_KEY => token_receiver_key,
    };

    let minting_contract_hash = get_minting_contract_hash(&builder);

    let non_approved_transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_TRANSFER,
        transfer_runtime_arguments,
    )
    .build();

    builder.exec(non_approved_transfer_request).expect_failure();

    let error = builder
        .get_error()
        .expect("non approved transfer must have failed");

    assert_expected_error(error, 6u16, "InvalidTokenOwner(6) must be raised");
}

#[test]
fn should_disallow_transfer_from_contract_without_package_operator_mode_with_package_as_operator() {
    let mut builder = genesis();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Mixed)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA ,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();
    let token_receiver_key = *ACCOUNT_1_KEY;

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_receiver_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let token_id = 0u64;

    let transfer_runtime_arguments = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_ID => token_id,
        ARG_SOURCE_KEY => *DEFAULT_ACCOUNT_KEY,
        ARG_TARGET_KEY => token_receiver_key,
    };

    let minting_contract_package_hash = get_minting_contract_package_hash(&builder);

    let set_approve_for_all_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => Key::Hash(minting_contract_package_hash.value())
        },
    )
    .build();

    builder
        .exec(set_approve_for_all_request)
        .expect_success()
        .commit();

    let minting_contract_hash = get_minting_contract_hash(&builder);

    let non_approved_transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_TRANSFER,
        transfer_runtime_arguments,
    )
    .build();

    builder.exec(non_approved_transfer_request).expect_failure();

    let error = builder
        .get_error()
        .expect("non approved transfer must have failed");

    assert_expected_error(error, 6u16, "InvalidTokenOwner(6) must be raised");
}

#[test]
fn should_allow_transfer_from_contract_with_package_operator_mode_with_operator() {
    let mut builder = genesis();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let package_operator_mode = true;

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Mixed)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_package_operator_mode(package_operator_mode)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA ,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_receiver_key = *ACCOUNT_1_KEY;

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_receiver_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let token_id = 0u64;

    let transfer_runtime_arguments = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_ID => token_id,
        ARG_SOURCE_KEY => *DEFAULT_ACCOUNT_KEY,
        ARG_TARGET_KEY => token_receiver_key,
    };

    let minting_contract_package_hash = get_minting_contract_package_hash(&builder);

    let set_approve_for_all_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => Key::Hash(minting_contract_package_hash.value())
        },
    )
    .build();

    builder
        .exec(set_approve_for_all_request)
        .expect_success()
        .commit();

    let minting_contract_hash = get_minting_contract_hash(&builder);

    let approved_transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_TRANSFER,
        transfer_runtime_arguments,
    )
    .build();

    builder
        .exec(approved_transfer_request)
        .expect_success()
        .commit();

    let actual_token_owner = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    );

    assert_eq!(actual_token_owner, token_receiver_key);
}

#[test]
fn should_disallow_package_operator_to_approve_without_package_operator_mode() {
    let mut builder = genesis();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Mixed)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA ,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_receiver_key = *ACCOUNT_1_KEY;

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_receiver_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let minting_contract_hash = get_minting_contract_hash(&builder);
    let minting_contract_package_hash = get_minting_contract_package_hash(&builder);

    let set_approve_for_all_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => Key::Hash(minting_contract_package_hash.value())
        },
    )
    .build();

    builder
        .exec(set_approve_for_all_request)
        .expect_success()
        .commit();

    let token_id = 0u64;
    let spender_key = *ACCOUNT_2_KEY;

    let approve_runtime_arguments = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_ID => token_id,
        ARG_SPENDER => spender_key
    };

    let non_approved_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_APPROVE,
        approve_runtime_arguments,
    )
    .build();

    builder.exec(non_approved_approve_request).expect_failure();

    let error = builder.get_error().expect("non approved must have failed");

    assert_expected_error(error, 6u16, "InvalidTokenOwner(6) must be raised");

    let actual_token_owner = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    );

    assert_eq!(actual_token_owner, *DEFAULT_ACCOUNT_KEY);
}

#[test]
fn should_allow_package_operator_to_approve_with_package_operator_mode() {
    let mut builder = genesis();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let package_operator_mode = true;

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Mixed)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_package_operator_mode(package_operator_mode)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA ,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_receiver_key = *ACCOUNT_1_KEY;

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_receiver_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let minting_contract_package_hash = get_minting_contract_package_hash(&builder);

    let set_approve_for_all_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => Key::Hash(minting_contract_package_hash.value())
        },
    )
    .build();

    builder
        .exec(set_approve_for_all_request)
        .expect_success()
        .commit();

    let token_id = 0u64;
    let spender = ACCOUNT_2_ADDR.to_owned();
    let spender_key = *ACCOUNT_2_KEY;

    let approve_runtime_arguments = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_ID => token_id,
        ARG_SPENDER => spender_key
    };

    let minting_contract_hash = get_minting_contract_hash(&builder);

    // Approval by contract
    let approved_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_APPROVE,
        approve_runtime_arguments,
    )
    .build();

    builder
        .exec(approved_approve_request)
        .expect_success()
        .commit();

    let transfer_runtime_arguments = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_ID => token_id,
        ARG_SOURCE_KEY => *DEFAULT_ACCOUNT_KEY,
        ARG_TARGET_KEY => token_receiver_key,
    };

    let approved_transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
        spender,
        nft_contract_hash,
        ENTRY_POINT_TRANSFER,
        transfer_runtime_arguments,
    )
    .build();

    builder
        .exec(approved_transfer_request)
        .expect_success()
        .commit();

    let actual_token_owner = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    );

    assert_eq!(actual_token_owner, token_receiver_key);
}

#[test]
fn should_allow_account_to_approve_spender_with_package_operator() {
    let mut builder = genesis();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let package_operator_mode = true;

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Mixed)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_package_operator_mode(package_operator_mode)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA ,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_receiver_key = *ACCOUNT_1_KEY;

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_receiver_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let minting_contract_package_hash = get_minting_contract_package_hash(&builder);

    let set_approve_for_all_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => Key::Hash(minting_contract_package_hash.value())
        },
    )
    .build();

    builder
        .exec(set_approve_for_all_request)
        .expect_success()
        .commit();

    let token_id = 0u64;
    let spender_key = *ACCOUNT_2_KEY;

    let approve_runtime_arguments = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_ID => token_id,
        ARG_SPENDER => spender_key
    };

    let approved_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        approve_runtime_arguments,
    )
    .build();

    builder
        .exec(approved_approve_request)
        .expect_success()
        .commit();

    let transfer_runtime_arguments = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_ID => token_id,
        ARG_SOURCE_KEY => *DEFAULT_ACCOUNT_KEY,
        ARG_TARGET_KEY => token_receiver_key,
    };

    let approved_transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
        *ACCOUNT_2_ADDR,
        nft_contract_hash,
        ENTRY_POINT_TRANSFER,
        transfer_runtime_arguments,
    )
    .build();

    builder
        .exec(approved_transfer_request)
        .expect_success()
        .commit();

    let actual_token_owner = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    );

    assert_eq!(actual_token_owner, token_receiver_key);
}

#[test]
fn should_allow_package_operator_to_revoke_with_package_operator_mode() {
    let mut builder = genesis();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let package_operator_mode = true;

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_holder_mode(NFTHolderMode::Mixed)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_package_operator_mode(package_operator_mode)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = get_nft_contract_hash_key(&builder);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => *DEFAULT_ACCOUNT_KEY,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA ,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let token_receiver_key = *ACCOUNT_1_KEY;

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_receiver_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let minting_contract_package_hash = get_minting_contract_package_hash(&builder);

    let set_approve_for_all_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => Key::Hash(minting_contract_package_hash.value())
        },
    )
    .build();

    builder
        .exec(set_approve_for_all_request)
        .expect_success()
        .commit();

    let token_id = 0u64;
    let spender = ACCOUNT_2_ADDR.to_owned();
    let spender_key = *ACCOUNT_2_KEY;

    let approve_runtime_arguments = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_ID => token_id,
        ARG_SPENDER => spender_key
    };

    let approved_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        approve_runtime_arguments,
    )
    .build();

    builder
        .exec(approved_approve_request)
        .expect_success()
        .commit();

    let revoke_runtime_arguments = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_ID => token_id,
    };

    let minting_contract_hash = get_minting_contract_hash(&builder);

    // Revoke approval by contract
    let approved_revoke_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_REVOKE,
        revoke_runtime_arguments,
    )
    .build();

    builder
        .exec(approved_revoke_request)
        .expect_success()
        .commit();

    let transfer_runtime_arguments = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_ID => token_id,
        ARG_SOURCE_KEY => *DEFAULT_ACCOUNT_KEY,
        ARG_TARGET_KEY => token_receiver_key,
    };

    let non_approved_transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
        spender,
        nft_contract_hash,
        ENTRY_POINT_TRANSFER,
        transfer_runtime_arguments,
    )
    .build();

    builder.exec(non_approved_transfer_request).expect_failure();

    let error = builder.get_error().expect("transfer must have failed");

    assert_expected_error(error, 6u16, "InvalidTokenOwner(6) must be raised");

    let actual_token_owner = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    );

    assert_eq!(actual_token_owner, *DEFAULT_ACCOUNT_KEY);
}
