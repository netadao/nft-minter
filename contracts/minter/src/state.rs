use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub maintainer_addr: Option<Addr>,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub max_token_supply: u32,
    pub max_per_address_mint: u32,
    pub mint_price: Uint128,
    pub mint_denom: String,
    pub base_token_uri: String,
    pub name: String,
    pub symbol: String,
    pub token_code_id: u64,
    pub extension: CollectionInfo,
    pub escrow_funds: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Eq, Ord, PartialOrd)]
pub struct CollectionInfo {
    pub secondary_metadata_uri: Option<String>,
    pub mint_revenue_share: Vec<RoyaltyInfo>,
    pub secondary_market_royalties: Vec<RoyaltyInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Eq, Ord, PartialOrd)]
pub struct RoyaltyInfo {
    pub addr: Addr,
    pub bps: u32,
    pub is_primary: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CURRENT_TOKEN_SUPPLY: Item<u32> = Item::new("current_token_supply");
pub const ADDRESS_MINT_TRACKER: Map<Addr, u32> = Map::new("address_mint_tracker");
// (idx, tokenid)
pub const SHUFFLED_TOKEN_IDS: Map<u32, u32> = Map::new("shuffled_token_ids");
// (tokenid, idx)
pub const TOKEN_ID_POSITIONS: Map<u32, u32> = Map::new("token_id_positions");
pub const CW721_ADDR: Item<Addr> = Item::new("cw721_addr");
pub const AIRDROPPER_ADDR: Item<Addr> = Item::new("airdropper_addr");
pub const WHITELIST_ADDR: Item<Addr> = Item::new("whitelist_addr");
pub const BANK_BALANCES: Map<Addr, Uint128> = Map::new("bank_balances");
