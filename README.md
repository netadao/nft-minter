# Overview

Collection of CosmWasm contracts with NFT minting functionality built by [Neta DAO](https://netadao.zone) at the discretion and direction of the [Idols NFT collection](https://beholdidols.zone).  

|Contract|Description|
|-|--|
|minter|The core contract to mint an NFT collection|
|airdropper|An "airdropping" module that allow creators to promise specific token_ids or promise mints|
|whitelist|A module that allows creators to run whitelist minting campaigns where they control access, price, and mint count|

## TODO
- v1: test cases are a mess right now. needs a little love
- v2: modularize logic to separate contracts

## Developer Commands

Run these to lint, format code, build and test.  Crate/binary generation commands will come later

```
cargo build
cargo clippy --all-targets -- --D warnings
cargo fmt
cargo test
```

```
sh ./scripts/schema.sh

cosmwasm-ts-codegen generate \
    --plugin client \
    --schema ./contracts/airdropper/schema \
    --out ./ts \
    --name Airdropper \
    --no-bundle
cosmwasm-ts-codegen generate \
    --plugin client \
    --schema ./contracts/minter/schema \
    --out ./ts \
    --name Minter \
    --no-bundle
cosmwasm-ts-codegen generate \
    --plugin client \
    --schema ./contracts/whitelist/schema \
    --out ./ts \
    --name Whitelist \
    --no-bundle

docker run --rm -v "$(pwd)":/code \
		--mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		--platform linux/amd64 \
		cosmwasm/workspace-optimizer:0.12.10
```

Misc notes:

```
sudo docker cp ./artifacts/minter.wasm juno_node_1:/opt/minter.wasm
sudo docker exec -i juno_node_1 junod tx wasm store /opt/minter.wasm \
    --gas-prices 0.1ujunox --gas auto --gas-adjustment 1.3 \
    -y -b block --chain-id testing \
    --from localnet --output json 
```

### References and credits

Leveraging open source work, some code and inspiration may have come from these repos:

- [CosmWasm cw-nfts](https://github.com/CosmWasm/cw-nfts/tree/main/contracts)
    - This repo contains some of the first NFT minting contracts in CosmWasm
- [DAODAO](https://github.com/DA0-DA0/dao-contracts)
    - A lot of growth, experience and contributions are thanks to the DAODAO team
- [Stargaze Minter](https://github.com/public-awesome/launchpad/blob/c425d5fc45fc44391dc231b31c740f9a53eee2fb/contracts/vending-minter/src/contract.rs#L266)
    - The `shuffle` logic from Stargaze's contracts.

### Disclaimer

NETA DAO TOOLING IS PROVIDED “AS IS”, AT YOUR OWN RISK, AND WITHOUT WARRANTIES OF ANY KIND. No developer or entity involved in creating the NETA DAO UI or smart contracts will be liable for any claims or damages whatsoever associated with your use, inability to use, or your interaction with other users of NETA DAO tooling, including any direct, indirect, incidental, special, exemplary, punitive or consequential damages, or loss of profits, cryptocurrencies, tokens, or anything else of value.
