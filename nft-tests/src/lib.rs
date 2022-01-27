#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
        DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_GENESIS_CONFIG,
        DEFAULT_GENESIS_CONFIG_HASH, DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
    };
    use casper_execution_engine::core::engine_state::{
        run_genesis_request::RunGenesisRequest, GenesisAccount,
    };
    use casper_types::{account::AccountHash, runtime_args, Key, Motes, PublicKey, RuntimeArgs, SecretKey, U512, ContractHash};

    const ARG_TOKEN_OWNER: &str = "token_owner";
    const NFT_CONTRACT_WASM: &str = "nft-installer.wasm";
    const CONTRACT_NAME: &str = "nft_contract";
    const ENTRY_POINT_SET_VARIABLES: &str = "set_variables";

    #[test]
    fn should_install_contract() {
        let mut builder = InMemoryWasmTestBuilder::default();

        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            NFT_CONTRACT_WASM,
            runtime_args! {
                ARG_TOKEN_OWNER => "Hans".to_string()
            }
        )
            .build();

        builder.exec(install_request)
            .expect_success()
            .commit();

        let nft_contract_key = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR)
            .named_keys()
            .get(CONTRACT_NAME)
            .expect("must have key in named keys")
            .clone();

        let query_result = builder.query(
            None,
            nft_contract_key,
            &["token_owner".to_string()]
        )
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
                ARG_TOKEN_OWNER => "Austin".to_string()
            }
        )
            .build();

        builder.exec(set_variables_request)
            .expect_success()
            .commit();

        let query_result = builder.query(
            None,
            nft_contract_key,
            &["token_owner".to_string()]
        )
            .expect("must have stored value")
            .as_cl_value()
            .cloned()
            .expect("must have cl value")
            .into_t::<String>()
            .expect("must get string value");

        assert_eq!(query_result, "Austin".to_string());

    }
}