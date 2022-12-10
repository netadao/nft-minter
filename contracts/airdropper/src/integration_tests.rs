#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::{
        AddressPromisedTokensResponse, AddressTokenMsg, AddressValMsg, ExecuteMsg, InstantiateMsg,
        QueryMsg, TokenMsg,
    };
    use crate::state::Config;
    use cosmwasm_std::{Addr, Coin, Empty, Timestamp, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::query::query,
        );
        Box::new(contract)
    }

    const USER: &str = "user";
    const USER1: &str = "user1";
    const USER2: &str = "user2";
    const USER3: &str = "user3";
    const ADMIN: &str = "admin";
    const MAINTAINER: &str = "maintainer";
    const INVALID: &str = "invalid";
    const NATIVE_DENOM: &str = "juno";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1_000_000),
                    }],
                )
                .unwrap();
        })
    }

    fn proper_instantiate() -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());

        let msg = InstantiateMsg {
            start_time: Timestamp::from_seconds(1571797420), // 1571797419
            end_time: None,
            maintainer_address: Some(MAINTAINER.to_string()),
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

    mod execute {
        use super::*;

        #[test]
        fn update_maintainer_address_1() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let config: Config = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            println!("config {:?}", config);

            let init_maintainer_address = config.maintainer_addr;
            println!("init_maintainer_address {:?}", init_maintainer_address);
            assert_eq!(
                init_maintainer_address,
                Some(Addr::unchecked(MAINTAINER.to_owned()))
            );

            let junk_address: String = "junkcontract1".to_string();
            let test_address_addr = Addr::unchecked(junk_address.clone());

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

            let config: Config = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            let updated_maintainer_address = config.maintainer_addr;
            println!(
                "updated_maintainer_address {:?}",
                updated_maintainer_address
            );
            assert_eq!(updated_maintainer_address, Some(test_address_addr));

            // Remove maintainer
            let _res = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    cw_template_contract.addr(),
                    &ExecuteMsg::UpdateMaintainerAddress(None),
                    &[],
                )
                .unwrap();

            let config: Config = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            let updated_maintainer_address = config.maintainer_addr;
            println!(
                "updated_maintainer_address {:?}",
                updated_maintainer_address
            );
            assert_eq!(updated_maintainer_address, None);

            app.update_block(|mut block| block.time = Timestamp::from_seconds(1771797428));

            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            let res = app
                .execute(Addr::unchecked(INVALID), cosmos_msg)
                .unwrap_err();
            println!("res {:?}", res);
        }

        #[test]
        fn update_start_time() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let config: Config = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let maintainer_address: Option<String> =
                config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = InstantiateMsg {
                maintainer_address,
                start_time: config.start_time,
                end_time: config.end_time,
            };

            let new_start_time = Timestamp::from_seconds(1571797419);

            // unauthorized
            msg.start_time = new_start_time;
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            // before block time
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            let new_start_time = Timestamp::from_seconds(1571797421);
            msg.start_time = new_start_time;
            // success
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap();

            let new_start_time = Timestamp::from_seconds(1571797422);
            msg.start_time = new_start_time;
            app.execute_contract(
                Addr::unchecked(MAINTAINER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg),
                &[],
            )
            .unwrap();
        }

        #[test]
        fn update_end_time() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let config: Config = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let maintainer_address: Option<String> =
                config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = InstantiateMsg {
                maintainer_address,
                start_time: config.start_time,
                end_time: config.end_time,
            };

            let new_end_time = Timestamp::from_seconds(1571797419);
            msg.end_time = Some(new_end_time);
            // unauthorized
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            let new_end_time = Timestamp::from_seconds(1561797419);
            msg.end_time = Some(new_end_time);
            // before start/block time
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            let new_end_time = Timestamp::from_seconds(1581797419);
            msg.end_time = Some(new_end_time);
            // before start/block time
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap();

            // unauthorized
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            // before start/block time
            msg.end_time = None;
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg),
                &[],
            )
            .unwrap();

            let config: Config = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            assert_eq!(config.end_time, None);
        }
    }

    mod interaction_promised_token_ids {
        use super::*;

        #[test]
        fn add_promised_token_ids() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let promised_tokens_arr: Vec<AddressTokenMsg> = vec![
                AddressTokenMsg {
                    address: USER.to_owned(),
                    token: TokenMsg {
                        collection_id: 1,
                        token_id: 5,
                    },
                },
                AddressTokenMsg {
                    address: USER.to_owned(),
                    token: TokenMsg {
                        collection_id: 1,
                        token_id: 7,
                    },
                },
                AddressTokenMsg {
                    address: USER.to_owned(),
                    token: TokenMsg {
                        collection_id: 1,
                        token_id: 1,
                    },
                },
            ];

            // unauthorized
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::AddPromisedTokenIDs(promised_tokens_arr.clone()),
                &[],
            )
            .unwrap_err();

            // success
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::AddPromisedTokenIDs(promised_tokens_arr),
                &[],
            )
            .unwrap();

            let promised_tokens_response: Vec<AddressPromisedTokensResponse> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetAddressPromisedTokenIDs {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("{:?}", promised_tokens_response);

            let get_assigned_token_ids: Vec<TokenMsg> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetAssignedTokenIDs {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("get_assigned_token_ids {:?}", get_assigned_token_ids);
        }

        #[test]
        fn remove_promised_token_ids_by_address() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let promised_tokens_arr: Vec<AddressTokenMsg> = vec![
                AddressTokenMsg {
                    address: USER.to_owned(),
                    token: TokenMsg {
                        collection_id: 1,
                        token_id: 5,
                    },
                },
                AddressTokenMsg {
                    address: USER.to_owned(),
                    token: TokenMsg {
                        collection_id: 1,
                        token_id: 7,
                    },
                },
                AddressTokenMsg {
                    address: USER.to_owned(),
                    token: TokenMsg {
                        collection_id: 1,
                        token_id: 1,
                    },
                },
                AddressTokenMsg {
                    address: ADMIN.to_owned(),
                    token: TokenMsg {
                        collection_id: 1,
                        token_id: 2,
                    },
                },
            ];

            // success
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::AddPromisedTokenIDs(promised_tokens_arr),
                &[],
            )
            .unwrap();

            let promised_tokens_response: Vec<AddressPromisedTokensResponse> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetAddressPromisedTokenIDs {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("promised_tokens_response {:?}", promised_tokens_response);

            let get_assigned_token_ids: Vec<TokenMsg> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetAssignedTokenIDs {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("get_assigned_token_ids {:?}", get_assigned_token_ids);

            assert_eq!(
                get_assigned_token_ids,
                vec![
                    TokenMsg {
                        collection_id: 1,
                        token_id: 1
                    },
                    TokenMsg {
                        collection_id: 1,
                        token_id: 2
                    },
                    TokenMsg {
                        collection_id: 1,
                        token_id: 5
                    },
                    TokenMsg {
                        collection_id: 1,
                        token_id: 7
                    }
                ]
            );

            let addresses: Vec<String> = vec![USER.to_owned()];

            // unauthorized
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::RemovePromisedTokensByAddress(addresses.clone()),
                &[],
            )
            .unwrap_err();

            // success
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::RemovePromisedTokensByAddress(addresses),
                &[],
            )
            .unwrap();

            let get_assigned_token_ids: Vec<TokenMsg> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetAssignedTokenIDs {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("get_assigned_token_ids {:?}", get_assigned_token_ids);

            assert_eq!(
                get_assigned_token_ids,
                vec![TokenMsg {
                    collection_id: 1,
                    token_id: 2,
                }]
            );
        }

        #[test]
        fn remove_promised_token_ids_by_id() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let promised_tokens_arr: Vec<AddressTokenMsg> = vec![
                AddressTokenMsg {
                    address: USER.to_owned(),
                    token: TokenMsg {
                        collection_id: 1,
                        token_id: 5,
                    },
                },
                AddressTokenMsg {
                    address: USER.to_owned(),
                    token: TokenMsg {
                        collection_id: 1,
                        token_id: 7,
                    },
                },
                AddressTokenMsg {
                    address: USER1.to_owned(),
                    token: TokenMsg {
                        collection_id: 1,
                        token_id: 1,
                    },
                },
            ];

            // success
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::AddPromisedTokenIDs(promised_tokens_arr),
                &[],
            )
            .unwrap();

            let promised_tokens_response: Vec<AddressPromisedTokensResponse> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetAddressPromisedTokenIDs {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("promised_tokens_response {:?}", promised_tokens_response);

            let get_assigned_token_ids: Vec<TokenMsg> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetAssignedTokenIDs {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("get_assigned_token_ids {:?}", get_assigned_token_ids);

            assert_eq!(
                get_assigned_token_ids,
                vec![
                    TokenMsg {
                        collection_id: 1,
                        token_id: 1,
                    },
                    TokenMsg {
                        collection_id: 1,
                        token_id: 5,
                    },
                    TokenMsg {
                        collection_id: 1,
                        token_id: 7,
                    }
                ]
            );

            let remove_token_ids: Vec<TokenMsg> = vec![
                TokenMsg {
                    collection_id: 1,
                    token_id: 1,
                },
                TokenMsg {
                    collection_id: 1,
                    token_id: 5,
                },
            ];

            // unauthorized
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::RemovePromisedTokenIDs(remove_token_ids.clone()),
                &[],
            )
            .unwrap_err();

            // success
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::RemovePromisedTokenIDs(remove_token_ids),
                &[],
            )
            .unwrap();

            let get_assigned_token_ids: Vec<TokenMsg> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetAssignedTokenIDs {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("get_assigned_token_ids {:?}", get_assigned_token_ids);

            assert_eq!(
                get_assigned_token_ids,
                vec![TokenMsg {
                    collection_id: 1,
                    token_id: 7,
                }]
            );
        }

        #[test]
        fn mark_token_id_claimed() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let promised_tokens_arr: Vec<AddressTokenMsg> = vec![
                AddressTokenMsg {
                    address: USER.to_owned(),
                    token: TokenMsg {
                        collection_id: 1,
                        token_id: 5,
                    },
                },
                AddressTokenMsg {
                    address: USER.to_owned(),
                    token: TokenMsg {
                        collection_id: 1,
                        token_id: 7,
                    },
                },
                AddressTokenMsg {
                    address: USER1.to_owned(),
                    token: TokenMsg {
                        collection_id: 1,
                        token_id: 1,
                    },
                },
            ];

            // unauthorized
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::AddPromisedTokenIDs(promised_tokens_arr.clone()),
                &[],
            )
            .unwrap_err();

            // success
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::AddPromisedTokenIDs(promised_tokens_arr),
                &[],
            )
            .unwrap();

            let promised_tokens_response: Vec<AddressPromisedTokensResponse> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetAddressPromisedTokenIDs {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("{:?}", promised_tokens_response);

            let get_assigned_token_ids: Vec<TokenMsg> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetAssignedTokenIDs {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("get_assigned_token_ids {:?}", get_assigned_token_ids);

            let claim_msg: AddressTokenMsg = AddressTokenMsg {
                address: USER.to_owned(),
                token: TokenMsg {
                    collection_id: 1,
                    token_id: 5,
                },
            };

            // unauthorized
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::MarkTokenIDClaimed(claim_msg.clone()),
                &[],
            )
            .unwrap_err();

            // success
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::MarkTokenIDClaimed(claim_msg),
                &[],
            )
            .unwrap();

            let claimed_token_ids: Vec<AddressTokenMsg> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetClaimedTokenIDsWithAddress {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("get_assigned_token_ids {:?}", claimed_token_ids);

            assert_eq!(claimed_token_ids[0].address, USER.to_owned());
            assert_eq!(
                claimed_token_ids[0].token,
                TokenMsg {
                    collection_id: 1,
                    token_id: 5
                }
            );
        }
    }

    mod interaction_promised_mints {
        use super::*;

        #[test]
        fn add_promised_mints() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let promised_mints_arr: Vec<AddressValMsg> = vec![
                AddressValMsg {
                    address: USER.to_owned(),
                    value: 5,
                },
                AddressValMsg {
                    address: USER1.to_owned(),
                    value: 1,
                },
                AddressValMsg {
                    address: USER2.to_owned(),
                    value: 1,
                },
                AddressValMsg {
                    address: USER3.to_owned(),
                    value: 1,
                },
                AddressValMsg {
                    address: USER.to_owned(),
                    value: 7,
                },
            ];

            // unauthorized
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::AddPromisedMints(promised_mints_arr.clone()),
                &[],
            )
            .unwrap_err();

            // success
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::AddPromisedMints(promised_mints_arr),
                &[],
            )
            .unwrap();

            let promised_mints_response: Vec<AddressValMsg> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetAddressPromisedMints {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("### promised_mints_response {:?}", promised_mints_response);
        }

        #[test]
        fn remove_promised_mints() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let promised_mints_arr: Vec<AddressValMsg> = vec![
                AddressValMsg {
                    address: USER.to_owned(),
                    value: 5,
                },
                AddressValMsg {
                    address: USER1.to_owned(),
                    value: 1,
                },
                AddressValMsg {
                    address: USER2.to_owned(),
                    value: 1,
                },
                AddressValMsg {
                    address: USER3.to_owned(),
                    value: 1,
                },
                AddressValMsg {
                    address: USER.to_owned(),
                    value: 7,
                },
            ];

            // unauthorized
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::AddPromisedMints(promised_mints_arr.clone()),
                &[],
            )
            .unwrap_err();

            // success
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::AddPromisedMints(promised_mints_arr),
                &[],
            )
            .unwrap();

            let remove_addresses: Vec<String> = vec![USER2.to_owned()];

            // unauthorized
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::RemovePromisedMints(remove_addresses.clone()),
                &[],
            )
            .unwrap_err();

            // success
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::RemovePromisedMints(remove_addresses),
                &[],
            )
            .unwrap();

            let promised_mints_response: Vec<AddressValMsg> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetAddressPromisedMints {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("### promised_mints_response {:?}", promised_mints_response);
        }

        #[test]
        fn check_claimed_promised_mints() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let promised_mints_arr: Vec<AddressValMsg> = vec![
                AddressValMsg {
                    address: USER.to_owned(),
                    value: 5,
                },
                AddressValMsg {
                    address: USER1.to_owned(),
                    value: 1,
                },
                AddressValMsg {
                    address: USER2.to_owned(),
                    value: 1,
                },
                AddressValMsg {
                    address: USER3.to_owned(),
                    value: 3,
                },
                AddressValMsg {
                    address: USER.to_owned(),
                    value: 4,
                },
            ];

            // unauthorized
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::AddPromisedMints(promised_mints_arr.clone()),
                &[],
            )
            .unwrap_err();

            // success
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::AddPromisedMints(promised_mints_arr),
                &[],
            )
            .unwrap();

            // unauthorized
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::IncrementAddressClaimedPromisedMintCount(USER.to_owned()),
                &[],
            )
            .unwrap_err();

            // success
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::IncrementAddressClaimedPromisedMintCount(USER.to_owned()),
                &[],
            )
            .unwrap();

            // success
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::IncrementAddressClaimedPromisedMintCount(USER3.to_owned()),
                &[],
            )
            .unwrap();

            // success
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::AddPromisedMints(vec![AddressValMsg {
                    address: USER3.to_owned(),
                    value: 69,
                }]),
                &[],
            )
            .unwrap();

            // success - increment USER again
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::IncrementAddressClaimedPromisedMintCount(USER.to_owned()),
                &[],
            )
            .unwrap();

            let promised_mints_response: Vec<AddressValMsg> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetAddressPromisedMints {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("### promised_mints_response {:?}", promised_mints_response);

            assert_eq!(
                promised_mints_response,
                vec![
                    AddressValMsg {
                        address: USER.to_owned(),
                        value: 4,
                    },
                    AddressValMsg {
                        address: USER1.to_owned(),
                        value: 1,
                    },
                    AddressValMsg {
                        address: USER2.to_owned(),
                        value: 1,
                    },
                    AddressValMsg {
                        address: USER3.to_owned(),
                        value: 69,
                    },
                ]
            );

            let promised_mints_claimed_response: Vec<AddressValMsg> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetClaimedAddressPromisedMints {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!(
                "### promised_mints_claimed_response {:?}",
                promised_mints_claimed_response
            );

            assert_eq!(promised_mints_claimed_response[0].address, USER.to_owned());
            assert_eq!(promised_mints_claimed_response[0].value, 2);
            assert_eq!(promised_mints_claimed_response[1].address, USER3.to_owned());
            assert_eq!(promised_mints_claimed_response[1].value, 1);
        }
    }
}
