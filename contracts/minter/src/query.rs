#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;

use crate::msg::{AddressValMsg, ConfigResponse, QueryMsg, TokenDataResponse};
use crate::state::{
    ADDRESS_MINT_TRACKER, AIRDROPPER_ADDR, CONFIG, CURRENT_TOKEN_SUPPLY, CW721_ADDR, WHITELIST_ADDR,
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
        /*
        QueryMsg::GetShuffledTokenIds { start_after, limit } => to_binary(
            &query_get_shuffled_token_ids(deps, env, start_after, limit)?,
        ),
        QueryMsg::GetTokenIndices { start_after, limit } => {
            to_binary(&query_get_token_indices(deps, env, start_after, limit)?)
        }
        QueryMsg::GetShuffledTokenPosition { start_after, limit } => to_binary(
            &query_get_shuffled_token_position(deps, env, start_after, limit)?,
        ),
        */
        QueryMsg::GetRemainingTokens {} => query_get_remaining_tokens(deps, env),
    }
}

fn query_config(deps: Deps, _env: Env) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let cw721_addr = CW721_ADDR.may_load(deps.storage)?;
    let airdropper_addr = AIRDROPPER_ADDR.may_load(deps.storage)?;
    let whitelist_addr = WHITELIST_ADDR.may_load(deps.storage)?;

    Ok(ConfigResponse {
        admin: config.admin,
        maintainer_addr: config.maintainer_addr,
        start_time: config.start_time,
        end_time: config.end_time,
        max_token_supply: config.max_token_supply,
        max_per_address_mint: config.max_per_address_mint,
        mint_price: config.mint_price,
        mint_denom: config.mint_denom,
        base_token_uri: config.base_token_uri,
        name: config.name,
        symbol: config.symbol,
        token_code_id: config.token_code_id,
        cw721_addr,
        airdropper_addr,
        whitelist_addr,
        extension: config.extension,
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
        max_token_supply: config.max_token_supply,
        remaining_token_supply,
    })
}
/*
fn query_get_token_indices(
    deps: Deps,
    _env: Env,
    start_after: Option<u32>,
    limit: Option<u32>,
) -> StdResult<Vec<(u32, u32)>> {
    let start = start_after.map(Bound::exclusive);

    let limit = limit.unwrap_or(100).min(100) as usize;

    let tokens = TOKEN_ID_POSITIONS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (token_id, position) = item?;
            Ok((token_id, position))
        })
        .collect::<StdResult<Vec<_>>>();

    Ok(tokens.unwrap())
}

fn query_get_shuffled_token_position(
    deps: Deps,
    _env: Env,
    start_after: Option<u32>,
    limit: Option<u32>,
) -> StdResult<Vec<(u32, u32)>> {
    let start = start_after.map(Bound::exclusive);

    let limit = limit.unwrap_or(100).min(100) as usize;

    let tokens = SHUFFLED_TOKEN_IDS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (token_id, position) = item?;
            Ok((token_id, position))
        })
        .collect::<StdResult<Vec<_>>>();

    Ok(tokens.unwrap())
}

fn query_get_shuffled_token_ids(
    deps: Deps,
    _env: Env,
    start_after: Option<u32>,
    limit: Option<u32>,
) -> StdResult<Vec<u32>> {
    let start = start_after.map(Bound::exclusive);

    let limit = limit.unwrap_or(100).min(100) as usize;

    let tokens = SHUFFLED_TOKEN_IDS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.unwrap().1)
        .collect::<Vec<u32>>();

    Ok(tokens)
}
*/
