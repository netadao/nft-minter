use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid Start Time chosen")]
    InvalidStartTime {},

    #[error("TokenID: {0}-{1} already assigned")]
    TokenIDAlreadyAssigned(u64, u32),

    #[error("User was not promised this token id")]
    InvalidUserNotPromisedToken {},

    #[error("Token ID chosen is greater than max: {max} input:{input}")]
    InvalidTokenIDAssignment { max: u32, input: u32 },

    #[error("TokenID: {0} already claimed by address:{1}")]
    TokenIDAlreadyClaimed(u64, u32, String),

    #[error("User has already reached their max mints: {0}")]
    ReachedMaxMints(u32),

    #[error("Invalid End Time.")]
    InvalidEndTime {},

    #[error("InvalidMaxTokenSupply max: {max} input:{input}")]
    InvalidMaxTokenSupply { max: u32, input: u32 },

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
