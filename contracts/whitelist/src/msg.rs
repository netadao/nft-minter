use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Address for whoever wants to maintain/manage the contract
    pub maintainer_address: Option<String>,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Leverages InstantiateMsg to pass updates to the config for the contract
    UpdateConfig(InstantiateMsg),
    /// Update's the maintainer's address for the contract.  This address also has
    /// the ability to manage the contract alongside the admin.
    /// This is a separate function so the minter contract can easily pass an update
    /// if the maintainer needs to be updated
    UpdateMaintainerAddress(Option<String>),
    /// Adds each address in the list of strings to the whitelist
    AddToWhitelist(Vec<String>),
    /// Removes each address in the list of stirngs from the whitelist
    RemoveFromWhitelist(Vec<String>),
    /// For the address passed in, we'll increment their mint count by 1
    /// in the `ADDRESS_MINT_TRACKER`
    UpdateAddressMintTracker(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Gets `state::Config` a
    GetConfig {},
    /// Check's an address' whitelist eligibility as well as open window
    /// information on the WL mint. Returns closed, inprogress, address is
    /// on whitelist, current WL mint count for address, max mints per address
    /// and the mint price of the WL.  Returns a `CheckWhitelistResponse` obj
    CheckWhitelist { minter_address: String },
    /// Lists all addresses and the number of mints they have in `WHITELIST`
    /// default sort is ASCENDING by address. returns Vec<String>
    GetWhitelistAddresses {
        /// address
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Lists all address and their count of mints that are in
    /// `ADDRESS_MINT_TRACKER`. default sort is ASCENDING by address.
    /// Returns Vec<AddressValMsg>
    GetAddressMints {
        /// address
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    /// should be the minting contract or creator of the campaign
    pub admin: Addr,
    /// alternate address for address that maintains/manages this contract
    pub maintainer_addr: Option<Addr>,
    /// Time the whitelist addresses are allowed to mint
    pub start_time: Timestamp,
    /// time when the whitelist ends. There is a hard "stop" once the
    /// public mint starts, but this will allow users to schedule WL
    /// to end before public mint begins
    pub end_time: Timestamp,
    /// max number of addresses allowed on `WHITELIST`
    pub max_whitelist_address_count: u32,
    /// max mints per address for this whitelist
    pub max_per_address_mint: u32,
    /// (calculated field) whitelist is currently open and in progress
    pub whitelist_in_progress: bool,
    /// (calculated field) whitelist is closed and completed
    pub whitelist_is_closed: bool,
    /// mint price for WL. The denom is controlled via the main minting
    /// contract. only native and ibc/ denoms are allowed
    pub mint_price: Uint128,
    /// (calculated field) the count of addresses on `WHITELIST`
    pub whitelist_address_count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CheckWhitelistResponse {
    /// address that was checked for WL eligibility
    pub minter_addr: Addr,
    /// whitelist is closed and completed
    pub whitelist_is_closed: bool,
    /// whitelist is currently open and in progress
    pub whitelist_in_progress: bool,
    /// the address checked is on the whitelist
    pub is_on_whitelist: bool,
    /// the address' current WL mint count
    pub current_mint_count: u32,
    /// the WL campaign's max mint count per address
    pub max_per_address_mint: u32,
    /// mint price for WL. The denom is controlled via the main minting
    /// contract. only native and ibc/ denoms are allowed
    pub mint_price: Uint128,
}
