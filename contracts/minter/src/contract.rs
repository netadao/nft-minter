use crate::error::ContractError;
use crate::msg::{
    AddrBal, Admin, BaseInitMsg, CollectionInfoMsg, ExecuteMsg, ExecutionTarget, InstantiateMsg,
    MintType, ModuleInstantiateInfo, RoyaltyInfoMsg,
};
use crate::state::{
    CollectionInfo, Config, RoyaltyInfo, ADDRESS_MINT_TRACKER, AIRDROPPER_ADDR, BANK_BALANCES,
    CONFIG, CURRENT_TOKEN_SUPPLY, CW721_ADDR, SHUFFLED_TOKEN_IDS, TOKEN_ID_POSITIONS,
    WHITELIST_ADDR,
};
use airdropper::{
    msg::ExecuteMsg::{
        IncrementAddressClaimedPromisedMintCount as AD_IncrementAddressClaimedPromisedMintCount,
        MarkTokenIDClaimed as AD_MarkTokenIDClaimed,
        UpdateMaintainerAddress as AD_UpdateMaintainerAddress,
    },
    msg::QueryMsg as AirdropperQueryMsg,
    msg::{CheckAirdropPromisedMintResponse, CheckAirdropPromisedTokensResponse},
};
use cosmwasm_std::{
    coin, entry_point, to_binary, Addr, BankMsg, Coin, CosmosMsg, Deps, DepsMut, Empty, Env,
    MessageInfo, Order, Reply, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw721_base::{
    msg::ExecuteMsg as Cw721ExecuteMsg, msg::InstantiateMsg as Cw721InstantiateMsg, MintMsg,
};
use cw_utils::{may_pay, maybe_addr, parse_reply_instantiate_data};
use rand_core::SeedableRng;
use rand_xoshiro::Xoshiro128StarStar;
use sha2::{Digest, Sha256};
use shuffle::{fy::FisherYates, shuffler::Shuffler};
use std::cmp;
#[cfg(not(feature = "library"))]
use std::convert::{TryFrom, TryInto};
use url::Url;
use whitelist::{
    msg::CheckWhitelistResponse,
    msg::ExecuteMsg::{
        UpdateAddressMintTracker as WL_UpdateAddressMintTracker,
        UpdateMaintainerAddress as WL_UpdateMaintainerAddress,
    },
    msg::QueryMsg as WhitelistQueryMsg,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:nft-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO: migrate to a shared package?
// globals

/// Default denom for fees, pmt, etc, only NATIVE (NO CW20s ALLOWED) denoms allowed.
/// To start ujuno or IBC denoms allowed
const NATIVE_DENOM: &str = "ujuno";
/// The global maximum token supply for any campaign.
/// Initialized here and does not push down to Airdropper or Whitelist contracts
/// This should be moved to a DAO controlled contract
/// TODO: need more opinions on this
const MAX_TOKEN_SUPPLY: u32 = 50000;
/// The global maximum mints allowed per address.
/// TODO: decide if this is necessary to have in contract
const MAX_PER_ADDRESS_MINT: u32 = 50000;
/// REPLY_IDs are returned if the submessage returns ok
/// in this app we will ensure the reply_ids match the code_id
/// stored on the server /shrug
const INSTANTIATE_TOKEN_REPLY_ID: u64 = 2;
const INSTANTIATE_AIRDROPPER_REPLY_ID: u64 = 3;
const INSTANTIATE_WHITELIST_REPLY_ID: u64 = 4;
/// Acts as max/total basis points for initial mint revenue and as divisor.
/// Total mint shares can have 2 decimal places for percentages
const MAX_BPS: u32 = 10_000;
/// Max/total basis points for secondary royalty revenue.
/// This has a cap of 50% going to the addresses that are listed.
/// The intent of a 5000 bps cap or 50% is in the case of "free mints"
/// so the creator can make the majority of revenue in secondary sales
/// rather than initial mint earnings
const MAX_BPS_FOR_SECONDARY: u32 = 5_000;
/// MintParametersResponse is a response object to determine if a public/whitelist
/// is valid, the mint price and in the case of promised tokenids, if there are
/// remaining tokenids to be claimed
struct MintParametersResponse {
    /// If ture, then the public/WL mint will proceed, so long as a valid `mint_price`
    /// is also returned.  Otherwise, this will kick out and fall down to the next check.
    can_mint: bool,
    /// The mint price of the mint type, valid values are >= 0.
    /// There is a minor failsafe built in in case None is returned,
    /// we should error out
    mint_price: Option<Uint128>,
    /// Used for Airdrop Promised Token IDs to track which ones have not been claimed
    /// by the address that they were promised to
    remaining_token_ids: Vec<u32>,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // validate fields
    if msg.base_fields.start_time <= env.block.time {
        return Err(ContractError::InvalidStartTime {});
    }

    if msg.base_fields.end_time.unwrap_or(env.block.time) < env.block.time {
        return Err(ContractError::InvalidEndTime {});
    }

    if msg.base_fields.start_time
        >= msg
            .base_fields
            .end_time
            .unwrap_or_else(|| msg.base_fields.start_time.plus_nanos(1u64))
    {
        return Err(ContractError::InvalidStartTime {});
    }

    if !(1..=MAX_TOKEN_SUPPLY).contains(&msg.max_token_supply) {
        return Err(ContractError::InvalidMaxTokenSupply {
            max: MAX_TOKEN_SUPPLY,
            input: msg.max_token_supply,
        });
    }

    // this may be simplified to just checking against `max_token_supply`
    if msg.base_fields.max_per_address_mint < 1
        || msg.base_fields.max_per_address_mint > MAX_PER_ADDRESS_MINT
        || msg.base_fields.max_per_address_mint > msg.max_token_supply
    {
        return Err(ContractError::InvalidMaxPerAddressMint {
            max: cmp::min(MAX_PER_ADDRESS_MINT, msg.max_token_supply),
            input: msg.base_fields.max_per_address_mint,
        });
    }

    // if both an address and instantiate info are given, then error out
    if msg.airdrop_address.is_some() && msg.airdropper_instantiate_info.is_some() {
        return Err(ContractError::InvalidSubmoduleInstantiation {});
    }

    // if both an address and instantiate info are given, then error out
    if msg.whitelist_address.is_some() && msg.whitelist_instantiate_info.is_some() {
        return Err(ContractError::InvalidSubmoduleInstantiation {});
    }

    validate_uri(msg.base_fields.base_token_uri.clone())?;

    // validate the initial mint revenue split as well as royalty split
    let collection_info: CollectionInfo = validate_collection_info(deps.as_ref(), msg.extension)?;

    // validate the denom the user selected is one that is allowed.
    // cw20 banned
    if msg.base_fields.mint_denom != NATIVE_DENOM && !msg.base_fields.mint_denom.starts_with("ibc/")
    {
        return Err(ContractError::InvalidPaymentType {});
    }

    // TODO: add required fee that goes to neta dao's treasury dao OR if the treasury dao
    // is included in rev share then allow this to bypass
    /*
        let dao_fee_payment = must_pay(&info, NATIVE_DENOM)?;

        if dao_fee_payment != CAMPAIGN_CREATION_FEE.into() {
            return Err(ContractError::InvalidCampaignCreationFee {
                fee: CAMPAIGN_CREATION_FEE,
                denom: NATIVE_DENOM,
            });
        }
    */

    let config = Config {
        admin: info.sender.clone(),
        maintainer_addr: maybe_addr(deps.api, msg.base_fields.maintainer_address)?,
        start_time: msg.base_fields.start_time,
        end_time: msg.base_fields.end_time,
        max_token_supply: msg.max_token_supply,
        max_per_address_mint: msg.base_fields.max_per_address_mint,
        mint_price: msg.base_fields.mint_price,
        mint_denom: msg.base_fields.mint_denom,
        base_token_uri: msg.base_fields.base_token_uri,
        name: msg.name.clone(),
        symbol: msg.symbol.clone(),
        token_code_id: msg.base_fields.token_code_id,
        extension: collection_info,
        escrow_funds: msg.base_fields.escrow_funds,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &config)?;
    CURRENT_TOKEN_SUPPLY.save(deps.storage, &msg.max_token_supply)?;

    let mut sub_msgs: Vec<SubMsg> = vec![];

    // This was previously validated, so should be okay to instantiate
    if let Some(module_info) = msg.airdropper_instantiate_info {
        let airdropper_instantiate_msg = module_info.into_wasm_msg(env.contract.address.clone());

        let airdropper_instantiate_msg: SubMsg<Empty> =
            SubMsg::reply_on_success(airdropper_instantiate_msg, INSTANTIATE_AIRDROPPER_REPLY_ID);

        sub_msgs.push(airdropper_instantiate_msg);
    }

    // This was previously validated, so should be okay to instantiate
    if let Some(module_info) = msg.whitelist_instantiate_info {
        let whitelist_instantiate_msg = module_info.into_wasm_msg(env.contract.address.clone());

        let whitelist_instantiate_msg: SubMsg<Empty> =
            SubMsg::reply_on_success(whitelist_instantiate_msg, INSTANTIATE_WHITELIST_REPLY_ID);

        sub_msgs.push(whitelist_instantiate_msg);
    }

    // borrowed from stargaze's minter.
    // shuffles the token ids for an element of randomness
    let shuffled_token_ids = shuffle_token_ids(
        &env,
        info.sender.clone(),
        (1..=msg.max_token_supply).collect::<Vec<u32>>(),
    )?;

    let mut token_index = 1;
    for token_id in shuffled_token_ids {
        SHUFFLED_TOKEN_IDS.save(deps.storage, token_index, &token_id)?;
        TOKEN_ID_POSITIONS.save(deps.storage, token_id, &token_index)?;
        token_index += 1;
    }

    // instantiate cw721 contract
    let cw721_instantiate_info: ModuleInstantiateInfo = ModuleInstantiateInfo {
        code_id: msg.base_fields.token_code_id,
        msg: to_binary(&Cw721InstantiateMsg {
            name: msg.name.clone(),
            symbol: msg.symbol,
            minter: env.contract.address.clone().into_string(),
        })?,
        admin: Admin::None {},
        label: String::from("Instantiate fixed price NFT contract"),
    };

    let cw721_instantiate_msg = cw721_instantiate_info.into_wasm_msg(env.contract.address);

    let cw721_instantiate_msg: SubMsg<Empty> =
        SubMsg::reply_on_success(cw721_instantiate_msg, INSTANTIATE_TOKEN_REPLY_ID);

    sub_msgs.push(cw721_instantiate_msg);

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", info.sender)
        .add_submessages(sub_msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // mint
    match msg {
        ExecuteMsg::UpdateConfig(msg) => execute_update_config(deps, env, info, msg),
        ExecuteMsg::InitSubmodule(module_info) => {
            execute_init_submodule(deps, env, info, module_info)
        }
        ExecuteMsg::UpdateWhitelistAddress(address) => {
            execute_update_whitelist_address(deps, env, info, address)
        }
        ExecuteMsg::UpdateAirdropAddress(address) => {
            execute_update_airdrop_address(deps, env, info, address)
        }
        ExecuteMsg::Mint {} => execute_mint(deps, env, info),
        ExecuteMsg::AirdropMint { minter_address } => {
            execute_airdrop_mint(deps, env, info, minter_address)
        }
        ExecuteMsg::AirdropClaim { minter_address } => {
            execute_airdrop_token_distribution(deps, env, info, minter_address)
        }
        ExecuteMsg::CleanClaimedTokensFromShuffle {} => {
            execute_clean_claimed_tokens_from_shuffle(deps, env, info)
        }
        ExecuteMsg::ShuffleTokenOrder {} => execute_shuffle_token_order(deps, env, info),
        ExecuteMsg::SubmoduleHook(target, msg) => {
            execute_submodule_hook(deps, env, info, target, msg)
        }
        ExecuteMsg::DisburseFunds {} => execute_disburse_funds(deps, env, info),
    }
}

fn execute_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: BaseInitMsg,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info)?;

    let mut config = CONFIG.load(deps.storage)?;

    let mut res: Response = Response::new();

    let maintainer_addr: Option<Addr> = maybe_addr(deps.api, msg.maintainer_address.clone())?;

    if maintainer_addr != config.maintainer_addr {
        config.maintainer_addr = maintainer_addr;

        // dispatch calls to these methods if addresses exist
        if let Some(addr) = WHITELIST_ADDR.may_load(deps.storage)? {
            let update_msg = WL_UpdateMaintainerAddress(msg.maintainer_address.clone());
            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: addr.into_string(),
                msg: to_binary(&update_msg)?,
                funds: vec![],
            });

            res = res.add_message(msg);
        }

        if let Some(addr) = AIRDROPPER_ADDR.may_load(deps.storage)? {
            let update_msg = AD_UpdateMaintainerAddress(msg.maintainer_address.clone());
            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: addr.into_string(),
                msg: to_binary(&update_msg)?,
                funds: vec![],
            });

            res = res.add_message(msg);
        }
    }

    if msg.start_time != config.start_time {
        if msg.start_time <= env.block.time {
            return Err(ContractError::InvalidStartTime {});
        }

        if msg.start_time
            >= config
                .end_time
                .unwrap_or_else(|| msg.start_time.plus_nanos(1u64))
        {
            return Err(ContractError::InvalidStartTime {});
        }

        config.start_time = msg.start_time;
    }

    if msg.end_time != config.end_time {
        if msg
            .end_time
            .unwrap_or_else(|| env.block.time.plus_nanos(1u64))
            <= env.block.time
        {
            return Err(ContractError::InvalidEndTime {});
        }

        if msg
            .end_time
            .unwrap_or_else(|| config.start_time.plus_nanos(1u64))
            <= config.start_time
        {
            return Err(ContractError::InvalidEndTime {});
        }

        config.end_time = msg.end_time;
    }

    if msg.max_per_address_mint != config.max_per_address_mint {
        // I'm failing to see how clippy is making this easier/more efficient
        // if count < 1 || count > MAX_PER_ADDRESS_MINT || count > config.max_token_supply {
        if !(1..=MAX_PER_ADDRESS_MINT).contains(&msg.max_per_address_mint) {
            return Err(ContractError::InvalidMaxPerAddressMint {
                max: cmp::min(MAX_PER_ADDRESS_MINT, config.max_token_supply),
                input: msg.max_per_address_mint,
            });
        }

        config.max_per_address_mint = msg.max_per_address_mint;
    }

    if msg.mint_denom != config.mint_denom {
        if msg.mint_denom != NATIVE_DENOM && !msg.mint_denom.starts_with("ibc/") {
            return Err(ContractError::InvalidPaymentType {});
        }

        config.mint_denom = msg.mint_denom;
    }

    if msg.mint_price != config.mint_price {
        config.mint_price = msg.mint_price;
    }

    if msg.max_per_address_mint != config.max_per_address_mint {
        // this may be simplified to just checking against `max_token_supply`
        if msg.max_per_address_mint < 1
            || msg.max_per_address_mint > MAX_PER_ADDRESS_MINT
            || msg.max_per_address_mint > config.max_token_supply
        {
            return Err(ContractError::InvalidMaxPerAddressMint {
                max: cmp::min(MAX_PER_ADDRESS_MINT, config.max_token_supply),
                input: msg.max_per_address_mint,
            });
        }

        config.max_per_address_mint = msg.max_per_address_mint;
    }

    if msg.base_token_uri != config.base_token_uri {
        validate_uri(msg.base_token_uri.clone())?;

        config.base_token_uri = msg.base_token_uri;
    }

    if msg.token_code_id != config.token_code_id {
        config.token_code_id = msg.token_code_id;
    }

    if msg.escrow_funds != config.escrow_funds {
        config.escrow_funds = msg.escrow_funds;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(res
        .add_attribute("method", "update_config")
        .add_attribute("sender", info.sender))
}

fn execute_init_submodule(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module_info: ModuleInstantiateInfo,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info)?;

    // needs to be valid reply_id
    if module_info.code_id != INSTANTIATE_AIRDROPPER_REPLY_ID
        && module_info.code_id != INSTANTIATE_WHITELIST_REPLY_ID
    {
        println!("{:?}", module_info.code_id);
        Err(ContractError::InvalidSubmoduleCodeId {})
    } else {
        println!("env.contract.addressenv.contract.address {:?}", module_info);
        let msg = module_info.clone().into_wasm_msg(env.contract.address);

        let msg: SubMsg<Empty> = SubMsg::reply_on_success(msg, module_info.code_id);

        Ok(Response::new().add_submessage(msg))
    }
}

fn execute_update_whitelist_address(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: Option<String>,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info)?;

    match maybe_addr(deps.api, address)? {
        Some(addr) => WHITELIST_ADDR.save(deps.storage, &addr)?,
        None => WHITELIST_ADDR.remove(deps.storage),
    }

    Ok(Response::new()
        .add_attribute("method", "update_whitelist_address")
        .add_attribute("sender", info.sender))
}

fn execute_update_airdrop_address(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: Option<String>,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info)?;

    match maybe_addr(deps.api, address)? {
        Some(addr) => AIRDROPPER_ADDR.save(deps.storage, &addr)?,
        None => AIRDROPPER_ADDR.remove(deps.storage),
    }

    Ok(Response::new()
        .add_attribute("method", "update_whitelist_address")
        .add_attribute("sender", info.sender))
}

/// main public/whitelist minting method
pub fn execute_mint(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // check token supply
    let current_token_supply = CURRENT_TOKEN_SUPPLY.load(deps.storage)?;

    if current_token_supply == 0 {
        return Err(ContractError::MintCompleted {});
    }

    let config = CONFIG.load(deps.storage)?;

    // ensure campaign has not ended
    // TODO: move to optional?
    if config
        .end_time
        .unwrap_or_else(|| env.block.time.plus_nanos(1u64))
        <= env.block.time
    {
        return Err(ContractError::CampaignHasEnded {});
    }

    let mut mint_price: Uint128 = config.mint_price;
    let mut _mint_type: MintType = MintType::None;
    let mut _can_mint: bool = false;

    // if start time has NOT occurred then assess whitelist criteria, otherwise check public mint
    if env.block.time < config.start_time {
        // if this user is whitelist eligible via `can_mint` then we'll allow them through
        // else we error out as it is before start time of campaign
        let check_wl = check_whitelist(deps.as_ref(), &info)?;
        if check_wl.can_mint {
            if check_wl.mint_price.is_none() {
                return Err(ContractError::InvalidMintPrice {});
            }

            _mint_type = MintType::Whitelist;
            mint_price = check_wl.mint_price.unwrap();
            _can_mint = check_wl.can_mint;
        } else {
            return Err(ContractError::BeforeStartTime {});
        }
    } else {
        // if this user has public mints left then we allow them through
        let check_public_mint = check_public_mint(deps.as_ref(), env.clone(), &info)?;
        if check_public_mint {
            _can_mint = check_public_mint;
            _mint_type = MintType::Public;
        }
    }

    if _can_mint {
        let minter_addr: Addr = info.sender.clone();
        return _execute_mint(deps, env, info, _mint_type, mint_price, minter_addr);
    }

    Err(ContractError::UnableToMint {})
}

/// Airdrop Promised mint method
fn execute_airdrop_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    minter_address: Option<String>,
) -> Result<Response, ContractError> {
    // check token supply
    let current_token_supply = CURRENT_TOKEN_SUPPLY.load(deps.storage)?;

    if current_token_supply == 0 {
        return Err(ContractError::MintCompleted {});
    }

    // establish minter address. if no `minter_address` is provided, then we default to sender
    let minter_addr: Addr =
        (maybe_addr(deps.api, minter_address)?).unwrap_or_else(|| info.sender.clone());

    let config = CONFIG.load(deps.storage)?;

    // allow admin or maintainer to also execute this would allow for a push and pulls of airdrops
    if minter_addr != info.sender
        && config.admin != info.sender
        && config.maintainer_addr != Some(info.sender.clone())
    {
        return Err(ContractError::Unauthorized {});
    }

    // check the address' promised mints
    let check_airdropper_mint_res = check_airdrop_promises(
        deps.as_ref(),
        &info,
        MintType::PromisedMint,
        minter_addr.clone(),
    )?;

    // if mint eligible, execute mint (probably 0 token mint fee)
    if check_airdropper_mint_res.can_mint {
        _execute_mint(
            deps,
            _env,
            info,
            MintType::PromisedMint,
            check_airdropper_mint_res.mint_price.unwrap(),
            minter_addr,
        )
    } else {
        Err(ContractError::NoPromisedMints {})
    }
}

/// method that finalizes the mint and generates the submessages
fn _execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    mint_type: MintType,
    mint_price: Uint128,
    minter_addr: Addr,
) -> Result<Response, ContractError> {
    // check supply
    let current_token_supply = CURRENT_TOKEN_SUPPLY.load(deps.storage)?;

    if current_token_supply == 0 {
        return Err(ContractError::MintCompleted {});
    }

    let config = CONFIG.load(deps.storage)?;

    // check payment
    let payment = may_pay(&info, &config.mint_denom)?;

    if payment != mint_price {
        return Err(ContractError::IncorrectPaymentAmount {
            token: config.mint_denom,
            amt: mint_price,
        });
    }

    // TODO: add another element of randomness here?
    let token_index = SHUFFLED_TOKEN_IDS
        .keys(deps.storage, None, None, Order::Ascending)
        .take(1)
        .collect::<StdResult<Vec<_>>>()?[0];

    let token_id = SHUFFLED_TOKEN_IDS.load(deps.storage, token_index)?;

    // Create mint msgs
    let mint_msg = Cw721ExecuteMsg::Mint(MintMsg::<CollectionInfo> {
        token_id: token_id.to_string(),
        owner: minter_addr.clone().into_string(),
        token_uri: Some(format!("{}/{}", config.base_token_uri, token_id)),
        extension: config.extension.clone(),
    });

    let token_address = CW721_ADDR.load(deps.storage)?;

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_address.into_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    });

    let mut res = Response::new().add_message(msg);

    // remove token id from
    SHUFFLED_TOKEN_IDS.remove(deps.storage, token_index);
    TOKEN_ID_POSITIONS.remove(deps.storage, token_id);
    CURRENT_TOKEN_SUPPLY.save(deps.storage, &(current_token_supply - 1))?;

    match mint_type {
        MintType::Public => {
            // update internal mint tracker
            let current_mint_count =
                (ADDRESS_MINT_TRACKER.may_load(deps.storage, minter_addr.clone())?).unwrap_or(0);

            ADDRESS_MINT_TRACKER.save(deps.storage, minter_addr, &(current_mint_count + 1))?;
        }
        MintType::Whitelist => {
            // fire call to update whitelist
            let whitelist_addr = WHITELIST_ADDR.load(deps.storage)?;
            let update_msg = WL_UpdateAddressMintTracker(minter_addr.into_string());
            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: whitelist_addr.into_string(),
                msg: to_binary(&update_msg)?,
                funds: vec![],
            });

            res = res.add_message(msg);
        }
        MintType::PromisedMint => {
            // update airdropper mint tracker
            let airdropper_addr = AIRDROPPER_ADDR.load(deps.storage)?;
            let update_msg = AD_IncrementAddressClaimedPromisedMintCount(minter_addr.into_string());
            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: airdropper_addr.into_string(),
                msg: to_binary(&update_msg)?,
                funds: vec![],
            });

            res = res.add_message(msg);
        }
        _ => {
            return Err(ContractError::UnableToMint {});
        }
    }

    // TODO: add other disbursement methods eg contract escrow so we dont blow up
    // an address' tx history
    if mint_price > Uint128::zero() {
        let mut filtered_royalties = config.extension.mint_revenue_share;

        // place the is_primary address at the bottom
        // this address absorbs the remaining funds at the end of the calcs
        filtered_royalties.sort_by(|a, b| b.is_primary.cmp(&a.is_primary));

        let mut primary_royalty_addr: Option<Addr> = None;
        let mut remaining_mint_amount: Uint128 = mint_price;
        for (i, royalty) in filtered_royalties.iter().enumerate() {
            if remaining_mint_amount > Uint128::zero() {
                if primary_royalty_addr.is_none() && royalty.is_primary {
                    primary_royalty_addr = Some(royalty.addr.clone())
                }

                let amt: Uint128 = if i == filtered_royalties.len() && royalty.is_primary {
                    remaining_mint_amount
                } else {
                    calculate_royalty_amount(mint_price, royalty.bps, remaining_mint_amount)
                };

                remaining_mint_amount -= amt;

                if config.escrow_funds {
                    let balance = (BANK_BALANCES.may_load(deps.storage, royalty.addr.clone())?)
                        .unwrap_or(Uint128::zero());

                    BANK_BALANCES.save(deps.storage, royalty.addr.clone(), &(balance + amt))?;
                } else {
                    let msg = BankMsg::Send {
                        to_address: royalty.addr.clone().into_string(),
                        amount: vec![coin(u128::from(amt), config.mint_denom.clone())],
                    };

                    res = res.add_message(msg);
                }
            }
        }

        if remaining_mint_amount > Uint128::zero() {
            if config.escrow_funds {
                let balance = (BANK_BALANCES
                    .may_load(deps.storage, primary_royalty_addr.clone().unwrap())?)
                .unwrap_or(Uint128::zero());

                BANK_BALANCES.save(
                    deps.storage,
                    primary_royalty_addr.unwrap(),
                    &(balance + remaining_mint_amount),
                )?;
            } else {
                let msg = BankMsg::Send {
                    to_address: primary_royalty_addr.unwrap().into_string(),
                    amount: vec![coin(u128::from(remaining_mint_amount), config.mint_denom)],
                };

                res = res.add_message(msg);
            }
        }
    }

    Ok(res)
}

fn execute_airdrop_token_distribution(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    minter_address: Option<String>,
) -> Result<Response, ContractError> {
    // default to self if no address passed in
    let minter_addr: Addr =
        (maybe_addr(deps.api, minter_address)?).unwrap_or_else(|| info.sender.clone());

    let config = CONFIG.load(deps.storage)?;
    if minter_addr != info.sender
        && config.admin != info.sender
        && config.maintainer_addr != Some(info.sender.clone())
    {
        return Err(ContractError::Unauthorized {});
    }

    // check if we have promised tokens
    let check_airdropper_mint_res = check_airdrop_promises(
        deps.as_ref(),
        &info,
        MintType::PromisedToken,
        minter_addr.clone(),
    )?;

    if check_airdropper_mint_res.can_mint {
        _execute_claim_by_token_id(
            deps,
            env,
            info,
            minter_addr,
            check_airdropper_mint_res.remaining_token_ids,
        )
    } else {
        Err(ContractError::NoPromisedMints {})
    }
}

fn _execute_claim_by_token_id(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    minter_addr: Addr,
    token_ids: Vec<u32>,
) -> Result<Response, ContractError> {
    if token_ids.is_empty() {
        return Err(ContractError::NoPromisedMints {});
    }

    let config = CONFIG.load(deps.storage)?;
    let token_address = CW721_ADDR.load(deps.storage)?;
    let promised_token_id = token_ids[0];
    let mut res: Response = Response::new();
    // this is an error and we'll need to go remove it
    if promised_token_id > config.max_token_supply {
    } else {
        let token_index: u32 =
            (TOKEN_ID_POSITIONS.may_load(deps.storage, promised_token_id)?).unwrap_or(0);

        // if maintainer already cleared out the queue, then this wont be necessary
        if token_index > 0 {
            let token_id = (SHUFFLED_TOKEN_IDS.may_load(deps.storage, token_index)?).unwrap_or(0);
            SHUFFLED_TOKEN_IDS.remove(deps.storage, token_index);
            TOKEN_ID_POSITIONS.remove(deps.storage, token_id);

            let current_token_supply = CURRENT_TOKEN_SUPPLY.load(deps.storage)?;
            CURRENT_TOKEN_SUPPLY.save(deps.storage, &(current_token_supply - 1))?;
        }

        // Create mint msgs
        let mint_msg = Cw721ExecuteMsg::Mint(MintMsg::<CollectionInfo> {
            token_id: promised_token_id.to_string(),
            owner: minter_addr.to_string(),
            token_uri: Some(format!("{}/{}", config.base_token_uri, promised_token_id)),
            extension: config.extension,
        });
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_address.to_string(),
            msg: to_binary(&mint_msg)?,
            funds: vec![],
        });

        res = res.add_message(msg);
    }

    let airdropper_addr = AIRDROPPER_ADDR.load(deps.storage)?;
    let update_msg = AD_MarkTokenIDClaimed(minter_addr.to_string(), promised_token_id);
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: airdropper_addr.into_string(),
        msg: to_binary(&update_msg)?,
        funds: vec![],
    });

    res = res.add_message(msg);

    Ok(res)
}

fn execute_clean_claimed_tokens_from_shuffle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info)?;

    if let Some(addr) = AIRDROPPER_ADDR.may_load(deps.storage)? {
        let assigned_token_ids: Vec<u32> = deps.querier.query_wasm_smart(
            addr,
            &AirdropperQueryMsg::GetAssignedTokenIDs {
                start_after: None,
                limit: None,
            },
        )?;

        for token_id in assigned_token_ids {
            let position = (TOKEN_ID_POSITIONS.may_load(deps.storage, token_id)?).unwrap_or(0);
            if position > 0 {
                SHUFFLED_TOKEN_IDS.remove(deps.storage, position);
                TOKEN_ID_POSITIONS.remove(deps.storage, token_id);
            }
        }

        Ok(Response::new()
            .add_attribute("method", "clean_claimed_tokens_with_shuffle")
            .add_attribute("sender", info.sender))
    } else {
        Err(ContractError::InvalidAirdropperAddress {})
    }
}

fn execute_shuffle_token_order(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let current_token_supply = CURRENT_TOKEN_SUPPLY.load(deps.storage)?;

    if current_token_supply == 0 {
        return Err(ContractError::MintCompleted {});
    }

    let config = CONFIG.load(deps.storage)?;

    if config.admin != info.sender && config.maintainer_addr != Some(info.sender.clone()) {
        return Err(ContractError::Unauthorized {});
    }

    let token_ids: Vec<u32> = TOKEN_ID_POSITIONS
        .keys(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<u32>>>()?;

    let positions: Vec<u32> = SHUFFLED_TOKEN_IDS
        .keys(deps.storage, None, None, Order::Descending)
        .take(1)
        .collect::<StdResult<Vec<u32>>>()?;

    let shuffled_token_ids = shuffle_token_ids(&env, info.sender.clone(), token_ids.clone())?;

    let mut token_index = 1;
    for token_id in shuffled_token_ids {
        SHUFFLED_TOKEN_IDS.save(deps.storage, token_index, &token_id)?;
        TOKEN_ID_POSITIONS.save(deps.storage, token_id, &token_index)?;
        token_index += 1;
    }

    // trim the edges off from the shuffled tokenids list
    if usize::try_from(positions[0]).unwrap() > token_ids.len() {
        for i in token_index..=config.max_token_supply {
            SHUFFLED_TOKEN_IDS.remove(deps.storage, i);
        }
    }

    Ok(Response::new()
        .add_attribute("method", "shuffle_token_order")
        .add_attribute("sender", info.sender))
}

fn execute_submodule_hook(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    target: ExecutionTarget,
    msg: CosmosMsg<Empty>,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info)?;

    // extract target contract address from cosmosmsg::wasmmsg
    let target_contract_address: String = match msg.clone() {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg: _,
            funds: _,
        }) => contract_addr.to_lowercase(),
        _ => {
            return Err(ContractError::Unauthorized {});
        }
    };

    // extract stored contract address
    let contract_address: String = match target {
        ExecutionTarget::Airdropper => (AIRDROPPER_ADDR.load(deps.storage)?)
            .into_string()
            .to_lowercase(),
        ExecutionTarget::Whitelist => (WHITELIST_ADDR.load(deps.storage)?)
            .into_string()
            .to_lowercase(),
        _ => {
            return Err(ContractError::Unauthorized {});
        }
    };

    // ensure addresses match
    if contract_address != target_contract_address {
        return Err(ContractError::InvalidTargetAddress {});
    }

    Ok(Response::default()
        .add_attribute("method", "submodule_hook")
        .add_message(msg))
}

fn execute_disburse_funds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // EITHER admin (minting contract) or maintainer can update/
    if config.admin != info.sender && config.maintainer_addr != Some(info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let contract_balance: Coin = deps
        .querier
        .query_balance(&env.contract.address, config.mint_denom.clone())?;

    let balances: Vec<AddrBal> = BANK_BALANCES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (addr, balance) = item?;
            Ok(AddrBal { addr, balance })
        })
        .collect::<StdResult<Vec<AddrBal>>>()
        .unwrap();

    let mut remaining_balance: Uint128 = contract_balance.amount;
    let mut msgs: Vec<BankMsg> = vec![];

    for addr_bal in balances {
        if addr_bal.balance > Uint128::zero() && remaining_balance >= addr_bal.balance {
            msgs.push(BankMsg::Send {
                to_address: addr_bal.addr.clone().into_string(),
                amount: vec![coin(
                    u128::from(addr_bal.balance),
                    config.mint_denom.clone(),
                )],
            });

            remaining_balance -= addr_bal.balance;
            BANK_BALANCES.save(deps.storage, addr_bal.addr, &Uint128::zero())?;
        }
    }
    Ok(Response::default()
        .add_attribute("method", "disburse_funds")
        .add_messages(msgs))
}

// #region helper functions

fn validate_collection_info(
    deps: Deps,
    msg: CollectionInfoMsg,
) -> Result<CollectionInfo, ContractError> {
    let secondary_metadata_uri: Option<String> = match msg.secondary_metadata_uri {
        Some(uri) => {
            validate_uri(uri.clone())?;
            Some(uri)
        }
        None => None,
    };
    let collection_info: CollectionInfo = CollectionInfo {
        secondary_metadata_uri,
        mint_revenue_share: validate_royalties(deps, msg.mint_revenue_share, true)?,
        secondary_market_royalties: validate_royalties(
            deps,
            msg.secondary_market_royalties,
            false,
        )?,
    };

    Ok(collection_info)
}

fn validate_uri(uri: String) -> Result<String, ContractError> {
    // url is too short
    if uri.len() < 4 {
        return Err(ContractError::InvalidBaseTokenURI {});
    }

    // validate url is of ipfs schema
    let parsed_base_token_uri = Url::parse(&uri)?;
    if parsed_base_token_uri.scheme() != "ipfs" {
        Err(ContractError::InvalidBaseTokenURI {})
    } else {
        Ok(uri)
    }
}

fn validate_royalties(
    deps: Deps,
    royalties: Vec<RoyaltyInfoMsg>,
    is_mint_royalties: bool,
) -> Result<Vec<RoyaltyInfo>, ContractError> {
    let mut royalty_infos: Vec<RoyaltyInfo> = vec![];
    let mut running_bps: u32 = 0;
    let mut is_primary_ctr: u32 = 0;

    for royalty_info in royalties {
        running_bps += royalty_info.bps;

        if royalty_info.is_primary {
            is_primary_ctr += 1;
        }

        royalty_infos.push(RoyaltyInfo {
            addr: deps.api.addr_validate(&royalty_info.address)?,
            bps: royalty_info.bps,
            is_primary: royalty_info.is_primary,
        });
    }

    if is_mint_royalties {
        if running_bps != MAX_BPS {
            return Err(ContractError::InvalidBPS {
                running: running_bps,
                max: MAX_BPS,
            });
        }

        if is_primary_ctr != 1 {
            return Err(ContractError::NoRoyalPrimaryAddress {});
        }
    } else {
        if running_bps >= MAX_BPS_FOR_SECONDARY {
            return Err(ContractError::InvalidBPS {
                running: running_bps,
                max: MAX_BPS_FOR_SECONDARY,
            });
        }

        if is_primary_ctr > 1 {
            return Err(ContractError::NoRoyalPrimaryAddress {});
        }
    }

    Ok(royalty_infos)
}

fn calculate_royalty_amount(
    mint_price: Uint128,
    bps: u32,
    remaining_royalty_amount: Uint128,
) -> Uint128 {
    // bps should have been verified earlier
    // for mints, total should not be more than MAX_BPS (10000) -- representing 100.00%
    // for secondary, up to `MAX_BPS_FOR_SECONDARY` (5000), but ideally a portion will go to the original owner of the cw721
    let mut amt: Uint128 = mint_price * Uint128::from(bps) / Uint128::from(MAX_BPS);

    if amt >= remaining_royalty_amount {
        amt = remaining_royalty_amount;
    }

    amt
}

fn shuffle_token_ids(
    env: &Env,
    sender: Addr,
    mut tokens: Vec<u32>,
) -> Result<Vec<u32>, ContractError> {
    let sha256 = Sha256::digest(
        format!("{}{}{}", sender, env.block.height + 69, tokens.len() + 69).into_bytes(),
    );
    // Cut first 16 bytes from 32 byte value
    let randomness: [u8; 16] = sha256.to_vec()[0..16].try_into().unwrap();
    let mut rng = Xoshiro128StarStar::from_seed(randomness);
    let mut shuffler = FisherYates::default();

    shuffler
        .shuffle(&mut tokens, &mut rng)
        .map_err(StdError::generic_err)?;

    Ok(tokens)
}

// #endregion

// #region gates

fn check_can_update(deps: Deps, env: &Env, info: &MessageInfo) -> Result<bool, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // EITHER admin (minting contract) or maintainer can update/
    if config.admin == info.sender.clone() || config.maintainer_addr == Some(info.sender.clone()) {
        // campaign started
        if config.start_time <= env.block.time {
            return Err(ContractError::MintIsActive {});
        }

        // campaing ended
        if config
            .end_time
            .unwrap_or_else(|| env.block.time.plus_nanos(1u64))
            <= env.block.time
        {
            return Err(ContractError::CampaignHasEnded {});
        }

        // check token supply
        let current_token_supply = CURRENT_TOKEN_SUPPLY.load(deps.storage)?;

        if current_token_supply == 0 {
            return Err(ContractError::MintCompleted {});
        }

        Ok(true)
    } else {
        Err(ContractError::Unauthorized {})
    }
}

fn check_whitelist(
    deps: Deps,
    info: &MessageInfo,
) -> Result<MintParametersResponse, ContractError> {
    if let Some(whitelist_addr) = WHITELIST_ADDR.may_load(deps.storage)? {
        let wl_config: CheckWhitelistResponse = deps.querier.query_wasm_smart(
            whitelist_addr,
            &WhitelistQueryMsg::CheckWhitelist {
                minter_address: info.sender.clone().to_string(),
            },
        )?;

        if !wl_config.is_on_whitelist {
            return Err(ContractError::NotOnWhitelist {});
        }

        // must NOT be closed. AND in progress AND user is on whitelist
        if !wl_config.whitelist_is_closed {
            if !wl_config.whitelist_in_progress {
                return Err(ContractError::WhitelistNotInProgress {});
            }

            if wl_config.current_mint_count >= wl_config.max_per_address_mint {
                return Err(ContractError::WhitelistMaxMintReached(
                    wl_config.max_per_address_mint,
                ));
            }

            Ok(MintParametersResponse {
                can_mint: true,
                mint_price: Some(wl_config.mint_price),
                remaining_token_ids: vec![],
            })
        } else {
            Err(ContractError::WhitelistClosed {})
        }
    } else {
        Err(ContractError::InvalidWhitelistAddress {})
    }
}

fn check_airdrop_promises(
    deps: Deps,
    info: &MessageInfo,
    mint_type: MintType,
    minter_addr: Addr,
) -> Result<MintParametersResponse, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if minter_addr != info.sender
        && config.admin != info.sender
        && config.maintainer_addr.unwrap_or(config.admin) != info.sender
    {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(airdropper_addr) = AIRDROPPER_ADDR.may_load(deps.storage)? {
        let mut mint_params: MintParametersResponse = MintParametersResponse {
            can_mint: false,
            mint_price: None,
            remaining_token_ids: vec![],
        };

        match mint_type {
            MintType::PromisedMint => {
                let promised_mints: CheckAirdropPromisedMintResponse =
                    deps.querier.query_wasm_smart(
                        airdropper_addr,
                        &AirdropperQueryMsg::CheckAddressPromisedMints {
                            minter_address: minter_addr.into_string(),
                        },
                    )?;

                if promised_mints.promised_mint_count == 0 {
                    return Err(ContractError::NoPromisedMints {});
                }

                if !promised_mints.airdrop_mint_is_closed {
                    if !promised_mints.airdrop_mint_in_progress {
                        return Err(ContractError::BeforePremintStarttime {});
                    }

                    // if an address' claimed mint count >= promised mint count, then kick them out
                    if promised_mints.claimed_mint_count >= promised_mints.promised_mint_count {
                        return Err(ContractError::AllPromisesFulfilled {});
                    } else {
                        mint_params.can_mint = true;
                        mint_params.mint_price = Some(Uint128::zero());
                    }
                } else {
                    return Err(ContractError::AirdropClosed {});
                }
            }
            MintType::PromisedToken => {
                let promised_tokens: CheckAirdropPromisedTokensResponse =
                    deps.querier.query_wasm_smart(
                        airdropper_addr,
                        &AirdropperQueryMsg::CheckAddressPromisedTokens {
                            minter_address: minter_addr.into_string(),
                        },
                    )?;

                if !promised_tokens.airdrop_mint_is_closed {
                    if !promised_tokens.airdrop_mint_in_progress {
                        return Err(ContractError::BeforePremintStarttime {});
                    }

                    if promised_tokens.address_promised_token_ids.is_empty() {
                        return Err(ContractError::AllPromisesFulfilled {});
                    } else {
                        mint_params.can_mint = true;
                        mint_params.mint_price = Some(Uint128::zero());
                        mint_params.remaining_token_ids =
                            promised_tokens.address_promised_token_ids;
                    }
                } else {
                    return Err(ContractError::AirdropClosed {});
                }
            }
            _ => {
                return Err(ContractError::NoPromisedMints {});
            }
        }

        Ok(mint_params)
    } else {
        Err(ContractError::InvalidAirdropperAddress {})
    }
}

fn check_public_mint(deps: Deps, env: Env, info: &MessageInfo) -> Result<bool, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut can_mint: bool = false;

    if env.block.time < config.start_time {
        return Err(ContractError::BeforeStartTime {});
    }

    if config
        .end_time
        .unwrap_or_else(|| env.block.time.plus_nanos(1u64))
        <= env.block.time
    {
        return Err(ContractError::CampaignHasEnded {});
    }

    if config.start_time <= env.block.time
        && env.block.time
            < config
                .end_time
                .unwrap_or_else(|| env.block.time.plus_nanos(1u64))
    {
        can_mint = true;
    }

    let current_mint_count =
        (ADDRESS_MINT_TRACKER.may_load(deps.storage, info.sender.clone())?).unwrap_or(0);

    if cmp::max(config.max_per_address_mint - current_mint_count, 0) == 0 {
        return Err(ContractError::PublicMaxMintReached(
            config.max_per_address_mint,
        ));
    }

    Ok(can_mint)
}

// #endregion

// #region funds

//fn send_funds()

// #endregion

// Reply callback triggered from cw721 contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match parse_reply_instantiate_data(msg.clone()) {
        Ok(res) => {
            let addr = deps.api.addr_validate(&res.contract_address)?;
            match msg.id {
                INSTANTIATE_TOKEN_REPLY_ID => {
                    CW721_ADDR.save(deps.storage, &addr)?;
                }
                INSTANTIATE_AIRDROPPER_REPLY_ID => {
                    println!("{:?}", INSTANTIATE_AIRDROPPER_REPLY_ID);
                    println!("{:?}", res.contract_address);
                    AIRDROPPER_ADDR.save(deps.storage, &addr)?;
                }
                INSTANTIATE_WHITELIST_REPLY_ID => {
                    WHITELIST_ADDR.save(deps.storage, &addr)?;
                }
                _ => {
                    return Err(ContractError::InvalidTokenReplyId {});
                }
            }
            Ok(Response::default())
        }
        Err(error) => Err(ContractError::ContractInstantiateError {
            contract: match msg.id {
                INSTANTIATE_TOKEN_REPLY_ID => "CW721_ADDR".to_string(),
                INSTANTIATE_AIRDROPPER_REPLY_ID => "AIRDROPPER_ADDR".to_string(),
                INSTANTIATE_WHITELIST_REPLY_ID => "WHITELIST_ADDR".to_string(),
                _ => "ERROR".to_string(),
            },
            error,
        }),
    }
}
