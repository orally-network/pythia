all: local_deploy_pythia

local_deploy_siwe: 
	dfx deploy siwe_signer
	dfx deploy siwe_signer_mock

local_deploy_pythia:  local_deploy_siwe
ifndef SYBIL_CANISTER
	$(error SYBIL_CANISTER ENV is undefined)
endif

	dfx canister create pythia && dfx build pythia && gzip -f -1 ./.dfx/local/canisters/pythia/pythia.wasm
	dfx canister install --wasm ./.dfx/local/canisters/pythia/pythia.wasm.gz --argument \
		"(3000000:nat, \"dfx_test_key\",  principal \"${SYBIL_CANISTER}\")" pythia


local_upgrade: local_upgrade_pythia 

local_upgrade_pythia: 
	dfx build pythia 
	gzip -f -1 ./.dfx/local/canisters/pythia/pythia.wasm
	dfx canister install --mode upgrade --wasm ./.dfx/local/canisters/pythia/pythia.wasm.gz pythia



ic_upgrade: ic_upgrade_pythia


ic_upgrade_pythia:
	dfx build pythia --network ic && gzip -f -1 ./.dfx/ic/canisters/pythia/pythia.wasm
	dfx canister install --mode upgrade --wasm ./.dfx/ic/canisters/pythia/pythia.wasm.gz --network ic pythia