#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, Addr, Order};
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};
use cw_storage_plus::Bound;

use crate::msg::{CheckWhitelistResponse, ConfigResponse, QueryMsg};
use crate::state::{ADDRESS_MINT_TRACKER, CONFIG, WHITELIST, WHITELIST_ADDRESS_COUNT};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps, env)?),
        QueryMsg::CheckWhitelist { minter_address } => {
            to_binary(&query_check_whitelist(deps, env, minter_address)?)
        }
        QueryMsg::GetWhitelistAddresses { start_after, limit } => to_binary(
            &query_get_whitelist_addresses(deps, env, start_after, limit)?,
        ),
        QueryMsg::GetAddressMints { start_after, limit } => to_binary(
            &query_get_address_mint_tracker(deps, env, start_after, limit)?,
        ),
    }
}

fn query_config(deps: Deps, env: Env) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let whitelist_address_count = WHITELIST_ADDRESS_COUNT.load(deps.storage)?;

    Ok(ConfigResponse {
        admin: config.admin,
        maintainer_addr: config.maintainer_addr,
        start_time: config.start_time,
        end_time: config.end_time,
        max_whitelist_address_count: config.max_whitelist_address_count,
        max_per_address_mint: config.max_per_address_mint,
        whitelist_in_progress: (config.start_time <= env.block.time
            && env.block.time < config.end_time),
        whitelist_is_closed: config.end_time <= env.block.time,
        mint_price: config.mint_price,
        whitelist_address_count,
    })
}

fn query_check_whitelist(
    deps: Deps,
    env: Env,
    minter_address: String,
) -> StdResult<CheckWhitelistResponse> {
    let minter_addr = deps.api.addr_validate(&minter_address).unwrap();

    let config = CONFIG.load(deps.storage)?;

    let whitelist_is_closed = config.end_time <= env.block.time;
    let whitelist_in_progress =
        config.start_time <= env.block.time && env.block.time < config.end_time;

    let is_on_whitelist = WHITELIST.has(deps.storage, minter_addr.clone());
    let current_mint_count =
        (ADDRESS_MINT_TRACKER.may_load(deps.storage, minter_addr.clone())?).unwrap_or(0);

    Ok(CheckWhitelistResponse {
        minter_addr,
        whitelist_is_closed,
        whitelist_in_progress,
        is_on_whitelist,
        current_mint_count,
        max_per_address_mint: config.max_per_address_mint,
        mint_price: config.mint_price,
    })
}

fn query_get_whitelist_addresses(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<String>> {
    let start_after = start_after
        .map(|addr| deps.api.addr_validate(&addr))
        .transpose()?;
    let start = start_after.map(Bound::<Addr>::exclusive);

    let limit = limit.unwrap_or(100).min(100) as usize;

    let addresses = WHITELIST
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.unwrap().0.to_string())
        .collect::<Vec<String>>();

    Ok(addresses)
}

fn query_get_address_mint_tracker(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<(String, u32)>> {
    let start_after = start_after
        .map(|addr| deps.api.addr_validate(&addr))
        .transpose()?;
    let start = start_after.map(Bound::<Addr>::exclusive);

    let limit = limit.unwrap_or(100).min(100) as usize;

    let addresses = ADDRESS_MINT_TRACKER
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (addr, count) = item?;
            Ok((addr.to_string(), count))
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(addresses)
}
