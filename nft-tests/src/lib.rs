#[cfg(test)]
mod tests {

    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder,
        ARG_AMOUNT, DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_INITIAL_BALANCE,
        DEFAULT_ACCOUNT_PUBLIC_KEY, DEFAULT_GENESIS_CONFIG, DEFAULT_GENESIS_CONFIG_HASH,
        DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
    };
    use casper_execution_engine::{
        core::{
            engine_state::{
                run_genesis_request::RunGenesisRequest, Error, ExecuteRequest, GenesisAccount,
            },
            execution,
        },
        storage::global_state::in_memory::InMemoryGlobalState,
    };
    use casper_types::{
        account::AccountHash, bytesrepr::FromBytes, runtime_args, system::mint, ApiError, CLTyped,
        CLValue, ContractHash, Key, Motes, PublicKey, RuntimeArgs, SecretKey, URef, U256,
    };

    const COLLECTION_NAME: &str = "collection_name";
    const NFT_CONTRACT_WASM: &str = "nft-installer.wasm";
    const CONTRACT_NAME: &str = "nft_contract";

    const INSTALLER: &str = "installer";
    const NFT_TEST_COLLECTION: &str = "nft_test";
    const NFT_TEST_SYMBOL: &str = "TEST";

    pub const ENTRY_POINT_INIT: &str = "init";
    pub const ENTRY_POINT_SET_VARIABLES: &str = "set_variables";
    pub const ENTRY_POINT_MINT: &str = "mint";
    pub const ENTRY_POINT_BURN: &str = "burn";
    pub const ENTRY_POINT_TRANSFER: &str = "transfer";
    const ENTRY_POINT_APPROVE: &str = "approve";
    pub const ENTRY_POINT_BALANCE_OF: &str = "balance_of";

    const ARG_COLLECTION_NAME: &str = "collection_name";
    const ARG_COLLECTION_SYMBOL: &str = "collection_symbol";
    const ARG_TOTAL_TOKEN_SUPPLY: &str = "total_token_supply";
    const ARG_ALLOW_MINTING: &str = "allow_minting";
    const ARG_PUBLIC_MINTING: &str = "public_minting";
    const NUMBER_OF_MINTED_TOKENS: &str = "number_of_minted_tokens";
    const ARG_TOKEN_META_DATA: &str = "token_meta_data";
    const TOKEN_META_DATA: &str = "token_meta_data";
    pub const ARG_TOKEN_OWNER: &str = "token_owner";

    const TOKEN_OWNERS: &str = "token_owners";
    const OWNED_TOKENS: &str = "owned_tokens";
    const BURNT_TOKENS: &str = "burnt_tokens";
    const APPROVED_FOR_TRANSFER: &str = "approved_for_transfer";

    const ARG_TO_ACCOUNT_HASH: &str = "to_account_hash";
    pub const ARG_FROM_ACCOUNT_HASH: &str = "from_account_hash";
    const ARG_TOKEN_ID: &str = "token_id";
    const ARG_APPROVE_TRANSFER_FOR_ACCOUNT_HASH: &str = "approve_transfer_for_account_hash";

    const ACCOUNT_USER_1: [u8; 32] = [1u8; 32];
    const ACCOUNT_USER_2: [u8; 32] = [2u8; 32];
    const TEST_META_DATA: &str = "test meta";

    #[test]
    fn should_install_contract() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_collection_name(NFT_TEST_COLLECTION.to_string())
                .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
                .with_total_token_supply(U256::from(1u64))
                .build();

        builder.exec(install_request).expect_success().commit();

        let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_key = account
            .named_keys()
            .get(CONTRACT_NAME)
            .expect("must have key in named keys");

        let query_result: String = query_stored_value(
            &mut builder,
            *nft_contract_key,
            vec![ARG_COLLECTION_NAME.to_string()],
        );

        assert_eq!(
            query_result,
            NFT_TEST_COLLECTION.to_string(),
            "collection_name initialized at installation should exist"
        );

        let query_result: String = query_stored_value(
            &mut builder,
            *nft_contract_key,
            vec![ARG_COLLECTION_SYMBOL.to_string()],
        );

        assert_eq!(
            query_result,
            NFT_TEST_SYMBOL.to_string(),
            "collection_symbol initialized at installation should exist"
        );

        let query_result: U256 = query_stored_value(
            &mut builder,
            *nft_contract_key,
            vec![ARG_TOTAL_TOKEN_SUPPLY.to_string()],
        );

        assert_eq!(
            query_result,
            U256::from(1u64),
            "total_token_supply initialized at installation should exist"
        );

        let query_result: bool = query_stored_value(
            &mut builder,
            *nft_contract_key,
            vec![ARG_ALLOW_MINTING.to_string()],
        );

        assert!(query_result, "Allow minting should default to true");

        let query_result: bool = query_stored_value(
            &mut builder,
            *nft_contract_key,
            vec![ARG_PUBLIC_MINTING.to_string()],
        );

        assert!(!query_result, "public minting should default to false");

        let query_result: U256 = query_stored_value(
            &mut builder,
            *nft_contract_key,
            vec![NUMBER_OF_MINTED_TOKENS.to_string()],
        );

        assert_eq!(
            query_result,
            U256::zero(),
            "number_of_minted_tokens initialized at installation should exist"
        );
    }

    #[test]
    fn calling_init_entrypoint_after_intallation_should_error() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request_builder =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_total_token_supply(U256::from(2u64));
        builder
            .exec(install_request_builder.build())
            .expect_success()
            .commit();

        let init_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_INIT,
            runtime_args! {
                ARG_COLLECTION_NAME => "collection_name".to_string(),
                ARG_COLLECTION_SYMBOL => "collection_symbol".to_string(),
                ARG_TOTAL_TOKEN_SUPPLY => "total_token_supply".to_string(),
                ARG_ALLOW_MINTING => true,
                ARG_PUBLIC_MINTING => false,
            },
        )
        .build();
        builder.exec(init_request).expect_failure();

        let error = builder.get_error().expect("must have error");

        assert_expected_error(error, 58u16);
    }

    // This test is already done should_install_contract() test.
    // On the other hand we do not test setting allow_minting to false and public minting to true
    // #[test]
    // fn should_default_allow_minting() {
    //     let mut builder = InMemoryWasmTestBuilder::default();
    //     builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    //     let install_builder =
    //         InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM);
    //     // The allow_minting arg is defaulted to true if user does not provide value.
    //     builder.exec(install_builder.build()).expect_success();
    // }

    #[test]
    fn should_reject_invalid_typed_name() {
        let install_request_builder =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_invalid_collection_name(
                    CLValue::from_t::<U256>(U256::from(0)).expect("expected CLValue"),
                );

        assert_expected_invalid_installer_request(install_request_builder, 18);
    }

    #[test]
    fn should_reject_invalid_typed_symbol() {
        let install_request_builder =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_invalid_collection_symbol(
                    CLValue::from_t::<U256>(U256::from(0)).expect("expected CLValue"),
                );

        assert_expected_invalid_installer_request(install_request_builder, 24);
    }

    #[test]
    fn should_reject_invalid_typed_total_token_supply() {
        let install_request_builder =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_invalid_total_token_supply(
                    CLValue::from_t::<String>("".to_string()).expect("expected CLValue"),
                );
        assert_expected_invalid_installer_request(install_request_builder, 26);
    }

    #[test]
    fn should_disallaw_minting_when_allow_minting_is_set_to_false() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request_builder =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_total_token_supply(U256::from(2u64))
                .with_allowing_minting(Some(false));

        builder
            .exec(install_request_builder.build())
            .expect_success()
            .commit();

        let mint_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
            },
        )
        .build();
        builder.exec(mint_request).expect_failure();

        // Error should be MintingIsPaused=59
        let actual_error = builder.get_error().expect("must have error");
        assert_expected_error(actual_error, 59u16);
    }

    #[test]
    fn mint_should_increment_number_of_minted_tokens_by_one_and_add_public_key_to_token_owners() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request_builder =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_total_token_supply(U256::from(2u64));
        builder
            .exec(install_request_builder.build())
            .expect_success()
            .commit();

        let mint_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
            },
        )
        .build();
        builder.exec(mint_request).expect_success().commit();

        //Let's start querying
        let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_key = account
            .named_keys()
            .get(CONTRACT_NAME)
            .expect("must have key in named keys");

        //mint should have incremented number_of_minted_tokens by one
        let query_result: U256 = query_stored_value(
            &mut builder,
            *nft_contract_key,
            vec![NUMBER_OF_MINTED_TOKENS.to_string()],
        );

        assert_eq!(
            query_result,
            U256::one(),
            "number_of_minted_tokens initialized at installation should have incremented by one"
        );

        let minter_account_hash = get_dictionary_value_from_key::<String>(
            &builder,
            nft_contract_key,
            TOKEN_OWNERS,
            &U256::zero().to_string(),
        );

        assert_eq!(
            DEFAULT_ACCOUNT_ADDR.clone().to_string(),
            minter_account_hash
        );

        let actual_token_ids = get_dictionary_value_from_key::<Vec<U256>>(
            &builder,
            nft_contract_key,
            OWNED_TOKENS,
            &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
        );

        let expected_token_ids = vec![U256::zero()];
        assert_eq!(expected_token_ids, actual_token_ids);

        // If total_token_supply is initialized to 1 the following test should fail.
        // If we set total_token_supply > 1 it should pass
        let mint_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
            },
        )
        .build();
        builder.exec(mint_request).expect_success().commit();
    }

    #[test]
    fn mint_should_correctly_set_meta_data() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request_builder =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_total_token_supply(U256::from(2));
        builder
            .exec(install_request_builder.build())
            .expect_success()
            .commit();

        let mint_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_META_DATA=> TEST_META_DATA.to_string(),
            },
        )
        .build();
        builder.exec(mint_request).expect_success().commit();

        //Let's start querying
        let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_key = account
            .named_keys()
            .get(CONTRACT_NAME)
            .expect("must have key in named keys");

        let actual_token_meta_data = get_dictionary_value_from_key::<String>(
            &builder,
            nft_contract_key,
            TOKEN_META_DATA,
            &U256::zero().to_string(),
        );

        assert_eq!(actual_token_meta_data, TEST_META_DATA);
    }

    #[test]
    fn should_allow_public_minting_with_flag_set_to_true() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_total_token_supply(U256::from(100u64))
                .with_public_minting(Some(true))
                .build();
        builder.exec(install_request).expect_success().commit();

        let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_key = installing_account
            .named_keys()
            .get(CONTRACT_NAME)
            .expect("must have key in named keys");
        let nft_contract_hash = nft_contract_key
            .into_hash()
            .expect("must convert to hash addr");

        let (_account_1_secret_key, account_1_public_key) = create_dummy_key_pair(ACCOUNT_USER_1);

        let transfer_to_account_1 = ExecuteRequestBuilder::transfer(
            *DEFAULT_ACCOUNT_ADDR,
            runtime_args! {
                mint::ARG_AMOUNT => 100_000_000_000_000u64,
                mint::ARG_TARGET => account_1_public_key.to_account_hash(),
                mint::ARG_ID => Option::<u64>::None,
            },
        )
        .build();

        builder
            .exec(transfer_to_account_1)
            .expect_success()
            .commit();

        let public_minting_status = query_stored_value::<bool>(
            &mut builder,
            *nft_contract_key,
            vec![ARG_PUBLIC_MINTING.to_string()],
        );

        assert!(
            public_minting_status,
            "public minting should be set to true"
        );

        let nft_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            account_1_public_key.to_account_hash(),
            ContractHash::new(nft_contract_hash),
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
            },
        )
        .build();

        builder.exec(nft_mint_request).expect_success().commit();

        let minter_public_key = get_dictionary_value_from_key::<String>(
            &builder,
            nft_contract_key,
            TOKEN_OWNERS,
            &U256::zero().to_string(),
        );

        assert_eq!(
            account_1_public_key.to_account_hash().to_string(),
            minter_public_key
        );
    }

    #[test]
    fn should_disallow_public_minting_with_flag_set_to_false() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_total_token_supply(U256::from(100u64))
                .with_public_minting(Some(false))
                .build();
        builder.exec(install_request).expect_success().commit();

        let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_key = installing_account
            .named_keys()
            .get(CONTRACT_NAME)
            .expect("must have key in named keys");
        let nft_contract_hash = nft_contract_key
            .into_hash()
            .expect("must convert to hash addr");

        let (_account_1_secret_key, account_1_public_key) = create_dummy_key_pair(ACCOUNT_USER_1);

        let transfer_to_account_1 = ExecuteRequestBuilder::transfer(
            *DEFAULT_ACCOUNT_ADDR,
            runtime_args! {
                mint::ARG_AMOUNT => 100_000_000_000_000u64,
                mint::ARG_TARGET => account_1_public_key.to_account_hash(),
                mint::ARG_ID => Option::<u64>::None,
            },
        )
        .build();

        builder
            .exec(transfer_to_account_1)
            .expect_success()
            .commit();

        let public_minting_status = query_stored_value::<bool>(
            &mut builder,
            *nft_contract_key,
            vec![ARG_PUBLIC_MINTING.to_string()],
        );

        assert!(
            !public_minting_status,
            "public minting should be set to false"
        );

        let nft_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            account_1_public_key.to_account_hash(),
            ContractHash::new(nft_contract_hash),
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_META_DATA => TEST_META_DATA.to_string(),
            },
        )
        .build();

        builder.exec(nft_mint_request).expect_failure();
    }

    #[test]
    fn should_allow_minting_for_different_public_key_with_public_minting_set_to_true() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_total_token_supply(U256::from(100u64))
                .with_public_minting(Some(true))
                .build();
        builder.exec(install_request).expect_success().commit();

        let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_key = installing_account
            .named_keys()
            .get(CONTRACT_NAME)
            .expect("must have key in named keys");
        let nft_contract_hash = nft_contract_key
            .into_hash()
            .expect("must convert to hash addr");

        let (_account_1_secret_key, account_1_public_key) = create_dummy_key_pair(ACCOUNT_USER_1);
        let (_account_2_secret_key, account_2_public_key) = create_dummy_key_pair(ACCOUNT_USER_2);

        let transfer_to_account_1 = ExecuteRequestBuilder::transfer(
            *DEFAULT_ACCOUNT_ADDR,
            runtime_args! {
                mint::ARG_AMOUNT => 100_000_000_000_000u64,
                mint::ARG_TARGET => account_1_public_key.to_account_hash(),
                mint::ARG_ID => Option::<u64>::None,
            },
        )
        .build();

        let transfer_to_account_2 = ExecuteRequestBuilder::transfer(
            *DEFAULT_ACCOUNT_ADDR,
            runtime_args! {
                mint::ARG_AMOUNT => 100_000_000_000_000u64,
                mint::ARG_TARGET => account_1_public_key.to_account_hash(),
                mint::ARG_ID => Option::<u64>::None,
            },
        )
        .build();

        let transfer_requests = vec![transfer_to_account_1, transfer_to_account_2];
        for request in transfer_requests {
            builder.exec(request).expect_success().commit();
        }

        let public_minting_status = query_stored_value::<bool>(
            &mut builder,
            *nft_contract_key,
            vec![ARG_PUBLIC_MINTING.to_string()],
        );

        assert!(
            public_minting_status,
            "public minting should be set to true"
        );

        let incorrect_nft_minting_request = ExecuteRequestBuilder::contract_call_by_hash(
            account_1_public_key.to_account_hash(),
            ContractHash::new(nft_contract_hash),
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
            },
        )
        .build();

        builder
            .exec(incorrect_nft_minting_request)
            .expect_success()
            .commit();
    }

    #[test]
    fn should_burn_minted_token() {
        const TOKEN_ID: U256 = U256::zero();

        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request_builder =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_total_token_supply(U256::from(100u64))
                .build();

        builder
            .exec(install_request_builder)
            .expect_success()
            .commit();

        let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_key = installing_account
            .named_keys()
            .get(CONTRACT_NAME)
            .expect("must have key in named keys");

        let mint_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
            },
        )
        .build();

        builder.exec(mint_request).expect_success().commit();

        let actual_owned_tokens = get_dictionary_value_from_key::<Vec<U256>>(
            &builder,
            nft_contract_key,
            OWNED_TOKENS,
            &DEFAULT_ACCOUNT_PUBLIC_KEY
                .clone()
                .to_account_hash()
                .to_string(),
        );

        let expected_owned_tokens = vec![U256::zero()];
        assert_eq!(expected_owned_tokens, actual_owned_tokens);

        let burn_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_BURN,
            runtime_args! {
                ARG_TOKEN_ID => U256::zero(),
            },
        )
        .build();

        builder.exec(burn_request).expect_success().commit();

        let is_token_burnt = get_dictionary_value_from_key::<()>(
            &builder,
            nft_contract_key,
            BURNT_TOKENS,
            &TOKEN_ID.to_string(),
        );
    }

    #[test]
    fn should_not_burn_previously_burnt_token() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request_builder =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_total_token_supply(U256::from(100u64))
                .build();

        builder
            .exec(install_request_builder)
            .expect_success()
            .commit();

        let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_key = installing_account
            .named_keys()
            .get(CONTRACT_NAME)
            .expect("must have key in named keys");

        let mint_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
            },
        )
        .build();

        builder.exec(mint_request).expect_success().commit();

        let actual_owned_tokens = get_dictionary_value_from_key::<Vec<U256>>(
            &builder,
            nft_contract_key,
            OWNED_TOKENS,
            &DEFAULT_ACCOUNT_PUBLIC_KEY
                .clone()
                .to_account_hash()
                .to_string(),
        );

        let expected_owned_tokens = vec![U256::zero()];
        assert_eq!(expected_owned_tokens, actual_owned_tokens);

        let burn_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_BURN,
            runtime_args! {
                ARG_TOKEN_ID => U256::zero(),
            },
        )
        .build();

        builder.exec(burn_request).expect_success().commit();

        let re_burn_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_BURN,
            runtime_args! {
                ARG_TOKEN_ID => U256::zero(),
            },
        )
        .build();

        builder.exec(re_burn_request).expect_failure();

        let error = builder.get_error().expect("must have error");
        assert!(matches!(
            error,
            Error::Exec(execution::Error::Revert(ApiError::User(42)))
        ))
    }

    #[test]
    fn should_not_burn_un_minted_token() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request_builder =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_total_token_supply(U256::from(100u64))
                .build();

        builder
            .exec(install_request_builder)
            .expect_success()
            .commit();

        let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_key = installing_account
            .named_keys()
            .get(CONTRACT_NAME)
            .expect("must have key in named keys");
        let nft_contract_hash = nft_contract_key
            .into_hash()
            .expect("must convert to hash addr");

        let burn_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_BURN,
            runtime_args! {
                ARG_TOKEN_ID => U256::zero(),
            },
        )
        .build();

        builder.exec(burn_request).expect_failure();

        let error = builder.get_error().expect("must have error");
        assert!(matches!(
            error,
            Error::Exec(execution::Error::Revert(ApiError::User(28)))
        ))
    }

    #[test]
    fn should_disallow_burning_of_others_users_token() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request_builder =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_total_token_supply(U256::from(100u64))
                .build();

        builder
            .exec(install_request_builder)
            .expect_success()
            .commit();

        let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_key = installing_account
            .named_keys()
            .get(CONTRACT_NAME)
            .expect("must have key in named keys");
        let nft_contract_hash = nft_contract_key
            .into_hash()
            .expect("must convert to hash addr");

        let (_, account_user_1) = create_dummy_key_pair(ACCOUNT_USER_1);

        let transfer_to_account_1 = ExecuteRequestBuilder::transfer(
            *DEFAULT_ACCOUNT_ADDR,
            runtime_args! {
                mint::ARG_AMOUNT => 100_000_000_000_000u64,
                mint::ARG_TARGET => account_user_1.to_account_hash(),
                mint::ARG_ID => Option::<u64>::None,
            },
        )
        .build();

        builder
            .exec(transfer_to_account_1)
            .expect_success()
            .commit();

        let mint_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
            },
        )
        .build();

        builder.exec(mint_request).expect_success().commit();

        let actual_owned_tokens = get_dictionary_value_from_key::<Vec<U256>>(
            &builder,
            nft_contract_key,
            OWNED_TOKENS,
            &DEFAULT_ACCOUNT_PUBLIC_KEY
                .clone()
                .to_account_hash()
                .to_string(),
        );

        let expected_owned_tokens = vec![U256::zero()];
        assert_eq!(expected_owned_tokens, actual_owned_tokens);

        let incorrect_burn_request = ExecuteRequestBuilder::contract_call_by_hash(
            account_user_1.to_account_hash(),
            ContractHash::new(nft_contract_hash),
            ENTRY_POINT_BURN,
            runtime_args! {
                ARG_TOKEN_ID => U256::zero(),
            },
        )
        .build();

        builder.exec(incorrect_burn_request).expect_failure();

        let error = builder.get_error().expect("must have error");

        assert_expected_error(error, 6u16);
    }

    #[test]
    fn should_prevent_burning_on_owner_key_mismatch() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request_builder =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_total_token_supply(U256::from(100u64))
                .build();

        builder
            .exec(install_request_builder)
            .expect_success()
            .commit();

        let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_key = installing_account
            .named_keys()
            .get(CONTRACT_NAME)
            .expect("must have key in named keys");
        let nft_contract_hash = nft_contract_key
            .into_hash()
            .expect("must convert to hash addr");

        let (_, account_user_1) = create_dummy_key_pair(ACCOUNT_USER_1);

        let transfer_to_account_1 = ExecuteRequestBuilder::transfer(
            *DEFAULT_ACCOUNT_ADDR,
            runtime_args! {
                mint::ARG_AMOUNT => 100_000_000_000_000u64,
                mint::ARG_TARGET => account_user_1.to_account_hash(),
                mint::ARG_ID => Option::<u64>::None,
            },
        )
        .build();

        builder
            .exec(transfer_to_account_1)
            .expect_success()
            .commit();

        let mint_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
            },
        )
        .build();

        builder.exec(mint_request).expect_success().commit();

        let actual_owned_tokens = get_dictionary_value_from_key::<Vec<U256>>(
            &builder,
            nft_contract_key,
            OWNED_TOKENS,
            &DEFAULT_ACCOUNT_PUBLIC_KEY
                .clone()
                .to_account_hash()
                .to_string(),
        );

        let expected_owned_tokens = vec![U256::zero()];
        assert_eq!(expected_owned_tokens, actual_owned_tokens);

        let incorrect_burn_request = ExecuteRequestBuilder::contract_call_by_hash(
            account_user_1.to_account_hash(),
            ContractHash::new(nft_contract_hash),
            ENTRY_POINT_BURN,
            runtime_args! {
                ARG_TOKEN_ID => U256::zero()
            },
        )
        .build();

        builder.exec(incorrect_burn_request).expect_failure();

        let error = builder.get_error().expect("must get error");
        assert!(matches!(
            error,
            Error::Exec(execution::Error::Revert(ApiError::User(6)))
        ))
    }

    #[test]
    fn only_installer_should_be_able_to_toggle_allow_minting() {
        let (_, other_user_public_key) = create_dummy_key_pair(ACCOUNT_USER_1); //<-- Choose MINTER2 for failing red test
        let other_user_account = other_user_public_key.to_account_hash();
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_collection_name(NFT_TEST_COLLECTION.to_string())
                .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
                .with_total_token_supply(U256::from(1))
                .with_allowing_minting(Some(false))
                .build();

        builder.exec(install_request).expect_success().commit();

        let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_hash = account
            .named_keys()
            .get(CONTRACT_NAME)
            .cloned()
            .and_then(Key::into_hash)
            .map(ContractHash::new)
            .expect("failed to find nft contract");

        //Account other than installer account should not be able to change allow_minting
        // Red test
        let other_user_set_variables_request = ExecuteRequestBuilder::contract_call_by_hash(
            other_user_account,
            nft_contract_hash,
            ENTRY_POINT_SET_VARIABLES,
            runtime_args! { ARG_ALLOW_MINTING => Some(true) },
        )
        .build();

        builder
            .exec(other_user_set_variables_request)
            .expect_failure()
            .commit();

        // Don't just use expect_failure. Match and actual error!
        // let error = builder.get_error().expect("must have error");
        // assert_expected_error(error, NFTCoreError::InvalidAccount as u16);

        //Installer account should be able to change allow_minting
        // Green test
        let installer_set_variables_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash,
            ENTRY_POINT_SET_VARIABLES,
            runtime_args! { ARG_ALLOW_MINTING => Some(true) },
        )
        .build();

        builder
            .exec(installer_set_variables_request)
            .expect_success()
            .commit();
    }

    #[test]
    fn should_transfer_token_from_sender_to_receiver() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_collection_name(NFT_TEST_COLLECTION.to_string())
                .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
                .with_total_token_supply(U256::from(1u64))
                .build();

        builder.exec(install_request).expect_success().commit();

        let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_hash = account
            .named_keys()
            .get(CONTRACT_NAME)
            .cloned()
            .and_then(Key::into_hash)
            .map(ContractHash::new)
            .expect("failed to find nft contract");

        let token_owner = DEFAULT_ACCOUNT_PUBLIC_KEY
            .clone()
            .to_account_hash()
            .to_string();

        let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
            },
        )
        .build();

        builder.exec(mint_request).expect_success().commit();

        let (_, token_receiver) = create_dummy_key_pair(ACCOUNT_USER_1);
        let transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash,
            ENTRY_POINT_TRANSFER,
            runtime_args! {
                ARG_TOKEN_ID => U256::zero(),// We need mint to return the token_id!!
                ARG_FROM_ACCOUNT_HASH => token_owner.clone(),
                ARG_TO_ACCOUNT_HASH => token_receiver.clone().to_account_hash().to_string(),
            },
        )
        .build();
        builder.exec(transfer_request).expect_success().commit();

        //let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_key = installing_account
            .named_keys()
            .get(CONTRACT_NAME)
            .expect("must have key in named keys");

        let actual_token_owner: String = get_dictionary_value_from_key(
            &builder,
            nft_contract_key,
            TOKEN_OWNERS,
            &U256::zero().to_string(),
        );

        assert_eq!(
            actual_token_owner,
            token_receiver.to_account_hash().to_string()
        ); // Change  token_receiver to token_owner for red test

        let actual_owned_tokens: Vec<U256> = get_dictionary_value_from_key(
            &builder,
            nft_contract_key,
            OWNED_TOKENS,
            &token_receiver.to_account_hash().to_string(),
        );

        let expected_owned_tokens = vec![U256::zero()]; //Change zero() to one() for red test
        assert_eq!(actual_owned_tokens, expected_owned_tokens);
    }

    #[test]
    fn approve_token_for_transfer_should_add_entry_to_approved_dictionary() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_collection_name(NFT_TEST_COLLECTION.to_string())
                .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
                .with_total_token_supply(U256::one())
                .build();

        builder.exec(install_request).expect_success().commit();

        let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_hash = account
            .named_keys()
            .get(CONTRACT_NAME)
            .cloned()
            .and_then(Key::into_hash)
            .map(ContractHash::new)
            .expect("failed to find nft contract");

        let token_owner = DEFAULT_ACCOUNT_PUBLIC_KEY.clone();
        let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
            },
        )
        .build();

        builder.exec(mint_request).expect_success().commit();

        let (_, approve_public_key) = create_dummy_key_pair(ACCOUNT_USER_1);
        let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash,
            ENTRY_POINT_APPROVE,
            runtime_args! {
                ARG_TOKEN_ID => U256::zero(),
                ARG_APPROVE_TRANSFER_FOR_ACCOUNT_HASH => approve_public_key.clone().to_account_hash().to_string()
            },
        )
        .build();
        builder.exec(approve_request).expect_success().commit();

        let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_key = installing_account
            .named_keys()
            .get(CONTRACT_NAME)
            .expect("must have key in named keys");

        let actual_approved_account_hash: Option<String> = get_dictionary_value_from_key(
            &builder,
            nft_contract_key,
            APPROVED_FOR_TRANSFER,
            &U256::zero().to_string(),
        );

        assert_eq!(
            actual_approved_account_hash,
            Some(approve_public_key.to_account_hash().to_string())
        );
    }

    #[test]
    fn should_be_able_to_transfer_token_using_approved_operator() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_collection_name(NFT_TEST_COLLECTION.to_string())
                .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
                .with_total_token_supply(U256::one())
                .build();

        builder.exec(install_request).expect_success().commit();

        let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_hash = account
            .named_keys()
            .get(CONTRACT_NAME)
            .cloned()
            .and_then(Key::into_hash)
            .map(ContractHash::new)
            .expect("failed to find nft contract");

        // mint token for DEFAULT_ACCOUNT_ADDR
        let token_owner = DEFAULT_ACCOUNT_PUBLIC_KEY.clone();
        let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_META_DATA=>TEST_META_DATA.to_string(),
            },
        )
        .build();
        builder.exec(mint_request).expect_success().commit();

        // Create operator account and transfer funds
        let (_, operator) = create_dummy_key_pair(ACCOUNT_USER_1);
        let transfer_to_operator = ExecuteRequestBuilder::transfer(
            *DEFAULT_ACCOUNT_ADDR,
            runtime_args! {
                mint::ARG_AMOUNT => 100_000_000_000_000u64,
                mint::ARG_TARGET => operator.to_account_hash(),
                mint::ARG_ID => Option::<u64>::None,
            },
        )
        .build();
        builder.exec(transfer_to_operator).expect_success().commit();

        // Approve operator
        let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash,
            ENTRY_POINT_APPROVE,
            runtime_args! {
                ARG_TOKEN_ID => U256::zero(),
                ARG_APPROVE_TRANSFER_FOR_ACCOUNT_HASH => operator.clone().to_account_hash().to_string()
            },
        )
        .build();
        builder.exec(approve_request).expect_success().commit();

        // Create to_account and transfer minted token using operator
        let (_, to_account_hash) = create_dummy_key_pair(ACCOUNT_USER_2);
        let transfer_to_to_account = ExecuteRequestBuilder::transfer(
            *DEFAULT_ACCOUNT_ADDR,
            runtime_args! {
                mint::ARG_AMOUNT => 100_000_000_000_000u64,
                mint::ARG_TARGET => to_account_hash.to_account_hash(),
                mint::ARG_ID => Option::<u64>::None,
            },
        )
        .build();
        builder
            .exec(transfer_to_to_account)
            .expect_success()
            .commit();

        let transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
            operator.to_account_hash(),
            nft_contract_hash,
            ENTRY_POINT_TRANSFER,
            runtime_args! {
                ARG_TOKEN_ID => U256::zero(),
                ARG_FROM_ACCOUNT_HASH => token_owner.clone().to_account_hash().to_string(),
                ARG_TO_ACCOUNT_HASH => to_account_hash.to_account_hash().to_string(),
            },
        )
        .build();
        builder.exec(transfer_request).expect_success().commit();

        // // Start querying...
        // let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        // let nft_contract_key = installing_account
        //     .named_keys()
        //     .get(CONTRACT_NAME)
        //     .expect("must have key in named keys");

        // let actual_approved_account_hash: Option<String> = get_dictionary_value_from_key(
        //     &builder,
        //     nft_contract_key,
        //     APPROVED_FOR_TRANSFER,
        //     &U256::zero().to_string(),
        // );

        // assert_eq!(
        //     actual_approved_account_hash,
        //     Some(operator.to_account_hash().to_string())
        // );
    }

    ////////////////////////////////////////////////////////////////////////////////
    //////////////////////////////// Helper methods ////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////
    fn get_dictionary_value_from_key<T: CLTyped + FromBytes>(
        builder: &WasmTestBuilder<InMemoryGlobalState>,
        nft_contract_key: &Key,
        dictionary_name: &str,
        dictionary_key: &str,
    ) -> T {
        let seed_uref = *builder
            .query(None, *nft_contract_key, &vec![])
            .expect("must have nft contract")
            .as_contract()
            .expect("must convert contract")
            .named_keys()
            .get(dictionary_name)
            .expect("must have key")
            .as_uref()
            .expect("must convert to seed uref");

        builder
            .query_dictionary_item(None, seed_uref, dictionary_key)
            .expect("should have dictionary value")
            .as_cl_value()
            .expect("T should be CLValue")
            .to_owned()
            .into_t()
            .unwrap()
    }

    fn create_dummy_key_pair(account_string: [u8; 32]) -> (SecretKey, PublicKey) {
        let secrete_key =
            SecretKey::ed25519_from_bytes(account_string).expect("failed to create secret key");
        let public_key = PublicKey::from(&secrete_key);
        (secrete_key, public_key)
    }

    fn assert_expected_invalid_installer_request(
        install_request_builder: InstallerRequestBuilder,
        expected_error_code: u16,
    ) {
        let mut builder = InMemoryWasmTestBuilder::default();

        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
        builder
            .exec(install_request_builder.build())
            .expect_failure(); // Should test against expected error

        let error = builder.get_error().expect("should have an error");
        assert_expected_error(error, expected_error_code);
    }

    fn assert_expected_error(error: Error, error_code: u16) {
        let actual = format!("{:?}", error);
        let expected = format!(
            "{:?}",
            Error::Exec(execution::Error::Revert(ApiError::User(error_code)))
        );

        assert_eq!(actual, expected, "Error should match {}", error_code)
    }

    fn query_stored_value<T: CLTyped + FromBytes>(
        builder: &mut InMemoryWasmTestBuilder,
        nft_contract_key: Key,
        path: Vec<String>,
    ) -> T {
        builder
            .query(None, nft_contract_key, &path)
            .expect("must have stored value")
            .as_cl_value()
            .cloned()
            .expect("must have cl value")
            .into_t::<T>()
            .expect("must get value")
    }

    #[derive(Debug)]
    struct InstallerRequestBuilder {
        account_hash: AccountHash,
        session_file: String,
        collection_name: CLValue,
        collection_symbol: CLValue,
        total_token_supply: CLValue,
        allow_minting: CLValue,
        public_minting: CLValue,
    }

    impl InstallerRequestBuilder {
        fn new(account_hash: AccountHash, session_file: &str) -> Self {
            Self::default()
                .with_account_hash(account_hash)
                .with_session_file(session_file.to_string())
        }

        fn default() -> Self {
            InstallerRequestBuilder {
                account_hash: AccountHash::default(),
                session_file: String::default(),
                collection_name: CLValue::from_t("name".to_string())
                    .expect("name is legit CLValue"),
                collection_symbol: CLValue::from_t("SYM")
                    .expect("collection_symbol is legit CLValue"),
                total_token_supply: CLValue::from_t(U256::one())
                    .expect("total_token_supply is legit CLValue"),
                allow_minting: CLValue::from_t(Some(true)).unwrap(),
                public_minting: CLValue::from_t(Some(false)).unwrap(),
            }
        }

        fn with_account_hash(mut self, account_hash: AccountHash) -> Self {
            self.account_hash = account_hash;
            self
        }

        fn with_session_file(mut self, session_file: String) -> Self {
            self.session_file = session_file;
            self
        }

        fn with_collection_name(mut self, collection_name: String) -> Self {
            self.collection_name =
                CLValue::from_t(collection_name).expect("collection_name is legit CLValue");
            self
        }

        fn with_invalid_collection_name(mut self, collection_name: CLValue) -> Self {
            self.collection_name = collection_name;
            self
        }

        fn with_collection_symbol(mut self, collection_symbol: String) -> Self {
            self.collection_symbol =
                CLValue::from_t(collection_symbol).expect("collection_symbol is legit CLValue");
            self
        }

        fn with_invalid_collection_symbol(mut self, collection_symbol: CLValue) -> Self {
            self.collection_symbol = collection_symbol;
            self
        }

        fn with_total_token_supply(mut self, total_token_supply: U256) -> Self {
            self.total_token_supply =
                CLValue::from_t(total_token_supply).expect("total_token_supply is legit CLValue");
            self
        }

        fn with_invalid_total_token_supply(mut self, total_token_supply: CLValue) -> Self {
            self.total_token_supply = total_token_supply;
            self
        }

        // Why Option here? The None case should be taken care of when running default
        fn with_allowing_minting(mut self, allow_minting: Option<bool>) -> Self {
            self.allow_minting =
                CLValue::from_t(allow_minting).expect("allow minting is legit CLValue");
            self
        }

        fn with_public_minting(mut self, public_minting: Option<bool>) -> Self {
            self.public_minting =
                CLValue::from_t(public_minting).expect("public minting is legit CLValue");
            self
        }

        fn build(self) -> ExecuteRequest {
            let mut runtime_args = RuntimeArgs::new();
            runtime_args.insert_cl_value(ARG_COLLECTION_NAME, self.collection_name);
            runtime_args.insert_cl_value(ARG_COLLECTION_SYMBOL, self.collection_symbol);
            runtime_args.insert_cl_value(ARG_TOTAL_TOKEN_SUPPLY, self.total_token_supply);
            runtime_args.insert_cl_value(ARG_ALLOW_MINTING, self.allow_minting);
            runtime_args.insert_cl_value(ARG_PUBLIC_MINTING, self.public_minting);
            ExecuteRequestBuilder::standard(self.account_hash, &self.session_file, runtime_args)
                .build()
        }
    }
}

// #[test]
// fn minted_tokens_dictionary_should_exist() {
//     let mut builder = InMemoryWasmTestBuilder::default();
//     builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
//     let install_request_builder =
//         InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM);
//     builder
//         .exec(install_request_builder.build())
//         .expect_success()
//         .commit();get_stoget_stored_valuered_value

//     let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
//     let nft_contract_key = account
//         .named_keys()
//         .get(CONTRACT_NAME)
//         .expect("must have key in named keys");

//     let query_result: BTreeMap<U256, PublicKey> = query_stored_value(
//         &mut builder,
//         *nft_contract_key,
//         vec![TOKEN_OWNER.to_string()],
//     );
// }

// #[test]
// fn should_mint_nft() {
//     let mut builder = InMemoryWasmTestBuilder::default();
//     builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

//     // Install contract
//     let install_request = ExecuteRequestBuilder::standard(
//         *DEFAULT_ACCOUNT_ADDR,
//         NFT_CONTRACT_WASM,
//         runtime_args! {
//             ARG_COLLECTION_NAME => "Hans".to_string()
//         },
//     )
//     .build();
//     builder.exec(install_request).expect_success().commit();

//     let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
//     let set_variables_request = ExecuteRequestBuilder::contract_call_by_name(
//         *DEFAULT_ACCOUNT_ADDR,
//         CONTRACT_NAME,
//         ENTRY_POINT_SET_VARIABLES,
//         runtime_args! {
//             ARG_COLLECTION_NAME => "Austin".to_string()
//         },
//     )
//     .build();
//     builder
//         .exec(set_variables_request)
//         .expect_success()
//         .commit();

//     let nft_contract_key = account
//         .named_keys()
//         .get(CONTRACT_NAME)
//         .expect("must have key in named keys");

//     let query_result = builder
//         .query(None, *nft_contract_key, &[COLLECTION_NAME.to_string()])
//         .expect("must have stored value")
//         .as_cl_value()
//         .cloned()
//         .expect("must have cl value")
//         .into_t::<String>()
//         .expect("must get string value");

//     assert_eq!(query_result, "Austin".to_string());

//     let mint_request = ExecuteRequestBuilder::contract_call_by_name(
//         *DEFAULT_ACCOUNT_ADDR,
//         CONTRACT_NAME,
//         ENTRY_POINT_MINT,
//         runtime_args! {
//             ARG_TOKEN_OWNER => DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
//             ARG_TOKEN_NAME => "Austin".to_string(),
//             ARG_TOKEN_META => "Austin".to_string(),
//         },
//     )
//     .build();
//     builder.exec(mint_request).expect_success().commit();

//     //This one will fail because: thou shalt not return!
//     let balance_of_request = ExecuteRequestBuilder::contract_call_by_name(
//         *DEFAULT_ACCOUNT_ADDR,
//         CONTRACT_NAME,
//         ENTRY_POINT_BALANCE_OF,
//         runtime_args! {
//             ARG_TOKEN_OWNER => DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
//         },
//     )
//     .build();
//     builder.exec(balance_of_request).expect_success().commit();
// }

// #[test]
// fn should_transfer_nft_to_existing_account() {
//     let mut builder = InMemoryWasmTestBuilder::default();
//     builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

//     // Install contract
//     let install_request = ExecuteRequestBuilder::standard(
//         *DEFAULT_ACCOUNT_ADDR,
//         NFT_CONTRACT_WASM,
//         runtime_args! {
//             ARG_COLLECTION_NAME => "Hans".to_string()
//         },
//     )
//     .build();
//     builder.exec(install_request).expect_success().commit();

//     //Create reciever account
//     let receiver_secret_key =
//         SecretKey::ed25519_from_bytes([7; 32]).expect("failed to create secret key");
//     let receiver_public_key = PublicKey::from(&receiver_secret_key);

//     let receiver_account_hash = receiver_public_key.to_account_hash();

//     let fund_receiver_request = ExecuteRequestBuilder::transfer(
//         *DEFAULT_ACCOUNT_ADDR,
//         runtime_args! {
//             mint::ARG_AMOUNT => U512::from(30_000_000_000_000_u64), //The actual amount being tranferred?
//             mint::ARG_TARGET => receiver_public_key.clone(),//Recipient account
//             mint::ARG_ID => <Option::<u64>>::None, //What is ARG_ID for?
//         },
//     )
//     .build();

//     builder
//         .exec(fund_receiver_request)
//         .expect_success()
//         .commit();

//     let _ = builder.get_expected_account(receiver_account_hash);

//     let mint_request = ExecuteRequestBuilder::contract_call_by_name(
//         *DEFAULT_ACCOUNT_ADDR,
//         CONTRACT_NAME,
//         ENTRY_POINT_MINT,
//         runtime_args! {
//             ARG_TOKEN_OWNER => DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
//             ARG_TOKEN_NAME => "Austin".to_string(),
//             ARG_TOKEN_META => "Austin".to_string(),
//         },
//     )
//     .build();
//     builder.exec(mint_request).expect_success().commit();

//     let transfer_request = ExecuteRequestBuilder::contract_call_by_name(
//         *DEFAULT_ACCOUNT_ADDR,
//         CONTRACT_NAME,
//         ENTRY_POINT_TRANSFER,
//         runtime_args! {
//             ARG_TOKEN_OWNER => DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
//             ARG_TOKEN_RECEIVER => receiver_public_key,

//         },
//     );
// }

//     let token_owner: PublicKey = runtime::get_named_arg(ARG_TOKEN_OWNER);
//     let token_receiver: PublicKey = runtime::get_named_arg(ARG_TOKEN_RECEIVER);
//     let token_id: U256 = runtime::get_named_arg(ARG_TOKEN_ID);
