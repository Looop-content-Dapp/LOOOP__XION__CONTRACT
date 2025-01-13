use cosmwasm_std::StdError;
use thiserror::Error;


#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    // Authorization errors
    #[error("Unauthorized - only minter can perform this action")]
    Unauthorized {},

    #[error("Invalid token ID format")]
    InvalidTokenId {},

    // custom 

    #[error("{0}")]
    Custom(String),

    // Payment errors
    #[error("Insufficient payment - expected {expected} uxion, got {received} uxion")]
    InsufficientPayment { expected: u128, received: u128 },

    #[error("No payment sent with transaction")]
    NoPayment {},

    // Pass status errors
    #[error("Pass is not expired yet")]
    PassStillValid {},

    #[error("Pass has expired and grace period has ended")]
    PassExpiredAndGracePeriodEnded {},

    #[error("Pass is in grace period and can still be renewed")]
    PassInGracePeriod {},

    #[error("Pass does not exist")]
    PassNotFound {},

    // Supply limit error
    #[error("Maximum token supply reached")]
    MaxSupplyReached {},

    // Base contract operation errors
    #[error("Direct minting not allowed - use MintPass instead")]
    DirectMintNotAllowed {},

    #[error("Pass cannot be transferred - soulbound NFT")]
    NoTransfer {},
}

