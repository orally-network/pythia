# Pythia
Pythia is a canister that provides the SubPub functionality for the Ethereum family smart contracts.

## Deploy
```sh
dfx deploy siwe_signer_mock --specified-id bkyz2-fmaaa-aaaaa-qaaaq-cai # or siwe_signer, if it's a production
dfx canister create pythia_backend
dfx build pythia_backend
gzip -f -1 ./target/wasm32-unknown-unknown/release/pythia_backend.wasm
dfx canister install -m reinstall --wasm target/wasm32-unknown-unknown/release/pythia_backend.wasm.gz --argument '(0:nat, "dfx_test_key", principal "bkyz2-fmaaa-aaaaa-qaaaq-cai")' pythia_backend
```

## Usage
```sh
# init the canister controllers in the canister storage
dfx canister call pythia_backend get_controllers
# add a new supported chain
dfx canister call pythia_backend add_chain '(11155111:nat, "http://some.rpc.net", 10000000000000000:nat)'
# add a new user
dfx canister call pythia_backend add_user '("service.org wants you to sign in with your Ethereum account:
0xE86C4A45C1Da21f8838a1ea26Fc852BD66489ce9


URI: https://service.org/login
Version: 1
Chain ID: 11155111
Nonce: 00000000
Issued At: 2023-05-04T18:39:24Z", "fa7b336d271b7ed539b6db3034d57be294ef889b42534fa95689afd0989ab6d27878c837a14ed1b4c3ab6b7052180ce87198934cb7712a81ea413fd8ebb29e8c1c")'
# init a new sub
dfx canister call pythia_backend subscribe '(11155111:nat, "a59BCe2A90e8Ee71bE0EfdA6Ee361B3f308aE50A", "{ \"inputs\": [ { \"internalType\": \"string\", \"name\": \"_price\", \"type\": \"string\" } ], \"name\": \"set_price\", \"outputs\": [], \"stateMutability\": \"nonpayable\", \"type\": \"function\" }", 60:nat, "service.org wants you to sign in with your Ethereum account:
0xE86C4A45C1Da21f8838a1ea26Fc852BD66489ce9


URI: https://service.org/login
Version: 1
Chain ID: 11155111
Nonce: 00000000
Issued At: 2023-05-04T18:39:24Z", "fa7b336d271b7ed539b6db3034d57be294ef889b42534fa95689afd0989ab6d27878c837a14ed1b4c3ab6b7052180ce87198934cb7712a81ea413fd8ebb29e8c1c")'
# refresh subs
dfx canister call pythia_backend refresh_subs '(11155111:nat, "service.org wants you to sign in with your Ethereum account:
0xE86C4A45C1Da21f8838a1ea26Fc852BD66489ce9


URI: https://service.org/login
Version: 1
Chain ID: 11155111
Nonce: 00000000
Issued At: 2023-05-04T18:39:24Z", "fa7b336d271b7ed539b6db3034d57be294ef889b42534fa95689afd0989ab6d27878c837a14ed1b4c3ab6b7052180ce87198934cb7712a81ea413fd8ebb29e8c1c")'
```