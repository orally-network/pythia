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
Chain ID: 324
Nonce: NUY87tYWuZwkxrTZM
Issued At: 2023-11-03T11:40:39.690Z" &&
SIWE_SIG="31f8f8ea2104062e242dc13b9729c75b866e1ab1635c69404a1e7438221ff23849ea6a82e2544d28b4a16075f27fd3db6569e8664191af501572ad342e616c0300" &&
CONTRACT_ADDR="0x8540Bca176E8566e3F26B2c23A542934d26DAc29" &&
METHOD_ABI="increment_counter()" &&
GAS_LIMIT=1000000 &&
MULTICALL_CONTRACT="0xfEBe4A1F840D5cf52184D4062cE46DE5A948E70f" &&
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
