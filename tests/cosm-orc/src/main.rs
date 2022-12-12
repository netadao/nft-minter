use anyhow::Result;
use cosm_orc::{
    config::{
        cfg::Config,
        key::{Key, SigningKey},
    },
    orchestrator::cosm_orc::CosmOrc,
};

use cosmwasm_std::{Timestamp, Uint128};

use cw_denom::UncheckedDenom;
use serde_json::Value;
use std::fs;

use minter::msg::{
    BaseInitMsg, CollectionInfoMsg, InstantiateMsg, RoyaltyInfoMsg, SharedCollectionInfoMsg,
};

fn main() -> Result<()> {
    env_logger::init();

    let cfg = Config::from_yaml("juno_local.yaml")?;
    let mut cosm_orc = CosmOrc::new(cfg.clone(), true)?;

    // juno19f3njfqjs7fcj352ysz08skqzfrl335rjjh9w2

    let key = SigningKey {
        name: "validator".to_string(),
        key: Key::Mnemonic("car thumb elevator wagon arch chase same kiss item super razor insane napkin walnut member amount air hazard advice hammer minimum uniform little blanket".to_string()),
    };
    let _account = key.to_account(&cfg.chain_cfg.prefix)?;

    cosm_orc.store_contracts("../../artifacts", &key, None)?;

    cosm_orc.instantiate(
        "minter",
        "instantiate_minter",
        &InstantiateMsg {
            base_fields: BaseInitMsg {
                maintainer_address: Some("juno1u20j62nwkmkcwq5mp06azgr3cgkyp6s88q63mn".to_string()),
                start_time: Timestamp::from_nanos(1668161200000000000),
                end_time: None,
                max_per_address_mint: 5,
                max_per_address_bundle_mint: 10000,
                mint_price: Uint128::from(5_000_000u128),
                bundle_mint_price: Uint128::from(30_000_000u128),
                mint_denom: UncheckedDenom::Native("ujunox".to_string()),
                escrow_funds: true,
                bundle_enabled: false,
            },
            name: "test1".to_string(),
            airdrop_address: None,
            airdropper_instantiate_info: None,
            whitelist_address: None,
            whitelist_instantiate_info: None,
            token_code_id: 2,
            collection_infos: vec![CollectionInfoMsg {
                token_supply: 10000,
                name: "nft1".to_string(),
                symbol: "nft1".to_string(),
                base_token_uri: "ipfs://asdf".to_string(),
                secondary_metadata_uri: None,
            }],
            extension: SharedCollectionInfoMsg {
                mint_revenue_share: vec![RoyaltyInfoMsg {
                    address: "juno1u20j62nwkmkcwq5mp06azgr3cgkyp6s88q63mn".to_string(),
                    bps: 10000,
                    is_primary: true,
                }],
                secondary_market_royalties: vec![RoyaltyInfoMsg {
                    address: "juno1u20j62nwkmkcwq5mp06azgr3cgkyp6s88q63mn".to_string(),
                    bps: 4999,
                    is_primary: true,
                }],
            },
        },
        &key,
        None,
        vec![],
    )?;

    let report = cosm_orc.gas_profiler_report().unwrap();

    let j: Value = serde_json::to_value(report)?;
    fs::write("./gas_report.json", j.to_string())?;

    Ok(())
}
