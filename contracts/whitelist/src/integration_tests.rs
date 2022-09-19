#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::{
        CheckWhitelistResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
    };
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, Addr, Coin, Empty, Timestamp, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use crate::contract::instantiate;
    use cw20::Cw20Coin;

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::query::query,
        );
        Box::new(contract)
    }

    pub fn cw20_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "user";
    const ADMIN: &str = "admin";
    const INVALID: &str = "invalid";
    const NATIVE_DENOM: &str = "juno";
    const INITIAL_BALANCE: u128 = 2_000_000_000;
    const MINT_PRICE: u128 = 1_000_000;
    const TEST_MINT_PRICE: u128 = 1_500_000;

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1),
                    }],
                )
                .unwrap();
        })
    }

    fn proper_instantiate() -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());

        // 1571797419879305533
        let msg = InstantiateMsg {
            start_time: Timestamp::from_seconds(1571797420), // 1571797419
            end_time: Timestamp::from_seconds(1656801750),
            maintainer_address: Some(USER.to_owned()),
            max_whitelist_address_count: 5,
            max_per_address_mint: 3,
            mint_price: Uint128::from(MINT_PRICE),
        };

        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let cw_template_contract = CwTemplateContract(cw_template_contract_addr);

        (app, cw_template_contract)
    }

    fn proper_init_whitelist(
        app: &mut App,
        cw_template_contract: CwTemplateContract,
    ) -> Vec<String> {
        let mut addresses = vec![];
        for i in 0..5 {
            addresses.push(format!("test_addr{}", i));
        }

        let init_addresses = addresses.clone();

        println!("addresses {:?}", addresses);

        let msg = ExecuteMsg::AddToWhitelist(addresses);
        let cosmos_msg = cw_template_contract.call(msg).unwrap();
        let res = app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();

        println!("{:?}", res);

        init_addresses
    }

    mod init {
        use super::*;
        use crate::msg::QueryMsg;

        #[test]
        fn proper_init() {
            let (app, cw_template_contract) = proper_instantiate();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert_eq!(config.max_whitelist_address_count, 5)
        }

        #[test]
        fn init_cw20() {
            let (mut app, _cw_template_contract) = proper_instantiate();

            let cw20_addr = instantiate_cw20(&mut app, vec![]);
            println!("{:?}", cw20_addr);
            assert_eq!(5, 5);
        }

        fn instantiate_cw20(app: &mut App, initial_balances: Vec<Cw20Coin>) -> Addr {
            let cw20_id = app.store_code(cw20_contract());
            let msg = cw20_base::msg::InstantiateMsg {
                name: String::from("Test"),
                symbol: String::from("TEST"),
                decimals: 6,
                initial_balances,
                mint: None,
                marketing: None,
            };

            app.instantiate_contract(cw20_id, Addr::unchecked(ADMIN), &msg, &[], "cw20", None)
                .unwrap()
        }

        #[test]
        fn proper_init_2() {
            let mut app = mock_app();
            let cw_template_id = app.store_code(contract_template());
            let _deps = mock_dependencies_with_balance(&coins(2, "token"));
            let _info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));

            let _cw20_addr = instantiate_cw20(&mut app, vec![]);

            // 1571797419879305533
            let msg = InstantiateMsg {
                start_time: Timestamp::from_seconds(1571797420), // 1571797419
                end_time: Timestamp::from_seconds(1656801750),
                maintainer_address: Some(USER.to_owned()),
                max_whitelist_address_count: 5,
                max_per_address_mint: 3,
                mint_price: Uint128::from(MINT_PRICE),
            };

            let cw_template_contract_addr = app
                .instantiate_contract(
                    cw_template_id,
                    Addr::unchecked(ADMIN),
                    &msg,
                    &[],
                    "test",
                    None,
                )
                .unwrap();

            let _cw_template_contract = CwTemplateContract(cw_template_contract_addr);
        }

        #[test]
        fn fail_init_start_time() {
            let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
            let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));

            // 1571797419879305533
            let msg = InstantiateMsg {
                start_time: Timestamp::from_seconds(1571797418), // 1571797419
                end_time: Timestamp::from_seconds(1656801750),
                maintainer_address: Some("randomcontractaddr0".to_string()),
                max_whitelist_address_count: 5,
                max_per_address_mint: 3,
                mint_price: Uint128::from(MINT_PRICE),
            };

            let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

            let err_str = format!("{:?}", err);

            assert_eq!(err_str, "InvalidStartTime".to_string());
        }

        #[test]
        fn fail_init_end_time_before_start_time() {
            let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
            let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));

            // 1571797419879305533
            let msg = InstantiateMsg {
                start_time: Timestamp::from_seconds(1571797420), // 1571797419
                end_time: Timestamp::from_seconds(1571797417),
                maintainer_address: Some("randomcontractaddr0".to_string()),
                max_whitelist_address_count: 5,
                max_per_address_mint: 3,
                mint_price: Uint128::from(MINT_PRICE),
            };

            let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

            let err_str = format!("{:?}", err);

            assert_eq!(err_str, "InvalidEndTime".to_string());
        }

        //#[test]
        fn _fail_init_invalid_maintainer_address() {
            let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
            let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));

            // 1571797419879305533
            let msg = InstantiateMsg {
                start_time: Timestamp::from_seconds(1571797419879305533), // 1571797419
                end_time: Timestamp::from_seconds(1571797420),
                maintainer_address: Some(
                    "juno1jcx6n6pz6ryl9wr42wm2h4rsjgyjk2wjxfjwmxr5gmcwyktedpgqnn0n7j".to_string(),
                ),
                max_whitelist_address_count: 5,
                max_per_address_mint: 3,
                mint_price: Uint128::from(MINT_PRICE),
            };

            let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

            let err_str = format!("{:?}", err);

            assert_eq!(err_str, "InvalidEndTime".to_string());
        }

        #[test]
        fn fail_init_max_whitelist_address_count() {
            let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
            let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));

            // 1571797419879305533
            let msg = InstantiateMsg {
                start_time: Timestamp::from_seconds(1571797420), // 1571797419
                end_time: Timestamp::from_seconds(1581797417),
                maintainer_address: Some("randomcontractaddr0".to_string()),
                max_whitelist_address_count: 10001,
                max_per_address_mint: 3,
                mint_price: Uint128::from(MINT_PRICE),
            };

            let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

            let err_str = format!("{:?}", err);

            assert_eq!(
                err_str,
                "InvalidMaxWhitelistAddressCount(10000)".to_string()
            );
        }

        #[test]
        fn fail_init_max_per_address_mint() {
            let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
            let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));

            // 1571797419879305533
            let msg = InstantiateMsg {
                start_time: Timestamp::from_seconds(1571797420), // 1571797419
                end_time: Timestamp::from_seconds(1581797417),
                maintainer_address: Some("randomcontractaddr0".to_string()),
                max_whitelist_address_count: 5,
                max_per_address_mint: 101,
                mint_price: Uint128::from(MINT_PRICE),
            };

            let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

            let err_str = format!("{:?}", err);

            assert_eq!(err_str, "InvalidMaxPerAddressMint(100)".to_string());
        }

        //#[test]
        fn _fail_init_() {
            let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
            let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));

            // 1571797419879305533
            let msg = InstantiateMsg {
                start_time: Timestamp::from_seconds(1571797420), // 1571797419
                end_time: Timestamp::from_seconds(1581797417),
                maintainer_address: Some("randomcontractaddr0".to_string()),
                max_whitelist_address_count: 5,
                max_per_address_mint: 3,
                mint_price: Uint128::from(MINT_PRICE),
            };

            let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

            let err_str = format!("{:?}", err);

            assert_eq!(err_str, "".to_string());
        }
    }

    mod execute {
        use super::*;

        #[test]
        fn update_maintainer_address_1() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            println!("config {:?}", config);

            let init_maintainer_address = config.maintainer_addr.clone();
            println!("init_maintainer_address {:?}", init_maintainer_address);
            assert_eq!(
                init_maintainer_address,
                Some(Addr::unchecked(USER.to_owned()))
            );

            let junk_address: String = "junkcontract1".to_string();
            let test_address_addr = Some(Addr::unchecked(junk_address.clone()));

            let maintainer_address: Option<String> =
                config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = InstantiateMsg {
                start_time: config.start_time,
                end_time: config.end_time,
                maintainer_address,
                max_whitelist_address_count: config.max_whitelist_address_count,
                max_per_address_mint: config.max_per_address_mint,
                mint_price: config.mint_price,
            };

            msg.maintainer_address = Some(junk_address);

            // failed only admin can update
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();
            app.execute_contract(
                Addr::unchecked(INVALID),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            // successful execution
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            let updated_maintainer_address = config.maintainer_addr;
            println!(
                "updated_maintainer_address {:?}",
                updated_maintainer_address
            );
            assert_eq!(updated_maintainer_address, test_address_addr);

            // Remove maintainer
            msg.maintainer_address = None;
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let updated_maintainer_address = config.maintainer_addr;
            println!(
                "updated_maintainer_address {:?}",
                updated_maintainer_address
            );
            assert_eq!(updated_maintainer_address, None);

            app.update_block(|mut block| block.time = Timestamp::from_seconds(1771797428));

            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg),
                &[],
            )
            .unwrap_err();
        }

        #[test]
        fn update_maintainer_address_2() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            println!("config {:?}", config);

            let init_maintainer_address = config.maintainer_addr;
            println!("init_maintainer_address {:?}", init_maintainer_address);
            assert_eq!(
                init_maintainer_address,
                Some(Addr::unchecked(USER.to_owned()))
            );

            let junk_address: String = "junkcontract1".to_string();
            let test_address_addr = Some(Addr::unchecked(junk_address.clone()));

            // failed only admin can update
            let msg = ExecuteMsg::UpdateMaintainerAddress(Some(junk_address.clone()));

            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let _res = app
                .execute(Addr::unchecked(USER), cosmos_msg.clone())
                .unwrap_err();
            let _res = app
                .execute(Addr::unchecked(INVALID), cosmos_msg)
                .unwrap_err();

            // successful execution
            let msg = ExecuteMsg::UpdateMaintainerAddress(Some(junk_address));

            let cosmos_msg = cw_template_contract.call(msg.clone()).unwrap();
            let _res = app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            let updated_maintainer_address = config.maintainer_addr;
            println!(
                "updated_maintainer_address {:?}",
                updated_maintainer_address
            );
            assert_eq!(updated_maintainer_address, test_address_addr);

            // Remove maintainer
            let _res = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    cw_template_contract.addr(),
                    &ExecuteMsg::UpdateMaintainerAddress(None),
                    &[],
                )
                .unwrap();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let updated_maintainer_address = config.maintainer_addr;
            println!(
                "updated_maintainer_address {:?}",
                updated_maintainer_address
            );
            assert_eq!(updated_maintainer_address, None);

            app.update_block(|mut block| block.time = Timestamp::from_seconds(1771797428));

            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let res = app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap_err();
            println!("res {:?}", res);
        }

        #[test]
        fn update_start_time() {
            let (mut app, cw_template_contract) = proper_instantiate();

            // init with  1571797420
            // block time 1571797419879305533
            // end time   1656801750

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let maintainer_address: Option<String> =
                config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = InstantiateMsg {
                start_time: config.start_time,
                end_time: config.end_time,
                maintainer_address,
                max_whitelist_address_count: config.max_whitelist_address_count,
                max_per_address_mint: config.max_per_address_mint,
                mint_price: config.mint_price,
            };

            // FAIL before block time
            let new_start_time = Timestamp::from_seconds(1571797419);
            msg.start_time = new_start_time;
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            // FAIL after END time
            let new_start_time = Timestamp::from_seconds(1676801750);
            msg.start_time = new_start_time;
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            // SUCCESS for user
            let new_start_time = Timestamp::from_seconds(1571797429);
            msg.start_time = new_start_time;
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert_eq!(config.start_time, new_start_time);

            // SUCCESS for admin
            let new_start_time = Timestamp::from_seconds(1571797430);
            msg.start_time = new_start_time;
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert_eq!(config.start_time, new_start_time);

            // fail, not a user
            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            let old_start_time = config.start_time;
            let new_start_time = Timestamp::from_seconds(1571797431);
            msg.start_time = new_start_time;
            app.execute_contract(
                Addr::unchecked(INVALID),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert_eq!(config.start_time, old_start_time);

            // tiem travel
            app.update_block(|mut block| block.time = Timestamp::from_seconds(1771797428));
            let new_start_time = Timestamp::from_seconds(1571797429);
            msg.start_time = new_start_time;
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            msg.start_time = new_start_time;
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg),
                &[],
            )
            .unwrap_err();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert!(config.whitelist_is_closed);
        }

        #[test]
        fn update_end_time() {
            let (mut app, cw_template_contract) = proper_instantiate();

            // init with  1571797420
            // block time 1571797419879305533
            // end time   1656801750

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let maintainer_address: Option<String> =
                config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = InstantiateMsg {
                start_time: config.start_time,
                end_time: config.end_time,
                maintainer_address,
                max_whitelist_address_count: config.max_whitelist_address_count,
                max_per_address_mint: config.max_per_address_mint,
                mint_price: config.mint_price,
            };

            // FAIL before start time
            let new_end_time = Timestamp::from_nanos(1571797419879305534);
            msg.end_time = new_end_time;
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            // FAIL before block time
            let new_end_time = Timestamp::from_seconds(1571797419);
            msg.end_time = new_end_time;
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            // SUCCESS for user
            let new_end_time = Timestamp::from_seconds(1571797429);
            msg.end_time = new_end_time;
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert_eq!(config.end_time, new_end_time);

            // SUCCESS for admin
            let new_end_time = Timestamp::from_seconds(1571797430);
            msg.end_time = new_end_time;
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert_eq!(config.end_time, new_end_time);

            // fail, not a user
            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            let old_end_time = config.end_time;
            let new_end_time = Timestamp::from_seconds(1571797431);
            msg.end_time = new_end_time;
            app.execute_contract(
                Addr::unchecked(INVALID),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert_eq!(config.end_time, old_end_time);

            // SUCCESS for admin AFTER current end time
            let new_end_time = Timestamp::from_seconds(1656801751);
            msg.end_time = new_end_time;
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert_eq!(config.end_time, new_end_time);

            // tiem travel
            app.update_block(|mut block| block.time = Timestamp::from_seconds(1771797428));
            let new_end_time = Timestamp::from_seconds(1571797429);
            msg.end_time = new_end_time;
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg),
                &[],
            )
            .unwrap_err();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert!(config.whitelist_is_closed);
        }

        #[test]
        fn update_max_whitelist_address_count() {
            let (mut app, cw_template_contract) = proper_instantiate();

            // init with  5
            // end time   10000
            // 1571797421 jump to time

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let maintainer_address: Option<String> =
                config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = InstantiateMsg {
                start_time: config.start_time,
                end_time: config.end_time,
                maintainer_address,
                max_whitelist_address_count: config.max_whitelist_address_count,
                max_per_address_mint: config.max_per_address_mint,
                mint_price: config.mint_price,
            };

            // FAIL too high
            let new_count: u32 = 10001;
            msg.max_whitelist_address_count = new_count;
            app.execute_contract(
                Addr::unchecked(INVALID),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            // good to go for USER
            let new_count: u32 = 6;
            msg.max_whitelist_address_count = new_count;
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert_eq!(config.max_whitelist_address_count, new_count);

            // good to go for ADMIN
            let new_count: u32 = 7;
            msg.max_whitelist_address_count = new_count;
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert_eq!(config.max_whitelist_address_count, new_count);

            // tiem travel
            app.update_block(|mut block| block.time = Timestamp::from_seconds(1771797428));
            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            let old_count = config.max_whitelist_address_count;
            let new_count: u32 = 7;
            msg.max_whitelist_address_count = new_count;
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg),
                &[],
            )
            .unwrap_err();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert_eq!(config.max_whitelist_address_count, old_count);
        }

        #[test]
        fn update_max_per_address_mint() {
            let (mut app, cw_template_contract) = proper_instantiate();

            // init with  3
            // max value  100
            // 1571797421 jump to time

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let maintainer_address: Option<String> =
                config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = InstantiateMsg {
                start_time: config.start_time,
                end_time: config.end_time,
                maintainer_address,
                max_whitelist_address_count: config.max_whitelist_address_count,
                max_per_address_mint: config.max_per_address_mint,
                mint_price: config.mint_price,
            };

            // FAIL too high
            let new_count: u32 = 10001;
            msg.max_per_address_mint = new_count;
            app.execute_contract(
                Addr::unchecked(INVALID),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            // good to go for USER
            let new_count: u32 = 6;
            msg.max_per_address_mint = new_count;
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert_eq!(config.max_per_address_mint, new_count);

            // good to go for ADMIN
            let new_count: u32 = 7;
            msg.max_per_address_mint = new_count;
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert_eq!(config.max_per_address_mint, new_count);

            // tiem travel
            app.update_block(|mut block| block.time = Timestamp::from_seconds(1771797428));
            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            let old_count = config.max_per_address_mint;
            let new_count: u32 = 7;
            msg.max_per_address_mint = new_count;
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();
            msg.max_per_address_mint = new_count;
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg),
                &[],
            )
            .unwrap_err();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert_eq!(config.max_per_address_mint, old_count);
        }

        #[test]
        fn update_max_whitelist_address_count_2() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let maintainer_address: Option<String> =
                config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = InstantiateMsg {
                start_time: config.start_time,
                end_time: config.end_time,
                maintainer_address,
                max_whitelist_address_count: config.max_whitelist_address_count,
                max_per_address_mint: config.max_per_address_mint,
                mint_price: config.mint_price,
            };

            let init_max_whitelist_address_count = config.max_whitelist_address_count;

            let test_update: u32 = 10000;
            msg.max_whitelist_address_count = test_update;
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg),
                &[],
            )
            .unwrap();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            println!("config {:?}", config);

            let updated_max_whitelist_address_count = config.max_whitelist_address_count;

            assert_eq!(init_max_whitelist_address_count, 5);
            assert_ne!(
                init_max_whitelist_address_count,
                updated_max_whitelist_address_count
            );
            assert_eq!(updated_max_whitelist_address_count, test_update);
            //assert_eq!(5, 7);
        }

        #[test]
        fn update_max_per_address_mint_2() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let maintainer_address: Option<String> =
                config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = InstantiateMsg {
                start_time: config.start_time,
                end_time: config.end_time,
                maintainer_address,
                max_whitelist_address_count: config.max_whitelist_address_count,
                max_per_address_mint: config.max_per_address_mint,
                mint_price: config.mint_price,
            };

            let init_max_per_address_mint = config.max_per_address_mint;

            let test_update: u32 = 100;
            msg.max_per_address_mint = test_update;
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg),
                &[],
            )
            .unwrap();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            println!("config {:?}", config);

            let updated_max_per_address_mint = config.max_per_address_mint;

            assert_eq!(init_max_per_address_mint, 3);
            assert_ne!(init_max_per_address_mint, updated_max_per_address_mint);
            assert_eq!(updated_max_per_address_mint, test_update);
            //assert_eq!(5, 7);
        }

        #[test]
        fn update_mint_price() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let maintainer_address: Option<String> =
                config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = InstantiateMsg {
                start_time: config.start_time,
                end_time: config.end_time,
                maintainer_address,
                max_whitelist_address_count: config.max_whitelist_address_count,
                max_per_address_mint: config.max_per_address_mint,
                mint_price: config.mint_price,
            };

            // failed only admin can update
            msg.mint_price = Uint128::from(TEST_MINT_PRICE);
            app.execute_contract(
                Addr::unchecked(INVALID),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap();
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg),
                &[],
            )
            .unwrap();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert_eq!(config.mint_price, Uint128::from(TEST_MINT_PRICE));

            app.update_block(|mut block| block.time = Timestamp::from_seconds(1771797428));
        }
    }

    mod whitelist_checks {
        use super::*;

        #[test]
        fn add_to_whitelist_1() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            let init_addresses = proper_init_whitelist(&mut app, cw_template_contract.clone());

            let addresses_response: Vec<String> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetWhitelistAddresses {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("addresses_response {:?}", addresses_response);

            // 5, 5
            assert_eq!(init_addresses.len(), addresses_response.len());
        }

        #[test]
        fn add_to_whitelist_2() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let maintainer_address: Option<String> =
                config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = InstantiateMsg {
                start_time: config.start_time,
                end_time: config.end_time,
                maintainer_address,
                max_whitelist_address_count: config.max_whitelist_address_count,
                max_per_address_mint: config.max_per_address_mint,
                mint_price: config.mint_price,
            };

            let _init_addresses = proper_init_whitelist(&mut app, cw_template_contract.clone());

            let mut addresses = vec![];

            addresses.push(format!("test_addr{}", 1));
            addresses.push(format!("test_addr{}", 6));
            addresses.push(format!("test_addr{}", 7));
            addresses.push(format!("test_addr{}", 8));
            addresses.push(format!("test_addr{}", 9));
            addresses.push(format!("test_addr{}", 6));
            addresses.push(format!("test_addr{}", 6));

            println!("addresses {:?}", addresses);

            msg.max_whitelist_address_count = 100;
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg),
                &[],
            )
            .unwrap();

            let msg = ExecuteMsg::AddToWhitelist(addresses);
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let res = app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();

            println!("{:?}", res);

            let addresses_response: Vec<String> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetWhitelistAddresses {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("addresses_response {:?}", addresses_response);

            assert_eq!(9, addresses_response.len());
        }

        #[test]
        fn add_to_whitelist_3() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let _init_addresses = proper_init_whitelist(&mut app, cw_template_contract.clone());

            let mut addresses = vec![];

            addresses.push(format!("test_addr{}", 1));
            addresses.push(format!("test_addr{}", 3));

            println!("addresses {:?}", addresses);

            let msg = ExecuteMsg::RemoveFromWhitelist(addresses);
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let res = app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();

            println!("{:?}", res);

            let addresses_response: Vec<String> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetWhitelistAddresses {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("addresses_response {:?}", addresses_response);

            assert_eq!(3, addresses_response.len());
        }

        #[test]
        fn add_to_whitelist_4() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let _init_addresses = proper_init_whitelist(&mut app, cw_template_contract.clone());

            let mut addresses = vec![];

            addresses.push(format!("test_addr{}", 1));
            addresses.push(format!("test_addr{}", 6));
            addresses.push(format!("test_addr{}", 7));
            addresses.push(format!("test_addr{}", 8));
            addresses.push(format!("test_addr{}", 9));
            addresses.push(format!("test_addr{}", 6));
            addresses.push(format!("test_addr{}", 6));
            addresses.push(format!("test_addr{}", 7));

            println!("addresses {:?}", addresses);

            let msg = ExecuteMsg::AddToWhitelist(addresses);
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let res = app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap_err();
            // errors out, max slots reached
            println!("{:?}", res);

            let addresses_response: Vec<String> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetWhitelistAddresses {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("addresses_response {:?}", addresses_response);

            assert_eq!(5, addresses_response.len());
        }

        // lots of add/removes
        #[test]
        fn alter_whitelist_1() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let maintainer_address: Option<String> =
                config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = InstantiateMsg {
                start_time: config.start_time,
                end_time: config.end_time,
                maintainer_address,
                max_whitelist_address_count: config.max_whitelist_address_count,
                max_per_address_mint: config.max_per_address_mint,
                mint_price: config.mint_price,
            };

            let _init_addresses = proper_init_whitelist(&mut app, cw_template_contract.clone());

            let mut addresses = vec![];

            addresses.push(format!("test_addr{}", 1));
            addresses.push(format!("test_addr{}", 3));

            println!("addresses {:?}", addresses);

            app.execute_contract(
                Addr::unchecked(INVALID),
                cw_template_contract.addr(),
                &ExecuteMsg::RemoveFromWhitelist(addresses),
                &[],
            )
            .unwrap_err();

            let addresses_response: Vec<String> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetWhitelistAddresses {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("addresses_response {:?}", addresses_response);

            assert_eq!(addresses_response.len(), 5);

            let mut addresses = vec![];

            addresses.push(format!("test_addr{}", 1));
            addresses.push(format!("test_addr{}", 6));
            addresses.push(format!("test_addr{}", 10));
            addresses.push(format!("test_addr{}", 3));
            addresses.push(format!("test_addr{}", 9));
            addresses.push(format!("test_addr{}", 6));
            addresses.push(format!("test_addr{}", 6));

            println!("addresses {:?}", addresses);

            // failed only admin can update
            msg.max_whitelist_address_count = 100;
            app.execute_contract(
                Addr::unchecked(INVALID),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg),
                &[],
            )
            .unwrap();

            let msg = ExecuteMsg::AddToWhitelist(addresses);
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let _res = app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();

            let addresses_response: Vec<String> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetWhitelistAddresses {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("addresses_response {:?}", addresses_response);

            assert_eq!(8, addresses_response.len());

            let mut addresses = vec![];

            addresses.push(format!("test_addr{}", 11));

            println!("addresses {:?}", addresses);

            let msg = ExecuteMsg::AddToWhitelist(addresses);
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let _res = app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();

            assert_eq!(8, addresses_response.len());

            let mut addresses = vec![];

            addresses.push(format!("test_addr{}", 13));

            println!("addresses {:?}", addresses);

            let msg = ExecuteMsg::AddToWhitelist(addresses);
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let _res = app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();

            assert_eq!(8, addresses_response.len());

            let mut addresses = vec![];

            addresses.push(format!("test_addr{}", 11));
            addresses.push(format!("test_addr{}", 30));
            addresses.push(format!("test_addr{}", 6));
            addresses.push(format!("test_addr{}", 6));
            addresses.push(format!("test_addr{}", 6));

            println!("addresses {:?}", addresses);

            let msg = ExecuteMsg::RemoveFromWhitelist(addresses);
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let res = app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();

            println!("{:?}", res);

            let addresses_response: Vec<String> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetWhitelistAddresses {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("addresses_response {:?}", addresses_response);

            assert_eq!(8, addresses_response.len());
        }

        #[test]
        fn user_whitelist_add() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let mut addresses = vec![];
            for i in 0..5 {
                addresses.push(format!("test_addr{}", i));
            }

            let _init_addresses = addresses.clone();

            println!("addresses {:?}", addresses);

            let msg = ExecuteMsg::AddToWhitelist(addresses);
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let res = app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();

            println!("{:?}", res);

            let addresses_response: Vec<String> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetWhitelistAddresses {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("addresses_response {:?}", addresses_response);

            assert_eq!(5, addresses_response.len());
        }

        #[test]
        fn user_whitelist_remove() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let _init_addresses = proper_init_whitelist(&mut app, cw_template_contract.clone());

            let mut addresses = vec![];

            addresses.push(format!("test_addr{}", 1));
            addresses.push(format!("test_addr{}", 3));

            println!("addresses {:?}", addresses);

            let msg = ExecuteMsg::RemoveFromWhitelist(addresses);
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let res = app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();

            println!("{:?}", res);

            let addresses_response: Vec<String> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetWhitelistAddresses {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("addresses_response {:?}", addresses_response);

            assert_eq!(3, addresses_response.len());
        }

        #[test]
        fn invalid_whitelist_add() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let mut addresses = vec![];
            for i in 0..5 {
                addresses.push(format!("test_addr{}", i));
            }

            let _init_addresses = addresses.clone();

            println!("addresses {:?}", addresses);

            let msg = ExecuteMsg::AddToWhitelist(addresses);
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let res = app
                .execute(Addr::unchecked(INVALID), cosmos_msg)
                .unwrap_err();

            println!("{:?}", res);

            let addresses_response: Vec<String> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetWhitelistAddresses {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("addresses_response {:?}", addresses_response);

            assert_eq!(0, addresses_response.len());
        }

        #[test]
        fn invalid_whitelist_remove() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let _init_addresses = proper_init_whitelist(&mut app, cw_template_contract.clone());

            let mut addresses = vec![];

            addresses.push(format!("test_addr{}", 1));
            addresses.push(format!("test_addr{}", 3));

            println!("addresses {:?}", addresses);

            let msg = ExecuteMsg::RemoveFromWhitelist(addresses);
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let res = app
                .execute(Addr::unchecked(INVALID), cosmos_msg)
                .unwrap_err();

            println!("{:?}", res);

            let addresses_response: Vec<String> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetWhitelistAddresses {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("addresses_response {:?}", addresses_response);

            assert_eq!(5, addresses_response.len());
        }
    }

    mod query_checks {
        use super::*;

        #[test]
        fn check_whitelist_response() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let _init_addresses = proper_init_whitelist(&mut app, cw_template_contract.clone());
            let valid_minter_address = "test_addr1".to_string();
            let invalid_minter_address = "test_addr69".to_string();

            // check for valid result on whitelist
            let check_whitelist_response: CheckWhitelistResponse = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::CheckWhitelist {
                        minter_address: valid_minter_address.clone(),
                    },
                )
                .unwrap();

            println!("check_whitelist_response {:?}", check_whitelist_response);

            assert!(check_whitelist_response.is_on_whitelist);

            // check invalid address is not on whitelist
            let check_whitelist_response: CheckWhitelistResponse = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::CheckWhitelist {
                        minter_address: invalid_minter_address,
                    },
                )
                .unwrap();

            println!("check_whitelist_response {:?}", check_whitelist_response);

            assert!(!check_whitelist_response.is_on_whitelist);

            //check if whitelist is in progress
            //app.update_block(|mut block| block.height += 1);
            app.update_block(|mut block| block.time = Timestamp::from_seconds(1571797428));

            let check_whitelist_response: CheckWhitelistResponse = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::CheckWhitelist {
                        minter_address: valid_minter_address.clone(),
                    },
                )
                .unwrap();

            println!("check_whitelist_response {:?}", check_whitelist_response);

            assert!(check_whitelist_response.whitelist_in_progress);

            // update tracker then check result
            let msg = ExecuteMsg::UpdateAddressMintTracker(valid_minter_address.clone());
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let res = app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();

            println!("{:?}", res);

            let check_whitelist_response: CheckWhitelistResponse = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::CheckWhitelist {
                        minter_address: valid_minter_address.clone(),
                    },
                )
                .unwrap();

            println!("check_whitelist_response {:?}", check_whitelist_response);
            let remaining_mint_count = check_whitelist_response.max_per_address_mint
                - check_whitelist_response.current_mint_count;
            assert_eq!(remaining_mint_count, 2);

            // try to update tracker with non admin address should FAIL
            let msg = ExecuteMsg::UpdateAddressMintTracker(valid_minter_address.clone());
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let res = app
                .execute(Addr::unchecked(INVALID), cosmos_msg)
                .unwrap_err();

            println!("{:?}", res);

            let check_whitelist_response: CheckWhitelistResponse = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::CheckWhitelist {
                        minter_address: valid_minter_address.clone(),
                    },
                )
                .unwrap();

            println!("check_whitelist_response {:?}", check_whitelist_response);
            let remaining_mint_count = check_whitelist_response.max_per_address_mint
                - check_whitelist_response.current_mint_count;
            assert_eq!(remaining_mint_count, 2);

            // update time then ensure whitelist is closed
            app.update_block(|mut block| block.time = Timestamp::from_seconds(1771797428));

            let check_whitelist_response: CheckWhitelistResponse = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::CheckWhitelist {
                        minter_address: valid_minter_address,
                    },
                )
                .unwrap();

            println!("check_whitelist_response {:?}", check_whitelist_response);

            assert!(!check_whitelist_response.whitelist_in_progress);
            assert!(check_whitelist_response.whitelist_is_closed);
        }

        #[test]
        fn ensure_cannot_update_after_end() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let maintainer_address: Option<String> =
                config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = InstantiateMsg {
                start_time: config.start_time,
                end_time: config.end_time,
                maintainer_address,
                max_whitelist_address_count: config.max_whitelist_address_count,
                max_per_address_mint: config.max_per_address_mint,
                mint_price: config.mint_price,
            };

            app.update_block(|mut block| block.time = Timestamp::from_seconds(1771797428));

            // FAILED update maintainer due to USER
            msg.maintainer_address = Some("junk_address1".to_string());
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            // FAILED update maintainer due to TIME
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg),
                &[],
            )
            .unwrap_err();

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            assert_eq!(
                config.maintainer_addr,
                Some(Addr::unchecked(USER.to_owned()))
            )
        }
    }
}
