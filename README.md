# Pythia
Pythia is a canister that provides the SubPub functionality for the Ethereum family smart contracts.

## Deploy
```sh
dfx deploy siwe_signer_mock # or siwe_signer, if it's a production
export SIWE_SIGNER_CANISTER_ID="$(dfx canister id siwe_signer_mock)"
dfx canister create pythia
dfx build pythia
gzip -f -1 ./.dfx/local/canisters/pythia/pythia.wasm
dfx canister install -m upgrade --wasm ./.dfx/local/canisters/pythia/pythia.wasm.gz pythia
```

## Usage
```sh
# add a new supported chain
dfx canister call pythia add_chain '(11155111:nat, "https://sepolia.infura.io/v3/d20be327500c45819a1a3b850daec0e2", 10000000000000000:nat, "0000000000000000000000000000000000000000")'
# get the PMA
dfx canister call pythia get_pma
# init a new sub
dfx canister call pythia deposit '("0x3cea2779207d192355975f87a8dcd6a1fb25ccffb1169979137d886673dd3df0", 11155111:nat, "service.org wants you to sign in with your Ethereum account:
0xE86C4A45C1Da21f8838a1ea26Fc852BD66489ce9
    

URI: https://service.org/login
Version: 1
Chain ID: 11155111
Nonce: 00000000
Issued At: 2023-05-04T18:39:24Z", "fa7b336d271b7ed539b6db3034d57be294ef889b42534fa95689afd0989ab6d27878c837a14ed1b4c3ab6b7052180ce87198934cb7712a81ea413fd8ebb29e8c1c")'
dfx canister call pythia subscribe '(11155111:nat, null, "0x5615156085DEC243767B19d9C914d4413b42e2CF", "increment_counter()", 300:nat, false, 50000:nat, "service.org wants you to sign in with your Ethereum account:
0xE86C4A45C1Da21f8838a1ea26Fc852BD66489ce9
    

URI: https://service.org/login
Version: 1
Chain ID: 11155111
Nonce: 00000000
Issued At: 2023-05-04T18:39:24Z", "fa7b336d271b7ed539b6db3034d57be294ef889b42534fa95689afd0989ab6d27878c837a14ed1b4c3ab6b7052180ce87198934cb7712a81ea413fd8ebb29e8c1c")'
dfx canister call pythia withdraw '(11155111:nat, "service.org wants you to sign in with your Ethereum account:
0xE86C4A45C1Da21f8838a1ea26Fc852BD66489ce9
    

URI: https://service.org/login
Version: 1
Chain ID: 11155111
Nonce: 00000000
Issued At: 2023-05-04T18:39:24Z", "fa7b336d271b7ed539b6db3034d57be294ef889b42534fa95689afd0989ab6d27878c837a14ed1b4c3ab6b7052180ce87198934cb7712a81ea413fd8ebb29e8c1c", "0xE86C4A45C1Da21f8838a1ea26Fc852BD66489ce9")'
```