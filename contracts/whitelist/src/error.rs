use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Whitelist in progress")]
    WhitelistInProgress {},

    #[error("Whitelist closed")]
    WhitelistHasEnded {},

    #[error("Invalid Start Time chosen")]
    InvalidStartTime {},

    #[error("Invalid End Time.")]
    InvalidEndTime {},

    #[error("Address is not whitelisted.")]
    InvalidMintAttempt {},

    #[error("Invalid Max Per Address Mint: Max {0}")]
    InvalidMaxPerAddressMint(u32),

    #[error("Invalid Max Whitelist Address Count: Max {0}")]
    InvalidMaxWhitelistAddressCount(u32),

    #[error("Max whitelist slots reached: {0}")]
    MaxWhitelistSlots(u32),

    #[error("Max mint reached for address: max {0}")]
    MaxMintsReached(u32),

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
