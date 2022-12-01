use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Config {
    /// Ideally this is the minting contract
    pub admin: Addr,
    /// Address for whoever wants to maintain/manage the contract
    pub maintainer_addr: Option<Addr>,
    /// Time the whitelist addresses are allowed to mint
    pub start_time: Timestamp,
    /// time when the whitelist ends. There is a hard "stop" once the
    /// public mint starts, but this will allow users to schedule WL
    /// to end before public mint begins
    pub end_time: Timestamp,
    /// max number of addresses on the whitelist
    pub max_whitelist_address_count: u32,
    /// max mints per address
    pub max_per_address_mint: u32,
    /// mint price for WL. The denom is controlled via the main minting
    /// contract. only native and ibc/ denoms are allowed
    pub mint_price: Uint128,
}

/// config
pub const CONFIG: Item<Config> = Item::new("config");
/// Map that holds the addresses on the WL
pub const WHITELIST: Map<Addr, bool> = Map::new("wl");
/// Map that tracks how many mints an address has made in the WL
pub const ADDRESS_MINT_TRACKER: Map<Addr, u32> = Map::new("address_mint_tracker");
/// Item that keeps count of the number of addresses on the WL
pub const WHITELIST_ADDRESS_COUNT: Item<u32> = Item::new("whitelist_address_count");
