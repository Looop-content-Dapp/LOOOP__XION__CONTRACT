use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    // Factory Contract Errors
    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid symbol format. Must be uppercase and no spaces")]
    InvalidSymbol {},

    #[error("No data in reply")]
    InvalidSymbolA {},

    #[error("Invalid UTF-8")]
    InvalidSymbolB {},

    #[error("Symbol is already taken")]
    SymbolAlreadyTaken {},

    #[error("Collection not found")]
    CollectionNotFound {},

    #[error("Unknown reply ID: {id}")]
    UnknownReplyId { id: u64 },

    // NFT Contract Errors
    #[error("No uxion payment found")]
    NoPayment {},

    #[error("Invalid payment amount")]
    InvalidPayment {},

    #[error("Token ID already exists")]
    TokenExists {},

    #[error("Token not found")]
    TokenNotFound {},

    #[error("Pass is not expired")]
    PassNotExpired {},

    #[error("Pass has expired")]
    PassExpired {},

    #[error("Pass is in grace period")]
    InGracePeriod {},

    #[error("Pass is soulbound and cannot be transferred")]
    Soulbound {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    // General Errors
    #[error("Invalid address format")]
    InvalidAddress {},

    #[error("Invalid token ID format")]
    InvalidTokenId {},

    // Optional: More specific errors
    #[error("Invalid pass duration")]
    InvalidPassDuration {},

    #[error("Invalid grace period")]
    InvalidGracePeriod {},

    #[error("Maximum supply reached")]
    MaxSupplyReached {},

    #[error("Operation not permitted during grace period")]
    GracePeriodOperation {},

    #[error("Invalid contract instantiation")]
    InvalidInstantiation {},
}