# Basic NFT

A minimal mintable non-fungible token contract with ownership queries, transfer operations, enumeration, and approval patterns.

## Features

- Mint new tokens as contract admin
- Transfer tokens between owners
- Query `owner_of` and `balance_of`
- Enumerate all tokens and owner-specific token lists
- Approve a single spender for a token
- Approve an operator for all owned tokens
- `transfer_from` with approval and operator checks

## Usage

- `initialize(admin, name, symbol)`
- `mint(admin, to, token_id)`
- `owner_of(token_id)`
- `balance_of(owner)`
- `token_by_index(index)`
- `tokens_of_owner(owner)`
- `transfer(from, to, token_id)`
- `approve(owner, approved, token_id)`
- `set_approval_for_all(owner, operator, approved)`
- `transfer_from(spender, from, to, token_id)`

## Running tests

```bash
cargo test -p basic-nft
```
