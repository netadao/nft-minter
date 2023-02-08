use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};

/// General Instantiation message. also used to pass updates for the config
#[cw_serde]
pub struct InstantiateMsg {
    /// alternate address for maintaining/management of this contract
    pub maintainer_address: Option<String>,
    /// start time for the airdropper portion of the campaign
    pub start_time: Timestamp,
    /// hard stop for claims
    /// todo: implement stop. for s1 of project, no end time for claiming
    pub end_time: Option<Timestamp>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Leverages InstantiateMsg to pass updates to the config for the contract
    UpdateConfig(InstantiateMsg),
    /// Update's the maintainer's address for the contract.  This address also has
    /// the ability to manage the contract alongside the admin.
    /// This is a separate function so the minter contract can easily pass an update
    /// if the maintainer needs to be updated
    UpdateMaintainerAddress(Option<String>),
    /// `Value` used here is a `token_id`. This function will validate the token_id and
    /// add it to an address' list of promised `token_id`s
    AddPromisedTokenIds(Vec<AddressTokenMsg>),
    /// For every `token_id` passed in, grab the address it was promised to, then
    /// remove it from that address' promised tokens. Also remove it from the
    /// assigned list that tracks which address it was promised to
    RemovePromisedTokenIds(Vec<TokenMsg>),
    /// Given an a list of addresses, we'll iterate through and unassign each token_id
    /// from the assigned tokens tracker and then remove the address
    /// this will remove ALL promised token ids for an address
    RemovePromisedTokensByAddress(Vec<String>),
    /// Value used here is a count/number of promised mints to an address
    /// Also performs updates if the count needs to change
    AddPromisedMints(Vec<AddressValMsg>),
    /// Removes addresses from the list of addresses with promised mints
    RemovePromisedMints(Vec<String>),
    /// Marks a token_id as claimed by an address
    MarkTokenIdClaimed(AddressTokenMsg),
    /// Increments an address' claimed promised mint count
    IncrementAddressClaimedPromisedMintCount(String),
}

#[cw_serde]
pub enum QueryMsg {
    /// Gets `state::Config` and returns it
    GetConfig {},
    /// Lists all promised token_ids for addresses from `ADDRESS_PROMISED_TOKEN_IDS`
    /// default sort is ASCENDING by ADDRESS. returns `Vec<AddressPromisedTokens>`
    /// which is in Vec(address-Vec[u32]) form
    GetAddressPromisedTokenIds {
        /// address
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Lists all token_ids that have been assigned in `ASSIGNED_TOKEN_IDS`
    /// default sort is ASCENDING by token_id. returns Vec<u32>
    GetAssignedTokenIds {
        /// token_id
        start_after: Option<(u64, u32)>,
        limit: Option<u32>,
    },
    /// Lists all assigned token_id-address pairs in `ASSIGNED_TOKEN_IDS`
    /// default sort is ASCENDING by token_id. returns Vec<AddressValMsg>
    GetAssignedTokenIdsWithAddress {
        /// (collection_id, token_id)
        start_after: Option<(u64, u32)>,
        limit: Option<u32>,
    },
    /// Lists all token_ids that are claimed in `CLAIMED_TOKEN_IDS`
    /// default sort is ASCENDING by token_id. returns Vec<u32>
    GetClaimedTokenIds {
        /// (collection_id, token_id)
        start_after: Option<(u64, u32)>,
        limit: Option<u32>,
    },
    /// Lists all token_ids and which address claimed themin `CLAIMED_TOKEN_IDS`
    /// default sort is ASCENDING by token_id. returns Vec<AddressValMsg>
    GetClaimedTokenIdsWithAddress {
        /// (collection_id, token_id)
        start_after: Option<(u64, u32)>,
        limit: Option<u32>,
    },
    /// Lists all addresses and the number of promised mints they have in `ADDRESS_PROMISED_MINTS`
    /// default sort is ASCENDING by address. returns Vec<AddressValMsg>
    GetAddressPromisedMints {
        /// address
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Lists all address and their count of claimed promised mints that are in
    /// `ADDRESS_CLAIMED_PROMISED_MINTS`. default sort is ASCENDING by address.
    /// Returns Vec<AddressValMsg>
    GetClaimedAddressPromisedMints {
        /// address
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Checks an address' promised mints as well as if the airdrop window info (closed, inprogress)
    /// Returns `CheckAirdropPromisedMintResponse` which has mint counts that were promised, claimed
    /// and remaining (diff between claimed and promised)
    CheckAddressPromisedMints { minter_address: String },
    /// Checks an address' promised tokens.  This is intended for use for specialized tokens, 1:1s,
    /// general promises, etc. Also returns info on airdrop window (closed/inprogress).
    /// Returns `CheckAirdropPromisedTokensResponse` which has the promised and claimedtoken_ids
    CheckAddressPromisedTokens { minter_address: String },
}

/// Used as execution msg and query response for single Address-Value pairs
/// eg address is promised token_id 8 or address is promised 3 mints or
/// token_id 6 was minted by address: "juno1addr"
#[cw_serde]
pub struct AddressValMsg {
    /// address for the promised values
    pub address: String,
    /// promised token_id OR promised mint count
    pub value: u32,
}

/// Used as query response for single collection_id-token_id pairs
#[cw_serde]
pub struct TokenMsg {
    /// address for the promised values
    pub collection_id: u64,
    /// promised token_id
    pub token_id: u32,
}

/// Used as execution msg and query response for single Address-TokenMsg pairs
/// eg address is promised collection_id 3's token_id 8
#[cw_serde]
pub struct AddressTokenMsg {
    /// address for the promised values
    pub address: String,
    /// promised token_id OR promised mint count
    pub token: TokenMsg,
}

/// Response object used to return a list of `token_id`s
/// promised/claimed by an address
#[cw_serde]
pub struct AddressPromisedTokensResponse {
    /// address for the promised values
    pub address: String,
    /// list of token_ids promised to an address
    pub token_ids: Vec<TokenMsg>,
}

/// Response object used to check an address' promised and claimed mints
/// should be used as giveaways and claimable when the window opens
#[cw_serde]
pub struct CheckAirdropPromisedMintResponse {
    /// minter's address being checked
    pub minter_addr: Addr,
    /// checks if airdrop is closed or not
    pub airdrop_mint_is_closed: bool,
    /// checks if airdrop is in progress
    pub airdrop_mint_in_progress: bool,
    /// count of promised mints
    pub promised_mint_count: u32,
    /// count of claimed promised mints for an address
    pub claimed_mint_count: u32,
}

/// Response object used to check an address' promised and claimed `token_id`s
/// Promised `token_id`s should generally be used for 1:1s, other giveaways, etc
#[cw_serde]
pub struct CheckAirdropPromisedTokensResponse {
    /// minter's address being checked
    pub minter_addr: Addr,
    /// checks if airdrop is closed or not
    pub airdrop_mint_is_closed: bool,
    /// checks if airdrop is in progress
    pub airdrop_mint_in_progress: bool,
    /// an address' promised token ids. These get removed
    /// once the user claims these token_ids (pull) or if
    /// an admin pushes the token_id to their address
    pub address_promised_token_ids: Vec<TokenMsg>,
    /// an address' claimed promised token ids
    pub address_claimed_token_ids: Vec<TokenMsg>,
}
