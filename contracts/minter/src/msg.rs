use crate::state::CollectionInfo;
use cosmwasm_std::{Addr, Binary, CosmosMsg, Empty, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub base_fields: BaseInitMsg,
    /// max token supply
    /// v2TODO: allow uncapped
    /// v2TODO: allow update to this field
    pub max_token_supply: u32,
    /// name of nft project
    pub name: String,
    /// symbol for nft project, this seems really optional for cw721 standard
    pub symbol: String,
    /// airdropper address if it was manaully instantiated elsewhere
    pub airdrop_address: Option<String>,
    /// airdropper instantiation info must have either or none
    /// against `airdrop_address`
    pub airdropper_instantiate_info: Option<ModuleInstantiateInfo>,
    /// whitelist address if it was manaully instantiated elsewhere
    pub whitelist_address: Option<String>,
    /// whitelist contract instantiation info. must have either or none
    /// against `whitelist_address`
    pub whitelist_instantiate_info: Option<ModuleInstantiateInfo>,
    /// extension info that will be passed to
    pub extension: CollectionInfoMsg,
}

/// Base fields that are used for instantiation
/// dual purpose: also used for update config funciton
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BaseInitMsg {
    /// alternate address for maintaining/management of this contract
    pub maintainer_address: Option<String>,
    /// start time for the public mint portion of the campaign. this will
    /// hard stop the WL campaign
    pub start_time: Timestamp,
    /// hard stop for public mint
    /// TODO: move to optional?
    /// TODO: allow admin/maintainer to update this if needed
    pub end_time: Option<Timestamp>,
    /// max mint per address
    pub max_per_address_mint: u32,
    /// mint price fee for PUBLIC mint. This can be overridden by WL mint_price
    pub mint_price: Uint128,
    /// only native and ibc/ denoms are allowed. onus is on user to verify if
    /// they manually instantiate this contract. otherwise, controlled via frontend
    pub mint_denom: String,
    /// uri for the metadata. intended to be a static metadata for the nft
    pub base_token_uri: String,
    /// code id for cw721 contract
    pub token_code_id: u64,
    /// determines if you want to escrow funds or just send funds per tx
    pub escrow_funds: bool,
}

/// Collection Info that stores revenue/royalty split as well the optional secondary metadata
/// uri that will allow creators to add evolving metadata in addition to the static metadata
/// that is in `base_token_uri`
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Eq, Ord, PartialOrd)]
pub struct CollectionInfoMsg {
    /// optional secondary metadata resource that is intended to be dynamic
    /// and extensible to the creator's desires
    pub secondary_metadata_uri: Option<String>,
    /// initial sales split. has a hardcap of 10000 bps equating to 100.00%
    pub mint_revenue_share: Vec<RoyaltyInfoMsg>,
    /// secondary sales royalty split. hardcap of 5000bps equating to 50.00%
    /// so the token owner gets roughly 50% of the sales revenue in the case of
    /// "free mints"
    pub secondary_market_royalties: Vec<RoyaltyInfoMsg>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Eq, Ord, PartialOrd)]
pub struct RoyaltyInfoMsg {
    /// address that receives this split
    pub address: String,
    /// this address' basis points to calculate the total split of revenue
    pub bps: u32,
    /// is_primary is the primary address and will receive the remaining dust from
    /// rev splits
    pub is_primary: bool,
}

/// Information about the admin of a contract.
/// may have been stolen from daodao
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Admin {
    /// A specific address.
    Address { address: String },
    /// The core contract itself. The contract will fill this in
    /// while instantiation takes place.
    CoreContract {},
    /// No admin.
    None {},
}

/// Execution Target enum used for submodule hooks
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionTarget {
    /// No target was specified
    None,
    /// targetting the config's `airdropper_addr`
    Airdropper,
    /// targetting the config's `whitelist_addr`
    Whitelist,
}

/// Mint Type enum that determines which mint type was intended by the
/// entry points
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MintType {
    /// default
    None,
    /// public mint type
    Public,
    /// whitelist
    Whitelist,
    /// airdropper promised mint. these are "free" mints that are promised to folks
    /// intended for use in giveaways, etc
    PromisedMint,
    /// airdropper promised `token_id`s that are intended for use for 1:1s
    /// and the like
    PromisedToken,
}

/// Information needed to instantiate a submodule.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ModuleInstantiateInfo {
    /// Code ID of the contract to be instantiated.
    pub code_id: u64,
    /// Instantiate message to be used to create the contract.
    pub msg: Binary,
    /// Admin of the instantiated contract.
    pub admin: Admin,
    /// Label for the instantiated contract.
    pub label: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Uses `BaseInitMsg` to update the the config
    UpdateConfig(BaseInitMsg),
    /// (Re)Initializes submodules if a user desires.  This will replace the
    /// existing submodule that its targeting. dependent on our reply_id's
    /// matching the code_id that is deployed to the chain
    InitSubmodule(ModuleInstantiateInfo),
    /// Update the attached `WHITELIST_ADDR`
    UpdateWhitelistAddress(Option<String>),
    /// Update the attached `AIRDROPPER_ADDR`
    UpdateAirdropAddress(Option<String>),
    /// General path for whitelist and public mints
    /// whitelist requires eligibility, public mint right now does not
    Mint {},
    /// AirdropMint allow users to mint an NFT that was promised to them
    /// feeless (`mint_price` = 0). the airdrop promised mint is managed in
    /// the contract attached to `AIRDROPPER_ADDR`
    /// the optional `minter_address` is if a maintainer wants to `push`
    /// an nft to the address rather than having the recipient come `pull`
    /// the promised mint by executing this function themselves
    AirdropMint { minter_address: Option<String> },
    /// airdrop claim is intended for 1:1s or other creator criteria for
    /// granting ownership of specific `token_id`s. This is controlled in the
    /// contract attached to `AIRDROPPER_ADDR`
    /// the optional `minter_address` allows an address to `pull` (execute
    /// this themselves) or an admin to `push` the token to them
    AirdropClaim { minter_address: Option<String> },
    /// Calls the attached airdropper contract and removes the `token_id`s
    /// from `SHUFFLED_TOKEN_IDS` and `TOKEN_ID_POSITIONS` so they will not
    /// accidentally get minted.  Once complete, it'll shuffle the token order
    CleanClaimedTokensFromShuffle {},
    /// shuffles the token order. takes a lot of gas
    ShuffleTokenOrder {},
    /// Allows this contract to pass execution messages to its submodules
    SubmoduleHook(ExecutionTarget, CosmosMsg<Empty>),
    /// Allows an admin/maintainer to disburse funds in escrow
    DisburseFunds {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Gets Config + some other fields and returns `ConfigResponse`
    GetConfig {},
    /// Checks an address' mint count and returns `AddressValMsg`
    CheckAddressMints { minter_address: String },
    /// Gets a list of all the addresses who have had a public mint
    /// in `ADDRESS_MINT_TRACKER`. Default sort is in ASCENDING based on
    /// addressreturns Vec<AddressValMsg>
    GetAddressMints {
        /// address
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// gets a list of all the balances in escrow
    /// returns Vec<AddrBal>
    GetEscrowBalances {
        /// address
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /*
    /// TODO: REMOVE, TESTING ONLY
    GetShuffledTokenIds {
        /// token_id
        start_after: Option<u32>,
        limit: Option<u32>,
    },
    GetTokenIndices {
        /// token_id
        start_after: Option<u32>,
        limit: Option<u32>,
    },
    GetShuffledTokenPosition {
        /// token_id
        start_after: Option<u32>,
        limit: Option<u32>,
    },
    */
    /// Gets count of remaining tokens available in `CURRENT_TOKEN_SUPPLY`
    GetRemainingTokens {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    /// admin address for this contract. has special permissions in contract
    pub admin: Addr,
    /// alternate manager of this contract
    pub maintainer_addr: Option<Addr>,
    /// public mint start time
    pub start_time: Timestamp,
    /// public mint end time.
    /// TODO: migrate to optional?
    pub end_time: Option<Timestamp>,
    /// maximum token supply
    /// TODO: move to uncapped for special project
    pub max_token_supply: u32,
    /// maximum mints per address
    pub max_per_address_mint: u32,
    /// mint price for the public mint
    pub mint_price: Uint128,
    /// only native and ibc/ denoms are allowed. onus is on user to verify if
    /// they manually instantiate this contract. otherwise, controlled via frontend
    pub mint_denom: String,
    /// base uri for the cw721. stores the location of the metadata for all nfts
    pub base_token_uri: String,
    /// useless? name for the token
    pub name: String,
    /// useless? symbol for the token
    pub symbol: String,
    /// cw721 contract code id
    pub token_code_id: u64,
    /// address to contract that holds the nfts"
    pub cw721_addr: Option<Addr>,
    /// address to contract that we'll read promised mints and token_ids data from
    pub airdropper_addr: Option<Addr>,
    /// address to contract that we'll read whitelist data from
    pub whitelist_addr: Option<Addr>,
    /// data we'll pass to the cw721 contract when a token is minted
    pub extension: CollectionInfo,
}

/// response object that has token supply data
/// was used for testing
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenDataResponse {
    pub max_token_supply: u32,
    pub remaining_token_supply: u32,
}

/// Used as execution msg and query response for single Address-Value pairs
/// eg: address has 4 mints
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Eq)]
pub struct AddressValMsg {
    /// address for the promised values
    pub address: String,
    /// mint count
    pub value: u32,
}

/// Simple struct for address-bal
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Eq)]
pub struct AddrBal {
    pub addr: Addr,
    pub balance: Uint128,
}
