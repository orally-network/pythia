# Pythia
Pythia is a canister that provides the SubPub functionality for the Ethereum family smart contracts.

## Deploy
```sh
dfx deploy siwe_signer
dfx canister create pythia_backend
dfx build pythia_backend
gzip -f -1 ./target/wasm32-unknown-unknown/release/pythia_backend.wasm
dfx canister install -m reinstall --wasm target/wasm32-unknown-unknown/release/pythia_backend.wasm.gz --argument '(0:nat, "dfx_test_key", principal "bkyz2-fmaaa-aaaaa-qaaaq-cai")' pythia_backend
```