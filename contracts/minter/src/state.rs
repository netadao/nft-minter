use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_denom::CheckedDenom;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub maintainer_addr: Option<Addr>,
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    pub total_token_supply: u32,
    pub max_per_address_mint: u32,
    pub max_per_address_bundle_mint: u32,
    pub mint_price: Uint128,
    pub bundle_mint_price: Uint128,
    pub mint_denom: CheckedDenom,
    pub token_code_id: u64,
    pub extension: SharedCollectionInfo,
    pub escrow_funds: bool,
    pub bundle_enabled: bool,
    pub bundle_completed: bool,
    pub bonded_denom: String,
}

#[cw_serde]
pub struct CollectionInfo {
    pub id: u64,
    pub token_supply: u32,
    pub name: String,
    pub symbol: String,
    pub base_token_uri: String,
    pub secondary_metadata_uri: Option<String>,
}

#[cw_serde]
pub struct SharedCollectionInfo {
    pub mint_revenue_share: Vec<RoyaltyInfo>,
    pub secondary_market_royalties: Vec<RoyaltyInfo>,
}

#[cw_serde]
pub struct RoyaltyInfo {
    pub addr: Addr,
    pub bps: u32,
    pub is_primary: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CW721_COLLECTION_INFO: Map<u64, CollectionInfo> = Map::new("cw721_collection_info");

// addresses
pub const FEE_COLLECTION_ADDR: Item<Addr> = Item::new("fee_collection_addr");
pub const AIRDROPPER_ADDR: Item<Addr> = Item::new("airdropper_addr");
pub const WHITELIST_ADDR: Item<Addr> = Item::new("whitelist_addr");
pub const CW721_ADDRS: Map<u64, Addr> = Map::new("cw721_addrs");

// supplies
pub const CURRENT_TOKEN_SUPPLY: Item<u32> = Item::new("current_token_supply");
pub const TOTAL_TOKEN_SUPPLY: Item<u32> = Item::new("total_token_supply");
pub const COLLECTION_CURRENT_TOKEN_SUPPLY: Map<u64, u32> =
    Map::new("collection_current_token_supply");

// trackers
pub const ADDRESS_MINT_TRACKER: Map<Addr, u32> = Map::new("address_mint_tracker");
pub const BUNDLE_MINT_TRACKER: Map<Addr, u32> = Map::new("bundle_mint_tracker");
pub const BANK_BALANCES: Map<Addr, Uint128> = Map::new("bank_balances");
pub const TOKEN_MINTED_BY: Map<String, Addr> = Map::new("token_minted_by");

//
// (idx, token_id)
// based on index (1-N), we'll dump whatever token_id back for minting
// eg: (1, 5), (2, 3), (3, 1), (4, 4), (5, 2)
///pub const SHUFFLED_BASE_TOKEN_IDS: Map<u32, u32> = Map::new("shuffled_base_token_ids");
// (token_id, idx)
// based on token_id, we can find where it is in the shuffled map
// eg: (1, 3), (2, 5), (3, 2), (4, 4), (5, 1)
///pub const BASE_TOKEN_ID_POSITIONS: Map<u32, u32> = Map::new("base_token_id_positions");
// (token_id, cw721_id)
// map a "token_id" to a cw721_collection:id
// eg: ()
///pub const BASE_TOKEN_ID_CW721_ID: Map<u32, String> = Map::new("base_token_id_cw721_id");
// (cw721_id, token_id) helps locate token_id
pub const CW721_SHUFFLED_TOKEN_IDS: Map<u64, Vec<u32>> = Map::new("cw721_shuffled_token_ids");
