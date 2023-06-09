# Pythia

Pythia is a canister that provides the SubPub functionality for the Ethereum family smart contracts.

## Upgrade local

```sh
dfx build pythia && gzip -f -1 ./.dfx/local/canisters/pythia/pythia.wasm
dfx canister install --mode upgrade --wasm ./.dfx/local/canisters/pythia/pythia.wasm.gz pythia
```

## Upgrade production

```sh
dfx build pythia --network ic && gzip -f -1 ./.dfx/ic/canisters/pythia/pythia.wasm
dfx canister install --network ic --mode upgrade --wasm ./.dfx/ic/canisters/pythia/pythia.wasm.gz pythia
```

## Enviroment

```sh
CHAIN_ID=11155111
UPDATE_TIME_FREQUENCY=60
RPC="https://sepolia.infura.io/v3/d20be327500c45819a1a3b850daec0e2"
MIN_BALANCE=10000000000000000
BLOCK_GAS_LIMIT=30000000
PLATFORM_FEE=1000
CHAIN_SYMBOL="SepoliaETH"
ADDRESS="e86c4a45c1da21f8838a1ea26fc852bd66489ce9"
SIWE_MSG="service.org wants you to sign in with your Ethereum account:
0xE86C4A45C1Da21f8838a1ea26Fc852BD66489ce9


URI: https://service.org/login
Version: 1
Chain ID: 11155111
Nonce: 00000000
Issued At: 2023-05-04T18:39:24Z"
SIWE_SIG="fa7b336d271b7ed539b6db3034d57be294ef889b42534fa95689afd0989ab6d27878c837a14ed1b4c3ab6b7052180ce87198934cb7712a81ea413fd8ebb29e8c1c"
CONTRACT_ADDR="5615156085DEC243767B19d9C914d4413b42e2CF"
METHOD_ABI="increment_counter()"
GAS_LIMIT=50000
```

## Usage

```sh
# update the timer frequency for debug
dfx canister call pythia update_timer_frequency "(${UPDATE_TIME_FREQUENCY}:nat)"
# add a new supported chain
dfx canister call pythia add_chain "(record {chain_id=${CHAIN_ID}:nat; rpc=\"${RPC}\"; min_balance=${MIN_BALANCE}:nat; block_gas_limit=${BLOCK_GAS_LIMIT}:nat; fee=${PLATFORM_FEE}:nat; symbol=\"${CHAIN_SYMBOL}\"})"
# add to whitelist
dfx canister call pythia add_to_whitelist "(\"${ADDRESS}\")"
# get the PMA
dfx canister call pythia get_pma
# deposit a funds to the pma
read -p "Tx hash: " TX_HASH
dfx canister call pythia deposit "(${CHAIN_ID}:nat, \"${TX_HASH}\", \"${SIWE_MSG}\", \"${SIWE_SIG}\")"
# create a subscription
dfx canister call pythia subscribe "(record {chain_id=${CHAIN_ID}:nat; pair_id=null; contract_addr=\"${CONTRACT_ADDR}\"; method_abi=\"${METHOD_ABI}\"; frequency=${UPDATE_TIME_FREQUENCY}:nat; is_random=false; gas_limit=${GAS_LIMIT}:nat; msg=\"${SIWE_MSG}\"; sig=\"${SIWE_SIG}\"})"
```
