use crate::error::ContractError;
use crate::msg::{
    AddrBal, AddressValMsg, Admin, BaseInitMsg, CollectionInfoMsg, ExecuteMsg, ExecutionTarget,
    InstantiateMsg, MintType, ModuleInstantiateInfo, RoyaltyInfoMsg, SharedCollectionInfoMsg,
    TokenMsg,
};
use crate::state::{
    CollectionInfo, Config, RoyaltyInfo, SharedCollectionInfo, ADDRESS_MINT_TRACKER,
    AIRDROPPER_ADDR, BANK_BALANCES, BUNDLE_MINT_TRACKER, COLLECTION_CURRENT_TOKEN_SUPPLY, CONFIG,
    CURRENT_TOKEN_SUPPLY, CW721_ADDRS, CW721_COLLECTION_INFO, CW721_SHUFFLED_TOKEN_IDS,
    FEE_COLLECTION_ADDR, TOTAL_TOKEN_SUPPLY, WHITELIST_ADDR, CUSTOM_BUNDLE_MINT_TRACKER, CUSTOM_BUNDLE_TOKENS,
};
use airdropper::{
    msg::ExecuteMsg::{
        IncrementAddressClaimedPromisedMintCount as AD_IncrementAddressClaimedPromisedMintCount,
        MarkTokenIDClaimed as AD_MarkTokenIDClaimed,
        UpdateMaintainerAddress as AD_UpdateMaintainerAddress,
    },
    msg::QueryMsg as AirdropperQueryMsg,
    msg::{
        AddressTokenMsg as AD_AddressTokenMsg, CheckAirdropPromisedMintResponse,
        CheckAirdropPromisedTokensResponse, TokenMsg as AD_TokenMsg,
    },
};
use cosmwasm_std::{
    coin, entry_point, to_binary, Addr, BankMsg, Coin, CosmosMsg, Deps, DepsMut, Empty, Env,
    MessageInfo, Order, Reply, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};

use cw2::set_contract_version;
use cw721_base::{
    msg::ExecuteMsg as Cw721ExecuteMsg, msg::InstantiateMsg as Cw721InstantiateMsg, MintMsg,
};
use cw_utils::{may_pay, maybe_addr, must_pay, parse_reply_instantiate_data};
use rand_core::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro128PlusPlus;
use sha2::{Digest, Sha256};
use shuffle::{fy::FisherYates, shuffler::Shuffler};
use std::cmp;
use std::collections::BTreeMap;
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
const INSTANTIATE_TOKEN_REPLY_ID: u64 = 100;
const INSTANTIATE_AIRDROPPER_REPLY_ID: u64 = 1;
const INSTANTIATE_WHITELIST_REPLY_ID: u64 = 2;
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
    remaining_token_ids: Vec<TokenMsg>,
}

/// Default fee collection address if no DAO address is provided
const DEFAULT_FEE_COLLECTION_ADDRESS: &str = "juno1jv65s3grqf6v6jl3dp4t6c9t9rk99cd83d88wr";

/// default fee amount assumes 6 decimal
const DEFAULT_FEE_AMOUNT: u128 = 1_000_000u128;

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

    let validate_collection_info_res: ValidateCollectionInfoResponse =
        validate_collection_info(deps.as_ref(), msg.collection_infos)?;

    // this may be simplified to just checking against `max_token_supply`
    if msg.base_fields.max_per_address_mint < 1
        || msg.base_fields.max_per_address_mint > MAX_PER_ADDRESS_MINT
        || msg.base_fields.max_per_address_mint > validate_collection_info_res.total_token_supply
    {
        return Err(ContractError::InvalidMaxPerAddressMint {
            max: cmp::min(
                MAX_PER_ADDRESS_MINT,
                validate_collection_info_res.total_token_supply,
            ),
            input: msg.base_fields.max_per_address_mint,
        });
    }

    // if both an address and instantiate info are given, then error out
    if msg.base_fields.airdropper_address.is_some() && msg.airdropper_instantiate_info.is_some() {
        return Err(ContractError::InvalidSubmoduleInstantiation {});
    }

    // if both an address and instantiate info are given, then error out
    if msg.base_fields.whitelist_address.is_some() && msg.whitelist_instantiate_info.is_some() {
        return Err(ContractError::InvalidSubmoduleInstantiation {});
    }

    // validate the initial mint revenue split as well as royalty split
    let shared_collection_info: SharedCollectionInfo =
        validate_shared_collection_info(deps.as_ref(), msg.extension)?;

    // validate the denom the user selected is one that is allowed.
    // cw20 banned

    validate_native_denom(msg.base_fields.mint_denom.clone())?;

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

    let bonded_denom: String = deps.querier.query_bonded_denom()?;

    let config = Config {
        admin: info.sender.clone(),
        maintainer_addr: maybe_addr(deps.api, msg.base_fields.maintainer_address)?,
        start_time: msg.base_fields.start_time,
        end_time: msg.base_fields.end_time,
        total_token_supply: validate_collection_info_res.total_token_supply,
        max_per_address_mint: msg.base_fields.max_per_address_mint,
        max_per_address_bundle_mint: msg.base_fields.max_per_address_bundle_mint,
        mint_price: msg.base_fields.mint_price,
        bundle_mint_price: msg.base_fields.bundle_mint_price,
        mint_denom: msg.base_fields.mint_denom,
        token_code_id: msg.token_code_id,
        extension: shared_collection_info,
        escrow_funds: msg.base_fields.escrow_funds,
        bundle_enabled: msg.base_fields.bundle_enabled,
        bundle_completed: false,
        bonded_denom,
        custom_bundle_enabled: false,
        custom_bundle_completed: false,
        custom_bundle_price: Uint128::from(1_000_000_000u128) // establishing default price
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &config)?;
    CURRENT_TOKEN_SUPPLY.save(
        deps.storage,
        &validate_collection_info_res.total_token_supply,
    )?;

    TOTAL_TOKEN_SUPPLY.save(
        deps.storage,
        &validate_collection_info_res.total_token_supply,
    )?;

    FEE_COLLECTION_ADDR.save(
        deps.storage,
        &deps.api.addr_validate(DEFAULT_FEE_COLLECTION_ADDRESS)?,
    )?;

    let mut sub_msgs: Vec<SubMsg> = vec![];

    // This was previously validated, so should be okay to instantiate
    if let Some(module_info) = msg.airdropper_instantiate_info {
        let airdropper_instantiate_msg = module_info.into_wasm_msg(env.contract.address.clone());

        let airdropper_instantiate_msg: SubMsg<Empty> =
            SubMsg::reply_on_success(airdropper_instantiate_msg, INSTANTIATE_AIRDROPPER_REPLY_ID);

        sub_msgs.push(airdropper_instantiate_msg);
    } else if msg.base_fields.airdropper_address.is_some() {
        AIRDROPPER_ADDR.save(
            deps.storage,
            &deps
                .api
                .addr_validate(&msg.base_fields.airdropper_address.unwrap())?,
        )?;
    }

    // This was previously validated, so should be okay to instantiate
    if let Some(module_info) = msg.whitelist_instantiate_info {
        let whitelist_instantiate_msg = module_info.into_wasm_msg(env.contract.address.clone());

        let whitelist_instantiate_msg: SubMsg<Empty> =
            SubMsg::reply_on_success(whitelist_instantiate_msg, INSTANTIATE_WHITELIST_REPLY_ID);

        sub_msgs.push(whitelist_instantiate_msg);
    } else if msg.base_fields.whitelist_address.is_some() {
        WHITELIST_ADDR.save(
            deps.storage,
            &deps
                .api
                .addr_validate(&msg.base_fields.whitelist_address.unwrap())?,
        )?;
    }

    let mut _token_index = 1;
    for coll_info in validate_collection_info_res.collection_infos {
        // instantiate cw721 contract
        let cw721_instantiate_info: ModuleInstantiateInfo = ModuleInstantiateInfo {
            code_id: msg.token_code_id,
            msg: to_binary(&Cw721InstantiateMsg {
                name: coll_info.name.clone(),
                symbol: coll_info.symbol.clone(),
                minter: env.contract.address.clone().into_string(),
            })?,
            admin: Admin::None {},
            label: String::from("Instantiate fixed price NFT contract"),
        };

        let cw721_instantiate_msg =
            cw721_instantiate_info.into_wasm_msg(env.contract.address.clone());

        let cw721_instantiate_msg: SubMsg<Empty> =
            SubMsg::reply_on_success(cw721_instantiate_msg, coll_info.id);

        sub_msgs.push(cw721_instantiate_msg);
        CW721_COLLECTION_INFO.save(deps.storage, coll_info.id, &coll_info)?;
        COLLECTION_CURRENT_TOKEN_SUPPLY.save(
            deps.storage,
            coll_info.id,
            &coll_info.token_supply,
        )?;

        // shuffle

        let shuffled_token_ids: Vec<u32> = shuffle_token_ids(
            &env,
            info.sender.clone(),
            (1..=coll_info.token_supply).collect::<Vec<u32>>(),
            coll_info.id,
        )?;

        CW721_SHUFFLED_TOKEN_IDS.save(deps.storage, coll_info.id, &shuffled_token_ids)?;
    }

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
        ExecuteMsg::InitSubmodule(reply_id, module_info) => {
            execute_init_submodule(deps, env, info, reply_id, module_info)
        }
        ExecuteMsg::Mint {
            is_promised_mint,
            minter_address,
        } => execute_mint(deps, env, info, is_promised_mint, minter_address),
        ExecuteMsg::MintBundle {} => execute_mint_bundle(deps, env, info),
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
        ExecuteMsg::ProcessCustomBundle { price, tokens } => execute_process_custom_bundle(deps, env, info, price, tokens),
        ExecuteMsg::MintCustomBundle {} => execute_mint_custom_bundle(deps, env, info),
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
        // if count < 1 || count > MAX_PER_ADDRESS_MINT || count > config.total_token_supply {
        if !(1..=MAX_PER_ADDRESS_MINT).contains(&msg.max_per_address_mint) {
            return Err(ContractError::InvalidMaxPerAddressMint {
                max: cmp::min(MAX_PER_ADDRESS_MINT, config.total_token_supply),
                input: msg.max_per_address_mint,
            });
        }

        config.max_per_address_mint = msg.max_per_address_mint;
    }

    if validate_native_denom(msg.mint_denom.clone())? {
        config.mint_denom = msg.mint_denom;
    }

    if msg.mint_price != config.mint_price {
        config.mint_price = msg.mint_price;
    }

    if msg.bundle_mint_price != config.bundle_mint_price {
        config.bundle_mint_price = msg.bundle_mint_price;
    }

    if msg.max_per_address_mint != config.max_per_address_mint {
        // this may be simplified to just checking against `max_token_supply`
        if msg.max_per_address_mint < 1
            || msg.max_per_address_mint > MAX_PER_ADDRESS_MINT
            || msg.max_per_address_mint > config.total_token_supply
        {
            return Err(ContractError::InvalidMaxPerAddressMint {
                max: cmp::min(MAX_PER_ADDRESS_MINT, config.total_token_supply),
                input: msg.max_per_address_mint,
            });
        }

        config.max_per_address_mint = msg.max_per_address_mint;
    }

    if msg.max_per_address_bundle_mint != config.max_per_address_bundle_mint {
        config.max_per_address_bundle_mint = msg.max_per_address_bundle_mint;
    }

    if msg.escrow_funds != config.escrow_funds {
        config.escrow_funds = msg.escrow_funds;
    }

    match maybe_addr(deps.api, msg.airdropper_address)? {
        Some(addr) => AIRDROPPER_ADDR.save(deps.storage, &addr)?,
        None => AIRDROPPER_ADDR.remove(deps.storage),
    }

    match maybe_addr(deps.api, msg.whitelist_address)? {
        Some(addr) => WHITELIST_ADDR.save(deps.storage, &addr)?,
        None => WHITELIST_ADDR.remove(deps.storage),
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
    reply_id: u64,
    module_info: ModuleInstantiateInfo,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info)?;

    // needs to be valid reply_id
    if reply_id != INSTANTIATE_AIRDROPPER_REPLY_ID && reply_id != INSTANTIATE_WHITELIST_REPLY_ID {
        Err(ContractError::InvalidSubmoduleCodeId {})
    } else {
        let msg = module_info.into_wasm_msg(env.contract.address);

        let msg: SubMsg<Empty> = SubMsg::reply_on_success(msg, reply_id);

        Ok(Response::new().add_submessage(msg))
    }
}

/// main public/whitelist minting method
pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    is_airdrop_mint: bool,
    minter_address: Option<String>,
) -> Result<Response, ContractError> {
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

    let minter_addr: Addr =
        (maybe_addr(deps.api, minter_address)?).unwrap_or_else(|| info.sender.clone());

    if is_airdrop_mint {
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

        if check_airdropper_mint_res.can_mint {
            // if mint eligible, execute mint (probably 0 token mint fee)
            _mint_type = MintType::PromisedMint;
            mint_price = check_airdropper_mint_res.mint_price.unwrap();
        } else {
            return Err(ContractError::NoPromisedMints {});
        }
    }
    // if start time has NOT occurred then assess whitelist criteria, otherwise check public mint
    else if env.block.time < config.start_time {
        // if this user is whitelist eligible via `can_mint` then we'll allow them through
        // else we error out as it is before start time of campaign
        let check_wl = check_whitelist(deps.as_ref(), &info)?;
        if check_wl.can_mint {
            if check_wl.mint_price.is_none() {
                return Err(ContractError::InvalidMintPrice {});
            }

            _mint_type = MintType::Whitelist;
            mint_price = check_wl.mint_price.unwrap();
        } else {
            return Err(ContractError::BeforeStartTime {});
        }
    } else {
        // if this user has public mints left then we allow them through
        if check_public_mint(deps.as_ref(), env.clone(), &info)? {
            _mint_type = MintType::Public;
        }
    }

    println!("{:?}", 1);

    if _mint_type != MintType::None {
        return _execute_mint(deps, env, info, _mint_type, mint_price, minter_addr);
    }

    Err(ContractError::UnableToMint {})
}

/// method that finalizes the mint and generates the submessages
fn _execute_mint(
    mut deps: DepsMut,
    env: Env,
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

    let mut res = Response::new();

    // TODO: add another element of randomness here?
    let (collection_id, token_index) =
        randomize_and_draw_mint(deps.as_ref(), &env, info.sender, None)?;

    res = res.add_message(process_and_get_mint_msg(
        deps.branch(),
        minter_addr.clone(),
        current_token_supply - 1,
        collection_id,
        None,
        Some(token_index),
    )?);

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

    res = disburse_or_escrow_funds(deps, res, mint_price)?;

    Ok(res)
}

fn process_and_get_mint_msg(
    deps: DepsMut,
    minter_addr: Addr,
    new_current_token_supply: u32,
    collection_id: u64,
    mut token_id: Option<u32>,
    token_index: Option<u32>,
) -> Result<CosmosMsg, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    let mut collection_token_ids: Vec<u32> =
        CW721_SHUFFLED_TOKEN_IDS.load(deps.storage, collection_id)?;

    match token_id {
        Some(_) => {}
        None => {
            token_id = Some(collection_token_ids[token_index.unwrap() as usize]);
        }
    }

    // Create mint msgs
    let coll_info: CollectionInfo = CW721_COLLECTION_INFO.load(deps.storage, collection_id)?;

    let mint_msg: Cw721ExecuteMsg<SharedCollectionInfo, Empty> =
        Cw721ExecuteMsg::Mint(MintMsg::<SharedCollectionInfo> {
            token_id: token_id.unwrap().to_string(),
            owner: minter_addr.into_string(),
            token_uri: Some(format!(
                "{}/{}",
                coll_info.base_token_uri,
                token_id.unwrap()
            )),
            extension: config.extension.clone(),
        });

    let token_address = CW721_ADDRS.load(deps.storage, coll_info.id)?;

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_address.into_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    });

    // if maintainer already cleared out the queue, then this wont be necessary
    if collection_token_ids.contains(&token_id.unwrap()) {
        collection_token_ids.retain(|&x| x != token_id.unwrap());
        CW721_SHUFFLED_TOKEN_IDS.save(deps.storage, collection_id, &collection_token_ids)?;
    }

    CW721_SHUFFLED_TOKEN_IDS.save(deps.storage, collection_id, &collection_token_ids)?;

    let collection_current_token_supply =
        COLLECTION_CURRENT_TOKEN_SUPPLY.load(deps.storage, collection_id)?;
    let new_collection_current_token_supply = collection_current_token_supply - 1;
    COLLECTION_CURRENT_TOKEN_SUPPLY.save(
        deps.storage,
        collection_id,
        &new_collection_current_token_supply,
    )?;

    if new_collection_current_token_supply == 0 {
        config.bundle_completed = true;
        CONFIG.save(deps.storage, &config)?;
    }

    CURRENT_TOKEN_SUPPLY.save(deps.storage, &new_current_token_supply)?;

    Ok(msg)
}

fn execute_mint_bundle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // check token supply
    let current_token_supply = CURRENT_TOKEN_SUPPLY.load(deps.storage)?;

    if current_token_supply == 0 {
        return Err(ContractError::MintCompleted {});
    }

    let config = CONFIG.load(deps.storage)?;

    if !config.bundle_enabled {
        return Err(ContractError::BundleMintDisabled {});
    }

    if config.bundle_completed {
        return Err(ContractError::BundleMintCompleted {});
    } else {
        let collection_supplies: Vec<u32> = COLLECTION_CURRENT_TOKEN_SUPPLY
            .range(deps.storage, None, None, Order::Ascending)
            .map(|item| {
                let (_, supply) = item?;
                Ok(supply)
            })
            .collect::<StdResult<Vec<u32>>>()
            .unwrap();

        for supply in collection_supplies {
            if supply == 0 {
                return Err(ContractError::BundleMintCompleted {});
            }
        }
    }

    let payment = may_pay(&info, &config.mint_denom)?;

    if payment != config.bundle_mint_price {
        return Err(ContractError::IncorrectPaymentAmount {
            token: config.mint_denom,
            amt: config.bundle_mint_price,
        });
    }

    if config
        .end_time
        .unwrap_or_else(|| env.block.time.plus_nanos(1u64))
        <= env.block.time
    {
        return Err(ContractError::CampaignHasEnded {});
    }

    let current_bundle_mint_count =
        (BUNDLE_MINT_TRACKER.may_load(deps.storage, info.sender.clone())?).unwrap_or(0);

    if current_bundle_mint_count >= config.max_per_address_bundle_mint {
        return Err(ContractError::BundleMaxMintReached(
            config.max_per_address_bundle_mint,
        ));
    }

    if config.start_time <= env.block.time {
        return _execute_mint_bundle(deps, env, info, current_token_supply);
    }

    Err(ContractError::UnableToMint {})
}

/// method that finalizes the mint and generates the submessages
fn _execute_mint_bundle(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut current_token_supply: u32,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // address - address
    // value - collection_id
    let collections: Vec<AddressValMsg> = CW721_ADDRS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (coll_id, addr) = item?;
            Ok(AddressValMsg {
                address: addr.into_string(),
                value: coll_id as u32,
            })
        })
        .collect::<StdResult<Vec<AddressValMsg>>>()
        .unwrap();

    let mut res: Response = Response::new();

    for collection in collections {
        current_token_supply -= 1;
        println!("collection.value {:?}", collection.value);

        let collection_current_token_supply =
            COLLECTION_CURRENT_TOKEN_SUPPLY.load(deps.storage, collection.value as u64)?;

        let token_index: u32 = randomize_and_draw_index(
            &env,
            info.sender.clone(),
            collection.value as u64, // collection's id
            collection_current_token_supply,
        )?;

        res = res.add_message(process_and_get_mint_msg(
            deps.branch(),
            info.sender.clone(),
            current_token_supply,
            collection.value as u64,
            None,
            Some(token_index),
        )?);
    }

    let current_bundle_mint_count =
        (BUNDLE_MINT_TRACKER.may_load(deps.storage, info.sender.clone())?).unwrap_or(0);

    BUNDLE_MINT_TRACKER.save(deps.storage, info.sender, &(current_bundle_mint_count + 1))?;

    res = disburse_or_escrow_funds(deps, res, config.bundle_mint_price)?;

    Ok(res)
}

fn disburse_or_escrow_funds(
    mut deps: DepsMut,
    mut res: Response,
    mint_price: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

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

                res = _disburse_or_escrow_funds(
                    deps.branch(),
                    res,
                    config.escrow_funds,
                    royalty.addr.clone(),
                    amt,
                    config.mint_denom.clone(),
                )?;
            }
        }

        if remaining_mint_amount > Uint128::zero() {
            res = _disburse_or_escrow_funds(
                deps.branch(),
                res,
                config.escrow_funds,
                primary_royalty_addr.unwrap(),
                remaining_mint_amount,
                config.mint_denom,
            )?;
        }
    }

    Ok(res)
}

fn _disburse_or_escrow_funds(
    deps: DepsMut,
    mut res: Response,
    escrow_funds: bool,
    royalty_addr: Addr,
    amount: Uint128,
    mint_denom: String,
) -> Result<Response, ContractError> {
    if escrow_funds {
        let balance = (BANK_BALANCES.may_load(deps.storage, royalty_addr.clone())?)
            .unwrap_or(Uint128::zero());

        BANK_BALANCES.save(deps.storage, royalty_addr, &(balance + amount))?;
    } else {
        let msg = BankMsg::Send {
            to_address: royalty_addr.to_string(),
            amount: vec![Coin {
                amount,
                denom: mint_denom,
            }],
        };

        res = res.add_message(msg);
    }

    Ok(res)
}

fn execute_airdrop_token_distribution(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    minter_address: Option<String>,
) -> Result<Response, ContractError> {
    // default to self if no address passed in
    let minter_addr: Addr =
        (maybe_addr(deps.api, minter_address)?).unwrap_or_else(|| info.sender.clone());

    let config = CONFIG.load(deps.storage)?;
    let mut current_token_supply = CURRENT_TOKEN_SUPPLY.load(deps.storage)?;
    let airdropper_addr = AIRDROPPER_ADDR.load(deps.storage)?;

    let mut res: Response = Response::new();

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
        for token in check_airdropper_mint_res.remaining_token_ids {
            current_token_supply -= 1;
            res = res.add_message(process_and_get_mint_msg(
                deps.branch(),
                minter_addr.clone(),
                current_token_supply,
                token.collection_id,
                Some(token.token_id),
                None,
            )?);

            let update_msg = AD_MarkTokenIDClaimed(AD_AddressTokenMsg {
                address: minter_addr.to_string(),
                token: AD_TokenMsg {
                    collection_id: token.collection_id,
                    token_id: token.token_id,
                },
            });

            res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: airdropper_addr.clone().into_string(),
                msg: to_binary(&update_msg)?,
                funds: vec![],
            }));
        }

        Ok(res)
    } else {
        Err(ContractError::NoPromisedMints {})
    }
}

fn execute_clean_claimed_tokens_from_shuffle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info)?;

    if let Some(addr) = AIRDROPPER_ADDR.may_load(deps.storage)? {
        let assigned_token_ids: Vec<AD_TokenMsg> = deps.querier.query_wasm_smart(
            addr,
            &AirdropperQueryMsg::GetAssignedTokenIDs {
                start_after: None,
                limit: None,
            },
        )?;

        for msg in assigned_token_ids {
            let mut ids: Vec<u32> =
                CW721_SHUFFLED_TOKEN_IDS.load(deps.storage, msg.collection_id)?;

            ids.retain(|&x| x != msg.token_id);
            CW721_SHUFFLED_TOKEN_IDS.save(deps.storage, msg.collection_id, &ids)?;

            let collection_current_token_supply =
                COLLECTION_CURRENT_TOKEN_SUPPLY.load(deps.storage, msg.collection_id)?;
            COLLECTION_CURRENT_TOKEN_SUPPLY.save(
                deps.storage,
                msg.collection_id,
                &(collection_current_token_supply - 1),
            )?;
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
    let mut res: Response = Response::new();

    // if not admin or maintainer, a fee is needed to execute this function
    if config.admin != info.sender && config.maintainer_addr != Some(info.sender.clone()) {
        // check payment
        let payment = must_pay(&info, &config.bonded_denom)?;

        if payment != Uint128::from(DEFAULT_FEE_AMOUNT) {
            return Err(ContractError::InvalidFeeAmount {
                denom: config.bonded_denom,
                fee: DEFAULT_FEE_AMOUNT,
                operation: "shuffle_token_order".to_string(),
            });
        }

        let fee_collection_addr = FEE_COLLECTION_ADDR.load(deps.storage)?;

        let msg = BankMsg::Send {
            to_address: fee_collection_addr.into_string(),
            amount: vec![coin(DEFAULT_FEE_AMOUNT, config.bonded_denom)],
        };

        res = res.add_message(msg);
    }

    // address - address
    // value - collection_id
    let collections: Vec<AddressValMsg> = CW721_ADDRS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (coll_id, addr) = item?;
            Ok(AddressValMsg {
                address: addr.into_string(),
                value: coll_id as u32,
            })
        })
        .collect::<StdResult<Vec<AddressValMsg>>>()
        .unwrap();

    for collection in collections {
        let collection_id: u64 = collection.value as u64;
        let collection_token_ids: Vec<u32> =
            CW721_SHUFFLED_TOKEN_IDS.load(deps.storage, collection_id)?;

        let shuffled_token_ids = shuffle_token_ids(
            &env,
            info.sender.clone(),
            collection_token_ids.clone(),
            collection.value as u64,
        )?;

        CW721_SHUFFLED_TOKEN_IDS.save(deps.storage, collection_id, &shuffled_token_ids)?;
    }

    Ok(res
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

    let mut remaining_balance: Uint128 = (deps
        .querier
        .query_balance(&env.contract.address, config.mint_denom.clone())?)
    .amount;

    let balances: Vec<AddrBal> = BANK_BALANCES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (addr, balance) = item?;
            Ok(AddrBal { addr, balance })
        })
        .collect::<StdResult<Vec<AddrBal>>>()
        .unwrap();

    //let mut remaining_balance: Uint128 = contract_balance.amount;
    let mut msgs: Vec<BankMsg> = vec![];

    for addr_bal in balances {
        if addr_bal.balance > Uint128::zero() && remaining_balance >= addr_bal.balance {
            msgs.push(BankMsg::Send {
                to_address: addr_bal.addr.to_string(),
                amount: vec![Coin {
                    amount: addr_bal.balance,
                    denom: config.mint_denom.clone(),
                }],
            });

            remaining_balance -= addr_bal.balance;
            BANK_BALANCES.save(deps.storage, addr_bal.addr, &Uint128::zero())?;
        }
    }
    Ok(Response::default()
        .add_attribute("method", "disburse_funds")
        .add_messages(msgs))
}

fn execute_process_custom_bundle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    price: Uint128,
    tokens: Option<Vec<TokenMsg>>,
) -> Result<Response, ContractError> {

    Ok(Response::new())
}

fn execute_mint_custom_bundle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // check token supply
    let current_token_supply = CURRENT_TOKEN_SUPPLY.load(deps.storage)?;

    if current_token_supply == 0 {
        return Err(ContractError::MintCompleted {});
    }

    let config = CONFIG.load(deps.storage)?;

    if config.start_time <= env.block.time {
        return Err(ContractError::BeforeStartTime {});
    }

    if !config.custom_bundle_enabled {
        return Err(ContractError::BundleMintDisabled {});
    }

    if config.custom_bundle_completed {
        return Err(ContractError::BundleMintCompleted {});
    }

    let payment = may_pay(&info, &config.mint_denom)?;

    if payment != config.custom_bundle_mint_price {
        return Err(ContractError::IncorrectPaymentAmount {
            token: config.mint_denom,
            amt: config.custom_bundle_mint_price,
        });
    }

    if config
        .end_time
        .unwrap_or_else(|| env.block.time.plus_nanos(1u64))
        <= env.block.time
    {
        return Err(ContractError::CampaignHasEnded {});
    }

    let current_custom_bundle_mint_count =
        (CUSTOM_BUNDLE_MINT_TRACKER.may_load(deps.storage, info.sender.clone())?).unwrap_or(0);

    if current_custom_bundle_mint_count >= config.max_per_address_bundle_mint {
        return Err(ContractError::BundleMaxMintReached(
            config.max_per_address_bundle_mint,
        ));
    }

    _execute_custom_mint_bundle(deps, env, info)
}

fn _execute_custom_mint_bundle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let custom_bundle_tokens: Vec<(u64, u32)> = CUSTOM_BUNDLE_TOKENS.load(deps.storage)?;

    // no tokens left so we exit
    if (custom_bundle_tokens.len() as u32) < config.custom_bundle_content_count {
        return Err(ContractError::BundleMintCompleted {})
    }

    let mut tokens_mapped: BTreeMap<(u64, u32), bool> = BTreeMap::new();

    for token in custom_bundle_tokens {
        tokens_mapped.insert(token, true);
    }

    for draw in 1..config.custom_bundle_content_count {

    }

    Ok(Response::new())
}

// #region helper functions

struct ValidateCollectionInfoResponse {
    pub collection_infos: Vec<CollectionInfo>,
    pub total_token_supply: u32,
}

fn validate_collection_info(
    _deps: Deps,
    msgs: Vec<CollectionInfoMsg>,
) -> Result<ValidateCollectionInfoResponse, ContractError> {
    let mut collection_infos: Vec<CollectionInfo> = vec![];

    let mut total_token_supply: u32 = 0;
    let mut id: u64 = INSTANTIATE_TOKEN_REPLY_ID;
    for msg in msgs {
        id += 1;
        validate_uri(msg.base_token_uri.clone())?;

        let secondary_metadata_uri: Option<String> = match msg.secondary_metadata_uri {
            Some(uri) => {
                validate_uri(uri.clone())?;
                Some(uri)
            }
            None => None,
        };

        total_token_supply += msg.token_supply;

        collection_infos.push(CollectionInfo {
            id,
            token_supply: msg.token_supply,
            name: msg.name,
            symbol: msg.symbol,
            base_token_uri: msg.base_token_uri,
            secondary_metadata_uri,
        })
    }

    if !(1..=MAX_TOKEN_SUPPLY).contains(&total_token_supply) {
        return Err(ContractError::InvalidMaxTokenSupply {
            max: MAX_TOKEN_SUPPLY,
            input: total_token_supply,
        });
    }

    Ok(ValidateCollectionInfoResponse {
        collection_infos,
        total_token_supply,
    })
}

fn validate_shared_collection_info(
    deps: Deps,
    msg: SharedCollectionInfoMsg,
) -> Result<SharedCollectionInfo, ContractError> {
    let shared_collection_info: SharedCollectionInfo = SharedCollectionInfo {
        mint_revenue_share: validate_royalties(deps, msg.mint_revenue_share, true)?,
        secondary_market_royalties: validate_royalties(
            deps,
            msg.secondary_market_royalties,
            false,
        )?,
    };

    Ok(shared_collection_info)
}

fn validate_uri(uri: String) -> Result<String, ContractError> {
    // url is too short
    if uri.len() < 4 {
        return Err(ContractError::InvalidBaseTokenURI {});
    }

    // url crate was causing wasm

    Ok(uri)
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
        if running_bps > MAX_BPS_FOR_SECONDARY {
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

/// base shuffle logic drawn from stargaze's minter
fn shuffle_token_ids(
    env: &Env,
    sender: Addr,
    mut tokens: Vec<u32>,
    seeded_randomness: u64,
) -> Result<Vec<u32>, ContractError> {
    let tx_index = if let Some(tx) = &env.transaction {
        tx.index
    } else {
        0
    };

    let sha256 = Sha256::digest(
        format!(
            "{}{}{}{}",
            sender,
            env.block.height + 69 + seeded_randomness,
            tokens.len() + 69 + seeded_randomness as usize,
            tx_index
        )
        .into_bytes(),
    );
    // Cut first 16 bytes from 32 byte value
    let randomness: [u8; 16] = sha256.to_vec()[0..16].try_into().unwrap();
    let mut rng = Xoshiro128PlusPlus::from_seed(randomness);
    let mut shuffler = FisherYates::default();

    shuffler
        .shuffle(&mut tokens, &mut rng)
        .map_err(StdError::generic_err)?;

    Ok(tokens)
}

/// base shuffle logic drawn from stargaze's minter
fn randomize_and_draw_mint(
    deps: Deps,
    env: &Env,
    sender: Addr,
    supply: Option<u32>,
) -> Result<(u64, u32), ContractError> {
    // get collections
    let collection_supplies: Vec<(u64, u32)> = COLLECTION_CURRENT_TOKEN_SUPPLY
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (collection_id, supply) = item?;
            Ok((collection_id, supply))
        })
        .collect::<StdResult<Vec<(u64, u32)>>>()
        .unwrap();

    // filter down to usable collection_ids
    let available_collections_ids: Vec<u64> = collection_supplies
        .into_iter()
        .filter(|&(_, v)| v > 0)
        .map(|item| Ok(item.0))
        .collect::<StdResult<Vec<u64>>>()
        .unwrap();

    // grab a collection id
    let collection_index_draw: u32 = randomize_and_draw_index(
        env,
        sender.clone(),
        69u64,
        available_collections_ids.len() as u32,
    )?;

    let collection_id: u64 = available_collections_ids[collection_index_draw as usize];

    // retrieve supply of collection
    let collection_current_token_supply: u32 =
        supply.unwrap_or(COLLECTION_CURRENT_TOKEN_SUPPLY.load(deps.storage, collection_id)?);

    // grab a collection id
    let index: u32 =
        randomize_and_draw_index(env, sender, collection_id, collection_current_token_supply)?;

    Ok((collection_id, index))
}

/// base shuffle logic drawn from stargaze's minter
fn randomize_and_draw_index(
    env: &Env,
    sender: Addr,
    collection_id: u64,
    limit: u32,
) -> Result<u32, ContractError> {
    let tx_index = if let Some(tx) = &env.transaction {
        tx.index
    } else {
        0
    };

    let sha256 = Sha256::digest(
        format!(
            "{}{}{}{}",
            sender,
            env.block.height + collection_id,
            limit + collection_id as u32,
            tx_index
        )
        .into_bytes(),
    );

    // Cut first 16 bytes from 32 byte value
    let randomness: [u8; 16] = sha256.to_vec()[0..16].try_into().unwrap();
    let mut rng = Xoshiro128PlusPlus::from_seed(randomness);

    let r = rng.next_u32();

    let mut rem = 50;
    if rem > limit {
        rem = limit;
    }
    let n = r % rem;

    // pull either front or go near back of vec
    let mut index: u32 = match r % 2 {
        1 => n,
        _ => limit - n,
    };

    // push index_id down to a 0 based index for the array
    // bound should be 0..(vec_length - 1)
    if index >= limit {
        index = limit - 1;
    } else if index > 0 {
        index -= 1;
    }

    Ok(index)
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
                        mint_params.remaining_token_ids = promised_tokens
                            .address_promised_token_ids
                            .into_iter()
                            .map(|item| {
                                Ok(TokenMsg {
                                    collection_id: item.collection_id,
                                    token_id: item.token_id,
                                })
                            })
                            .collect::<StdResult<Vec<TokenMsg>>>()
                            .unwrap();
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

    if current_mint_count >= config.max_per_address_mint {
        return Err(ContractError::PublicMaxMintReached(
            config.max_per_address_mint,
        ));
    }

    Ok(can_mint)
}

// #endregion

// #endregion

/// Follows cosmos SDK validation logic. Specifically, the regex
/// string `[a-zA-Z][a-zA-Z0-9/:._-]{2,127}`.
///
/// <https://github.com/cosmos/cosmos-sdk/blob/7728516abfab950dc7a9120caad4870f1f962df5/types/coin.go#L865-L867>
pub fn validate_native_denom(denom: String) -> Result<bool, ContractError> {
    if denom.len() < 3 || denom.len() > 128 {
        return Err(ContractError::NativeDenomLength { len: denom.len() });
    }
    let mut chars = denom.chars();
    // Really this means that a non utf-8 character is in here, but
    // non-ascii is also correct.
    let first = chars.next().ok_or(ContractError::NonAlphabeticAscii)?;
    if !first.is_ascii_alphabetic() {
        return Err(ContractError::NonAlphabeticAscii);
    }

    for c in chars {
        if !(c.is_ascii_alphanumeric() || c == '/' || c == ':' || c == '.' || c == '_' || c == '-')
        {
            return Err(ContractError::InvalidCharacter { c });
        }
    }

    Ok(true)
}

// Reply callback triggered from cw721 contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match parse_reply_instantiate_data(msg.clone()) {
        Ok(res) => {
            let addr = deps.api.addr_validate(&res.contract_address)?;
            match msg.id {
                INSTANTIATE_AIRDROPPER_REPLY_ID => {
                    AIRDROPPER_ADDR.save(deps.storage, &addr)?;
                }
                INSTANTIATE_WHITELIST_REPLY_ID => {
                    WHITELIST_ADDR.save(deps.storage, &addr)?;
                }
                _ => {
                    let cw721_info = CW721_COLLECTION_INFO.may_load(deps.storage, msg.id)?;

                    match cw721_info {
                        Some(info) => {
                            CW721_ADDRS.save(deps.storage, info.id, &addr)?;
                        }
                        None => {
                            return Err(ContractError::InvalidTokenReplyId {});
                        }
                    }
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
