# Overview

Collection of CosmWasm contracts with NFT minting functionality built by [Neta DAO](https://netadao.zone) at the discretion and direction of the [Idols NFT collection](https://beholdidols.zone).  

|Contract|Description|
|-|--|
|minter|The core contract to mint an NFT collection|
|airdropper|An "airdropping" module that allow creators to promise specific token_ids or promise mints|
|whitelist|A module that allows creators to run whitelist minting campaigns where they control access, price, and mint count|

## Developer Commands

Run these to lint, format code, build and test.  Crate/binary generation commands will come later

```
cargo build
cargo clippy --all-targets -- --D warnings
cargo fmt
cargo test
```

```
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

### References and credits

Leveraging open source work, some code and inspiration may have come from these repos:

- [CosmWasm cw-nfts](https://github.com/CosmWasm/cw-nfts/tree/main/contracts)
    - This repo contains some of the first minting contract work in CosmWasm
- [DAODAO Contracts](https://github.com/DA0-DA0/dao-contracts)
    - A lot of growth, experience and contributions are thanks to the DAODAO team
- [Stargaze Minter](https://github.com/public-awesome/launchpad/tree/main/contracts/minter/src)
    - The `shuffle` logic from Stargaze's contracts.

### Disclaimer

NETA DAO TOOLING IS PROVIDED “AS IS”, AT YOUR OWN RISK, AND WITHOUT WARRANTIES OF ANY KIND. No developer or entity involved in creating the NETA DAO UI or smart contracts will be liable for any claims or damages whatsoever associated with your use, inability to use, or your interaction with other users of NETA DAO tooling, including any direct, indirect, incidental, special, exemplary, punitive or consequential damages, or loss of profits, cryptocurrencies, tokens, or anything else of value.