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

## Usage
```sh
# update the timer frequency for debug
dfx canister call pythia update_timer_frequency '(60:nat)'
# add a new supported chain
dfx canister call pythia add_chain '(record {chain_id=11155111:nat; rpc="https://sepolia.infura.io/v3/d20be327500c45819a1a3b850daec0e2"; min_balance=10000000000000000:nat; block_gas_limit=30000000:nat; fee=10000:nat; symbol="SepoliaETH"})'
# add to whitelist
dfx canister call pythia add_to_whitelist '("e86c4a45c1da21f8838a1ea26fc852bd66489ce9")'
# get the PMA
dfx canister call pythia get_pma
# init a new sub
dfx canister call pythia deposit '(11155111:nat, "0c22e62a46d10b4993929aab41d4bea8e10977e0eb7e3c1c33f7dc83cc8e03c8", "service.org wants you to sign in with your Ethereum account:
0xE86C4A45C1Da21f8838a1ea26Fc852BD66489ce9
    

URI: https://service.org/login
Version: 1
Chain ID: 11155111
Nonce: 00000000
Issued At: 2023-05-04T18:39:24Z", "fa7b336d271b7ed539b6db3034d57be294ef889b42534fa95689afd0989ab6d27878c837a14ed1b4c3ab6b7052180ce87198934cb7712a81ea413fd8ebb29e8c1c")'
dfx canister call pythia subscribe '(record {chain_id=11155111:nat; pair_id=null; contract_addr="0x5615156085DEC243767B19d9C914d4413b42e2CF"; method_abi="increment_counter()"; frequency=60:nat; is_random=false; gas_limit=50000:nat; msg="service.org wants you to sign in with your Ethereum account:
0xE86C4A45C1Da21f8838a1ea26Fc852BD66489ce9
    

URI: https://service.org/login
Version: 1
Chain ID: 11155111
Nonce: 00000000
Issued At: 2023-05-04T18:39:24Z"; sig="fa7b336d271b7ed539b6db3034d57be294ef889b42534fa95689afd0989ab6d27878c837a14ed1b4c3ab6b7052180ce87198934cb7712a81ea413fd8ebb29e8c1c"})'
dfx canister call pythia withdraw '(11155111:nat, "service.org wants you to sign in with your Ethereum account:
0xE86C4A45C1Da21f8838a1ea26Fc852BD66489ce9
    

URI: https://service.org/login
Version: 1
Chain ID: 11155111
Nonce: 00000000
Issued At: 2023-05-04T18:39:24Z", "fa7b336d271b7ed539b6db3034d57be294ef889b42534fa95689afd0989ab6d27878c837a14ed1b4c3ab6b7052180ce87198934cb7712a81ea413fd8ebb29e8c1c", "0xE86C4A45C1Da21f8838a1ea26Fc852BD66489ce9")'
```