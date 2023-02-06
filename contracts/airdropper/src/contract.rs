#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, Addr, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use cw_utils::maybe_addr;

use crate::error::ContractError;
use crate::msg::{AddressTokenMsg, AddressValMsg, ExecuteMsg, InstantiateMsg, TokenMsg};
use crate::state::{
    Config, ADDRESS_CLAIMED_PROMISED_MINTS, ADDRESS_CLAIMED_TOKEN_IDS, ADDRESS_PROMISED_MINTS,
    ADDRESS_PROMISED_TOKEN_IDS, ASSIGNED_TOKEN_IDS, CLAIMED_TOKEN_IDS, CONFIG,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:airdropper";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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

    if msg.end_time.is_some()
        && (msg.end_time.unwrap() <= env.block.time || msg.end_time.unwrap() <= msg.start_time)
    {
        return Err(ContractError::InvalidEndTime {});
    }

    let config = Config {
        admin: info.sender.clone(),
        maintainer_addr: maybe_addr(deps.api, msg.maintainer_address)?,
        start_time: msg.start_time,
        end_time: msg.end_time,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
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
        ExecuteMsg::AddPromisedTokenIDs(msg) => {
            execute_add_promised_token_ids(deps, env, info, msg)
        }
        ExecuteMsg::RemovePromisedTokenIDs(ids) => {
            execute_remove_promised_token_ids(deps, env, info, ids)
        }
        ExecuteMsg::RemovePromisedTokensByAddress(addresses) => {
            execute_remove_promised_token_ids_by_address(deps, env, info, addresses)
        }
        ExecuteMsg::AddPromisedMints(addresses_msg) => {
            execute_add_promised_mints(deps, env, info, addresses_msg)
        }
        ExecuteMsg::RemovePromisedMints(addresses) => {
            execute_remove_promised_mints(deps, env, info, addresses)
        }
        ExecuteMsg::MarkTokenIDClaimed(msg) => execute_mark_token_id_claimed(deps, env, info, msg),
        ExecuteMsg::IncrementAddressClaimedPromisedMintCount(address) => {
            execute_increment_address_promised_mint_count(deps, env, info, address)
        }
    }
}

fn execute_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info)?;

    let mut config = CONFIG.load(deps.storage)?;

    // update maintainer address
    let maintainer_addr = maybe_addr(deps.api, msg.maintainer_address)?;
    if maintainer_addr != config.maintainer_addr {
        if config.admin != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        config.maintainer_addr = maintainer_addr;
    }

    // validate start_time. end_time should have been validated if it exists,
    // if not, we can use block time
    if msg.start_time != config.start_time {
        if msg.start_time <= config.end_time.unwrap_or(env.block.time) {
            return Err(ContractError::InvalidStartTime {});
        }

        config.start_time = msg.start_time;
    }

    if msg.end_time != config.end_time {
        // validate end time. None is an acceptable value
        if msg.end_time.is_some() && msg.end_time <= Some(config.start_time) {
            return Err(ContractError::InvalidEndTime {});
        }

        config.end_time = msg.end_time;
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
    check_can_update(deps.as_ref(), &env, &info)?;

    let mut config = CONFIG.load(deps.storage)?;

    config.maintainer_addr = maybe_addr(deps.api, address)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "update_minter_address")
        .add_attribute("sender", info.sender))
}

/// value in `AddressTokenMsg` used here represents the `token_id`
fn execute_add_promised_token_ids(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address_tokens: Vec<AddressTokenMsg>,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info)?;

    // iterate through each { address, value} we have
    // in this case value is the token_id and NOT mint count
    // airdropper is unaware of valid collection_ids, so FE/claim
    // contract will fail if an invalid collection_id is called
    // todo: optimize for large number of records
    for address_token in address_tokens.into_iter() {
        let token: (u64, u32) = (
            address_token.token.clone().collection_id,
            address_token.token.token_id,
        );
        // check if token has already been assigned
        if ASSIGNED_TOKEN_IDS.has(deps.storage, token) {
            return Err(ContractError::TokenIDAlreadyAssigned(token.0, token.1));
        }

        let mut address_assigned_token_ids: Vec<(u64, u32)> = vec![];

        let addr: Addr = deps.api.addr_validate(&address_token.address)?;

        // grab the address' promised token ids
        if ADDRESS_PROMISED_TOKEN_IDS.has(deps.storage, addr.clone()) {
            address_assigned_token_ids = (ADDRESS_PROMISED_TOKEN_IDS
                .may_load(deps.storage, addr.clone())?)
            .unwrap_or_default();
        }

        // add the assigned value to the vec and store
        address_assigned_token_ids.push(token);
        ASSIGNED_TOKEN_IDS.save(deps.storage, token, &addr.clone())?;
        ADDRESS_PROMISED_TOKEN_IDS.save(deps.storage, addr, &address_assigned_token_ids)?;
    }

    Ok(Response::new()
        .add_attribute("method", "add_promised_token_id")
        .add_attribute("sender", info.sender))
}

/// value used in `AddressTokenMsg` represents the `token_id`
fn execute_remove_promised_token_ids(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    ids: Vec<TokenMsg>,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info)?;

    for id in ids.into_iter() {
        // if token_id has been assigned then we'll remove it from an address'
        // vec of assigned tokens and then remove it from the assigned tracker
        // if empty, we'll remove the address from `ADDRESS_PROMISED_TOKEN_IDS`
        // entirely
        if ASSIGNED_TOKEN_IDS.has(deps.storage, (id.collection_id, id.token_id)) {
            let addr: Addr =
                ASSIGNED_TOKEN_IDS.load(deps.storage, (id.collection_id, id.token_id))?;

            // todo: optimize for large number of records
            let mut address_assigned_token_ids = (ADDRESS_PROMISED_TOKEN_IDS
                .may_load(deps.storage, addr.clone())?)
            .unwrap_or_default();

            address_assigned_token_ids.retain(|&val| val != (id.collection_id, id.token_id));

            // if the vec has no items left, remove it
            if address_assigned_token_ids.is_empty() {
                ADDRESS_PROMISED_TOKEN_IDS.remove(deps.storage, addr);
            } else {
                ADDRESS_PROMISED_TOKEN_IDS.save(deps.storage, addr, &address_assigned_token_ids)?;
            }
            ASSIGNED_TOKEN_IDS.remove(deps.storage, (id.collection_id, id.token_id));
        }
    }

    Ok(Response::new()
        .add_attribute("method", "remove_promised_token_id")
        .add_attribute("sender", info.sender))
}

fn execute_remove_promised_token_ids_by_address(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    addresses: Vec<String>,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info)?;

    // iterate through each address sent in, then iterate through its
    // promised token_ids. remove them from `ASSIGNED_TOKEN_IDS`.
    // finish by removing the address from `ADDRESS_PROMISED_TOKEN_IDS`
    for address in addresses.into_iter() {
        let addr: Addr = deps.api.addr_validate(&address)?;

        if ADDRESS_PROMISED_TOKEN_IDS.has(deps.storage, addr.clone()) {
            let address_assigned_token_ids = (ADDRESS_PROMISED_TOKEN_IDS
                .may_load(deps.storage, addr.clone())?)
            .unwrap_or_default();

            for token_id in address_assigned_token_ids {
                ASSIGNED_TOKEN_IDS.remove(deps.storage, token_id);
            }

            ADDRESS_PROMISED_TOKEN_IDS.remove(deps.storage, addr);
        }
    }

    Ok(Response::new()
        .add_attribute("method", "remove_promised_token_id")
        .add_attribute("sender", info.sender))
}

/// `AddressValMsg` value used here represents total promised(fee free) mint `count` promised to an address
fn execute_add_promised_mints(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address_vals: Vec<AddressValMsg>,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info)?;

    for address_val in address_vals.into_iter() {
        let addr: Addr = deps.api.addr_validate(&address_val.address)?;
        ADDRESS_PROMISED_MINTS.save(deps.storage, addr, &address_val.value)?;
    }

    Ok(Response::new()
        .add_attribute("method", "add_promised_mints")
        .add_attribute("sender", info.sender))
}

fn execute_remove_promised_mints(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    addresses: Vec<String>,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info)?;

    for address in addresses.into_iter() {
        let addr: Addr = deps.api.addr_validate(&address)?;
        ADDRESS_PROMISED_MINTS.remove(deps.storage, addr);
    }

    Ok(Response::new()
        .add_attribute("method", "remove_promised_mints")
        .add_attribute("sender", info.sender))
}

// `AddressTokenMsg` token used here represents the `token_id`
fn execute_mark_token_id_claimed(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address_token_msg: AddressTokenMsg,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info)?;

    let token_id: (u64, u32) = (
        address_token_msg.token.collection_id,
        address_token_msg.token.token_id,
    );

    // this should probably never happen
    if CLAIMED_TOKEN_IDS.has(deps.storage, token_id) {
        let addr: Addr = CLAIMED_TOKEN_IDS.load(deps.storage, token_id)?;
        return Err(ContractError::TokenIDAlreadyClaimed(
            token_id.0,
            token_id.1,
            addr.to_string(),
        ));
    }

    let addr: Addr = deps.api.addr_validate(&address_token_msg.address)?;

    let mut address_promised_token_ids =
        (ADDRESS_PROMISED_TOKEN_IDS.may_load(deps.storage, addr.clone())?).unwrap_or_default();

    if !address_promised_token_ids.contains(&token_id.clone()) {
        return Err(ContractError::InvalidUserNotPromisedToken {});
    }

    address_promised_token_ids.retain(|&val| val != token_id);

    let mut address_claimed_token_ids =
        (ADDRESS_CLAIMED_TOKEN_IDS.may_load(deps.storage, addr.clone())?).unwrap_or_default();

    address_claimed_token_ids.push(token_id);

    CLAIMED_TOKEN_IDS.save(deps.storage, token_id, &addr)?;
    ADDRESS_PROMISED_TOKEN_IDS.save(deps.storage, addr.clone(), &address_promised_token_ids)?;
    ADDRESS_CLAIMED_TOKEN_IDS.save(deps.storage, addr, &address_claimed_token_ids)?;

    Ok(Response::new()
        .add_attribute("method", "mark_token_id_claimed")
        .add_attribute("sender", info.sender))
}

fn execute_increment_address_promised_mint_count(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    check_can_update(deps.as_ref(), &env, &info)?;

    let addr: Addr = deps.api.addr_validate(&address)?;

    let promised_mint_count =
        (ADDRESS_PROMISED_MINTS.may_load(deps.storage, addr.clone())?).unwrap_or(0);

    let current_mint_count =
        (ADDRESS_CLAIMED_PROMISED_MINTS.may_load(deps.storage, addr.clone())?).unwrap_or(0);

    // check promised mint vs address' current mint count
    if current_mint_count >= promised_mint_count {
        return Err(ContractError::ReachedMaxMints(promised_mint_count));
    }

    ADDRESS_CLAIMED_PROMISED_MINTS.save(deps.storage, addr, &(current_mint_count + 1))?;

    Ok(Response::new()
        .add_attribute("method", "increment_address_promised_mint_count")
        .add_attribute("sender", info.sender))
}

/// check_can_update checks if the user attempting to execute is an
/// admin or the maintainer of the contract
fn check_can_update(deps: Deps, _env: &Env, info: &MessageInfo) -> Result<bool, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // EITHER admin, minting contract, or maintainer can update
    if config.admin == info.sender.clone() || config.maintainer_addr == Some(info.sender.clone()) {
        return Ok(true);
    }

    Err(ContractError::Unauthorized {})
}
