use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    /// should be the minting contract OR the creator of the campaign
    pub admin: Addr,
    /// alternate address to maintain and manage the contract
    pub maintainer_addr: Option<Addr>,
    /// the time when airdrops become valid
    pub start_time: Timestamp,
    /// *optional* time when airdrops no longer claimable
    pub end_time: Option<Timestamp>,
}

/// config? lol
pub const CONFIG: Item<Config> = Item::new("config");
/// Map that stores the `token_id`s promised to a particular address
/// This map will get modified as addresses claim their promises or if
/// a maintainer pushes the tokens to an address
pub const ADDRESS_PROMISED_TOKEN_IDS: Map<Addr, Vec<u32>> = Map::new("address_promised_token_ids");
/// Map that stores an address' claimed `token_id`s
/// dependent on the `ADDRESS_PROMISED_TOKEN_IDS` map
pub const ADDRESS_CLAIMED_TOKEN_IDS: Map<Addr, Vec<u32>> = Map::new("address_claimed_token_ids");
/// Helper map that holds what address a `token_id` is assigned to
pub const ASSIGNED_TOKEN_IDS: Map<u32, Addr> = Map::new("assigned_token_ids");
/// Helper map that holds what address a `token_id` has been claimed by
pub const CLAIMED_TOKEN_IDS: Map<u32, Addr> = Map::new("claimed_token_ids");
/// Map that holds the number of promised [free] mints for an address
pub const ADDRESS_PROMISED_MINTS: Map<Addr, u32> = Map::new("address_promised_free_mints");
/// Map that holds the number of CLAIMED promised mints for an address
pub const ADDRESS_CLAIMED_PROMISED_MINTS: Map<Addr, u32> =
    Map::new("address_claimed_promise_mints");
