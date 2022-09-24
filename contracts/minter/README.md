# Overview

The core contract for this NFT minting repo.

Key features include the following:
- Separate Airdropper module
- Airdrop claim functions
- Separate Whitelist module
- Single `pseudorandom` mint
- Shuffled collection with separate shuffle functionality
- Admin/maintainer are allowed to 'push' mints to addresses that were promised them

Caveats:
- Only native and ibc/ denoms accepted in this contract

## Workflow

Prior to any of the claims/mint windows opening, an admin or maintainer will be able to modify the contract and approve addresses for promises and whitelists.

Ideal window goes:

1. Airdrop opens
    - Optional close period for promised mints
2. Whitelist opens
    - Optional whitelist closes any time period before public mint opens
3. Public mint opens
    - Optional public mint ends
    - Once public mint begins, it is a hard stop on Whitelist mints