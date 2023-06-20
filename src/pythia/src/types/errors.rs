use thiserror::Error;

#[derive(Error, Debug)]
pub enum PythiaError {
    #[error("Not a controller")]
    NotAController,
    #[error("Controllers were not initialized")]
    ControllersWereNotInitialized,
    #[error("Chain already exists")]
    ChainAlreadyExists,
    #[error("Chain does not exist")]
    ChainDoesNotExist,
    #[error("Subscribtion does not exist")]
    SubscriptionDoesNotExist,
    #[error("Balance does not exist")]
    BalanceDoesNotExist,
    #[error("Balance already exists")]
    BalanceAlreadyExists,
    #[error("Nonce already used")]
    NonceAlreadyExists,
    #[error("Tx does not exist")]
    TxDoesNotExist,
    #[error("Tx has failed")]
    TxHasFailed,
    #[error("Tx is not executed yet")]
    TxNotExecuted,
    #[error("User is not whitelisted")]
    UserIsNotWhitelisted,
    #[error("Tx without receiver")]
    TxWithoutReceiver,
    #[error("Tx was not sent to the PMA")]
    TxWasNotSentToPma,
    #[error("Unable to recover address")]
    UnableToRecoverAddress,
    #[error("Unable to get PMA address")]
    UnableToGetPmaAddress,
    #[error("Unable to add a new balance")]
    UnableToAddNewBalance,
    #[error("Unable to get tx")]
    UnableToGetTx,
    #[error("Unable to save nonce")]
    UnableToSaveNonce,
    #[error("Unable to inscrease balance")]
    UnableToIncreaseBalance,
    #[error("Unable to get gas price")]
    UnableToGetGasPrice,
    #[error("Unable to get value for withdraw")]
    UnableToGetValueForWithdraw,
    #[error("Unable to add a new chain")]
    UnableToAddNewChain,
    #[error("Unable to remove a chain")]
    UnableToRemoveChain,
    #[error("Invalid chain RPC")]
    InvalidChainRPC,
    #[error("Unable to update a chain")]
    UnableToUpdateChain,
    #[error("Unable to get a chain RPC")]
    UnableToGetChainRPC,
    #[error("Unable to get controllers")]
    UnableToGetControllers,    
    #[error("Pair does not exist")]
    PairDoesNotExist,
    #[error("Invalid ABI function Name")]
    InvalidABIFunctionName,
    #[error("Invalid ABI parameters")]
    InvalidABIParameters,
    #[error("Invalid ABI parameters number")]
    InvalidABIParametersNumber,
    #[error("Invalid ABI parameter types")]
    InvalidABIParameterTypes,
    #[error("Total subscriptions limit reached")]
    TotalSubscriptionsLimitReached,
    #[error("Wallet subscriptions limit reached")]
    WalletSubscriptionsLimitReached,
    #[error("Timer frequency is greater than subscription frequency")]
    TimerFrequencyIsGreaterThanSubscriptionFrequency,
    #[error("Subscribtion frequency is not multiple of timer frequency")]
    TimerFrequencyIsNotDivisibleBySubscriptionFrequency,
    #[error("Invalid subscription frequency")]
    InvalidSubscriptionFrequency,
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Unable to add subscription")]
    UnableToAddSubscription,
    #[error("Unable to stop subscription")]
    UnableToStopSubscription,
    #[error("Unable to start subscription")]
    UnableToStartSubscription,
    #[error("Invalid address format")]
    InvalidAddressFormat,
    #[error("Unable to update subscription")]
    UnableToUpdateSubscription,
    #[error("Unable to stop subscriptions")]
    UnableToStopSubscriptions,
    #[error("Unable to remove subscriptions")]
    UnableToRemoveSubscriptions,
    #[error("Invalid contract ABI")]
    InvalidContractABI,
    #[error("Unable to get the PMA")]
    UnableToGetPMA,
    #[error("Unable to sign contract call")]
    UnableToSignContractCall,
    #[error("Unable to execute a raw tx")]
    UnableToExecuteRawTx,
    #[error("Waiting for a successful tx execution failed")]
    WaitingForSuccessConfirmationFailed,
    #[error("Unable to add a withdraw request")]
    UnableToAddWithdrawRequest,
}
