# Pythia
Pythia is a canister that provides the SubPub functionality for the Ethereum family smart contracts.

## Upgrade
```sh
dfx build pythia && gzip -f -1 ./.dfx/local/canisters/pythia/pythia.wasm
dfx canister install --mode upgrade --wasm ./.dfx/local/canisters/pythia/pythia.wasm.gz pythia
```

## Usage
```sh
# add a new supported chain
dfx canister call pythia add_chain '(record {chain_id=11155111:nat; rpc="https://sepolia.infura.io/v3/d20be327500c45819a1a3b850daec0e2"; min_balance=10000000000000000:nat; block_gas_limit=30000000:nat; fee=10000:nat; symbol="SepoliaETH"})'
# add to whitelist
dfx canister call pythia add_to_whitelist '("e86c4a45c1da21f8838a1ea26fc852bd66489ce9")'
# get the PMA
dfx canister call pythia get_pma
# init a new sub
dfx canister call pythia deposit '(11155111:nat, "0xe5a8ecb5bcb2b81b615fb6f0dbdd74f019d6b7c8d46f2a456067f59773f14258", "service.org wants you to sign in with your Ethereum account:
0xE86C4A45C1Da21f8838a1ea26Fc852BD66489ce9
    

URI: https://service.org/login
Version: 1
Chain ID: 11155111
Nonce: 00000000
Issued At: 2023-05-04T18:39:24Z", "fa7b336d271b7ed539b6db3034d57be294ef889b42534fa95689afd0989ab6d27878c837a14ed1b4c3ab6b7052180ce87198934cb7712a81ea413fd8ebb29e8c1c")'
dfx canister call pythia subscribe '(record {chain_id=11155111:nat; pair_id=null; contract_addr="0x5615156085DEC243767B19d9C914d4413b42e2CF"; method_abi="increment_counter()"; frequency=300:nat; is_random=false; gas_limit=50000:nat; msg="service.org wants you to sign in with your Ethereum account:
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