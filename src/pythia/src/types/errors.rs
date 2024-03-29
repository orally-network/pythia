use thiserror::Error;

use super::methods::ExecutionConditionError;

#[derive(Error, Debug)]
pub enum PythiaError {
    #[error("Not a controller")]
    NotAController,
    #[error("Chain already exists")]
    ChainAlreadyExists,
    #[error("Chain does not exist")]
    ChainDoesNotExist,
    #[error("Chain does not exist in Balances")]
    ChainDoesNotExistInBalances,
    #[error("Chain does not exist in Subscriptions")]
    ChainDoesNotExistInSubscriptions,
    #[error("Chain does not exist in WithdrawalRequests")]
    ChainDoesNotExistInWithdrawalRequests,
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
    #[error("Feed does not exist")]
    FeedDoesNotExist,
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
    #[error("Unable to get a random")]
    UnableToGetRandom,
    #[error("Unable to stop insufficient subscriptions")]
    UnableToStopInsufficientSubscriptions,
    #[error("Unable to execute the multicall")]
    UnableToExecuteMulticall,
    #[error("Unable to form call data")]
    UnableToFormCallData,
    #[error("Unable to decode outputs")]
    UnableToDecodeOutputs,
    #[error("Invalid multicall result")]
    InvalidMulticallResult,
    #[error("Unable to transfer funds")]
    UnableToTransferFunds,
    #[error("Unable to get balance")]
    UnableToGetBalance,
    #[error("Unable to get asset data")]
    UnableToGetAssetData,
    #[error("Unable to reduce balance")]
    UnableToReduceBalance,
    #[error("Chain already initialized in Balances")]
    ChainAlreadyInitializedInBalances,
    #[error("Chain already initialized in Subscriptions")]
    ChainAlreadyInitializedInSubscriptions,
    #[error("Chain already initialized in WithdrawalRequests")]
    ChainAlreadyInitializedInWithdrawalRequests,
    #[error("Unable to activate timer")]
    UnableToActivateTimer,
    #[error("Unable to deactivate timer")]
    UnableToDeactivateTimer,
    #[error("Unable to update timer")]
    UnableToUpdateTimer,
    #[error("Timer is not initialized")]
    TimerIsNotInitialized,
    #[error("Tx timeout")]
    TxTimeout,
    #[error("Unable to get tx receipt")]
    UnableToGetTxReceipt,
    #[error("Subscription frequency is too low")]
    SubscriptionFrequencyIsTooLow,
    #[error("Unable to get input")]
    UnableToGetInput,
    #[error("Unable to encode call")]
    UnableToEncodeCall,
    #[error("Unable to clear balance")]
    UnableToClearBalance,
    #[error("Unable to get Sybil rate")]
    UnableToGetSybilRate,
    #[error("Unsupported asset data type")]
    UnsupportedAssetDataType,
    #[error("Execution condition error: {0}")]
    ExecutionConditionError(#[from] ExecutionConditionError),
    #[error("Unable to estimate gas")]
    UnableToEstimateGas,
    #[error("Sign error: {0}")]
    SignError(String),
}
