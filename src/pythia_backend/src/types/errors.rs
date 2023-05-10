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
    #[error("failed to get eth address: {0}")]
    FailedToGetEthAddress(String),
    #[error("user not found")]
    UserNotFound,
    #[error("not enoght funds on an execution addr")]
    InsufficientBalance,
    #[error("invalid a abi function")]
    InvalidABIFunction(String),
    #[allow(dead_code)]
    #[error("tx failed")]
    TxFailed,
    #[error("tx reached timeout")]
    TxTimeout,
}
