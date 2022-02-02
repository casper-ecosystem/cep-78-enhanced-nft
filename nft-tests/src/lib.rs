#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
        DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_ACCOUNT_PUBLIC_KEY,
        DEFAULT_GENESIS_CONFIG, DEFAULT_GENESIS_CONFIG_HASH, DEFAULT_PAYMENT,
        DEFAULT_RUN_GENESIS_REQUEST,
    };
    use casper_execution_engine::core::engine_state::{
        run_genesis_request::RunGenesisRequest, GenesisAccount,
    };
    use casper_types::{
        account::AccountHash, runtime_args, system::mint, ContractHash, Key, Motes, PublicKey,
        RuntimeArgs, SecretKey, U256, U512,
    };

    const COLLECTION_NAME: &str = "collection_name";
    const NFT_CONTRACT_WASM: &str = "nft-installer.wasm";
    const CONTRACT_NAME: &str = "nft_contract";

    pub const ENTRY_POINT_INIT: &str = "init";
    pub const ENTRY_POINT_SET_VARIABLES: &str = "set_variables";
    pub const ENTRY_POINT_MINT: &str = "mint";
    pub const ENTRY_POINT_BURN: &str = "burn";
    pub const ENTRY_POINT_TRANSFER: &str = "transfer";
    pub const ENTRY_POINT_BALANCE_OF: &str = "balance_of";

    const ARG_COLLECTION_NAME: &str = "collection_name";
    const ARG_TOKEN_OWNER: &str = "token_owner";
    const ARG_TOKEN_RECEIVER: &str = "token_receiver";
    const ARG_TOKEN_NAME: &str = "token_name";
    const ARG_TOKEN_META: &str = "token_meta";
    const ARG_TOKEN_ID: &str = "token_id";

    #[test]
    fn should_install_contract() {
        let mut builder = InMemoryWasmTestBuilder::default();

        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            NFT_CONTRACT_WASM,
            runtime_args! {
                ARG_COLLECTION_NAME => "Hans".to_string()
            },
        )
        .build();

        builder.exec(install_request).expect_success().commit();

        let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_key = account
            .named_keys()
            .get(CONTRACT_NAME)
            .expect("must have key in named keys");

        let query_result = builder
            .query(None, *nft_contract_key, &[COLLECTION_NAME.to_string()])
            .expect("must have stored value")
            .as_cl_value()
            .cloned()
            .expect("must have cl value")
            .into_t::<String>()
            .expect("must get string value");

        assert_eq!(query_result, "Hans".to_string());

        let set_variables_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_SET_VARIABLES,
            runtime_args! {
                ARG_COLLECTION_NAME => "Austin".to_string()
            },
        )
        .build();

        builder
            .exec(set_variables_request)
            .expect_success()
            .commit();

        let query_result = builder
            .query(None, *nft_contract_key, &[COLLECTION_NAME.to_string()])
            .expect("must have stored value")
            .as_cl_value()
            .cloned()
            .expect("must have cl value")
            .into_t::<String>()
            .expect("must get string value");

        assert_eq!(query_result, "Austin".to_string());
    }

    #[test]
    fn should_mint_nft() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        // Install contract
        let install_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            NFT_CONTRACT_WASM,
            runtime_args! {
                ARG_COLLECTION_NAME => "Hans".to_string()
            },
        )
        .build();
        builder.exec(install_request).expect_success().commit();

        let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let set_variables_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_SET_VARIABLES,
            runtime_args! {
                ARG_COLLECTION_NAME => "Austin".to_string()
            },
        )
        .build();
        builder
            .exec(set_variables_request)
            .expect_success()
            .commit();

        let nft_contract_key = account
            .named_keys()
            .get(CONTRACT_NAME)
            .expect("must have key in named keys");

        let query_result = builder
            .query(None, *nft_contract_key, &[COLLECTION_NAME.to_string()])
            .expect("must have stored value")
            .as_cl_value()
            .cloned()
            .expect("must have cl value")
            .into_t::<String>()
            .expect("must get string value");

        assert_eq!(query_result, "Austin".to_string());

        let mint_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_OWNER => DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
                ARG_TOKEN_NAME => "Austin".to_string(),
                ARG_TOKEN_META => "Austin".to_string(),
            },
        )
        .build();
        builder.exec(mint_request).expect_success().commit();

        //This one will fail because: thou shalt not return!
        let balance_of_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_BALANCE_OF,
            runtime_args! {
                ARG_TOKEN_OWNER => DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
            },
        )
        .build();
        builder.exec(balance_of_request).expect_success().commit();
    }

    #[test]
    fn should_transfer_nft_to_existing_account() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        // Install contract
        let install_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            NFT_CONTRACT_WASM,
            runtime_args! {
                ARG_COLLECTION_NAME => "Hans".to_string()
            },
        )
        .build();
        builder.exec(install_request).expect_success().commit();

        //Create reciever account
        let receiver_secret_key =
            SecretKey::ed25519_from_bytes([7; 32]).expect("failed to create secret key");
        let receiver_public_key = PublicKey::from(&receiver_secret_key);

        let receiver_account_hash = receiver_public_key.to_account_hash();

        let fund_receiver_request = ExecuteRequestBuilder::transfer(
            *DEFAULT_ACCOUNT_ADDR,
            runtime_args! {
                mint::ARG_AMOUNT => U512::from(30_000_000_000_000_u64), //The actual amount being tranferred?
                mint::ARG_TARGET => receiver_public_key.clone(),//Recipient account
                mint::ARG_ID => <Option::<u64>>::None, //What is ARG_ID for?
            },
        )
        .build();

        builder
            .exec(fund_receiver_request)
            .expect_success()
            .commit();

        let _ = builder.get_expected_account(receiver_account_hash);

        let mint_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_OWNER => DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
                ARG_TOKEN_NAME => "Austin".to_string(),
                ARG_TOKEN_META => "Austin".to_string(),
            },
        )
        .build();
        builder.exec(mint_request).expect_success().commit();

        let transfer_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_TRANSFER,
            runtime_args! {
                ARG_TOKEN_OWNER => DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
                ARG_TOKEN_RECEIVER => receiver_public_key,

            },
        );
    }

    //     let token_owner: PublicKey = runtime::get_named_arg(ARG_TOKEN_OWNER);
    //     let token_receiver: PublicKey = runtime::get_named_arg(ARG_TOKEN_RECEIVER);
    //     let token_id: U256 = runtime::get_named_arg(ARG_TOKEN_ID);
}
