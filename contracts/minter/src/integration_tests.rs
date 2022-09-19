#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::{
        Admin, BaseInitMsg, CollectionInfoMsg, ConfigResponse, ExecuteMsg, ExecutionTarget,
        InstantiateMsg, ModuleInstantiateInfo, QueryMsg, RoyaltyInfoMsg, TokenDataResponse,
    };
    use cosmwasm_std::{
        coin, coins, to_binary, Addr, Coin, CosmosMsg, Empty, Timestamp, Uint128, WasmMsg,
    };
    use cw721_base::{MinterResponse, QueryMsg as Cw721QueryMsg};
    use cw_multi_test::{App, AppBuilder, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};
    use prost::Message;

    use whitelist::{
        msg::ConfigResponse as WhitelistConfig, msg::ExecuteMsg as WhitelistExecuteMsg,
        msg::InstantiateMsg as WLInstantiateMsg, msg::QueryMsg as WhitelistQueryMsg,
    };

    use airdropper::{
        msg::AddressPromisedTokensResponse, msg::AddressValMsg as AD_AddressValMsg,
        msg::ExecuteMsg as AirdropperExecuteMsg, msg::InstantiateMsg as ADInstantiateMsg,
        msg::QueryMsg as AirdropperQueryMsg, state::Config as AirdropperConfig,
    };

    // Type for replies to contract instantiate messes
    #[derive(Clone, PartialEq, Message)]
    struct MsgInstantiateContractResponse {
        #[prost(string, tag = "1")]
        pub contract_address: ::prost::alloc::string::String,
        #[prost(bytes, tag = "2")]
        pub data: ::prost::alloc::vec::Vec<u8>,
    }

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::query::query,
        )
        .with_reply(crate::contract::reply);
        Box::new(contract)
    }

    fn cw721_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw721_base::entry::execute,
            cw721_base::entry::instantiate,
            cw721_base::entry::query,
        );
        Box::new(contract)
    }

    fn airdropper_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            airdropper::contract::execute,
            airdropper::contract::instantiate,
            airdropper::query::query,
        );
        Box::new(contract)
    }

    fn whitelist_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            whitelist::contract::execute,
            whitelist::contract::instantiate,
            whitelist::query::query,
        );
        Box::new(contract)
    }

    const USER: &str = "user";
    const USER2: &str = "user2";
    const USER3: &str = "user3";
    const USER10: &str = "user10";
    const USER25: &str = "user25";
    const ADMIN: &str = "admin";
    const NATIVE_DENOM: &str = "ujuno";
    const MAINTAINER_ADDR: &str = "whiskey";
    const INVALID: &str = "invalid";

    const MINT_PRICE: u128 = 2_000_000;
    const WL_MINT_PRICE: u128 = 1_000_000;
    const _TEST_MINT_PRICE: u128 = 1_500_000;

    const _BASE_BLOCK_HEIGHT: u64 = 12345;
    const _BASE_BLOCK_TIME: u64 = 1571797419879305533;
    const WHITELIST_START_TIME: u64 = 1571797420;
    const WHITELIST_END_TIME: u64 = 1591797421;
    const AIRDROPPER_START_TIME: u64 = 1571797420;
    const MINT_START_TIME: u64 = 1601797420;
    const MINT_END_TIME: u64 = 1657801750;

    const MAX_PER_ADDRESS_MINT: u32 = 4;
    const MAX_TOKEN_SUPPLY: u32 = 5;

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(ADMIN),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(10_000_000),
                    }],
                )
                .unwrap();
        })
    }

    fn proper_instantiate(
        init_airdropper: bool,
        init_whitelist: bool,
    ) -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());
        let cw721_id = app.store_code(cw721_contract());
        let airdropper_id = app.store_code(airdropper_contract());
        let whitelist_id = app.store_code(whitelist_contract());

        let mut airdropper_module_instantiate: Option<ModuleInstantiateInfo> = None;
        let mut whitelist_module_instantiate: Option<ModuleInstantiateInfo> = None;

        if init_airdropper {
            let airdropper_instantiate_msg = airdropper::msg::InstantiateMsg {
                maintainer_address: Some(MAINTAINER_ADDR.to_string()),
                start_time: Timestamp::from_seconds(AIRDROPPER_START_TIME),
                end_time: None,
            };

            airdropper_module_instantiate = Some(ModuleInstantiateInfo {
                code_id: airdropper_id,
                msg: to_binary(&airdropper_instantiate_msg).unwrap(),
                admin: Admin::CoreContract {},
                label: "airdropper".to_string(),
            });
        }

        if init_whitelist {
            let whitelist_instantiate_msg = whitelist::msg::InstantiateMsg {
                maintainer_address: Some(MAINTAINER_ADDR.to_string()),
                start_time: Timestamp::from_seconds(WHITELIST_START_TIME),
                end_time: Timestamp::from_seconds(WHITELIST_END_TIME),
                max_whitelist_address_count: 100,
                max_per_address_mint: 2,
                mint_price: Uint128::from(WL_MINT_PRICE),
            };

            whitelist_module_instantiate = Some(ModuleInstantiateInfo {
                code_id: whitelist_id,
                msg: to_binary(&whitelist_instantiate_msg).unwrap(),
                admin: Admin::CoreContract {},
                label: "whitelist".to_string(),
            });
        }

        let collection_info: CollectionInfoMsg = CollectionInfoMsg {
            secondary_metadata_uri: None,
            mint_revenue_share: vec![
                RoyaltyInfoMsg {
                    address: ADMIN.to_owned(),
                    bps: 7000,
                    is_primary: true,
                },
                RoyaltyInfoMsg {
                    address: MAINTAINER_ADDR.to_owned(),
                    bps: 3000,
                    is_primary: false,
                },
            ],
            secondary_market_royalties: vec![
                RoyaltyInfoMsg {
                    address: ADMIN.to_owned(),
                    bps: 1000,
                    is_primary: true,
                },
                RoyaltyInfoMsg {
                    address: MAINTAINER_ADDR.to_owned(),
                    bps: 1000,
                    is_primary: false,
                },
            ],
        };

        let msg = InstantiateMsg {
            base_fields: BaseInitMsg {
                maintainer_address: Some(MAINTAINER_ADDR.to_string()),
                start_time: Timestamp::from_seconds(MINT_START_TIME),
                end_time: Timestamp::from_seconds(MINT_END_TIME),
                max_per_address_mint: MAX_PER_ADDRESS_MINT,
                mint_price: Uint128::from(MINT_PRICE),
                mint_denom: NATIVE_DENOM.to_owned(),
                base_token_uri: "ipfs://QmSw2yJjwYbdVnn27KQFg5ex2Q6G24RxorgX7v72NpFs4v".to_string(),
                token_code_id: cw721_id,
            },
            whitelist_address: None,
            airdrop_address: None,

            max_token_supply: MAX_TOKEN_SUPPLY,

            name: "TESTNFTPROJECT".to_string(),
            symbol: "TESTNFT".to_string(),

            airdropper_instantiate_info: airdropper_module_instantiate,
            whitelist_instantiate_info: whitelist_module_instantiate,
            extension: collection_info,
        };

        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                //&[coin(1_000_000_000, NATIVE_DENOM)],
                "test",
                None,
            )
            .unwrap();

        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: USER.to_string(),
                amount: coins(1_000_001, NATIVE_DENOM),
            }
        }))
        .ok();

        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: USER2.to_string(),
                amount: coins(2_000_000, NATIVE_DENOM),
            }
        }))
        .ok();

        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: USER3.to_string(),
                amount: coins(3_000_000, NATIVE_DENOM),
            }
        }))
        .ok();

        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: USER10.to_string(),
                amount: coins(10_000_000, NATIVE_DENOM),
            }
        }))
        .ok();

        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: USER25.to_string(),
                amount: coins(25_000_000, NATIVE_DENOM),
            }
        }))
        .ok();

        let cw_template_contract = CwTemplateContract(
            cw_template_contract_addr,
            cw721_id,
            airdropper_id,
            whitelist_id,
        );
        (app, cw_template_contract)
    }

    mod init {
        use super::*;
        use crate::msg::QueryMsg;

        #[test]
        fn proper_init() {
            let (app, cw_template_contract) = proper_instantiate(true, true);

            println!(
                "cw_template_contract.addr() {:?}",
                cw_template_contract.addr()
            );

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("config {:?}", config);

            let nft_addr = config.cw721_addr.unwrap();

            println!("nft_addr {:?}", nft_addr);

            let airdropper_addr = config.airdropper_addr;

            println!("airdropper_addr {:?}", airdropper_addr);

            let whitelist_addr = config.whitelist_addr;

            println!("whitelist_addr {:?}", whitelist_addr);
            /*
            let shuffled_token_ids: TokensResponse = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetShuffledTokenIds {
                        start_after: None,
                        limit: Some(5),
                    },
                )
                .unwrap();

            println!("shuffled_token_ids {:?}", shuffled_token_ids);
            */
            let nft_minter_query: MinterResponse = app
                .wrap()
                .query_wasm_smart(&nft_addr.to_string(), &Cw721QueryMsg::Minter {})
                .unwrap();
            println!("nft_minter_query {:?}", nft_minter_query);

            assert_eq!(config.max_per_address_mint, 4);
            assert_eq!(
                cw_template_contract.addr().to_string(),
                nft_minter_query.minter
            );

            let balance = app
                .wrap()
                .query_balance(ADMIN.to_string(), NATIVE_DENOM)
                .unwrap();
            println!("balance {:?}", balance);
            assert_eq!(balance, coin(10_000_000, NATIVE_DENOM));

            let balance = app
                .wrap()
                .query_balance(USER.to_string(), NATIVE_DENOM)
                .unwrap();
            println!("balance {:?}", balance);
            assert_eq!(balance, coin(1_000_001, NATIVE_DENOM));

            let balance = app
                .wrap()
                .query_balance(USER2.to_string(), NATIVE_DENOM)
                .unwrap();
            println!("balance {:?}", balance);
            assert_eq!(balance, coin(2_000_000, NATIVE_DENOM));

            let balance = app
                .wrap()
                .query_balance(USER3.to_string(), NATIVE_DENOM)
                .unwrap();
            println!("balance {:?}", balance);
            assert_eq!(balance, coin(3_000_000, NATIVE_DENOM));
        }
    }

    mod updates {
        use super::*;

        #[test]
        fn test_update_maintainer() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let maintainer_address: Option<String> = config
                .maintainer_addr
                .clone()
                .map(|addr| addr.into_string());

            let mut msg: BaseInitMsg = BaseInitMsg {
                maintainer_address,
                start_time: config.start_time,
                end_time: config.end_time,
                max_per_address_mint: config.max_per_address_mint,
                mint_price: config.mint_price,
                mint_denom: config.mint_denom,
                base_token_uri: config.base_token_uri,
                token_code_id: config.token_code_id,
            };

            assert_eq!(
                config.maintainer_addr,
                Some(Addr::unchecked(MAINTAINER_ADDR.to_string()))
            );

            let ad_config: AirdropperConfig = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.unwrap(),
                    &AirdropperQueryMsg::GetConfig {},
                )
                .unwrap();

            assert_eq!(
                ad_config.maintainer_addr,
                Some(Addr::unchecked(MAINTAINER_ADDR.to_string()))
            );

            let wl_config: WhitelistConfig = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.unwrap(),
                    &WhitelistQueryMsg::GetConfig {},
                )
                .unwrap();

            assert_eq!(
                wl_config.maintainer_addr,
                Some(Addr::unchecked(MAINTAINER_ADDR.to_string()))
            );

            // unauthorized
            msg.maintainer_address = Some(USER25.to_owned());
            app.execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap_err();

            // success. this should update all 3 contracts
            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg.clone()),
                &[],
            )
            .unwrap();

            // ensure theyve been updated
            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            assert_eq!(
                config.maintainer_addr,
                Some(Addr::unchecked(USER25.to_string()))
            );

            let ad_config: AirdropperConfig = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.unwrap(),
                    &AirdropperQueryMsg::GetConfig {},
                )
                .unwrap();

            assert_eq!(
                ad_config.maintainer_addr,
                Some(Addr::unchecked(USER25.to_string()))
            );

            let wl_config: WhitelistConfig = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.unwrap(),
                    &WhitelistQueryMsg::GetConfig {},
                )
                .unwrap();

            assert_eq!(
                wl_config.maintainer_addr,
                Some(Addr::unchecked(USER25.to_string()))
            );

            msg.maintainer_address = None;
            app.execute_contract(
                Addr::unchecked(USER25),
                cw_template_contract.addr(),
                &ExecuteMsg::UpdateConfig(msg),
                &[],
            )
            .unwrap();

            // ensure theyve been updated to NONE
            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            assert_eq!(config.maintainer_addr, None);

            let ad_config: AirdropperConfig = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.unwrap(),
                    &AirdropperQueryMsg::GetConfig {},
                )
                .unwrap();

            assert_eq!(ad_config.maintainer_addr, None);

            let wl_config: WhitelistConfig = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.unwrap(),
                    &WhitelistQueryMsg::GetConfig {},
                )
                .unwrap();

            assert_eq!(wl_config.maintainer_addr, None);
        }

        #[test]
        fn test_shuffle_order() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);
            /*
                        let shuffled_token_ids: TokensResponse = app
                            .wrap()
                            .query_wasm_smart(
                                &cw_template_contract.addr(),
                                &QueryMsg::GetShuffledTokenIds {
                                    start_after: None,
                                    limit: Some(5),
                                },
                            )
                            .unwrap();

                        println!("shuffled_token_ids {:?}", shuffled_token_ids);
            */
            app.update_block(|mut block| block.height += 1);

            app.execute_contract(
                Addr::unchecked(USER.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::ShuffleTokenOrder {},
                &[],
            )
            .unwrap_err();

            app.execute_contract(
                Addr::unchecked(ADMIN.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::ShuffleTokenOrder {},
                &[],
            )
            .unwrap();
            /*
            let shuffled_token_ids: TokensResponse = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetShuffledTokenIds {
                        start_after: None,
                        limit: Some(5),
                    },
                )
                .unwrap();

            println!("shuffled_token_ids 2 {:?}", shuffled_token_ids);
            */
        }

        #[test]
        fn test_shuffle_order_2() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            /*
                        let shuffled_token_ids: TokensResponse = app
                            .wrap()
                            .query_wasm_smart(
                                &cw_template_contract.addr(),
                                &QueryMsg::GetShuffledTokenIds {
                                    start_after: None,
                                    limit: Some(5),
                                },
                            )
                            .unwrap();
                        assert_eq!(shuffled_token_ids.token_ids, vec![4, 2, 1, 5, 3]);
                        println!("shuffled_token_ids {:?}", shuffled_token_ids);
            */
            app.update_block(|mut block| block.height += 1);

            // public mint starts
            app.update_block(|mut block| block.time = Timestamp::from_seconds(MINT_START_TIME));

            // USER25 mints in public with less than amount
            app.execute_contract(
                Addr::unchecked(USER25),
                cw_template_contract.addr(),
                &ExecuteMsg::Mint {},
                &[coin(2_000_000, NATIVE_DENOM)],
            )
            .unwrap();
            /*
                        let shuffled_token_ids: TokensResponse = app
                            .wrap()
                            .query_wasm_smart(
                                &cw_template_contract.addr(),
                                &QueryMsg::GetShuffledTokenIds {
                                    start_after: None,
                                    limit: Some(5),
                                },
                            )
                            .unwrap();

                        assert_eq!(shuffled_token_ids.token_ids, vec![2, 1, 5, 3]);
                        println!("shuffled_token_ids 2 {:?}", shuffled_token_ids);
            */
            app.execute_contract(
                Addr::unchecked(USER.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::ShuffleTokenOrder {},
                &[],
            )
            .unwrap_err();

            app.execute_contract(
                Addr::unchecked(ADMIN.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::ShuffleTokenOrder {},
                &[],
            )
            .unwrap();
            /*
            let shuffled_token_ids: TokensResponse = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetShuffledTokenIds {
                        start_after: None,
                        limit: Some(5),
                    },
                )
                .unwrap();

            println!("shuffled_token_ids 2 {:?}", shuffled_token_ids);

            let mints: Vec<AddressValMsg> = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetAddressMints {
                        start_after: None,
                        limit: Some(5),
                    },
                )
                .unwrap();

            //println!("mints 2 {:?}", mints);

            //assert_eq!(shuffled_token_ids.token_ids, vec![2, 3, 1, 5]);
            */
        }

        #[test]
        fn test_clean_shuffle() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            app.execute_contract(
                cw_template_contract.addr(),
                config.airdropper_addr.unwrap(),
                &AirdropperExecuteMsg::AddPromisedTokenIDs(vec![
                    AD_AddressValMsg {
                        address: USER.to_owned(),
                        value: 3,
                    },
                    AD_AddressValMsg {
                        address: USER2.to_owned(),
                        value: 2,
                    },
                ]),
                &[],
            )
            .unwrap();

            // execute the list cleaner
            app.execute_contract(
                Addr::unchecked(USER.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::CleanClaimedTokensFromShuffle {},
                &[],
            )
            .unwrap_err();

            app.execute_contract(
                Addr::unchecked(ADMIN.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::CleanClaimedTokensFromShuffle {},
                &[],
            )
            .unwrap();
            /*
                        let shuffled_token_ids: TokensResponse = app
                            .wrap()
                            .query_wasm_smart(
                                &cw_template_contract.addr(),
                                &QueryMsg::GetShuffledTokenIds {
                                    start_after: None,
                                    limit: Some(5),
                                },
                            )
                            .unwrap();

                        println!("shuffled_token_ids 2 {:?}", shuffled_token_ids);
            */
            // first shuffle

            app.update_block(|mut block| block.height += 1);

            app.execute_contract(
                Addr::unchecked(ADMIN.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::ShuffleTokenOrder {},
                &[],
            )
            .unwrap();
            /*
                        let shuffled_token_ids: TokensResponse = app
                            .wrap()
                            .query_wasm_smart(
                                &cw_template_contract.addr(),
                                &QueryMsg::GetShuffledTokenIds {
                                    start_after: None,
                                    limit: Some(5),
                                },
                            )
                            .unwrap();

                        println!("shuffled_token_ids 2 {:?}", shuffled_token_ids);

                        assert_eq!(shuffled_token_ids.token_ids, vec![4, 1, 5]);
            */
            // second shuffle
            app.update_block(|mut block| block.height += 1);

            app.execute_contract(
                Addr::unchecked(ADMIN.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::ShuffleTokenOrder {},
                &[],
            )
            .unwrap();
            /*
            let shuffled_token_ids: TokensResponse = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetShuffledTokenIds {
                        start_after: None,
                        limit: Some(5),
                    },
                )
                .unwrap();

            println!("shuffled_token_ids 2 {:?}", shuffled_token_ids);

            assert_eq!(shuffled_token_ids.token_ids, vec![1, 4, 5]);
            */
        }

        #[test]
        fn reinit_airdropper_submodule() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            println!("config1 {:?}", config);

            let ad_config: AirdropperConfig = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.unwrap(),
                    &AirdropperQueryMsg::GetConfig {},
                )
                .unwrap();

            let module_instantiate = airdropper::msg::InstantiateMsg {
                maintainer_address: Some(USER.to_string()),
                start_time: Timestamp::from_seconds(AIRDROPPER_START_TIME),
                end_time: None,
            };

            let module_info: ModuleInstantiateInfo = ModuleInstantiateInfo {
                code_id: cw_template_contract.airdrop_contract_id(),
                msg: to_binary(&module_instantiate).unwrap(),
                admin: Admin::CoreContract {},
                label: "airdropper2".to_string(),
            };

            // fail ofc
            app.execute_contract(
                Addr::unchecked(INVALID),
                cw_template_contract.addr(),
                &ExecuteMsg::InitSubmodule(module_info.clone()),
                &[],
            )
            .unwrap_err();

            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::InitSubmodule(module_info),
                &[],
            )
            .unwrap();

            let old_ad_maintainer = ad_config.maintainer_addr;

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let ad_config: AirdropperConfig = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.unwrap(),
                    &AirdropperQueryMsg::GetConfig {},
                )
                .unwrap();

            assert_ne!(old_ad_maintainer, ad_config.maintainer_addr);
        }

        #[test]
        fn reinit_whitelist_submodule() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            println!("config1 {:?}", config);

            let wl_config: WhitelistConfig = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.unwrap(),
                    &WhitelistQueryMsg::GetConfig {},
                )
                .unwrap();

            let whitelist_instantiate_msg = whitelist::msg::InstantiateMsg {
                maintainer_address: Some(USER.to_string()),
                start_time: Timestamp::from_seconds(WHITELIST_START_TIME),
                end_time: Timestamp::from_seconds(WHITELIST_END_TIME),
                max_whitelist_address_count: 100,
                max_per_address_mint: 2,
                mint_price: Uint128::from(WL_MINT_PRICE),
            };

            let module_info: ModuleInstantiateInfo = ModuleInstantiateInfo {
                code_id: cw_template_contract.whitelist_contract_id(),
                msg: to_binary(&whitelist_instantiate_msg).unwrap(),
                admin: Admin::CoreContract {},
                label: "whitelist2".to_string(),
            };

            // fail ofc
            app.execute_contract(
                Addr::unchecked(INVALID),
                cw_template_contract.addr(),
                &ExecuteMsg::InitSubmodule(module_info.clone()),
                &[],
            )
            .unwrap_err();

            app.execute_contract(
                Addr::unchecked(ADMIN),
                cw_template_contract.addr(),
                &ExecuteMsg::InitSubmodule(module_info),
                &[],
            )
            .unwrap();

            let old_wl_maintainer = wl_config.maintainer_addr;

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let wl_config: WhitelistConfig = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.unwrap(),
                    &WhitelistQueryMsg::GetConfig {},
                )
                .unwrap();

            assert_ne!(old_wl_maintainer, wl_config.maintainer_addr);
        }
    }

    mod airdropper_interaction {
        use super::*;
        use airdropper::state::Config as AirdropperConfig;

        #[test]
        fn verify_airdropper_init() {
            let (app, cw_template_contract) = proper_instantiate(true, false);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let airdropper_config: AirdropperConfig = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.unwrap(),
                    &AirdropperQueryMsg::GetConfig {},
                )
                .unwrap();

            assert_eq!(
                airdropper_config.maintainer_addr,
                Some(Addr::unchecked(MAINTAINER_ADDR.to_string()))
            );
        }

        #[test]
        fn ad_update_maintainer_address() {
            let (mut app, cw_template_contract) = proper_instantiate(true, false);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            // INVALID EXECUTION
            app.execute_contract(
                Addr::unchecked(INVALID.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::UpdateMaintainerAddress(Some(
                            "notwhiskey".to_string(),
                        )))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::UpdateMaintainerAddress(Some(
                            "notwhiskey".to_string(),
                        )))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            // ADMIN EXECUTION
            app.execute_contract(
                Addr::unchecked(ADMIN.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::UpdateMaintainerAddress(Some(
                            "notwhiskey".to_string(),
                        )))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            let airdropper_config: AirdropperConfig = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.unwrap(),
                    &AirdropperQueryMsg::GetConfig {},
                )
                .unwrap();

            assert_ne!(
                airdropper_config.maintainer_addr,
                Some(Addr::unchecked(MAINTAINER_ADDR.to_string()))
            );
            assert_eq!(
                airdropper_config.maintainer_addr,
                Some(Addr::unchecked("notwhiskey".to_string()))
            );
        }

        #[test]
        fn ad_update_start_time() {
            let (mut app, cw_template_contract) = proper_instantiate(true, false);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let ad_config: AirdropperConfig = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.clone().unwrap(),
                    &AirdropperQueryMsg::GetConfig {},
                )
                .unwrap();

            let maintainer_address: Option<String> =
                ad_config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = ADInstantiateMsg {
                maintainer_address,
                start_time: ad_config.start_time,
                end_time: ad_config.end_time,
            };

            // INVALID EXECUTION
            msg.start_time = Timestamp::from_seconds(MINT_START_TIME);
            app.execute_contract(
                Addr::unchecked(INVALID.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::UpdateConfig(msg.clone())).unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::UpdateConfig(msg.clone())).unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            // ADMIN EXECUTION
            app.execute_contract(
                Addr::unchecked(ADMIN.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::UpdateConfig(msg.clone())).unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            println!("msgmsgmsgmsg {:?}", msg);
            let airdropper_config: AirdropperConfig = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.unwrap(),
                    &AirdropperQueryMsg::GetConfig {},
                )
                .unwrap();

            assert_ne!(
                airdropper_config.start_time,
                Timestamp::from_seconds(AIRDROPPER_START_TIME)
            );
            assert_eq!(
                airdropper_config.start_time,
                Timestamp::from_seconds(MINT_START_TIME)
            );
        }

        #[test]
        fn ad_update_end_time() {
            let (mut app, cw_template_contract) = proper_instantiate(true, false);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let ad_config: AirdropperConfig = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.clone().unwrap(),
                    &AirdropperQueryMsg::GetConfig {},
                )
                .unwrap();

            let maintainer_address: Option<String> =
                ad_config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = ADInstantiateMsg {
                maintainer_address,
                start_time: ad_config.start_time,
                end_time: ad_config.end_time,
            };

            // INVALID EXECUTION
            msg.end_time = Some(Timestamp::from_seconds(MINT_START_TIME));
            app.execute_contract(
                Addr::unchecked(INVALID.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::UpdateConfig(msg.clone())).unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::UpdateConfig(msg.clone())).unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            // ADMIN EXECUTION
            msg.end_time = Some(Timestamp::from_seconds(MINT_START_TIME));
            app.execute_contract(
                Addr::unchecked(ADMIN.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::UpdateConfig(msg.clone())).unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            let airdropper_config: AirdropperConfig = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.clone().unwrap(),
                    &AirdropperQueryMsg::GetConfig {},
                )
                .unwrap();

            println!(
                "airdropper_configairdropper_configairdropper_config{:?}",
                airdropper_config
            );
            assert_ne!(
                airdropper_config.end_time,
                Some(Timestamp::from_seconds(AIRDROPPER_START_TIME))
            );
            assert_eq!(
                airdropper_config.end_time,
                Some(Timestamp::from_seconds(MINT_START_TIME))
            );

            // ADMIN EXECUTION
            msg.end_time = None;
            app.execute_contract(
                Addr::unchecked(ADMIN.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::UpdateConfig(msg)).unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            let airdropper_config: AirdropperConfig = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.unwrap(),
                    &AirdropperQueryMsg::GetConfig {},
                )
                .unwrap();

            println!(
                "airdropper_configairdropper_configairdropper_config{:?}",
                airdropper_config
            );

            assert_eq!(airdropper_config.end_time, None);
        }

        #[test]
        fn ad_add_remove_promised_token_ids() {
            let (mut app, cw_template_contract) = proper_instantiate(true, false);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            // INVALID EXECUTION
            app.execute_contract(
                Addr::unchecked(INVALID.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::AddPromisedTokenIDs(vec![
                            AD_AddressValMsg {
                                address: USER.to_owned(),
                                value: 1,
                            },
                        ]))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::AddPromisedTokenIDs(vec![
                            AD_AddressValMsg {
                                address: USER.to_owned(),
                                value: 1,
                            },
                            AD_AddressValMsg {
                                address: USER2.to_owned(),
                                value: 2,
                            },
                        ]))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            // ADMIN EXECUTION
            app.execute_contract(
                Addr::unchecked(ADMIN.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::AddPromisedTokenIDs(vec![
                            AD_AddressValMsg {
                                address: USER3.to_owned(),
                                value: 3,
                            },
                            AD_AddressValMsg {
                                address: USER10.to_owned(),
                                value: 5,
                            },
                        ]))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            let res: Vec<AddressPromisedTokensResponse> = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.clone().unwrap(),
                    &AirdropperQueryMsg::GetAddressPromisedTokenIDs {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            assert_eq!(
                res,
                vec![
                    AddressPromisedTokensResponse {
                        address: USER.to_owned(),
                        token_ids: vec![1]
                    },
                    AddressPromisedTokensResponse {
                        address: USER10.to_owned(),
                        token_ids: vec![5]
                    },
                    AddressPromisedTokensResponse {
                        address: USER2.to_owned(),
                        token_ids: vec![2]
                    },
                    AddressPromisedTokensResponse {
                        address: USER3.to_owned(),
                        token_ids: vec![3]
                    },
                ]
            );

            //REMOVALS

            // INVALID EXECUTION
            app.execute_contract(
                Addr::unchecked(INVALID.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::RemovePromisedTokenIDs(vec![1, 2]))
                            .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::RemovePromisedTokenIDs(vec![1, 2]))
                            .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            let res: Vec<AddressPromisedTokensResponse> = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.clone().unwrap(),
                    &AirdropperQueryMsg::GetAddressPromisedTokenIDs {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            assert_eq!(
                res,
                vec![
                    AddressPromisedTokensResponse {
                        address: USER10.to_owned(),
                        token_ids: vec![5]
                    },
                    AddressPromisedTokensResponse {
                        address: USER3.to_owned(),
                        token_ids: vec![3]
                    },
                ]
            );

            // ADMIN EXECUTION
            app.execute_contract(
                Addr::unchecked(ADMIN.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::RemovePromisedTokensByAddress(vec![
                            USER10.to_owned(),
                        ]))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            let res: Vec<AddressPromisedTokensResponse> = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.unwrap(),
                    &AirdropperQueryMsg::GetAddressPromisedTokenIDs {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            assert_eq!(
                res,
                vec![AddressPromisedTokensResponse {
                    address: USER3.to_owned(),
                    token_ids: vec![3]
                },]
            );
        }

        #[test]
        fn ad_add_remove_promised_mints() {
            let (mut app, cw_template_contract) = proper_instantiate(true, false);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            // INVALID EXECUTION
            app.execute_contract(
                Addr::unchecked(INVALID.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::AddPromisedMints(vec![
                            AD_AddressValMsg {
                                address: USER.to_owned(),
                                value: 1,
                            },
                        ]))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::AddPromisedMints(vec![
                            AD_AddressValMsg {
                                address: USER.to_owned(),
                                value: 1,
                            },
                            AD_AddressValMsg {
                                address: USER2.to_owned(),
                                value: 2,
                            },
                        ]))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            let res: Vec<AD_AddressValMsg> = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.clone().unwrap(),
                    &AirdropperQueryMsg::GetAddressPromisedMints {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            assert_eq!(
                res,
                vec![
                    AD_AddressValMsg {
                        address: USER.to_owned(),
                        value: 1,
                    },
                    AD_AddressValMsg {
                        address: USER2.to_owned(),
                        value: 2,
                    },
                ]
            );

            // REMOVAL

            // INVALID EXECUTION
            app.execute_contract(
                Addr::unchecked(INVALID.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::RemovePromisedMints(vec![
                            USER.to_owned()
                        ]))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::RemovePromisedMints(vec![
                            USER.to_owned()
                        ]))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            let res: Vec<AD_AddressValMsg> = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.unwrap(),
                    &AirdropperQueryMsg::GetAddressPromisedMints {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            assert_eq!(
                res,
                vec![AD_AddressValMsg {
                    address: USER2.to_owned(),
                    value: 2,
                },]
            );
        }

        #[test]
        fn ad_mark_token_id_claimed() {
            let (mut app, cw_template_contract) = proper_instantiate(true, false);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::AddPromisedTokenIDs(vec![
                            AD_AddressValMsg {
                                address: USER.to_owned(),
                                value: 1,
                            },
                            AD_AddressValMsg {
                                address: USER2.to_owned(),
                                value: 2,
                            },
                        ]))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            // INVALID EXECUTION
            app.execute_contract(
                Addr::unchecked(INVALID.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::MarkTokenIDClaimed(
                            USER.to_owned(),
                            1,
                        ))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::MarkTokenIDClaimed(
                            USER.to_owned(),
                            1,
                        ))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            let res: Vec<u32> = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.clone().unwrap(),
                    &AirdropperQueryMsg::GetAssignedTokenIDs {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            assert_eq!(res, vec![1, 2]);

            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::MarkTokenIDClaimed(
                            USER.to_owned(),
                            2,
                        ))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            let get_claimed_token_ids: Vec<AD_AddressValMsg> = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.unwrap(),
                    &AirdropperQueryMsg::GetClaimedTokenIDsWithAddress {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            assert_eq!(
                get_claimed_token_ids,
                vec![AD_AddressValMsg {
                    address: USER.to_owned(),
                    value: 1,
                }]
            );
        }

        #[test]
        fn ad_increment_address_promised_mint_count() {
            let (mut app, cw_template_contract) = proper_instantiate(true, false);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            // INVALID EXECUTION
            app.execute_contract(
                Addr::unchecked(INVALID.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::AddPromisedMints(vec![
                            AD_AddressValMsg {
                                address: USER.to_owned(),
                                value: 1,
                            },
                        ]))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(&AirdropperExecuteMsg::AddPromisedMints(vec![
                            AD_AddressValMsg {
                                address: USER.to_owned(),
                                value: 1,
                            },
                            AD_AddressValMsg {
                                address: USER2.to_owned(),
                                value: 2,
                            },
                        ]))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            // INVALID EXECUTION
            app.execute_contract(
                Addr::unchecked(INVALID.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(
                            &AirdropperExecuteMsg::IncrementAddressClaimedPromisedMintCount(
                                USER.to_owned(),
                            ),
                        )
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(
                            &AirdropperExecuteMsg::IncrementAddressClaimedPromisedMintCount(
                                USER.to_owned(),
                            ),
                        )
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            // MAINTAINER EXECUTION ERROR, MAX REACHED
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(
                            &AirdropperExecuteMsg::IncrementAddressClaimedPromisedMintCount(
                                USER.to_owned(),
                            ),
                        )
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Airdropper,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.airdropper_addr.clone().unwrap().into_string(),
                        msg: to_binary(
                            &AirdropperExecuteMsg::IncrementAddressClaimedPromisedMintCount(
                                USER2.to_owned(),
                            ),
                        )
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            let promised_mints: Vec<AD_AddressValMsg> = app
                .wrap()
                .query_wasm_smart(
                    config.airdropper_addr.unwrap(),
                    &AirdropperQueryMsg::GetClaimedAddressPromisedMints {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            assert_eq!(
                promised_mints,
                vec![
                    AD_AddressValMsg {
                        address: USER.to_owned(),
                        value: 1,
                    },
                    AD_AddressValMsg {
                        address: USER2.to_owned(),
                        value: 1,
                    }
                ]
            );
        }
    }

    mod whitelist_interaction {
        use super::*;

        #[test]
        fn verify_whitelist_init() {
            let (app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let wl_config: WhitelistConfig = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.unwrap(),
                    &WhitelistQueryMsg::GetConfig {},
                )
                .unwrap();

            assert_eq!(
                wl_config.maintainer_addr,
                Some(Addr::unchecked(MAINTAINER_ADDR.to_string()))
            );
        }

        #[test]
        fn update_maintainer_address() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            println!("{:?}", config);
            println!("{:?}", cw_template_contract.addr());

            app.execute_contract(
                cw_template_contract.addr(),
                config.whitelist_addr.clone().unwrap(),
                &WhitelistExecuteMsg::UpdateMaintainerAddress(Some("notwhiskey".to_string())),
                &[],
            )
            .unwrap();

            let wl_config: WhitelistConfig = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.unwrap(),
                    &WhitelistQueryMsg::GetConfig {},
                )
                .unwrap();

            assert_ne!(
                wl_config.maintainer_addr,
                Some(Addr::unchecked(MAINTAINER_ADDR.to_string()))
            );
            assert_eq!(
                wl_config.maintainer_addr,
                Some(Addr::unchecked("notwhiskey".to_string()))
            );
        }

        #[test]
        fn update_whitelist() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            app.execute_contract(
                cw_template_contract.addr(),
                config.whitelist_addr.clone().unwrap(),
                &WhitelistExecuteMsg::AddToWhitelist(vec![
                    USER.to_string(),
                    USER10.to_string(),
                    USER25.to_string(),
                ]),
                &[],
            )
            .unwrap();

            let addresses: Vec<String> = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.unwrap(),
                    &WhitelistQueryMsg::GetWhitelistAddresses {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("### GetWhitelistAddresses {:?}", addresses);
            assert_eq!(
                addresses,
                vec![USER.to_string(), USER10.to_string(), USER25.to_string()]
            );
        }

        #[test]
        fn execute_whitelist_mint_not_in_progress() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("###config {:?}", config);

            // not yet block time

            let res = app
                .execute_contract(
                    Addr::unchecked(USER),
                    cw_template_contract.addr(),
                    &ExecuteMsg::Mint {},
                    &[coin(1_000_000, NATIVE_DENOM)],
                )
                .unwrap_err();

            println!("current_token_supply {:?}", res);
        }

        #[test]
        fn execute_whitelist_mint_not_on_list() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("###config {:?}", config);

            // not yet block time
            let msg = ExecuteMsg::Mint {};

            app.update_block(|mut block| {
                block.time = Timestamp::from_seconds(WHITELIST_START_TIME)
            });

            let res = app
                .execute_contract(
                    Addr::unchecked(USER),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(1_000_000, NATIVE_DENOM)],
                )
                .unwrap_err();

            println!("current_token_supply {:?}", res);
        }

        #[test]
        fn execute_whitelist_mint_whitelist_ended() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("###config {:?}", config);

            // not yet block time
            let msg = ExecuteMsg::Mint {};

            app.update_block(|mut block| {
                block.time = Timestamp::from_seconds(WHITELIST_END_TIME + 1)
            });

            let res = app
                .execute_contract(
                    Addr::unchecked(USER),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(1_000_000, NATIVE_DENOM)],
                )
                .unwrap_err();

            println!("current_token_supply {:?}", res);
        }

        #[test]
        fn execute_whitelist_mints_not_in_progress() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            app.execute_contract(
                cw_template_contract.addr(),
                config.whitelist_addr.clone().unwrap(),
                &WhitelistExecuteMsg::AddToWhitelist(vec![
                    USER.to_string(),
                    USER10.to_string(),
                    USER25.to_string(),
                ]),
                &[],
            )
            .unwrap();

            let addresses: Vec<String> = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.unwrap(),
                    &WhitelistQueryMsg::GetWhitelistAddresses {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("### GetWhitelistAddresses {:?}", addresses);
            assert_eq!(
                addresses,
                vec![USER.to_string(), USER10.to_string(), USER25.to_string()]
            );

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER2),
                    cw_template_contract.addr(),
                    &ExecuteMsg::Mint {},
                    &[coin(1_000_000, NATIVE_DENOM)],
                )
                .unwrap_err();
        }

        #[test]
        fn execute_whitelist_mints_not_on_whitelist() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            app.execute_contract(
                cw_template_contract.addr(),
                config.whitelist_addr.clone().unwrap(),
                &WhitelistExecuteMsg::AddToWhitelist(vec![
                    USER.to_string(),
                    USER10.to_string(),
                    USER25.to_string(),
                ]),
                &[],
            )
            .unwrap();

            let addresses: Vec<String> = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.unwrap(),
                    &WhitelistQueryMsg::GetWhitelistAddresses {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("### GetWhitelistAddresses {:?}", addresses);
            assert_eq!(
                addresses,
                vec![USER.to_string(), USER10.to_string(), USER25.to_string()]
            );

            app.update_block(|mut block| {
                block.time = Timestamp::from_seconds(WHITELIST_START_TIME)
            });

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER2),
                    cw_template_contract.addr(),
                    &ExecuteMsg::Mint {},
                    &[coin(1_000_000, NATIVE_DENOM)],
                )
                .unwrap_err();
        }

        #[test]
        fn execute_whitelist_mints_success() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let wl_config: WhitelistConfig = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.clone().unwrap(),
                    &WhitelistQueryMsg::GetConfig {},
                )
                .unwrap();

            let maintainer_address: Option<String> =
                wl_config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = WLInstantiateMsg {
                start_time: wl_config.start_time,
                end_time: wl_config.end_time,
                maintainer_address,
                max_whitelist_address_count: wl_config.max_whitelist_address_count,
                max_per_address_mint: wl_config.max_per_address_mint,
                mint_price: wl_config.mint_price,
            };

            app.execute_contract(
                cw_template_contract.addr(),
                config.whitelist_addr.clone().unwrap(),
                &WhitelistExecuteMsg::AddToWhitelist(vec![
                    USER.to_string(),
                    USER10.to_string(),
                    USER25.to_string(),
                ]),
                &[],
            )
            .unwrap();

            let addresses: Vec<String> = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.clone().unwrap(),
                    &WhitelistQueryMsg::GetWhitelistAddresses {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            println!("### GetWhitelistAddresses {:?}", addresses);
            assert_eq!(
                addresses,
                vec![USER.to_string(), USER10.to_string(), USER25.to_string()]
            );

            msg.mint_price = Uint128::from(1_000_001u128);
            app.execute_contract(
                cw_template_contract.addr(),
                config.whitelist_addr.clone().unwrap(),
                &WhitelistExecuteMsg::UpdateConfig(msg),
                &[],
            )
            .unwrap();

            let wl_config: WhitelistConfig = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.clone().unwrap(),
                    &WhitelistQueryMsg::GetConfig {},
                )
                .unwrap();

            assert_eq!(wl_config.mint_price, Uint128::from(1_000_001u128));
            assert_eq!(wl_config.whitelist_address_count, 3);

            app.update_block(|mut block| {
                block.time = Timestamp::from_seconds(WHITELIST_START_TIME)
            });

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER),
                    cw_template_contract.addr(),
                    &ExecuteMsg::Mint {},
                    &[coin(1_000_001, NATIVE_DENOM)],
                )
                .unwrap();

            let address_mint_tracker: Vec<(String, u32)> = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.unwrap(),
                    &WhitelistQueryMsg::GetAddressMints {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            assert_eq!(address_mint_tracker[0].0, USER.to_owned());
            assert_eq!(address_mint_tracker[0].1, 1);

            let admin_balance = app.wrap().query_all_balances(ADMIN.to_owned()).unwrap();
            println!("{:?}", admin_balance);
            assert_eq!(admin_balance[0].amount, Uint128::from(10_700_000u128));

            let maintainer_balance = app
                .wrap()
                .query_all_balances(MAINTAINER_ADDR.to_owned())
                .unwrap();
            assert_eq!(maintainer_balance[0].amount, Uint128::from(300_000u128));
        }

        #[test]
        fn execute_whitelist_mints_success_cleaned_out() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            app.execute_contract(
                cw_template_contract.addr(),
                config.whitelist_addr.clone().unwrap(),
                &WhitelistExecuteMsg::AddToWhitelist(vec![
                    USER.to_string(),
                    USER10.to_string(),
                    USER25.to_string(),
                ]),
                &[],
            )
            .unwrap();

            app.update_block(|mut block| {
                block.time = Timestamp::from_seconds(WHITELIST_START_TIME)
            });

            // first mint
            let _res = app
                .execute_contract(
                    Addr::unchecked(USER),
                    cw_template_contract.addr(),
                    &ExecuteMsg::Mint {},
                    &[coin(1_000_000, NATIVE_DENOM)],
                )
                .unwrap();

            // USER10 maxes out 2

            for _ in 1u32..=2 {
                let _res = app
                    .execute_contract(
                        Addr::unchecked(USER10),
                        cw_template_contract.addr(),
                        &ExecuteMsg::Mint {},
                        &[coin(1_000_000, NATIVE_DENOM)],
                    )
                    .unwrap();
            }

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER10),
                    cw_template_contract.addr(),
                    &ExecuteMsg::Mint {},
                    &[coin(1_000_000, NATIVE_DENOM)],
                )
                .unwrap_err();

            let token_data: TokenDataResponse = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetRemainingTokens {},
                )
                .unwrap();

            assert_eq!(token_data.remaining_token_supply, 2);

            println!("### config {:?}", token_data);

            // USER25 maxes out 2
            for _ in 1u32..=2 {
                let _res = app
                    .execute_contract(
                        Addr::unchecked(USER25),
                        cw_template_contract.addr(),
                        &ExecuteMsg::Mint {},
                        &[coin(1_000_000, NATIVE_DENOM)],
                    )
                    .unwrap();
            }

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER25),
                    cw_template_contract.addr(),
                    &ExecuteMsg::Mint {},
                    &[coin(1_000_000, NATIVE_DENOM)],
                )
                .unwrap_err();

            let address_mint_tracker: Vec<(String, u32)> = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.unwrap(),
                    &WhitelistQueryMsg::GetAddressMints {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            assert_eq!(address_mint_tracker[0].0, USER.to_owned());
            assert_eq!(address_mint_tracker[0].1, 1);

            assert_eq!(address_mint_tracker[1].0, USER10.to_owned());
            assert_eq!(address_mint_tracker[1].1, 2);

            assert_eq!(address_mint_tracker[2].0, USER25.to_owned());
            assert_eq!(address_mint_tracker[2].1, 2);
        }

        #[test]
        fn execute_whitelist_mints_success_partial_whitelist_mint() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            app.execute_contract(
                cw_template_contract.addr(),
                config.whitelist_addr.clone().unwrap(),
                &WhitelistExecuteMsg::AddToWhitelist(vec![
                    USER.to_string(),
                    USER10.to_string(),
                    USER25.to_string(),
                ]),
                &[],
            )
            .unwrap();

            app.update_block(|mut block| {
                block.time = Timestamp::from_seconds(WHITELIST_START_TIME)
            });

            // first mint
            let _res = app
                .execute_contract(
                    Addr::unchecked(USER),
                    cw_template_contract.addr(),
                    &ExecuteMsg::Mint {},
                    &[coin(1_000_000, NATIVE_DENOM)],
                )
                .unwrap();

            // USER10 maxes out 2

            for _ in 1u32..=2 {
                let _res = app
                    .execute_contract(
                        Addr::unchecked(USER10),
                        cw_template_contract.addr(),
                        &ExecuteMsg::Mint {},
                        &[coin(1_000_000, NATIVE_DENOM)],
                    )
                    .unwrap();
            }

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER10),
                    cw_template_contract.addr(),
                    &ExecuteMsg::Mint {},
                    &[coin(1_000_000, NATIVE_DENOM)],
                )
                .unwrap_err();

            let token_data: TokenDataResponse = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetRemainingTokens {},
                )
                .unwrap();

            assert_eq!(token_data.remaining_token_supply, 2);

            println!("### config {:?}", token_data);

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER25),
                    cw_template_contract.addr(),
                    &ExecuteMsg::Mint {},
                    &[coin(1_000_000, NATIVE_DENOM)],
                )
                .unwrap();

            let address_mint_tracker: Vec<(String, u32)> = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.unwrap(),
                    &WhitelistQueryMsg::GetAddressMints {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            assert_eq!(address_mint_tracker[0].0, USER.to_owned());
            assert_eq!(address_mint_tracker[0].1, 1);

            assert_eq!(address_mint_tracker[1].0, USER10.to_owned());
            assert_eq!(address_mint_tracker[1].1, 2);

            assert_eq!(address_mint_tracker[2].0, USER25.to_owned());
            assert_eq!(address_mint_tracker[2].1, 1);

            // WL ended
            app.update_block(|mut block| block.time = Timestamp::from_seconds(WHITELIST_END_TIME));

            // USER25 tries to mint again
            let _res = app
                .execute_contract(
                    Addr::unchecked(USER25),
                    cw_template_contract.addr(),
                    &ExecuteMsg::Mint {},
                    &[coin(1_000_000, NATIVE_DENOM)],
                )
                .unwrap_err();

            // public mint starts
            app.update_block(|mut block| block.time = Timestamp::from_seconds(MINT_START_TIME));

            // USER25 mints in public with less than amount
            let _res = app
                .execute_contract(
                    Addr::unchecked(USER25),
                    cw_template_contract.addr(),
                    &ExecuteMsg::Mint {},
                    &[coin(1_000_000, NATIVE_DENOM)],
                )
                .unwrap_err();

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER25),
                    cw_template_contract.addr(),
                    &ExecuteMsg::Mint {},
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap();

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER25),
                    cw_template_contract.addr(),
                    &ExecuteMsg::Mint {},
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap_err();
        }
    }

    mod whitelist_hooks {
        use super::*;

        #[test]
        fn wl_update_max_whitelist_address_count() {
            let (mut app, cw_template_contract) = proper_instantiate(false, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let wl_config: WhitelistConfig = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.clone().unwrap(),
                    &WhitelistQueryMsg::GetConfig {},
                )
                .unwrap();

            let maintainer_address: Option<String> =
                wl_config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = WLInstantiateMsg {
                start_time: wl_config.start_time,
                end_time: wl_config.end_time,
                maintainer_address,
                max_whitelist_address_count: wl_config.max_whitelist_address_count,
                max_per_address_mint: wl_config.max_per_address_mint,
                mint_price: wl_config.mint_price,
            };

            msg.max_whitelist_address_count = 32;
            // INVALID EXECUTION
            app.execute_contract(
                Addr::unchecked(INVALID.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::UpdateConfig(msg.clone())).unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::UpdateConfig(msg.clone())).unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            // ADMIN EXECUTION
            msg.max_whitelist_address_count = 64;
            app.execute_contract(
                Addr::unchecked(ADMIN.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::UpdateConfig(msg)).unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            let wl_config: WhitelistConfig = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.unwrap(),
                    &WhitelistQueryMsg::GetConfig {},
                )
                .unwrap();

            assert_eq!(wl_config.max_whitelist_address_count, 64);
        }

        #[test]
        fn wl_update_max_per_address_mint() {
            let (mut app, cw_template_contract) = proper_instantiate(false, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let wl_config: WhitelistConfig = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.clone().unwrap(),
                    &WhitelistQueryMsg::GetConfig {},
                )
                .unwrap();

            let maintainer_address: Option<String> =
                wl_config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = WLInstantiateMsg {
                start_time: wl_config.start_time,
                end_time: wl_config.end_time,
                maintainer_address,
                max_whitelist_address_count: wl_config.max_whitelist_address_count,
                max_per_address_mint: wl_config.max_per_address_mint,
                mint_price: wl_config.mint_price,
            };

            msg.max_per_address_mint = 32;
            // INVALID EXECUTION
            app.execute_contract(
                Addr::unchecked(INVALID.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::UpdateConfig(msg.clone())).unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::UpdateConfig(msg.clone())).unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            // ADMIN EXECUTION
            msg.max_per_address_mint = 64;
            app.execute_contract(
                Addr::unchecked(ADMIN.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::UpdateConfig(msg)).unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            let wl_config: WhitelistConfig = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.unwrap(),
                    &WhitelistQueryMsg::GetConfig {},
                )
                .unwrap();

            assert_eq!(wl_config.max_per_address_mint, 64);
        }

        #[test]
        fn wl_update_mint_price() {
            let (mut app, cw_template_contract) = proper_instantiate(false, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            let wl_config: WhitelistConfig = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.clone().unwrap(),
                    &WhitelistQueryMsg::GetConfig {},
                )
                .unwrap();

            let maintainer_address: Option<String> =
                wl_config.maintainer_addr.map(|addr| addr.into_string());

            let mut msg = WLInstantiateMsg {
                start_time: wl_config.start_time,
                end_time: wl_config.end_time,
                maintainer_address,
                max_whitelist_address_count: wl_config.max_whitelist_address_count,
                max_per_address_mint: wl_config.max_per_address_mint,
                mint_price: wl_config.mint_price,
            };

            // INVALID EXECUTION
            msg.mint_price = Uint128::from(500_000u128);
            app.execute_contract(
                Addr::unchecked(INVALID.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::UpdateConfig(msg.clone())).unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::UpdateConfig(msg.clone())).unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            // ADMIN EXECUTION
            msg.mint_price = Uint128::from(500_001u128);
            app.execute_contract(
                Addr::unchecked(ADMIN.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::UpdateConfig(msg)).unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            let wl_config: WhitelistConfig = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.unwrap(),
                    &WhitelistQueryMsg::GetConfig {},
                )
                .unwrap();

            assert_eq!(wl_config.mint_price, Uint128::from(500_001u128));
        }

        #[test]
        fn add_remove_update_whitelist_with_hook() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();

            // INVALID EXECUTION
            app.execute_contract(
                Addr::unchecked(INVALID.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::AddToWhitelist(vec![
                            USER.to_string(),
                            USER10.to_string(),
                            USER25.to_string(),
                        ]))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::AddToWhitelist(vec![
                            USER.to_string(),
                            USER10.to_string(),
                            USER25.to_string(),
                        ]))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            // ADMIN EXECUTION
            app.execute_contract(
                Addr::unchecked(ADMIN.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::AddToWhitelist(vec![
                            USER2.to_string(),
                            USER3.to_string(),
                        ]))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            let addresses: Vec<String> = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.clone().unwrap(),
                    &WhitelistQueryMsg::GetWhitelistAddresses {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            assert_eq!(
                addresses,
                vec![
                    USER.to_string(),
                    USER10.to_string(),
                    USER2.to_string(),
                    USER25.to_string(),
                    USER3.to_string()
                ]
            );

            // REMOVALS

            // INVALID EXECUTION
            app.execute_contract(
                Addr::unchecked(INVALID.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::RemoveFromWhitelist(vec![
                            USER25.to_string()
                        ]))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::RemoveFromWhitelist(vec![
                            USER.to_string()
                        ]))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            // ADMIN EXECUTION
            app.execute_contract(
                Addr::unchecked(ADMIN.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::RemoveFromWhitelist(vec![
                            USER2.to_string()
                        ]))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            let addresses: Vec<String> = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.clone().unwrap(),
                    &WhitelistQueryMsg::GetWhitelistAddresses {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            assert_eq!(
                addresses,
                vec![USER10.to_string(), USER25.to_string(), USER3.to_string()]
            );

            // INVALID EXECUTION
            app.execute_contract(
                Addr::unchecked(INVALID.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::UpdateAddressMintTracker(
                            USER25.to_string(),
                        ))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::UpdateAddressMintTracker(
                            USER.to_string(),
                        ))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            for _ in 1..=2 {
                // contract EXECUTION
                app.execute_contract(
                    Addr::unchecked(ADMIN.to_owned()),
                    cw_template_contract.addr(),
                    &ExecuteMsg::SubmoduleHook(
                        ExecutionTarget::Whitelist,
                        CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                            msg: to_binary(&WhitelistExecuteMsg::UpdateAddressMintTracker(
                                USER3.to_string(),
                            ))
                            .unwrap(),
                            funds: vec![],
                        }),
                    ),
                    &[],
                )
                .unwrap();
            }

            // contract EXECUTION FAIL
            app.execute_contract(
                Addr::unchecked(ADMIN.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::UpdateAddressMintTracker(
                            USER3.to_string(),
                        ))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap_err();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::UpdateAddressMintTracker(
                            USER25.to_string(),
                        ))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            // MAINTAINER EXECUTION
            app.execute_contract(
                Addr::unchecked(MAINTAINER_ADDR.to_owned()),
                cw_template_contract.addr(),
                &ExecuteMsg::SubmoduleHook(
                    ExecutionTarget::Whitelist,
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.whitelist_addr.clone().unwrap().into_string(),
                        msg: to_binary(&WhitelistExecuteMsg::UpdateAddressMintTracker(
                            USER10.to_string(),
                        ))
                        .unwrap(),
                        funds: vec![],
                    }),
                ),
                &[],
            )
            .unwrap();

            let address_mint_tracker: Vec<(String, u32)> = app
                .wrap()
                .query_wasm_smart(
                    config.whitelist_addr.unwrap(),
                    &WhitelistQueryMsg::GetAddressMints {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            assert_eq!(
                address_mint_tracker,
                vec![
                    (USER10.to_string(), 1),
                    (USER25.to_string(), 1),
                    (USER3.to_string(), 2)
                ]
            )
        }
    }

    mod mint_public {
        use super::*;

        #[test]
        fn execute_public_mint_success() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("###config {:?}", config);

            // not yet block time
            let msg = ExecuteMsg::Mint {};

            app.update_block(|mut block| block.time = Timestamp::from_seconds(MINT_START_TIME));

            let res = app
                .execute_contract(
                    Addr::unchecked(USER3),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap();

            println!("current_token_supply {:?}", res);

            let token_data: TokenDataResponse = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetRemainingTokens {},
                )
                .unwrap();

            assert_eq!(token_data.remaining_token_supply, 4);

            println!("### config {:?}", token_data);
        }

        // not enough tokens for user 1
        #[test]
        fn execute_public_mint_fail() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("###config {:?}", config);

            // not yet block time
            let msg = ExecuteMsg::Mint {};

            app.update_block(|mut block| block.time = Timestamp::from_seconds(MINT_START_TIME));

            let res = app
                .execute_contract(
                    Addr::unchecked(USER),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap_err();

            println!("current_token_supply {:?}", res);

            let token_data: TokenDataResponse = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetRemainingTokens {},
                )
                .unwrap();

            println!("### config {:?}", token_data);
        }

        // user 2 cannot mint twice
        #[test]
        fn execute_public_mint_fail_2() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("###config {:?}", config);

            // not yet block time
            let msg = ExecuteMsg::Mint {};

            app.update_block(|mut block| block.time = Timestamp::from_seconds(MINT_START_TIME));

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER2),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap();

            let res = app
                .execute_contract(
                    Addr::unchecked(USER2),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap_err();

            println!("current_token_supply {:?}", res);

            let token_data: TokenDataResponse = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetRemainingTokens {},
                )
                .unwrap();

            println!("### config {:?}", token_data);
        }

        // user25 cannot mint as there are no more tokens left
        #[test]
        fn execute_public_mint_fail_3() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("###config {:?}", config);

            // not yet block time
            let msg = ExecuteMsg::Mint {};

            app.update_block(|mut block| block.time = Timestamp::from_seconds(MINT_START_TIME));

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER2),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap();

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER3),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap();

            // cant mint again because no more funds
            let _res = app
                .execute_contract(
                    Addr::unchecked(USER3),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap_err();

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER10),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap();

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER10),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap();

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER10),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap();

            let res = app
                .execute_contract(
                    Addr::unchecked(USER25),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap_err();

            println!("current_token_supply {:?}", res);

            let token_data: TokenDataResponse = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetRemainingTokens {},
                )
                .unwrap();

            assert_eq!(token_data.remaining_token_supply, 0);
            println!("### config {:?}", token_data);
        }

        // user25 cannot mint over max
        #[test]
        fn execute_public_mint_fail_4() {
            let (mut app, cw_template_contract) = proper_instantiate(true, true);

            let config: ConfigResponse = app
                .wrap()
                .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetConfig {})
                .unwrap();
            println!("###config {:?}", config);

            // not yet block time
            let msg = ExecuteMsg::Mint {};

            app.update_block(|mut block| block.time = Timestamp::from_seconds(MINT_START_TIME));

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER25),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap();

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER25),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap();

            // cant mint again because no more funds
            let _res = app
                .execute_contract(
                    Addr::unchecked(USER25),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap();

            let _res = app
                .execute_contract(
                    Addr::unchecked(USER25),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap();

            let res = app
                .execute_contract(
                    Addr::unchecked(USER25),
                    cw_template_contract.addr(),
                    &msg,
                    &[coin(2_000_000, NATIVE_DENOM)],
                )
                .unwrap_err();

            println!("current_token_supply {:?}", res);

            let token_data: TokenDataResponse = app
                .wrap()
                .query_wasm_smart(
                    &cw_template_contract.addr(),
                    &QueryMsg::GetRemainingTokens {},
                )
                .unwrap();

            assert_eq!(token_data.remaining_token_supply, 1);
            println!("### config {:?}", token_data);
        }
    }

    mod misc_tests {
        use super::*;
        use crate::state::RoyaltyInfo;
        use cosmwasm_std::Decimal;

        #[test]
        fn test_decimal() {
            let share1 = 10000;
            let share2 = 5000;
            let share3 = 2500;
            let share4 = 1000;
            let share5 = 750;
            let share6 = 25;

            println!(
                "{:?}",
                (Uint128::from(MINT_PRICE) * Decimal::percent(share1) / Uint128::from(100u128))
            );
            println!(
                "{:?}",
                (Uint128::from(MINT_PRICE) * Decimal::percent(share2) / Uint128::from(100u128))
            );
            println!(
                "{:?}",
                (Uint128::from(MINT_PRICE) * Decimal::percent(share3) / Uint128::from(100u128))
            );
            println!(
                "{:?}",
                (Uint128::from(MINT_PRICE) * Decimal::percent(share4) / Uint128::from(100u128))
            );
            println!(
                "{:?}",
                (Uint128::from(MINT_PRICE) * Decimal::percent(share5) / Uint128::from(100u128))
            );
            println!(
                "{:?}",
                (Uint128::from(MINT_PRICE) * Decimal::percent(share6) / Uint128::from(100u128))
            );
        }

        #[test]
        fn test_sort() {
            let mut my_vec: Vec<RoyaltyInfo> = vec![
                RoyaltyInfo {
                    addr: Addr::unchecked(USER.to_owned()),
                    bps: 3300,
                    is_primary: false,
                },
                RoyaltyInfo {
                    addr: Addr::unchecked(USER2.to_owned()),
                    bps: 5000,
                    is_primary: false,
                },
                RoyaltyInfo {
                    addr: Addr::unchecked(USER3.to_owned()),
                    bps: 500,
                    is_primary: true,
                },
                RoyaltyInfo {
                    addr: Addr::unchecked(USER10.to_owned()),
                    bps: 1200,
                    is_primary: false,
                },
            ];

            my_vec.sort_by(|a, b| a.is_primary.cmp(&b.is_primary));

            println!("{:?}", my_vec);
            //assert_eq!(5, 7);
        }
    }
}
