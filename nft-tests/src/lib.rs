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
        account::AccountHash, runtime_args, ContractHash, Key, Motes, PublicKey, RuntimeArgs,
        SecretKey, U256, U512,
    };

    const COLLECTION_NAME: &str = "collection_name";
    const NFT_CONTRACT_WASM: &str = "nft-installer.wasm";
    const CONTRACT_NAME: &str = "nft_contract";

    const ENTRY_POINT_INIT: &str = "init";
    const ENTRY_POINT_SET_VARIABLES: &str = "set_variables";
    const ENTRY_POINT_MINT: &str = "mint";
    const ENTRY_POINT_BALANCE_OF: &str = "balance_of";

    const ARG_COLLECTION_NAME: &str = "collection_name";
    const ARG_TOKEN_OWNER: &str = "token_owner";
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

        let token_id = U256::from(0);
        let mint_request = ExecuteRequestBuilder::contract_call_by_name(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_NAME,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_TOKEN_OWNER => DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
                ARG_TOKEN_ID => token_id,
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
}
