use thiserror::Error;

#[derive(Error, Debug)]
pub enum PythiaError {
    #[error("Not a controller")]
    NotAController,
    #[error("Chain already exists")]
    ChainAlreadyExists,
    #[error("Chain does not exist")]
    ChainDoesNotExist,
    #[allow(dead_code)]
    #[error("Not implemented")]
    NotImplemented,
    #[error("Failed to get eth address: {0}")]
    FailedToGetEthAddress(String),
    #[error("User not found")]
    UserNotFound,
    #[error("Not enoght funds on an execution addr")]
    InsufficientBalance,
    #[error("Invalid an abi function")]
    InvalidABIFunction(String),
    #[error("Tx failed")]
    TxFailed,
    #[error("Tx reached timeout")]
    TxTimeout,
}