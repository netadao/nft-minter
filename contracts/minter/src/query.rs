#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;

use crate::msg::{AddrBal, AddressValMsg, ConfigResponse, QueryMsg, TokenDataResponse};
use crate::state::{
    CollectionInfo, ADDRESS_MINT_TRACKER, AIRDROPPER_ADDR, BANK_BALANCES, BUNDLE_MINT_TRACKER,
    COLLECTION_CURRENT_TOKEN_SUPPLY, CONFIG, CURRENT_TOKEN_SUPPLY, CW721_ADDRS,
    CW721_COLLECTION_INFO, WHITELIST_ADDR,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps, env)?),
        QueryMsg::CheckAddressMints { minter_address } => {
            query_check_address_mints(deps, minter_address)
        }
        QueryMsg::GetAddressMints { start_after, limit } => {
            query_get_address_mints(deps, env, start_after, limit)
        }
        QueryMsg::GetEscrowBalances { start_after, limit } => {
            query_get_escrow_balances(deps, env, start_after, limit)
        }
        QueryMsg::GetCw721CollectionInfo { start_after, limit } => to_binary(
            &query_get_cw721_collection_info(deps, env, start_after, limit)?,
        ),
        QueryMsg::GetBundleMintTracker { start_after, limit } => to_binary(
            &query_get_bundle_mint_tracker(deps, env, start_after, limit)?,
        ),
        QueryMsg::GetCollectionCurrentTokenSupply { start_after, limit } => to_binary(
            &query_get_collection_current_supply(deps, env, start_after, limit)?,
        ),

        QueryMsg::GetRemainingTokens {} => query_get_remaining_tokens(deps, env),
        QueryMsg::GetCW721Addrs {} => query_get_cw721_addrs(deps, env),
    }
}

fn query_config(deps: Deps, _env: Env) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let airdropper_addr = AIRDROPPER_ADDR.may_load(deps.storage)?;
    let whitelist_addr = WHITELIST_ADDR.may_load(deps.storage)?;

    Ok(ConfigResponse {
        admin: config.admin,
        maintainer_addr: config.maintainer_addr,
        start_time: config.start_time,
        end_time: config.end_time,
        total_token_supply: config.total_token_supply,
        max_per_address_mint: config.max_per_address_mint,
        max_per_address_bundle_mint: config.max_per_address_bundle_mint,
        mint_price: config.mint_price,
        bundle_mint_price: config.bundle_mint_price,
        mint_denom: config.mint_denom,
        token_code_id: config.token_code_id,
        airdropper_addr,
        whitelist_addr,
        extension: config.extension,
        bundle_enabled: config.bundle_enabled,
        bundle_completed: config.bundle_completed,
    })
}

fn query_check_address_mints(deps: Deps, minter_address: String) -> StdResult<Binary> {
    let minter_addr: Addr = deps.api.addr_validate(&minter_address)?;

    let tokens = (ADDRESS_MINT_TRACKER.may_load(deps.storage, minter_addr)?).unwrap_or(0);

    to_binary(&AddressValMsg {
        address: minter_address,
        value: tokens,
    })
}

fn query_get_address_mints(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let start_after = maybe_addr(deps.api, start_after)?;
    let start = start_after.map(Bound::<Addr>::exclusive);

    let limit = limit.unwrap_or(100).min(100) as usize;

    let address_mints = ADDRESS_MINT_TRACKER
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (address, value) = item?;
            Ok(AddressValMsg {
                address: address.into_string(),
                value,
            })
        })
        .collect::<StdResult<Vec<AddressValMsg>>>();

    to_binary(&address_mints.unwrap())
}

fn query_get_remaining_tokens(deps: Deps, _env: Env) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    let remaining_token_supply = CURRENT_TOKEN_SUPPLY.load(deps.storage)?;

    to_binary(&TokenDataResponse {
        total_token_supply: config.total_token_supply,
        remaining_token_supply,
    })
}

fn query_get_escrow_balances(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let start_after = maybe_addr(deps.api, start_after)?;
    let start = start_after.map(Bound::<Addr>::exclusive);

    let limit = limit.unwrap_or(100).min(100) as usize;

    let balances = BANK_BALANCES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (addr, balance) = item?;
            Ok(AddrBal { addr, balance })
        })
        .collect::<StdResult<Vec<AddrBal>>>();

    to_binary(&balances.unwrap())
}

fn query_get_cw721_addrs(deps: Deps, _env: Env) -> StdResult<Binary> {
    let addrs = CW721_ADDRS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (coll_id, addr) = item?;
            Ok(AddressValMsg {
                address: addr.into_string(),
                value: coll_id as u32,
            })
        })
        .collect::<StdResult<Vec<AddressValMsg>>>();

    to_binary(&addrs.unwrap())
}

fn query_get_cw721_collection_info(
    deps: Deps,
    _env: Env,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<(u64, CollectionInfo)>> {
    let start = start_after.map(Bound::exclusive);

    let limit = limit.unwrap_or(100).min(100) as usize;

    let tokens = CW721_COLLECTION_INFO
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (token_id, position) = item?;
            Ok((token_id, position))
        })
        .collect::<StdResult<Vec<_>>>();

    Ok(tokens.unwrap())
}

fn query_get_bundle_mint_tracker(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<(Addr, u32)>> {
    let start_after = maybe_addr(deps.api, start_after)?;
    let start = start_after.map(Bound::<Addr>::exclusive);

    let limit = limit.unwrap_or(100).min(100) as usize;

    let tokens = BUNDLE_MINT_TRACKER
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (token_id, position) = item?;
            Ok((token_id, position))
        })
        .collect::<StdResult<Vec<_>>>();

    Ok(tokens.unwrap())
}

fn query_get_collection_current_supply(
    deps: Deps,
    _env: Env,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<(u64, u32)>> {
    let start = start_after.map(Bound::exclusive);

    let limit = limit.unwrap_or(100).min(100) as usize;

    let tokens = COLLECTION_CURRENT_TOKEN_SUPPLY
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (token_id, position) = item?;
            Ok((token_id, position))
        })
        .collect::<StdResult<Vec<_>>>();

    Ok(tokens.unwrap())
}
