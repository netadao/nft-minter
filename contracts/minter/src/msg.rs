use crate::state::SharedCollectionInfo;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, CosmosMsg, Empty, Timestamp, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub base_fields: BaseInitMsg,
    /// name of nft project
    pub name: String,
    /// airdropper instantiation info must have either or none
    /// against `airdrop_address`
    pub airdropper_instantiate_info: Option<ModuleInstantiateInfo>,
    /// whitelist contract instantiation info. must have either or none
    /// against `whitelist_address`
    pub whitelist_instantiate_info: Option<ModuleInstantiateInfo>,
    /// code id for cw721 contract
    pub token_code_id: u64,
    /// vec of collection info
    pub collection_infos: Vec<CollectionInfoMsg>,
    /// extension info that will be passed to
    pub extension: SharedCollectionInfoMsg,
}

/// Base fields that are used for instantiation
/// dual purpose: also used for update config funciton
#[cw_serde]
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
    /// max bundles per address
    pub max_per_address_bundle_mint: u32,
    /// mint price fee for PUBLIC mint. This can be overridden by WL mint_price
    pub mint_price: Uint128,
    pub bundle_mint_price: Uint128,
    /// only native and ibc/ denoms are allowed. onus is on user to verify if
    /// they manually instantiate this contract. otherwise, controlled via frontend
    pub mint_denom: String,
    /// determines if you want to escrow funds or just send funds per tx
    pub escrow_funds: bool,
    pub bundle_enabled: bool,
    pub airdropper_address: Option<String>,
    pub whitelist_address: Option<String>,
}

#[cw_serde]
pub struct CollectionInfoMsg {
    /// token supply for this collection
    pub token_supply: u32,
    /// name of nft project
    pub name: String,
    /// symbol for nft project, this seems really optional for cw721 standard
    pub symbol: String,
    /// uri for the metadata. intended to be a static metadata for the nft
    pub base_token_uri: String,
    /// optional secondary metadata resource that is intended to be dynamic
    /// and extensible to the creator's desires
    pub secondary_metadata_uri: Option<String>,
}

/// Shared Collection Info that stores revenue/royalty split as well the optional secondary metadata
/// uri that will allow creators to add evolving metadata in addition to the static metadata
/// that is in `base_token_uri`
#[cw_serde]
pub struct SharedCollectionInfoMsg {
    /// initial sales split. has a hardcap of 10000 bps equating to 100.00%
    pub mint_revenue_share: Vec<RoyaltyInfoMsg>,
    /// secondary sales royalty split. hardcap of 5000bps equating to 50.00%
    /// so the token owner gets roughly 50% of the sales revenue in the case of
    /// "free mints"
    pub secondary_market_royalties: Vec<RoyaltyInfoMsg>,
}

#[cw_serde]
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
#[cw_serde]
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
#[cw_serde]
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
#[cw_serde]
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
#[cw_serde]
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

#[cw_serde]
pub enum ExecuteMsg {
    /// Uses `BaseInitMsg` to update the the config
    UpdateConfig(BaseInitMsg),
    /// General path for whitelist and public mints
    /// whitelist requires eligibility, public mint right now does not
    /// AirdropMint allow users to mint an NFT that was promised to them
    /// feeless (`mint_price` = 0). the airdrop promised mint is managed in
    /// the contract attached to `AIRDROPPER_ADDR`
    /// the optional `minter_address` is if a maintainer wants to `push`
    /// an nft to the address rather than having the recipient come `pull`
    /// the promised mint by executing this function themselves
    Mint {
        is_promised_mint: bool,
        minter_address: Option<String>,
    },
    MintBundle {},
    /// airdrop claim is intended for 1:1s or other creator criteria for
    /// granting ownership of specific `token_id`s. This is controlled in the
    /// contract attached to `AIRDROPPER_ADDR`
    /// the optional `minter_address` allows an address to `pull` (execute
    /// this themselves) or an admin to `push` the token to them
    AirdropClaim {
        minter_address: Option<String>,
    },
    /// Calls the attached airdropper contract and removes the `token_id`s
    /// from `SHUFFLED_TOKEN_IDS` and `TOKEN_ID_POSITIONS` so they will not
    /// accidentally get minted.  Once complete, it'll shuffle the token order
    CleanClaimedTokensFromShuffle {},
    /// shuffles the token order. takes a lot of gas
    ShuffleTokenOrder {},
    /// Allows this contract to pass execution messages to its submodules
    SubmoduleHook(ExecutionTarget, CosmosMsg<Empty>),
    /// Allows an admin/maintainer to disburse funds in escrow
    DisburseFunds {
        address: String,
    },
    ProcessCustomBundle {
        content_count: u32,
        mint_price: Uint128,
        tokens: Option<Vec<TokenMsg>>,
        purge: bool,
    },
    MintCustomBundle {},
}

#[cw_serde]
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
    GetCw721CollectionInfo {
        /// token_id
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    GetBundleMintTracker {
        /// token_id
        start_after: Option<String>,
        limit: Option<u32>,
    },
    GetCollectionCurrentTokenSupply {
        /// token_id
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    /// Gets count of remaining tokens available in `CURRENT_TOKEN_SUPPLY`
    GetRemainingTokens { address: Option<String> },
    /// Gets all the cw721 addresses attached to this contract
    GetCw721Addrs {},
}

#[cw_serde]
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
    pub total_token_supply: u32,
    /// maximum mints per address
    pub max_per_address_mint: u32,
    /// max bundles per address
    pub max_per_address_bundle_mint: u32,
    /// mint price for the public mint
    pub mint_price: Uint128,
    pub bundle_mint_price: Uint128,
    /// only native and ibc/ denoms are allowed. onus is on user to verify if
    /// they manually instantiate this contract. otherwise, controlled via frontend
    pub mint_denom: String,
    /// cw721 contract code id
    pub token_code_id: u64,
    /// address to contract that we'll read promised mints and token_ids data from
    pub airdropper_addr: Option<Addr>,
    /// address to contract that we'll read whitelist data from
    pub whitelist_addr: Option<Addr>,
    pub escrow_funds: bool,
    /// data we'll pass to the cw721 contract when a token is minted
    pub extension: SharedCollectionInfo,
    pub bundle_enabled: bool,
    pub bundle_completed: bool,
    pub custom_bundle_enabled: bool,
    pub custom_bundle_completed: bool,
    pub custom_bundle_mint_price: Uint128,
    pub custom_bundle_content_count: u32,
}

#[cw_serde]
pub struct CollectionInfoResponse {
    /// address to contract that holds the nfts"
    pub cw721_addr: Option<Addr>,
    /// useless? name for the token
    pub name: String,
    /// useless? symbol for the token
    pub symbol: String,
    pub token_supply: u64,
    /// base uri for the cw721. stores the location of the metadata for all nfts
    pub base_token_uri: String,
    pub secondary_metadata_uri: String,
}

/// response object that has token supply data
/// was used for testing
#[cw_serde]
pub struct TokenDataResponse {
    pub total_token_supply: u32,
    pub remaining_token_supply: u32,
    pub address_minted: u32,
    pub max_per_address_mint: u32,
    pub address_bundles_minted: u32,
    pub max_per_address_bundle_mint: u32,
    pub remaining_bundle_mints: u32,
    pub remaining_custom_bundle_mints: u32,
}

/// Used as execution msg and query response for single Address-Value pairs
/// eg: address has 4 mints
#[cw_serde]
pub struct AddressValMsg {
    /// address for the promised values
    pub address: String,
    /// mint count
    pub value: u32,
}

/// Simple struct for address-bal
#[cw_serde]
pub struct AddrBal {
    pub addr: Addr,
    pub balance: Uint128,
}

/// Used as query response for single collection_id-token_id pairs
#[cw_serde]
pub struct TokenMsg {
    /// address for the promised values
    pub collection_id: u64,
    /// promised token_id
    pub token_id: u32,
}
