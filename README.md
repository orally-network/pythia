# Pythia

Pythia is a canister that provides the SubPub functionality for the Ethereum family smart contracts.

## Deploy local
```sh
make SYBIL_CANISTER={SYBIL CANISTER ID}
```

## Upgrade local
```sh
make upgrade_local
```

## Upgrade production/staging

```sh
dfx build pythia --network ic && gzip -f -1 ./.dfx/ic/canisters/pythia/pythia.wasm
dfx canister install --wasm ./.dfx/ic/canisters/pythia/pythia.wasm.gz --argument "(30000000:nat, \"key_1\", principal \"vk6h6-zyaaa-aaaak-qceta-cai\", principal \"tysiw-qaaaa-aaaak-qcikq-cai\")" --network ic pythia
dfx canister install --wasm ./.dfx/ic/canisters/pythia/pythia.wasm.gz --network ic pythia -m upgrade
```

## Enviroment

```sh
CHAIN_ID=5 &&
UPDATE_TIME_FREQUENCY=60 &&
RPC="https://goerli.blockpi.network/v1/rpc/public" &&
MIN_BALANCE=1000000000 &&
BLOCK_GAS_LIMIT=300000000 &&
PLATFORM_FEE=1 &&
CHAIN_SYMBOL="ETH" &&
ADDRESS="0x6696eD42dFBe875E60779b8163fDCc39B088222A" &&
SIWE_MSG="localhost:4361 wants you to sign in with your Ethereum account:
0x6696eD42dFBe875E60779b8163fDCc39B088222A

Sign in with Ethereum.

URI: http://localhost:4361
Version: 1
Chain ID: 11155111
Nonce: 00000000
Issued At: 2023-05-04T18:39:24Z"
SIWE_SIG="fa7b336d271b7ed539b6db3034d57be294ef889b42534fa95689afd0989ab6d27878c837a14ed1b4c3ab6b7052180ce87198934cb7712a81ea413fd8ebb29e8c1c"
CONTRACT_ADDR="5615156085DEC243767B19d9C914d4413b42e2CF"
METHOD_ABI="increment_counter()"
GAS_LIMIT=50000
MUTATION_RATE=1
CONDITION_PRICE_ID="ETH/USD"
MUTATION_TYPE="Both"
MULTICALL_CONTRACT="0x88e33D0d7f9d130c85687FC73655457204E29467"
TX_HASH="{Enter tx hash here, where you sent some tokens to the sybil address}"
```

## Usage

```sh
# update the timer frequency for debug
dfx canister call pythia update_timer_frequency "(${UPDATE_TIME_FREQUENCY}:nat)"
# add a new supported chain
dfx canister call pythia add_chain "(record {chain_id=${CHAIN_ID}:nat; rpc=\"${RPC}\"; min_balance=${MIN_BALANCE}:nat; block_gas_limit=${BLOCK_GAS_LIMIT}:nat; fee=${PLATFORM_FEE}:nat; symbol=\"${CHAIN_SYMBOL}\"; multicall_contract=\"${MULTICALL_CONTRACT}\"})"
# add to whitelist
dfx canister call pythia add_to_whitelist "(\"${ADDRESS}\")"
# get the PMA
dfx canister call pythia get_pma
# deposit a funds to the pma
read -p "Tx hash: " TX_HASH
dfx canister call pythia deposit "(${CHAIN_ID}:nat, \"${TX_HASH}\", \"${SIWE_MSG}\", \"${SIWE_SIG}\")"
# create a subscription with a frequency condition
dfx canister call pythia subscribe "(record {chain_id=${CHAIN_ID}:nat; pair_id=null; contract_addr=\"${CONTRACT_ADDR}\"; method_abi=\"${METHOD_ABI}\"; is_random=false; gas_limit=${GAS_LIMIT}:nat; frequency_condition=opt ${UPDATE_TIME_FREQUENCY}; price_mutation_cond_req=null; msg=\"${SIWE_MSG}\"; sig=\"${SIWE_SIG}\"})"
# create a subscription with a price mutation condition
dfx canister call pythia subscribe "(record {chain_id=${CHAIN_ID}:nat; pair_id=null; contract_addr=\"${CONTRACT_ADDR}\"; method_abi=\"${METHOD_ABI}\"; is_random=false; gas_limit=${GAS_LIMIT}:nat; frequency_condition=null; price_mutation_cond_req=opt record {mutation_rate=${MUTATION_RATE}; pair_id=\"${CONDITION_PRICE_ID}\"; price_mutation_type=variant {${MUTATION_TYPE}}}; msg=\"${SIWE_MSG}\"; sig=\"${SIWE_SIG}\"})"
```
