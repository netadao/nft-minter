#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, Addr, Order};
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;

use crate::msg::{
    AddressPromisedTokensResponse, AddressTokenMsg, AddressValMsg,
    CheckAirdropPromisedMintResponse, CheckAirdropPromisedTokensResponse, QueryMsg, TokenMsg,
};
use crate::state::{
    ADDRESS_CLAIMED_PROMISED_MINTS, ADDRESS_CLAIMED_TOKEN_IDS, ADDRESS_PROMISED_MINTS,
    ADDRESS_PROMISED_TOKEN_IDS, ASSIGNED_TOKEN_IDS, CLAIMED_TOKEN_IDS, CONFIG,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => query_config(deps),
        QueryMsg::GetAddressPromisedTokenIds { start_after, limit } => {
            query_get_address_promised_token_ids(deps, start_after, limit)
        }
        QueryMsg::GetAssignedTokenIds { start_after, limit } => {
            query_get_assigned_token_ids(deps, start_after, limit)
        }
        QueryMsg::GetAssignedTokenIdsWithAddress { start_after, limit } => {
            query_get_assigned_token_ids_with_address(deps, start_after, limit)
        }
        QueryMsg::GetClaimedTokenIds { start_after, limit } => {
            query_get_claimed_token_ids(deps, start_after, limit)
        }
        QueryMsg::GetClaimedTokenIdsWithAddress { start_after, limit } => {
            query_get_claimed_token_ids_with_address(deps, start_after, limit)
        }
        QueryMsg::GetAddressPromisedMints { start_after, limit } => {
            query_get_address_promised_mints(deps, start_after, limit)
        }
        QueryMsg::GetClaimedAddressPromisedMints { start_after, limit } => {
            query_get_claimed_address_promised_mints(deps, start_after, limit)
        }
        QueryMsg::CheckAddressPromisedMints { minter_address } => {
            query_check_address_promised_mints(deps, env, minter_address)
        }
        QueryMsg::CheckAddressPromisedTokens { minter_address } => {
            query_check_address_promised_tokens(deps, env, minter_address)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_binary(&config)
}

fn query_get_address_promised_token_ids(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let start_after = maybe_addr(deps.api, start_after)?;
    let start = start_after.map(Bound::<Addr>::exclusive);

    let limit = limit.unwrap_or(100).min(100) as usize;

    let data: Vec<AddressPromisedTokensResponse> = ADDRESS_PROMISED_TOKEN_IDS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (address, ids) = item?;
            let token_ids: Vec<TokenMsg> = ids
                .into_iter()
                .map(|id| {
                    Ok(TokenMsg {
                        collection_id: id.0,
                        token_id: id.1,
                    })
                })
                .collect::<StdResult<Vec<TokenMsg>>>()?;

            Ok(AddressPromisedTokensResponse {
                address: address.to_string(),
                token_ids,
            })
        })
        .collect::<StdResult<Vec<_>>>()?;

    to_binary(&data)
}

fn query_get_assigned_token_ids(
    deps: Deps,
    start_after: Option<(u64, u32)>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let start = start_after.map(Bound::exclusive);

    let limit = limit.unwrap_or(100).min(100) as usize;

    let token_ids: Vec<TokenMsg> = ASSIGNED_TOKEN_IDS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (token, _) = item?;
            Ok(TokenMsg {
                collection_id: token.0,
                token_id: token.1,
            })
        })
        .collect::<StdResult<Vec<TokenMsg>>>()?;

    to_binary(&token_ids)
}

fn query_get_assigned_token_ids_with_address(
    deps: Deps,
    start_after: Option<(u64, u32)>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let start = start_after.map(Bound::exclusive);

    let limit = limit.unwrap_or(100).min(100) as usize;

    let token_ids: Vec<AddressTokenMsg> = ASSIGNED_TOKEN_IDS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (token_id, address) = item?;
            Ok(AddressTokenMsg {
                address: address.to_string(),
                token: TokenMsg {
                    collection_id: token_id.0,
                    token_id: token_id.1,
                },
            })
        })
        .collect::<StdResult<Vec<_>>>()?;

    to_binary(&token_ids)
}

fn query_get_claimed_token_ids(
    deps: Deps,
    start_after: Option<(u64, u32)>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let start = start_after.map(Bound::exclusive);

    let limit = limit.unwrap_or(100).min(100) as usize;

    let data: Vec<TokenMsg> = CLAIMED_TOKEN_IDS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (token, _) = item?;

            Ok(TokenMsg {
                collection_id: token.0,
                token_id: token.1,
            })
        })
        .collect::<StdResult<Vec<_>>>()?;

    to_binary(&data)
}

fn query_get_claimed_token_ids_with_address(
    deps: Deps,
    start_after: Option<(u64, u32)>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let start = start_after.map(Bound::exclusive);

    let limit = limit.unwrap_or(100).min(100) as usize;

    let data: Vec<AddressTokenMsg> = CLAIMED_TOKEN_IDS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (token_id, address) = item?;
            Ok(AddressTokenMsg {
                address: address.to_string(),
                token: TokenMsg {
                    collection_id: token_id.0,
                    token_id: token_id.1,
                },
            })
        })
        .collect::<StdResult<Vec<_>>>()?;

    to_binary(&data)
}

fn query_get_address_promised_mints(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let start_after = maybe_addr(deps.api, start_after)?;
    let start = start_after.map(Bound::<Addr>::exclusive);

    let limit = limit.unwrap_or(100).min(100) as usize;

    let data = ADDRESS_PROMISED_MINTS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (address, count) = item?;
            Ok(AddressValMsg {
                address: address.to_string(),
                value: count,
            })
        })
        .collect::<StdResult<Vec<_>>>()?;

    to_binary(&data)
}

fn query_get_claimed_address_promised_mints(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let start_after = maybe_addr(deps.api, start_after)?;
    let start = start_after.map(Bound::<Addr>::exclusive);

    let limit = limit.unwrap_or(100).min(100) as usize;

    let data = ADDRESS_CLAIMED_PROMISED_MINTS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (address, count) = item?;
            Ok(AddressValMsg {
                address: address.to_string(),
                value: count,
            })
        })
        .collect::<StdResult<Vec<_>>>()?;

    to_binary(&data)
}

fn query_check_address_promised_mints(
    deps: Deps,
    env: Env,
    minter_address: String,
) -> StdResult<Binary> {
    let minter_addr = deps.api.addr_validate(&minter_address)?;

    let config = CONFIG.load(deps.storage)?;

    let airdrop_mint_is_closed = match config.end_time {
        None => false,
        Some(time) => time < env.block.time,
    };

    let airdrop_mint_in_progress = config.start_time <= env.block.time && !airdrop_mint_is_closed;

    let promised_mint_count =
        (ADDRESS_PROMISED_MINTS.may_load(deps.storage, minter_addr.clone())?).unwrap_or(0);

    let claimed_mint_count =
        (ADDRESS_CLAIMED_PROMISED_MINTS.may_load(deps.storage, minter_addr.clone())?).unwrap_or(0);

    to_binary(&CheckAirdropPromisedMintResponse {
        minter_addr,
        airdrop_mint_is_closed,
        airdrop_mint_in_progress,
        promised_mint_count,
        claimed_mint_count,
    })
}

fn query_check_address_promised_tokens(
    deps: Deps,
    env: Env,
    minter_address: String,
) -> StdResult<Binary> {
    let minter_addr = deps.api.addr_validate(&minter_address)?;

    let config = CONFIG.load(deps.storage)?;

    let airdrop_mint_is_closed = match config.end_time {
        None => false,
        Some(time) => time < env.block.time,
    };

    let airdrop_mint_in_progress = config.start_time <= env.block.time && !airdrop_mint_is_closed;

    let address_promised_token_ids: Vec<TokenMsg> = (ADDRESS_PROMISED_TOKEN_IDS
        .may_load(deps.storage, minter_addr.clone())?)
    .unwrap_or_default()
    .into_iter()
    .map(|id| {
        Ok(TokenMsg {
            collection_id: id.0,
            token_id: id.1,
        })
    })
    .collect::<StdResult<Vec<TokenMsg>>>()?;

    let address_claimed_token_ids: Vec<TokenMsg> = (ADDRESS_CLAIMED_TOKEN_IDS
        .may_load(deps.storage, minter_addr.clone())?)
    .unwrap_or_default()
    .into_iter()
    .map(|id| {
        Ok(TokenMsg {
            collection_id: id.0,
            token_id: id.1,
        })
    })
    .collect::<StdResult<Vec<TokenMsg>>>()?;

    to_binary(&CheckAirdropPromisedTokensResponse {
        minter_addr,
        airdrop_mint_is_closed,
        airdrop_mint_in_progress,
        address_promised_token_ids,
        address_claimed_token_ids,
    })
}
