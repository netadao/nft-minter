use cosmwasm_std::{StdError, Uint128};
use cw721_base::ContractError as Cw721ContractError;
use cw_utils::ParseReplyError;
use cw_utils::PaymentError;
use thiserror::Error;
use url::ParseError;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid start_time")]
    InvalidStartTime {},

    #[error("Invalid end_time")]
    InvalidEndTime {},

    #[error("Mint campaign has not yet begun.")]
    BeforeStartTime {},

    #[error("Invalid Payment Type (native denoms only)")]
    InvalidPaymentType {},

    #[error("Cannot provide both address and instantiation info")]
    InvalidSubmoduleInstantiation {},
    #[error("Invalid code_id for submodule")]
    InvalidSubmoduleCodeId {},

    #[error("Mint campaign is currently active.")]
    MintIsActive {},

    #[error("Mint is closed.")]
    CampaignHasEnded {},

    #[error("Unable to mint.")]
    UnableToMint {},

    #[error("InvalidMaxTokenSupply max: {max} input:{input}")]
    InvalidMaxTokenSupply { max: u32, input: u32 },

    #[error("InvalidMaxPerAddressMaxMint max: {max} input:{input}")]
    InvalidMaxPerAddressMint { max: u32, input: u32 },

    #[error("Invalid base token URI. IPFS enforced.")]
    InvalidBaseTokenURI {},

    #[error("Campaign creation fee of {fee} {denom} OR royalties to Neta DAO are required.")]
    InvalidCampaignCreationFee { fee: u128, denom: String },

    #[error("{contract} Instantiate Error: {error}")]
    ContractInstantiateError {
        contract: String,
        error: ParseReplyError,
    },

    #[error("Incorrect Payment Amount for token: {token}. Expecting :{amt}")]
    IncorrectPaymentAmount { token: String, amt: Uint128 },

    #[error("Invalid mint price")]
    InvalidMintPrice {},

    #[error("Invalid Token Reply ID")]
    InvalidTokenReplyId {},

    #[error("Whitelist not yet active")]
    WhitelistNotInProgress {},

    #[error("You were not whitelisted for this mint")]
    NotOnWhitelist {},

    #[error("Max mints reached for whitelist: {0}")]
    WhitelistMaxMintReached(u32),

    #[error("Max mints reached: {0}")]
    PublicMaxMintReached(u32),

    #[error("Max bundles minted: {0}")]
    BundleMaxMintReached(u32),

    #[error("Premint window not yet open")]
    BeforePremintStarttime {},

    #[error("Whitelist Closed")]
    WhitelistClosed {},

    #[error("Premint Closed")]
    AirdropClosed {},

    #[error("Bundle mint has completed. No more bundles available.")]
    BundleMintCompleted {},

    #[error("Bundle Mints Disabled")]
    BundleMintDisabled {},

    #[error("No promised mints found")]
    NoPromisedMints {},

    #[error("All promises fulfilled")]
    AllPromisesFulfilled {},

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    // use error when 100% minted
    #[error("Mint has completed. No more tokens available.")]
    MintCompleted {},

    #[error("Invalid Target Contract Address")]
    InvalidTargetAddress {},

    #[error("Invalid Airdropper Address")]
    InvalidAirdropperAddress {},

    #[error("Invalid Whitelist Address")]
    InvalidWhitelistAddress {},

    #[error("Invalid basis points for {running}. Max of {max}")]
    InvalidBPS { running: u32, max: u32 },

    #[error("Single 'Primary' address required")]
    NoRoyalPrimaryAddress {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

impl From<ParseError> for ContractError {
    fn from(_err: ParseError) -> ContractError {
        ContractError::InvalidBaseTokenURI {}
    }
}

/*
impl From<ParseReplyError> for ContractError {
    fn from(err: ParseReplyError) -> ContractError {
        ContractError::ContractInstantiateError { contract, error: err }
    }
}
*/

impl From<ContractError> for Cw721ContractError {
    fn from(err: ContractError) -> Cw721ContractError {
        match err {
            ContractError::Unauthorized {} => Cw721ContractError::Unauthorized {},
            //ContractError::Claimed {} => Cw721ContractError::Claimed {},
            //ContractError::Expired {} => Cw721ContractError::Expired {},
            _ => unreachable!("cannot convert {:?} to Cw721ContractError", err),
        }
    }
}
