# Custom Token (SEP-41 + Multi-Sig)

A fully SEP-41 compliant custom token that demonstrates integrating multi-sig access control and pause mechanisms on top of the standard token interface.

## Features

- **SEP-41 Standard**: `initialize`, `transfer`, `transfer_from`, `approve`, `balance`, `allowance`, `total_supply`, `name`, `symbol`, `decimals`, `admin`, `mint`, `burn`
- **Multi-Sig Transfers**: Treasury-style transfers requiring M-of-N signer approval via `multi_sig_transfer`
- **Pausable**: Emergency pause/unpause by admin
- **Event Emission**: Transfer, approval, and multi-sig events for off-chain indexing

## Architecture

The contract builds on the SEP-41 token base (see `01-sep41-token`) and layers on multi-sig access control and pausing:

- **Factory Pattern**: Inspired by `examples/intermediate/ajo-factory/` — the token is deployable via a factory contract with pre-configured parameters.
- **Multi-Sig Access Control**: Inspired by `examples/intermediate/multi-sig-patterns/` — treasury-level `multi_sig_transfer` requires M-of-N authorized signers to each call `require_auth()`.

## Key Functions

| Function | Description |
| --- | --- |
| `initialize(admin, name, symbol, decimals, supply, threshold, signers)` | Initialize with admin, metadata, initial supply, and multi-sig configuration |
| `transfer(from, to, amount)` | Standard SEP-41 transfer |
| `approve(owner, spender, amount)` | Standard SEP-41 approve |
| `transfer_from(spender, owner, to, amount)` | Standard SEP-41 transfer_from |
| `mint(admin, to, amount)` | Admin-only minting |
| `burn(owner, amount)` | Burn tokens from caller's balance |
| `multi_sig_transfer(signers, to, amount)` | Treasury transfer requiring M-of-N signer auth |
| `set_pause(admin, paused)` | Admin-only pause toggle |
| `update_signers(admin, threshold, signers)` | Update multi-sig configuration |

## Usage

```bash
# Build
cargo build --target wasm32-unknown-unknown --release -p custom-token

# Test
cargo test -p custom-token
```

## Tests

20+ tests covering:
- Initialization with various parameters
- Standard token operations (transfer, approve, transfer_from)
- Edge cases (insufficient balance, invalid amounts, double init)
- Pause lifecycle (pause, reject, unpause, operate)
- Multi-sig flows (valid threshold, insufficient approvers, unauthorized signer)
- Signer updates
- Event emission verification
