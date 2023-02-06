#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use cw_utils::maybe_addr;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{Config, ADDRESS_MINT_TRACKER, CONFIG, WHITELIST, WHITELIST_ADDRESS_COUNT};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:neta-whitelist";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// globals
// TODO: move to a package
const MAX_PER_ADDRESS_MINT: u32 = 100;
const MAX_WHITELIST_ADDRESS_COUNT: u32 = 10000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // validate fields
    if msg.start_time <= env.block.time {
        return Err(ContractError::InvalidStartTime {});
    }

    if msg.end_time <= msg.start_time {
        return Err(ContractError::InvalidEndTime {});
    }

    // validate against global max
    if msg.max_per_address_mint > MAX_PER_ADDRESS_MINT {
        return Err(ContractError::InvalidMaxPerAddressMint(
            MAX_PER_ADDRESS_MINT,
        ));
    }

    // validate against global max
    if msg.max_whitelist_address_count > MAX_WHITELIST_ADDRESS_COUNT {
        return Err(ContractError::InvalidMaxWhitelistAddressCount(
            MAX_WHITELIST_ADDRESS_COUNT,
        ));
    }

    let config = Config {
        admin: info.sender.clone(),
        maintainer_addr: maybe_addr(deps.api, msg.maintainer_address)?,
        start_time: msg.start_time,
        end_time: msg.end_time,
        max_whitelist_address_count: msg.max_whitelist_address_count,
        max_per_address_mint: msg.max_per_address_mint,
        mint_price: msg.mint_price,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig(msg) => execute_update_config(deps, env, info, msg),
        ExecuteMsg::UpdateMaintainerAddress(address) => {
            execute_update_maintainer_address(deps, env, info, address)
        }
        ExecuteMsg::AddToWhitelist(addresses) => {
            execute_add_to_whitelist(deps, env, info, addresses)
        }
        ExecuteMsg::RemoveFromWhitelist(addresses) => {
            execute_remove_from_whitelist(deps, env, info, addresses)
        }
        ExecuteMsg::UpdateAddressMintTracker(minter_address) => {
            execute_update_address_mint_tracker(deps, env, info, &minter_address)
        }
    }
}

fn execute_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info, false)?;
    let mut config = CONFIG.load(deps.storage)?;

    // update maintainer address
    let maintainer_addr = maybe_addr(deps.api, msg.maintainer_address)?;
    if maintainer_addr != config.maintainer_addr {
        if config.admin != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        config.maintainer_addr = maintainer_addr;
    }

    // update start time
    if msg.start_time != config.start_time {
        if msg.start_time >= config.end_time {
            return Err(ContractError::InvalidStartTime {});
        }

        if msg.start_time <= env.block.time {
            return Err(ContractError::InvalidStartTime {});
        }

        config.start_time = msg.start_time;
    }

    // update end time
    if msg.end_time != config.end_time {
        if config.start_time >= msg.end_time {
            return Err(ContractError::InvalidEndTime {});
        }

        if msg.end_time <= env.block.time {
            return Err(ContractError::InvalidEndTime {});
        }

        config.end_time = msg.end_time;
    }

    if msg.max_whitelist_address_count != config.max_whitelist_address_count {
        if msg.max_whitelist_address_count > MAX_WHITELIST_ADDRESS_COUNT {
            return Err(ContractError::InvalidMaxWhitelistAddressCount(
                MAX_WHITELIST_ADDRESS_COUNT,
            ));
        }

        config.max_whitelist_address_count = msg.max_whitelist_address_count;
    }

    if msg.max_per_address_mint != config.max_per_address_mint {
        if msg.max_per_address_mint > MAX_PER_ADDRESS_MINT {
            return Err(ContractError::InvalidMaxPerAddressMint(
                MAX_PER_ADDRESS_MINT,
            ));
        }

        config.max_per_address_mint = msg.max_per_address_mint;
    }

    if msg.mint_price != config.mint_price {
        config.mint_price = msg.mint_price;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "update_config")
        .add_attribute("sender", info.sender))
}

fn execute_update_maintainer_address(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: Option<String>,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info, true)?;

    let mut config = CONFIG.load(deps.storage)?;

    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    config.maintainer_addr = maybe_addr(deps.api, address)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "update_minter_address")
        .add_attribute("sender", info.sender))
}

pub fn execute_add_to_whitelist(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut addresses: Vec<String>,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info, true)?;

    let config = CONFIG.load(deps.storage)?;
    let mut whitelist_address_count: u32 =
        (WHITELIST_ADDRESS_COUNT.may_load(deps.storage)?).unwrap_or(0);

    // remove dupes
    addresses.sort_unstable();
    addresses.dedup();

    // this is unsafe as someone can pass in an addresses with len > u32.max
    if (whitelist_address_count + (addresses.len() as u32)) > config.max_whitelist_address_count {
        return Err(ContractError::MaxWhitelistSlots(
            config.max_whitelist_address_count,
        ));
    }

    let mut address_already_in_whitelist_list: Vec<String> = vec![];

    for address in addresses.into_iter() {
        // this shouldnt happen, but just in case..
        if whitelist_address_count >= config.max_whitelist_address_count {
            return Err(ContractError::MaxWhitelistSlots(
                config.max_whitelist_address_count,
            ));
        }

        let addr = deps.api.addr_validate(&address)?;
        if WHITELIST.has(deps.storage, addr.clone()) {
            address_already_in_whitelist_list.push(addr.to_string());
        } else {
            whitelist_address_count += 1;
            WHITELIST.save(deps.storage, addr, &true)?;
            WHITELIST_ADDRESS_COUNT.save(deps.storage, &whitelist_address_count)?;
        }
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "add_to_whitelist")
        .add_attribute("sender", info.sender))
}

pub fn execute_remove_from_whitelist(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    addresses: Vec<String>,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info, true)?;

    let mut whitelist_address_count = WHITELIST_ADDRESS_COUNT.load(deps.storage)?;

    for address in addresses.into_iter() {
        let addr = deps.api.addr_validate(&address)?;

        if WHITELIST.has(deps.storage, addr.clone()) {
            whitelist_address_count -= 1;
            WHITELIST.remove(deps.storage, addr);
            WHITELIST_ADDRESS_COUNT.save(deps.storage, &whitelist_address_count)?;
        }
    }

    Ok(Response::new()
        .add_attribute("method", "remove_from_whitelist")
        .add_attribute("sender", info.sender))
}

pub fn execute_update_address_mint_tracker(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    minter_address: &str,
) -> Result<Response, ContractError> {
    check_can_execute(deps.as_ref(), &info)?;

    let addr = deps.api.addr_validate(minter_address)?;

    if !WHITELIST.has(deps.storage, addr.clone()) {
        return Err(ContractError::InvalidMintAttempt {});
    }

    let current_mint_count =
        (ADDRESS_MINT_TRACKER.may_load(deps.storage, addr.clone())?).unwrap_or(0);

    let config = CONFIG.load(deps.storage)?;

    if current_mint_count == config.max_per_address_mint {
        return Err(ContractError::MaxMintsReached(config.max_per_address_mint));
    }

    let new_mint_count = current_mint_count + 1;

    ADDRESS_MINT_TRACKER.save(deps.storage, addr, &new_mint_count)?;

    Ok(Response::new()
        .add_attribute("method", "update_address_mint_tracker")
        .add_attribute("sender", info.sender))
}

fn check_can_execute(deps: Deps, info: &MessageInfo) -> Result<bool, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // EITHER admin, or maintainer can execute
    if config.admin == info.sender.clone() || config.maintainer_addr == Some(info.sender.clone()) {
        Ok(true)
    } else {
        Err(ContractError::Unauthorized {})
    }
}

fn check_can_update(
    deps: Deps,
    env: &Env,
    info: &MessageInfo,
    allow_update_while_in_progress: bool,
) -> Result<bool, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // EITHER admin, or maintainer can update
    if config.admin == info.sender.clone() || config.maintainer_addr == Some(info.sender.clone()) {
        // wl over
        if config.end_time <= env.block.time {
            return Err(ContractError::WhitelistHasEnded {});
        }

        if !allow_update_while_in_progress
            && config.start_time <= env.block.time
            && env.block.time <= config.end_time
        {
            return Err(ContractError::WhitelistInProgress {});
        }

        Ok(true)
    } else {
        Err(ContractError::Unauthorized {})
    }
}
