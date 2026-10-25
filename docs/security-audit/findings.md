# Soroban Cookbook security audit findings

Read-only audit of the whole repository at `/home/vahalla/Desktop/Soroban-Cookbook-`, dated 2026-10-25. No fixes were applied. The published `docs/security-audit/audit-scope.md` covers only the intermediate examples and the token examples; this pass covered the whole repo.

The evidence is the inventory, the mechanical scan, and the manual reviews of fund-moving contracts, `examples/advanced/`, and the remaining trees. Where two notes describe the same defect, this report keeps one finding. A grouped finding names every path and function the notes confirmed by reading the body. The mechanical list of 87 `initialize` / `init` functions with no auth call in the body was a scan. Entries that no reviewer opened are not findings.

`scripts/malware_scan.py` and `.github/workflows/malware-scan.yml` are untracked and were not executed. Grep of the committed `.github/workflows` found no `curl` or `wget` piped into a shell.

Severity names follow `TRIAGE.md`: Critical, High, Medium, Low. Counts below are counts of findings. One finding may cover many files.

## Summary

| Severity | Findings |
| --- | ---: |
| Critical | 18 |
| High | 43 |
| Medium | 24 |
| Low | 16 |
| Total | 101 |

Formatting-only: 1 Low finding (`F-086`), covering 58 files that `cargo fmt --all -- --check` would rewrite, plus 2 test files rustfmt could not parse.

`F-004` is one Critical finding for every unauthenticated `initialize` / `init` a reviewer read. Some of those functions are also named under a later finding when a second defect has a different impact (a re-init that changes a live rate, an admin drain, a reward paid in the LP token, an immediate timelock call, a factory template swap). The notes rated the non-custody rows inside `F-004` High; they stay in that one finding so the pattern is not split.

## Findings

### F-001 — Critical — Self-transfer overwrites the debit and inflates the balance

**Paths and functions**

| Path | Function | Lines |
| --- | --- | --- |
| `examples/tokens/01-sep41-token/src/lib.rs` | `transfer` | 100–105 |
| `examples/tokens/01-sep41-token/src/lib.rs` | `transfer_from` | 164–169 |
| `examples/tokens/02-sep41-extensions/src/lib.rs` | `transfer` | 296–302 |
| `examples/tokens/02-sep41-extensions/src/lib.rs` | `transfer_from` | 363–370 |
| `examples/tokens/03-pausable-token/src/lib.rs` | `transfer` | 100–110 |
| `examples/tokens/03-pausable-token/src/lib.rs` | `transfer_from` | 160–174 |
| `examples/tokens/03-optimized-operations/src/lib.rs` | `OptimizedTokenOps::transfer` | 262–268 |
| `examples/tokens/04-snapshot-token/src/lib.rs` | `transfer` | 167–177 |
| `examples/tokens/06-reward-token/src/lib.rs` | `transfer` | 205–217 |
| `examples/tokens/06-token-wrapper/src/lib.rs` | `transfer` | 198–207 |
| `examples/tokens/07-token-metadata/src/lib.rs` | `transfer` | 263–272 |
| `examples/tokens/09-optimized-token-ops/src/lib.rs` | `transfer` | 71–80 |
| `examples/tokens/09-optimized-token-ops/src/lib.rs` | `batch_transfer_naive` | 103–113 |
| `examples/tokens/10-custom-token/src/lib.rs` | `transfer` | 115–125 |
| `examples/tokens/10-custom-token/src/lib.rs` | `transfer_from` | 175–189 |
| `examples/tokens/10-custom-token/src/lib.rs` | `multi_sig_transfer` | 361–371 |

**What is wrong.** The debit and the credit use the same storage key. The credit is a sum read before the debit. When `from == to`, the credit write is the one that remains, so the balance grows by `amount` and total supply does not change. In `multi_sig_transfer`, a transfer whose `to` is the contract address inflates the treasury balance the same way.

**Why it matters.** Any holder can mint unbacked balance to themselves. Later transfers and burns move that balance. The `01-sep41-token` README says the example follows SEP-41 and uses safe balance arithmetic. These functions do not.

The same pattern was read and does not apply in `examples/tokens/05-allowance-pattern` `transfer_from` (199–209), `examples/tokens/02-sep41-extensions` `batch_transfer` (226–233), `examples/tokens/03-optimized-operations` `batch_transfer` (218–227), `examples/tokens/09-optimized-token-ops` `batch_transfer_optimized` (135–148), and `templates/token-dapp/contracts/token/src/lib.rs` `transfer` / `transfer_from` (131–139, 200–212). Those paths re-read storage after the debit, or they reject `from == to`.

### F-002 — Critical — Reward claims pay into the balance that earns the reward

**Path.** `examples/tokens/06-reward-token/src/lib.rs`, `claim_rewards` (334–379), fed by `claimable_rewards` (320–329). `deposit_to_pool` (280–307) funds the reserve from the admin balance.

**What is wrong.** `gross` is `balance * rate_per_token / 1_000_000 - claimed`. The payout is added to that same balance, and `claimed` advances only to the previous gross.

**Why it matters.** With `rate_per_token == 1_000_000`, every successful claim pays the holder’s full pre-claim balance again out of `pool.total_deposited`. The holder repeats until the reserve is empty. A smaller rate still over-pays, because the new balance increases the next gross. This directory has no README (KI-5).

### F-003 — Critical — Admin checks compare an address and never call `require_auth`

**Paths and functions**

| Path | Helper | Callers |
| --- | --- | --- |
| `examples/defi/14-automatic-snapshot-triggers/src/lib.rs` | `require_admin` (248–258) | `set_frequency` (65–67), `set_enabled` (76–78), `set_prune_threshold` (87–93), `prune` (166–172) |
| `examples/nfts/05-automatic-snapshot-triggers/src/lib.rs` | `require_admin` (248–258) | same functions, same line numbers |
| `examples/governance/11-automatic-snapshot-triggers/src/lib.rs` | `require_admin` (248–258) | same functions, same line numbers |
| `examples/tokens/10-automatic-snapshot-triggers/src/lib.rs` | `require_admin` (248–258) | `set_frequency` (65–73), `set_enabled` (76–84), `set_prune_threshold` (87–99), `prune` (166–198) |
| `examples/advanced/26-data-aggregation-oracle/src/lib.rs` | `require_admin` (269–278) | `add_source` (93–94), `remove_source` (113), `pause` (244–247), `resume` (250–252) |
| `examples/basics/09-enum-types/src/lib.rs` | `set_user_role` (231–253) reads `get_user_role` (223–227) | the setter itself |

**What is wrong.** Each helper loads the stored admin (or, in the enum example, the stored role of the `admin` argument) and returns when the address matches. None of them call `require_auth`. Passing an `Address` does not prove the caller is that address.

**Why it matters.** On the four snapshot contracts, `initialize` does call `admin.require_auth()` (line 51 on the defi, nft, and governance copies; lines 47–62 on the token copy), so the stored admin can be a real account. Anyone who later passes that address can change frequency, disable snapshots, or delete history through `prune`. On the aggregation oracle, the same helper lets anyone who knows the admin address add or remove sources and pause or resume the feed. Oracle `initialize` (84–90) also stores that admin with no `require_auth` (`F-004`). On the enum example, after initialization anyone can pass the owner address and assign every role except `Owner` (`Owner` is rejected at 244–246). Forged snapshot values are `F-039`.

### F-004 — Critical — `initialize` / `init` grants a privileged role with no `require_auth`

**What is wrong.** Each function below stores an admin, owner, minter, token, implementation, signer set, or initial supply and does not call `require_auth` on that address. A one-time `has` check, where one exists, only blocks a second call. The first caller becomes the privileged address. On the proxy and beacon rows, the admin key is persistent and is never extended, so after TTL expiry `has` is false and `init` can run again.

**Why it matters.** `TRIAGE.md` treats an authentication bypass of an admin or governance role as Critical. The rows the manual notes rated Critical mint a supply, install an upgrade, or hand a custody contract to the first caller. The rows those notes rated High make the first caller the admin of a contract that does not, by itself, mint a supply or replace WASM. Later admin checks on those contracts do call `require_auth` on whatever address was stored. Both sets are listed here so the pattern stays one finding.

Instances the notes rated Critical:

| Path | Function | Lines | What the first caller gets |
| --- | --- | --- | --- |
| `examples/tokens/01-sep41-token/src/lib.rs` | `initialize` | 55–81 | admin role and `initial_supply` |
| `examples/tokens/02-minting-strategies/src/lib.rs` | `initialize` | 54–79 | admin; `mint_with_cap`, `mint_unlimited`, and `mint_scheduled` then mint (88–92, 128–132, 159–163) |
| `examples/tokens/02-sep41-extensions/src/lib.rs` | `initialize` | 125–139 | `initial_supply`; no admin key |
| `examples/tokens/03-pausable-token/src/lib.rs` | `initialize` | 59–86 | admin and `initial_supply` |
| `examples/tokens/03-optimized-operations/src/lib.rs` | `OptimizedTokenOps::initialize` | 150–165 | choice of `underlying` |
| `examples/tokens/04-mint-burn/src/lib.rs` | `initialize` | 42–56 | admin; `supply_cap == 0` means unlimited `mint` (69). `mint` (59–62) authenticates the stored admin |
| `examples/tokens/04-snapshot-token/src/lib.rs` | `initialize` | 81–101 | admin who can mint |
| `examples/tokens/05-allowance-pattern/src/lib.rs` | `initialize` | 114–131 | `initial_supply` credited to `admin` |
| `examples/tokens/05-vesting/src/lib.rs` | `initialize` | 65–71 | admin and token address (`F-013`) |
| `examples/tokens/01-vesting-management/src/lib.rs` | `initialize` | 108–129 | admin and default token (`F-013`) |
| `examples/tokens/06-reward-token/src/lib.rs` | `initialize` | 115–133 | admin who can mint |
| `examples/tokens/06-token-wrapper/src/lib.rs` | `initialize` | 71–81 | choice of the underlying token |
| `examples/tokens/07-token-metadata/src/lib.rs` | `initialize` | 100–130 | admin who can mint (`mint` at 202–205 authenticates the stored admin) |
| `examples/tokens/09-optimized-token-ops/src/lib.rs` | `initialize` | 46–58 | `total_supply` balance |
| `examples/tokens/10-custom-token/src/lib.rs` | `initialize` | 63–101 | admin, supply, and the signer set |
| `examples/defi/01-simple-swap/src/lib.rs` | `initialize` | 80–101 | owner, both tokens, and the rate, with no once-guard (`F-005`) |
| `examples/defi/08-liquidity-mining/src/lib.rs` | `initialize` | 109–116 | admin (`F-007`) |
| `examples/defi/12-farming-pool/src/lib.rs` | `initialize` | 39–47 | admin (`F-006`) |
| `examples/governance/09-timelock-governance/src/lib.rs` | `init` | 54–64 | admin and `min_delay` (`F-008`) |
| `examples/intermediate/ajo-factory/src/lib.rs` | `initialize` | 155–190 | `wasm_hash` (`F-012`) |
| `examples/advanced/02-beacon-proxy/src/beacon.rs` | `init` | 84–101 | admin and implementation; `upgrade` follows that admin |
| `examples/advanced/02-beacon-proxy/src/proxy.rs` | `init` | 51–62 | admin and beacon address |
| `examples/advanced/04-upgradeable-proxy/src/proxy.rs` | `init` | 12–20 | admin and implementation; `upgrade` (23–27) |
| `examples/advanced/07-upgrade-patterns/src/direct_upgrade.rs` | `initialize` | 108–118 | admin; `upgrade` (146–167) calls `update_current_contract_wasm`. `init_guard.rs` `initialize` (149–157) does authenticate |
| `examples/advanced/07-upgrade-patterns/src/versioned_upgrade.rs` | `initialize` | 157–169 | admin; `upgrade` (187–196) installs a WASM hash |
| `examples/advanced/10-contract-migrations/src/lib.rs` | `initialize` | 99–113 | admin; `require_admin` (317–324) later authenticates that address and `upgrade` (308–313) swaps WASM. `add_user` / `credit` can rewrite balances |
| `examples/advanced/14-bridge-validators/src/lib.rs` | `init` | 46–58 | admin and threshold. `add_validator`, `remove_validator`, `slash_validator`, and `set_threshold` trust it. `process_message` (130–168) releases a message once validator power reaches the threshold |
| `examples/advanced/19-diamond-security/src/lib.rs` | `init` | 66–72 | admin. `add_facet`, `remove_facet`, and `upgrade_admin` require that address. `execute` (161–177) routes to the facets |
| `examples/advanced/20-batch-transfer/src/lib.rs` | `initialize` | 130–146 | any positive `initial_supply` credited to the caller’s `admin`. `Unauthorized` exists and is unused here. `batch_transfer` later spends that balance |
| `examples/advanced/23-beacon-proxy-factory/src/beacon.rs` | `init` | 81–104 | admin and implementation. `factory.rs` `init` (91–103) does authenticate before it deploys a beacon; this beacon contract does not |
| `examples/advanced/23-beacon-proxy-factory/src/proxy.rs` | `init` | 44–55 | admin and beacon |
| `examples/advanced/26-data-aggregation-oracle/src/lib.rs` | `initialize` | 84–90 | admin (`F-003`, `F-017`) |
| `examples/advanced/27-gasless-relayer/src/lib.rs` | `initialize` | 72–82 | the same address stored as admin and `TrustedRelayer` (`F-055`) |
| `examples/advanced/28-merkle-airdrop/src/lib.rs` | `initialize` | 75–89 | admin, token, and Merkle `root` (`F-018`) |
| `examples/advanced/30-permit-pattern/src/lib.rs` | `initialize` | 51–61 | `initial_supply` credited to `admin`. `permit` (64–80) does use `require_auth_for_args` |
| `examples/advanced/31-proxy-admin/src/lib.rs` | `initialize` | 128–137 | admin. `execute_upgrade` (195–214) replaces the WASM |
| `examples/advanced/35-gas-optimization/src/lib.rs` | `initialize` | 147–159 | admin used by `batch_mint` (275–288), `batch_burn` (294–305), and pause (`F-054`) |
| `examples/basics/09-enum-types/src/lib.rs` | `initialize` | 192–209 | admin and `UserRole::Owner` (`F-003`) |

These `initialize` functions do call `admin.require_auth()` and are not part of this finding: `examples/tokens/08-multi-token-balance-manager` (76–83), `examples/tokens/10-automatic-snapshot-triggers` (47–51), `examples/tokens/10-pausable-permissions` (125–126), `examples/intermediate/02-role-based-access-control` (51–52), `examples/intermediate/access-control` (124), `examples/basics/03-authentication` (159–163), `examples/basics/06-validation-patterns` (92–102).

Instances the notes rated High. The first caller still becomes the admin, owner, or feed, and later gates trust that stored address:

| Path | Function | Lines | What the first caller gets |
| --- | --- | --- | --- |
| `examples/intermediate/03-pause-unpause/src/lib.rs` | `initialize` | 74–81 | admin. `pause`, `unpause`, and `reset` then call `admin.require_auth()` (87–93, 113–119, 160–161) |
| `examples/intermediate/03-priority-queue/src/lib.rs` | `initialize` | 40–45 | admin. `require_admin` (191–201) authenticates for `remove` / `merge` |
| `examples/intermediate/multi-sig-patterns/src/lib.rs` | `initialize` | 49–64 | signer set used by `create_proposal`, `approve`, and `execute`. After init, `execute` (181–213) flips `executed` and does not require the executor to be a signer. No funds move |
| `examples/intermediate/storage-migration/src/lib.rs` | `initialize` | 56–72 | admin (`F-060`). `admin_require_auth` (266–272) authenticates later calls |
| `examples/intermediate/event-history/src/lib.rs` | `initialize` | 67–82 | admin and `max_entries`. `append_event` loads admin and discards it (`let _ = admin`, 92–93) |
| `examples/intermediate/ajo/src/lib.rs` | `initialize` | 30–46 | creator and contribution amount. The factory can call this in the same transaction. A direct deploy does not. No deposit function in this crate |
| `examples/basics/05-error-handling/src/lib.rs` | `initialize` | 120–128 | admin. `pause` / `unpause` (247–282) authenticate and compare to it. `deposit` (145–176) is an internal ledger |
| `examples/defi/03-lending-pool/src/lib.rs` | `initialize` | 18–46 | `base_rate`, `kink_rate`, and `kink_utilization` (`F-019`, `F-063`) |
| `examples/defi/05-flash-loans/src/lib.rs` | `init` | 30–36 | admin that `set_fee` authorizes (`F-030`) |
| `examples/defi/06-flash-loan-use-cases/src/arbitrage.rs` | `init` | 52–57 | owner. `withdraw` (118–124) sends the contract’s full token balance to `to` (`F-031`) |
| `examples/defi/06-flash-loan-use-cases/src/refinancing.rs` | `init` | 58–63 | owner (`F-031`) |
| `examples/defi/06-flash-loan-use-cases/src/security.rs` | `init` | 37–43 | `Owner` and `Provider`. `set_provider` (46–49) is owner-authed after that |
| `examples/defi/07-staking-pool/src/lib.rs` | `initialize` | 153–181 | owner, staking token, reward token, and reward rate. `require_owner` is commented out (26–35) (`F-024`) |
| `examples/defi/09-vault-strategies/src/lib.rs` | `initialize` | 193–211 | admin (`F-021`) |
| `examples/defi/10-swap-liquidity/src/lib.rs` | `initialize` | 97–115 | owner, both pool tokens, and the LP token. `require_owner` is commented out (37–46) (`F-026`) |
| `examples/defi/02-constant-product-amm/src/lib.rs` | `initialize` | 67–82 | `token_x` and `token_y` (`F-025`) |
| `examples/defi/11-amm-price-oracle/src/lib.rs` | `AmmPoolContract::initialize` | 142–153 | owner and both tokens. `require_owner` is commented out (50–60). No function reads the owner for authorization (`F-027`) |
| `examples/defi/11-amm-price-oracle/src/lib.rs` | `initialize_oracle` | 270–287 | oracle owner (`F-027`) |
| `examples/governance/06-vote-delegation/src/lib.rs` | `init` | 52–58 | admin. `set_balance` (63–97) then lets that admin write any user’s voting balance. `delegate` / `undelegate` do authenticate `from` (103, 174) |
| `examples/advanced/03-oracle-pattern/src/lib.rs` | `initialize` | 89–101 | oracle admin or feed |
| `examples/advanced/04-circuit-breaker/src/lib.rs` | `initialize` | 61–81 | admin of `set_admin` and `configure`. `set_pause` (106–112) authenticates the stored admin (`F-050`) |
| `examples/advanced/05-rate-limiting/src/lib.rs` | `initialize` | 103–116 | admin |
| `examples/advanced/07-trusted-forwarder/src/lib.rs` | `initialize` | 63–74 | admin (`F-056`) |
| `examples/advanced/09-storage-optimization/src/lib.rs` | `initialize` | 49–60 | admin (`F-073`) |
| `examples/advanced/11-version-registry/src/lib.rs` | `initialize` | 46–50 | admin |
| `examples/advanced/12-oracle-consumer/aggregating/src/lib.rs` | `initialize` | 65–88 | feed admin |
| `examples/advanced/12-oracle-consumer/guarded/src/lib.rs` | `initialize` | 73–87 | feed admin |
| `examples/advanced/12-oracle-consumer/settlement/src/lib.rs` | `initialize` | 86–101 | feed admin. `settle` itself authenticates `account` |
| `examples/advanced/15-oracle-integration/src/lib.rs` | `OracleContract::initialize` | 138–153 | oracle admin |
| `examples/advanced/15-oracle-integration/src/lib.rs` | `ConsumerContract::initialize` | 329–333 | consumer admin |
| `examples/advanced/15-reentrancy-guard/src/lib.rs` | `init` | 27–28 | clears `Entered` on every call, including a reentrant one (`F-042`) |
| `examples/advanced/16-cross-contract-integration-testing/src/lib.rs` | `Factory::initialize` | 49–58 | factory admin |
| `examples/advanced/16-hierarchical-access-control/src/lib.rs` | `initialize` | 113–123 | admin |
| `examples/advanced/19-diamond-security/facet-adder/src/lib.rs` | `init_adder` | 28–33 | `FacetKey::Diamond`. Later `add` requires `caller == diamond` and `caller.require_auth()` |
| `examples/advanced/19-diamond-security/facet-multiplier/src/lib.rs` | `init_multiplier` | 28–33 | same pattern for `multiply` |
| `examples/advanced/22-batch-builder/src/lib.rs` | `initialize` | 66–70 | no admin stored (`F-044`) |
| `examples/advanced/32-rbac-modifiers/src/lib.rs` | `initialize` | 111–126 | admin. A comment says bootstrap skips auth. The first caller becomes ADMIN and can `grant_role` |
| `examples/advanced/33-registry-access-controls/src/lib.rs` | `init` | 23–35 | admin |
| `examples/advanced/36-price-oracle/src/lib.rs` | `initialize` | 67–75 | feed admin |
| `examples/advanced/37-beacon-management/src/lib.rs` | `initialize` | 33–44 | admin |
| `examples/advanced/43-payment-channels/src/lib.rs` | `init` | 71–90 | token, both participants, both pubkeys, and expiry, with a once-guard on `TOKEN` and no auth on either participant (`F-040`, `F-041`) |
| `examples/advanced/02-timelock/src/lib.rs` | `initialize` | 80–84 | admin. `execute` on this contract does not invoke a target (`F-087`) |

`examples/tokens/03-optimized-operations/src/lib.rs` `StandardTokenOps::initialize` (50–58) stores `Underlying` and `TotalSupply` with no once-guard and no auth. The notes rated that function Medium because the type is compiled only for non-wasm, tests, or `testutils` (`F-080`). `examples/defi/04-collateralized-lending` `initialize` (18–54) never stores an admin at all; that is `F-014`.

### F-005 — Critical — Simple-swap `initialize` can be called again and `swap` pays at the new rate

**Path.** `examples/defi/01-simple-swap/src/lib.rs`, `initialize` (80–101), abused by `swap` (150–198).

**What is wrong.** `initialize` stores `Owner`, `TokenA`, `TokenB`, `RateNum`, and `RateDen` with no `require_auth` and no already-initialized check. A later call overwrites the rate and the owner.

**Why it matters.** `swap` pulls `sell_amount` from the trader and sends `buy_amount` from the contract at that rate (`token::Client::transfer_from` / `transfer`, quote math at 174–186). One call can set an extreme ratio and drain whichever token the contract holds. The README describes a fixed-rate swap.

### F-006 — Critical — Farming admin can pull every token the contract holds

**Path.** `examples/defi/12-farming-pool/src/lib.rs`, `initialize` (39–47) and `emergency_withdraw_admin` (143–156). `check_admin` is 225–229.

**What is wrong.** `initialize` writes `Admin` if unset and never calls `admin.require_auth()`. `emergency_withdraw_admin` requires that stored admin and then `token::Client::transfer` of any `token` and any `amount` to any `to`.

**Why it matters.** The first caller becomes admin and can pull tokens the contract custodies, including user stakes. The function comment says it transfers tokens out of the contract. Reentrancy on `deposit` / `withdraw`, and `remove_pool` stranding stakes, are `F-028`.

### F-007 — Critical — Liquidity-mining rewards can be paid in the staked LP token

**Path.** `examples/defi/08-liquidity-mining/src/lib.rs`, `initialize` (109–116), `add_pool` (124–154), `harvest` (287–311). `require_admin` (435–441) does call `require_auth` on the stored admin. `stake` transfers LP tokens at 217–218.

**What is wrong.** `initialize` stores `admin` with no `admin.require_auth()` and only a one-time `has` check. `add_pool` lets that admin set `lp_token` and `reward_token` to the same address and `reward_rate` up to `MAX_REWARD_RATE` (1e15). `harvest` pays `total_claimable` of `reward_token` from the contract balance.

**Why it matters.** If the reward token is the LP token, the payout is other users’ stakes. Ordering of `stake` / `unstake` is `F-029`.

### F-008 — Critical — Timelock `emergency_execute` skips the delay

**Path.** `examples/governance/09-timelock-governance/src/lib.rs`, `init` (54–64), `emergency_execute` (157–174). `queue` (73–74), `execute` (104–105), and `cancel` (133–134) trust the same admin.

**What is wrong.** `init` stores `admin` and `min_delay` with no `admin.require_auth()`. `emergency_execute` skips the ETA check (170) and `invoke_contract`s `proposal.target`, `proposal.function`, and `proposal.args` (174).

**Why it matters.** The first caller is an admin who can run an arbitrary call immediately after queueing. This is a different contract from `examples/advanced/02-timelock`, whose `execute` does not invoke a target (`F-087`).

### F-009 — Critical — Vote weight is whatever the voter passes

**Paths and functions**

| Path | Function | Lines |
| --- | --- | --- |
| `examples/governance/04-proposal-lifecycle/src/lib.rs` | `vote` | 221–253 |
| `examples/governance/10-dao-treasury/src/lib.rs` | `vote` | 177–221 |
| `templates/dao-governance-dapp/contracts/governance/src/lib.rs` | `vote` | 150–207 |

**What is wrong.** After `voter.require_auth()`, the contract adds the caller-supplied `weight` to yes or no. There is no balance lookup. On the proposal-lifecycle contract there is also no `weight > 0` check, so a negative weight subtracts from the chosen side. On the DAO template, any positive `weight` is accepted (159–161) and added (183–193). The treasury contract uses the same caller-supplied weight (182, 205–214). `resolve_proposal_state` (391–406) and `resolve_state` (392–393) treat that tally as quorum.

**Why it matters.** One voter can meet `min_quorum` alone and pass a proposal. `execute_proposal` on the lifecycle contract then `invoke_contract`s the target (`F-037`). The treasury README says voting is weighted and token-based, and that weight is caller-supplied. The DAO template’s `initialize` (72–91) does authenticate the admin. Treasury token movement is `F-023`. Negative admin-set power on a different delegation contract is `F-065`.

### F-010 — Critical — Marketplace buy records a sale and does not collect payment

**Paths and functions**

| Path | Function | Lines |
| --- | --- | --- |
| `examples/nfts/04-nft-marketplace/src/lib.rs` | `create_fixed_price_listing` | 95–138 |
| `examples/nfts/04-nft-marketplace/src/lib.rs` | `create_auction_listing` | 141–195 |
| `examples/nfts/04-nft-marketplace/src/lib.rs` | `place_bid` | 198–247 |
| `examples/nfts/04-nft-marketplace/src/lib.rs` | `buy` | 250–273 |
| `examples/nfts/04-nft-marketplace/src/lib.rs` | `finalize_auction` | 276–310 |
| `examples/nfts/04-nft-marketplace/src/lib.rs` | `settle_sale` | 334–380 |
| `templates/nft-marketplace-dapp/contracts/marketplace/src/lib.rs` | `list_item` | 99–138 |
| `templates/nft-marketplace-dapp/contracts/marketplace/src/lib.rs` | `buy_item` | 171–194 |

**What is wrong.** Listings store an NFT id and a price and never call the NFT contract. `place_bid` stores `Bid { bidder, amount }` with no escrow. `buy` checks the offer equals `listing.price` and calls `settle_sale`. `settle_sale` writes a `TradeRecord`, increments the trade count, sets `sold`, and computes `royalty_paid = amount * royalty_bps / 10000` (341) without a token transfer, a royalty transfer, or an NFT transfer. `buy_item` authenticates the buyer, sets `nft.owner = buyer`, deletes the listing, and emits `listing.price`. There is no token client. `finalize_auction` takes `_executor` and does not authenticate it. The example README says anyone may finalize. The missing payment is separate from that sentence.

**Why it matters.** A buyer can take a listing off the market, and in the template can take the NFT, without paying. Both READMEs and the template doc comment describe a purchase. `initialize` on both contracts does authenticate the admin (example lines 82–88; template lines 58–69). Template `mint` (73–96) is open to any authenticated creator on purpose.

### F-011 — Critical — Custom-token multisig counts one signer many times

**Path.** `examples/tokens/10-custom-token/src/lib.rs`, `multi_sig_transfer` (332–375).

**What is wrong.** The loop requires `signers.len() == threshold`, `authorized.contains`, and `signer.require_auth()` (348–352). It does not require distinct addresses. One signer listed `threshold` times satisfies the loop. Each `require_auth` on that address is the same invocation check.

**Why it matters.** That signer can move the contract’s balance. Combined with `F-001`, a self-transfer to the contract address inflates that balance first. `update_signers` (294–314) is admin-gated and does authenticate.

### F-012 — Critical — Ajo factory lets any caller register executable WASM

**Path.** `examples/intermediate/ajo-factory/src/lib.rs`, `initialize` (155–190), `register_template` (198–228), `create_instance` (257–335).

**What is wrong.** `initialize` stores `wasm_hash` with no `require_auth`. `register_template` has no auth either, and it stays callable after initialization. `create_instance` deploys `meta.wasm_hash` from that registry. `get_template` (232–236) `expect`s on a missing id and traps.

**Why it matters.** An attacker who registers a template, or who wins `initialize`, chooses the code later users deploy. `create_instance` calls `initialize` on the new contract only when `template_id == TEMPLATE_AJO` (312–318). Savings and Escrow instances are left uninitialized (`F-082`).

### F-013 — Critical — Vesting pays from the contract without escrowing the schedule

**Paths and functions**

| Path | Functions |
| --- | --- |
| `examples/tokens/05-vesting/src/lib.rs` | `create_schedule` (76–136), `claim` (140–178) |
| `examples/tokens/01-vesting-management/src/lib.rs` | `create_schedule` (174–243), `claim` (282–317), `revoke` (332–372) |

**What is wrong.** `create_schedule` writes `total_allocation` or `total_amount` and does not pull tokens. `claim` then `token::Client::transfer`s from the contract to the beneficiary. `revoke` in the management contract sends the unvested remainder to the admin. Nothing caps the sum of schedules by the contract’s token balance.

**Why it matters.** Schedules share one pot, and the first successful transfer wins. With `F-004`, the first caller is that admin, can set cliff `0` and a short duration, and can withdraw tokens later sent to the contract. State is updated before the external transfer. The missing escrow is the defect.

### F-014 — Critical — Anyone can pause collateralized lending and delete positions

**Path.** `examples/defi/04-collateralized-lending/src/lib.rs`, `set_emergency_pause` (56–61) and `emergency_liquidate` (262–288). `initialize` is 18–54.

**What is wrong.** `initialize` never stores an admin. `set_emergency_pause` takes `admin: Address`, calls only `admin.require_auth()`, and writes `emergency_paused`. `emergency_liquidate` uses the same check (263). When paused, it `positions.remove(borrower)` (284). Collateral and debt for that user are deleted and are not paid out.

**Why it matters.** Any account can pause, then wipe every position. `deposit_collateral`, `withdraw_collateral`, and `borrow` also panic while `emergency_paused` is true. The map updates themselves move no token (`F-020`). The mechanical scan had not read `emergency_liquidate` through and had rated the pause High. The manual review read the delete. This report follows that.

### F-015 — Critical — Delegation marketplace mints the balance used as rent

**Path.** `examples/governance/08-delegation-marketplace/src/lib.rs`, `fund_account` (356–365) and `rent_voting_power` (206–264). `list_offer` is 137–166.

**What is wrong.** Module docs (lines 11–12) say the renter pays `units * price_per_unit` tokens. `fund_account` credits `DataKey::Balance` with no `require_auth` and no token transfer. `rent_voting_power` treats that balance as payment (242–264) and records `ActiveDelegation` voting power (274–282). `list_offer` never checks that the delegator holds a governance token. `voting_power` is whatever the caller writes.

**Why it matters.** Anyone can mint balance and rent listed power. Expiry accounting is `F-066`.

### F-016 — Critical — Bridge mint and release ignore validator signatures

**Path.** `examples/advanced/24-cross-chain-bridge/src/lib.rs`, `verify_validator_signatures` (351–377), called by `mint_tokens` (131–156) and `release_tokens` (217–240). `initialize` is 63–76.

**What is wrong.** The verifier counts a signature when the address is a stored validator. It never checks `signature` bytes against the transfer hash. `transfer_hash` is computed at 353 and unused. `initialize` also accepts `threshold == 0`, and `valid_signatures < threshold` is then false for an empty list.

**Why it matters.** Anyone can pass the public validator addresses, or no signatures if the threshold is 0, and `mint_tokens` or `release_tokens` will transfer the bridge’s token balance to a chosen recipient. The README calls this multi-signature validation. The constant validator count is `F-051`. Transfer-before-flag ordering is `F-052`.

### F-017 — Critical — Aggregation oracle accepts a price from anyone for a listed source

**Path.** `examples/advanced/26-data-aggregation-oracle/src/lib.rs`, `submit_data` (135–154), `require_authorized` (291–298).

**What is wrong.** `require_authorized` only checks that `source` is in the trusted-source list. It never calls `source.require_auth()`.

**Why it matters.** Any caller can write `SourceValue` for every trusted source and move the median that `aggregate_data` publishes. Admin checks that also skip `require_auth` are `F-003`.

### F-018 — Critical — Merkle airdrop root is set by whoever initializes

**Path.** `examples/advanced/28-merkle-airdrop/src/lib.rs`, `initialize` (75–89). `claim` is 97–137.

**What is wrong.** The one-time guard does not call `admin.require_auth()`. The caller chooses `admin`, `token`, and `root`.

**Why it matters.** `claim` pays `token` from the contract to whoever matches the root. The first caller can install their own root and later drain tokens sent to the contract. `claim` itself marks `Claimed`, extends TTL, and binds the leaf to `claimer.require_auth()` before `token::transfer`.

### F-019 — High — Lending pool updates balances and never moves a token

**Path.** `examples/defi/03-lending-pool/src/lib.rs`, `deposit` (48–79), `withdraw` (81–119), `borrow` (121–165), `repay` (167–203).

**What is wrong.** Each function updates `UserPosition` and totals with plain `+=` / `-=` and never calls a token client. There is no `token::Client` in the file.

**Why it matters.** The README describes deposit, withdraw, borrow, and repay of assets. A copied pool credits deposits and borrows with no payment. Borrow increases `position.borrow` and `total_borrows` (lines 154–164) with no asset movement. The arithmetic is unchecked. Release builds do not apply the overflow checks in the misspelled profile table (`F-062`). The rate view can also trap (`F-063`).

### F-020 — High — Collateralized lending positions are a map, not token balances

**Path.** `examples/defi/04-collateralized-lending/src/lib.rs`, `deposit_collateral` (63–90), `withdraw_collateral` (92–131), `borrow` (133–169), `repay` (171–195).

**What is wrong.** `position.collateral += amount` (85), `position.debt += amount` (164), and `position.debt -= repay_amount` (190). No token transfer. `user.require_auth()` is present.

**Why it matters.** Borrow and collateral are numbers in instance storage. The README lists collateral deposit, borrow, and repay and does not say the transfer is omitted. The pause-and-delete path is `F-014`.

### F-021 — High — Vault deposit and withdraw never transfer a token, and two strategies cannot be funded

**Path.** `examples/defi/09-vault-strategies/src/lib.rs`, `deposit` (222–288), `withdraw` (294–335), allocation check (257–262), `strategy_params` (145–159).

**What is wrong.** Both functions update `UserBalance` and `TotalDeposits` with `checked_add` / `checked_sub` and never transfer a token. The allocation check rejects when `amount * 10_000 > new_total * max_allocation_bps`, with `new_total = total + amount`. Balanced is 8_000 bps and Aggressive is 5_000. From `total == 0`, `amount * 10000 > amount * 8000` is always true.

**Why it matters.** Comments and the README say users deposit tokens and the vault allocates funds. Checked math does not create the tokens. The first deposit into Balanced or Aggressive always panics with “Exceeds strategy allocation cap”. Conservative (10_000 bps) passes. `pause`, `unpause`, and `switch_strategy` do check the stored admin.

### F-022 — High — Legacy staking pool records a stake and invents the payout

**Path.** `examples/defi/staking-pool/src/lib.rs`, `stake` (89–123), `withdraw` (125–158).

**What is wrong.** `stake` stores `StakeInfo.amount` and does not pull a token. `withdraw` deletes the stake and returns `net_amount`, which can be `stake.amount - penalty` (140) or `stake.amount + bonus` (143). The penalty is not sent anywhere. The bonus is not funded by a reward balance. `staker.require_auth()` is present.

**Why it matters.** The README says the contract lets a user stake a token amount and applies a 20% penalty or a maturity boost. The bonus exists only in the return value.

### F-023 — High — Treasury and DAO execution record a payment and do not transfer it

**Paths and functions**

| Path | Function | Lines |
| --- | --- | --- |
| `examples/governance/10-dao-treasury/src/lib.rs` | `deposit` | 125–137 |
| `examples/governance/10-dao-treasury/src/lib.rs` | `execute` for `ProposalKind::Transfer` | 224–269, subtraction at 249–257 |
| `templates/dao-governance-dapp/contracts/governance/src/lib.rs` | `execute` | 211–252 |

**What is wrong.** `deposit` adds `amount` to `TreasuryBalance` with `checked_add` and no token transfer. `execute` of a transfer subtracts `proposal.transfer_amount` from that key and does not transfer tokens to `recipient`. `ProposalKind::Upgrade` is marked `Executed` and never calls a wasm update. The template `execute` sets status to `Executed` and emits `proposal.target` and `proposal.amount` (241–250). No token transfer runs. Proposals carry an amount (124–130) that the contract never custodies.

**Why it matters.** The treasury README says the contract deposits and withdraws native token balances and disburses assets, and that upgrade replaces WASM. Execution changes a counter or a status. Vote weight on these contracts is `F-009`.

### F-024 — High — Staking pool auths the token contract, not the staker, and writes state after the transfer

**Path.** `examples/defi/07-staking-pool/src/lib.rs`, `stake` (183–198), `unstake` (200–217), `claim_rewards` (219–229).

**What is wrong.** `stake`, `unstake`, and `claim_rewards` never call `staker.require_auth()`. `stake` calls `token::Client.transfer` from `staker` to the contract (189) before writing `Balance` and `TotalSupply` (191–197). The credit is `balance_of_internal + amount`. `unstake` updates storage and then transfers staking tokens to the `staker` argument. `claim_rewards` sends reward tokens to that argument. The token `transfer` is from the contract, so the token authorizes the contract. Balance updates use raw `+` / `-` (191, 197, 207, 213).

**Why it matters.** Any caller can unstake a victim’s full balance. Tokens go to the staker, so this is a forced exit and a reward reset. Any caller can also push `claim_rewards` for any staker. A standard token’s `transfer` still requires the `from` address, so an honest staking token blocks staking as someone else. The pool itself does not. Stake is an external call before the balance write. Rewards are paid from the contract’s reward-token balance (229). Unchecked `+` / `-` can wrap in a release build (`F-062`).

### F-025 — High — Constant-product AMM locks `swap` only, so liquidity can move during the input transfer

**Path.** `examples/defi/02-constant-product-amm/src/lib.rs`, `SwapGuard` (41–59), `swap` (196–228), `add_liquidity` (84–146), `remove_liquidity` (148–186).

**What is wrong.** `SwapGuard` is taken only in `swap`. `add_liquidity` and `remove_liquidity` do not take it. The README (lines 44–46) says the mutex covers `swap` only. In `swap`, reserves and `amount_out` are fixed (206–225), then the input token is transferred (227), then the output (228). `add_liquidity` transfers both tokens (101–102) and only then `mint_lp` (144). `remove_liquidity` burns LP before transferring (182–184). A second `initialize` at line 419 is inside `#[cfg(test)] mod test` and is not a production entry.

**Why it matters.** A hook on the input transfer can `remove_liquidity` against balances that already include the input. The LP burn and token outs use the live balances. Control then returns to `swap`, which still sends the precomputed `amount_out`. That pays the swap output on top of the liquidity withdrawal. The README says the host currently rejects reentrancy. The crate still ships hook tests and an incomplete guard. The first-depositor share inflation is `F-064`.

### F-026 — High — Swap-liquidity reserves are written after the token calls, and the surplus is kept

**Path.** `examples/defi/10-swap-liquidity/src/lib.rs`, `add_liquidity` (117–175), `remove_liquidity` (177–223).

**What is wrong.** `add_liquidity` transfers both tokens (132–133), then mints LP (154), then writes reserves (156–164). `remove_liquidity` burns and transfers (201–203) before writing reserves (205–213). Shares are `min(share_a, share_b)` (140–150) with no refund of the other side. The surplus stays in the reserve counters (158–161). Initial shares are `amount_a * amount_b` (136–138).

**Why it matters.** A token hook can reenter `remove_liquidity` or `add_liquidity` while reserves and `TotalShares` are still the pre-call values, and the outer call then overwrites them. Stored reserves diverge from token balances. A provider who is off-ratio pays both amounts and receives the smaller share. The first mint is quadratically larger than the tokens deposited.

### F-027 — High — AMM price oracle takes deposits that only leave through `swap`, against stale reserves

**Path.** `examples/defi/11-amm-price-oracle/src/lib.rs`, `AmmPoolContract::deposit` (155–177), `swap` (179–235), `update` (289–327), `get_twap` (329–338).

**What is wrong.** `deposit` transfers both tokens (166–167) and then stores `reserve_a + amount_a` and `reserve_b + amount_b` (169–176). There is no LP token and no withdraw. `swap` transfers the input (204) before computing `amount_out` from the old reserves (206–213) and before writing new reserves (218–233). `update` is permissionless. The first observation stores `current_price * 0` (298–299). `get_twap` divides cumulative by elapsed time with no minimum window. `provider.require_auth()` is present on deposit (156).

**Why it matters.** Deposited tokens leave only through `swap`. Depositors have no claim on principal. A hook can reenter `swap` against stale reserves, and the outer write overwrites the inner reserve update. One `update` after a spot move defines the TWAP. Reserve adds are unchecked (`F-062`).

### F-028 — High — Farming deposit and withdraw call the token before the stake is saved

**Path.** `examples/defi/12-farming-pool/src/lib.rs`, `deposit` (159–188), `withdraw` (191–222), `remove_pool` (94–115).

**What is wrong.** `deposit` may `safe_reward_transfer` (171) and then transfer the staking token (177) before `user_info.amount += amount` (178) and before the storage writes (184–187). `withdraw` subtracts in memory (210–211), transfers (213), then stores (218–221). `remove_pool` deletes `DataKey::Pool` (114). Later `withdraw` and `deposit` call `get_pool`, which panics “Pool not found” (233–237). Amount updates and `acc_reward_per_share` (267) use plain arithmetic. Pending reward uses unchecked subtraction (168–169, 203–204).

**Why it matters.** A reward-token or staking-token hook can reenter `withdraw` against the still-stored full stake and move tokens twice. The outer save can restore or inflate `user_info.amount`. After `remove_pool`, staked tokens have no user path out. The comment at 92–93 says funds are not handled. Admin withdrawal of those tokens is `F-006`. Unchecked math can wrap in release (`F-062`).

### F-029 — High — Liquidity-mining `unstake` transfers before it saves the reduced stake

**Path.** `examples/defi/08-liquidity-mining/src/lib.rs`, `stake` (196–235), `unstake` (241–281).

**What is wrong.** `stake` transfers LP tokens (216–218) and then updates memory and saves (221–231). `unstake` decreases `info.staked` in memory (263), then `lp_client.transfer` (273–274), and only then `save_pool` / `save_user` (276–277). `user.require_auth()` is present on both.

**Why it matters.** Until `save_*`, storage still has the old stake. A token that reenters `unstake` can withdraw the same stake twice. That is a fund-loss case for an arbitrary LP token address. Setting the reward token equal to the LP token is `F-007`.

### F-030 — High — Flash-loan fee has no cap, and the admin is whoever initialized

**Path.** `examples/defi/05-flash-loans/src/lib.rs`, `set_fee` (96–99), fee math (56–57), repayment (82).

**What is wrong.** `set_fee` does not cap `new_fee_bps`. The fee is `(amount * fee_bps as i128) / 10000`, and repayment is `amount + fee`. A very large `fee_bps` makes `amount * fee_bps` overflow.

**Why it matters.** The first caller of `init` (`F-004`) is the admin `set_fee` authorizes, and can set the fee before users rely on the pool. `fee_bps` above 10_000 makes repayment larger than the borrowed amount by more than a 100% fee. Overflow panics when checks are on and wraps when the release profile is ignored (`F-062`). The reentrancy flag is set before the token transfer and callback (49–73) and cleared after repayment (86). That part matches the module comment.

### F-031 — High — Flash-loan callbacks do not check `initiator`

**Path.** `examples/defi/06-flash-loan-use-cases/`

| File | Function | Lines |
| --- | --- | --- |
| `src/arbitrage.rs` | `on_flash_loan` | 92–112 |
| `src/arbitrage.rs` | `withdraw` | 118–124 |
| `src/refinancing.rs` | `on_flash_loan` | 101–127 |
| `src/security.rs` | `on_flash_loan` | 62–64 |

**What is wrong.** `arbitrage` `on_flash_loan` does not check `initiator` against a stored provider. It approves `initiator` for `amount + fee` (112) and swaps whatever pools `execute` left in temporary storage. The comment at 90–91 says production code must validate `initiator`. `refinancing` `on_flash_loan` has the same gap. It approves pools and `initiator`, and calls `repay_and_withdraw`, `deposit_collateral`, and `borrow` on pool addresses stored by `execute`. `security` `on_flash_loan` trusts the stored provider. `withdraw` in `arbitrage` sends the contract’s full token balance to `to` for the owner set at init.

**Why it matters.** The example leaves the initiator check out. A hostile initializer (`F-004`) is that owner, and the callback approves and calls addresses the initiator influenced.

### F-032 — High — AMM router quotes a swap and never transfers, and anyone can overwrite reserves

**Path.** `examples/defi/13-amm-router/src/lib.rs`, `add_pool` (33–48), `swap_exact_tokens_for_tokens` (77–117), `calculate_swap_output` (64–74).

**What is wrong.** `add_pool` has no auth. Any caller overwrites `Map` reserves for a pair (39–44). `swap_exact_tokens_for_tokens` authenticates `user` and returns a quoted `current_amount`. It never transfers tokens. `_to` is unused (83). The quote uses raw `amount_in * 997` and `amount_in_with_fee * reserve_out`.

**Why it matters.** The README title and “Multi-hop swaps” describe execution. The function returns a number.

### F-033 — High — Reward strategies vest against a caller-supplied timestamp and do not pay a token

**Path.** `examples/defi/15-reward-strategies/src/lib.rs`, `claim` (163–181), `claimable` (152–159), elapsed-time math (208–224).

**What is wrong.** `now: u64` is an argument. `entitled` uses that value for linear elapsed time and exponential periods, not `env.ledger().timestamp()`. `claim` authorizes the participant and records `Claimed = entitled` for that caller-chosen time. `claim` does not transfer a token. It returns the `i128` and emits an event. The pool’s reward is the `total_reward` integer set in `initialize` (98–100).

**Why it matters.** Passing `now` far in the future marks the full `total_reward` share as claimed immediately. `initialize` does call `admin.require_auth()` (96). `register` authenticates the admin (275–284).

### F-034 — High — Proposals become executable with zero yes votes

**Path.** `examples/governance/03-voting-time-constraints/src/lib.rs`, `finalize` (371–397), `execute` (404–428).

**What is wrong.** `finalize` moves `Active` to `GracePeriod` when `now > voting_deadline`, then `GracePeriod` to `Executable` when `now >= executable_after`. It does not read `votes_for`, `votes_against`, or `quorum_threshold`. `execute` only checks `state == Executable` (415–417) and marks `Executed`.

**Why it matters.** A proposal with zero yes votes becomes executable after the clock. `close_early` (338–339) does check quorum. The normal path does not. `initialize` authenticates admin (165). `vote` is one address, one vote (297–300).

### F-035 — High — Anyone can create or close a proposal

**Path.** `examples/governance/05-proposal-validation/src/lib.rs`, `create_proposal` (52–99), `close_proposal` (108–130).

**What is wrong.** Neither function takes an address or calls `require_auth`. Any caller creates a proposal. Any caller sets `active = false` on any id.

**Why it matters.** Window checks (137–154) do not replace authorization.

### F-036 — High — Token-voting execution runs before the proposal is marked executed, and the snapshot includes the same ledger

**Path.** `examples/governance/02-token-voting/src/lib.rs`, `execute_proposal` (500–534), `submit_proposal` (327–333), `balance_at` (621–636), `vote` (385), `vote_for` (449), `mint` (189–193).

**What is wrong.** `execute_proposal` calls `env.invoke_contract` (521) before it stores `ProposalState::Executed` (523–526). `submit_proposal` sets `snapshot_ledger` to the current sequence. `balance_at` returns the latest checkpoint with `checkpoint.ledger <= snapshot`. Checkpoints written later in that same ledger still match. `mint` uses raw `old_supply + amount` and `current_balance + amount`.

**Why it matters.** The target can call `execute_proposal` again while `resolve_proposal_state` still returns `Passed` (513–516, 676–690). The action runs twice. Tokens received after submit, in the same ledger, count as voting power. `initialize` and `mint` do authenticate the stored admin (162, 178–179).

### F-037 — High — Proposal execution runs before state is marked executed

**Path.** `examples/governance/04-proposal-lifecycle/src/lib.rs`, `execute_proposal` (272–313).

**What is wrong.** `invoke_contract` (296–300) runs, then `state = Executed` (302–305).

**Why it matters.** Reentering `execute_proposal` from the target sees `Passed` again. The caller-supplied weight that decides `Passed` is `F-009`.

### F-038 — High — NFT `transfer` cannot be used by an approved operator

**Path.** `examples/nfts/03-nft-metadata-standards/src/lib.rs`, `transfer` (400–404), `approve` (419–449), `set_approval_for_all` (453–468).

**What is wrong.** Docs (397–399) say the caller must be the owner or an approved address. `transfer` calls `from.require_auth()` and then `check_approved(&env, &from, token_id)`. The authorized address is `from`, so the check asks whether the owner is approved for themselves. There is no `transfer_from`.

**Why it matters.** An operator or `Approved` address cannot move the token unless the owner signs. `approve` and `set_approval_for_all` store approvals that `transfer` cannot use. `mint` and `initialize` do require the stored admin (337–338, and initialize at 211).

### F-039 — High — Anyone can append another account’s snapshot history

**Paths.** `record_value` (103–128) and `auto_snapshot` (132–149) in:

- `examples/defi/14-automatic-snapshot-triggers/src/lib.rs`
- `examples/nfts/05-automatic-snapshot-triggers/src/lib.rs`
- `examples/governance/11-automatic-snapshot-triggers/src/lib.rs`

and `record_value` (103–129) and `auto_snapshot` (132–150) in `examples/tokens/10-automatic-snapshot-triggers/src/lib.rs`.

**What is wrong.** `record_value(env, owner, value)` appends a `SnapshotRecord` for `owner` with no `owner.require_auth()`. `auto_snapshot` calls `record_value` with `latest_value`. On the token copy, `total += 1` (122) is unchecked.

**Why it matters.** Anyone can forge another account’s snapshot history. The next auto snapshot copies the forged last value. The admin-setter bypass is `F-003`. The tokens review described this gap as Medium and separate from that bypass. This report files the four copies once, at High, with the defi, nft, and governance review.

### F-040 — High — Payment-channel `close` and `deposit` move tokens before they update channel state

**Path.** `examples/advanced/43-payment-channels/src/lib.rs`, `deposit` (93–115), `close` (150–181).

**What is wrong.** `deposit` transfers tokens in (99) and then does `bal + amount` (103–110). `close` runs both `token::Client::transfer` calls before `CLOSED` is set and before `BAL_A` / `BAL_B` are zeroed. There is no reentrancy flag.

**Why it matters.** The contract holds the tokens from `deposit`. A token whose `transfer` calls back into `close` pays the same on-chain balances twice. The balance add is unchecked (`F-062`). Who may call `close`, and which keys `init` binds, are `F-041` and `F-004`. `submit_state` does check both Ed25519 signatures over a message that includes the contract address.

### F-041 — High — Either payment-channel participant can close on the last on-chain state immediately

**Path.** `examples/advanced/43-payment-channels/src/lib.rs`, `close` (150–157).

**What is wrong.** `close` does not require a sequence, a signature, or `timestamp >= expiry`. Any participant who authorizes can exit.

**Why it matters.** Off-chain updates that were not yet `submit_state`’d are ignored. One party can settle a stale split and take the tokens. A front-run `init` (`F-004`) also binds later deposits to attacker-chosen Ed25519 keys.

### F-042 — High — Reentrancy-guard `init` clears the entered flag

**Path.** `examples/advanced/15-reentrancy-guard/src/lib.rs`, `init` (27–28), `withdraw` (53–86), `deposit` (31–47).

**What is wrong.** `init` always sets `Entered` to false. There is no already-initialized check. `withdraw` sets `Entered`, then `invoke_contract` to `receive_funds`, then clears the flag. `deposit` credits `Balance` with no token transfer.

**Why it matters.** The callback the README says the guard stops can call `init` and then `withdraw` again. The balance the guard is protecting was never deposited. The mechanical scan did not flag `withdraw`’s write-before-call order. The manual review read the `init` bypass. This report follows that.

### F-043 — High — Diamond token mints ignore the minter role

**Paths and functions**

| Path | Function | Lines |
| --- | --- | --- |
| `examples/advanced/18-diamond-facets/src/lib.rs` | `mint` | 119–143 |
| `examples/advanced/38-diamond-pattern/src/lib.rs` | `tf_mint` | 488–511 |

**What is wrong.** The facet doc comment says the minter must have `ROLE_MINTER` or higher. The body only calls `minter.require_auth()` and then adds `amount` to balance and total supply. `has_role` (299–305) is never called. `tf_mint` has the same shape. No admin or role is stored for that facet.

**Why it matters.** Any address can mint the facet token to itself and increase `TfBalance` and `TfTotalSupply`. `AccessFacet::initialize` (253) and `DiamondBase::initialize` (147) do authenticate an admin. Those inits are not this defect.

### F-044 — High — Batch contracts rewrite balances with no authorization

**Paths and functions**

| Path | Functions |
| --- | --- |
| `examples/advanced/22-batch-builder/src/lib.rs` | `set_balance` (179–183), `execute_batch` (162–175), `apply_op` (283–309) |
| `examples/advanced/39-batch-operations/src/lib.rs` | `set_balance` (52–59), `apply_operation` for `SetPaused` (123–126), `execute_batch_atomic` (76–90), `execute_batch_partial` (93–109) |

**What is wrong.** `set_balance` writes any account’s balance with no auth. `add_mint`, `add_transfer`, and `execute_batch` also have no auth. `apply_op` updates storage balances and does not transfer a token. On the batch-operations contract, `Credit`, `Debit`, and `Transfer` only change `DataKey::Balance`, and `SetPaused` writes the pause flag from the batch. No `require_auth` on any of these.

**Why it matters.** Any caller can mint or move the balances these contracts treat as the result of a batch, drain them through `Transfer`, or pause and unpause. Checked arithmetic on the batch-operations contract does not fix the missing authorization.

### F-045 — High — Merkle claim is not bound to the caller

**Path.** `examples/advanced/21-merkle-proofs/src/lib.rs`, `verify_and_claim` (254–298).

**What is wrong.** A valid proof marks `Claimed(generation, index)` with no `require_auth` and no claimant address in the claim key. `initialize` (136–143) does authenticate the admin, and `set_root` authenticates the admin.

**Why it matters.** Proofs are public in this pattern. Anyone can consume a leaf index and block the party the leaf was built for. `examples/advanced/28-merkle-airdrop` `claim` does bind `claimer.require_auth()` and the leaf to that address (`F-018`).

### F-046 — High — Any address can mark a bridge transfer fraudulent

**Path.** `examples/advanced/17-bridge-security/src/lib.rs`, `submit_fraud_proof` (218–247).

**What is wrong.** `reviewer.require_auth()` only proves the caller chose their own address. There is no admin, operator, or validator check, and `proof_hash` is stored without being checked. Status becomes `Fraudulent` from `Pending` or `Challenged`.

**Why it matters.** Anyone can permanently block `finalize_transfer` for every transfer id. `initialize` (85–99) does call `admin.require_auth()`. Pause helpers authenticate the stored admin.

### F-047 — High — One state-channel party sets the dispute split

**Path.** `examples/advanced/07-state-channels/src/lib.rs`, `challenge` (273–317), `finalize` (394–411).

**What is wrong.** The comment at 267–268 says both parties must authorize. The code only calls `challenger.require_auth()` and checks that the caller is one party. Balances are accepted when they sum to the recorded deposit.

**Why it matters.** One party sets the final split, waits out `dispute_period`, and `finalize` closes on that split. `open` (208–216) documents that deposits are not token transfers (`F-076`). The authorization bug is the dispute outcome.

### F-048 — High — A fraud proof awards the whole channel without a proof

**Path.** `examples/advanced/34-state-channel-disputes/src/lib.rs`, `submit_fraud_proof` (313–361), `open_dispute` (162–226).

**What is wrong.** If the caller is a participant, is not `dispute.proposer`, and passes `invalid_sequence == dispute.sequence`, the function assigns `total_deposit` to the caller and closes the channel. No signature or state check runs. `open_dispute` lets one participant set any split that sums to the total.

**Why it matters.** Opening a dispute lets the counterparty take 100% of the recorded deposit by echoing the sequence. `open_channel` records balances with no token transfer (`F-076`). The dispute result is still the contract’s settlement rule.

### F-049 — High — Virtual-channel materialize requires both endpoints

**Path.** `examples/advanced/13-virtual-channel/src/lib.rs`, `materialize` (319–335).

**What is wrong.** The module docs at lines 15–16 and 28 say either endpoint can materialize the latest signed state. The function requires `vc.alice.require_auth()` and `vc.bob.require_auth()`.

**Why it matters.** The dispute path the file describes is not available. One endpoint can refuse to materialize and keep the recorded collateral in place. The deposits themselves are storage (`F-069`).

### F-050 — High — Any caller can trip the circuit breaker

**Path.** `examples/advanced/04-circuit-breaker/src/lib.rs`, `fail` (167–170), `record_failure` (246–268).

**What is wrong.** `fail` only requires `caller.require_auth()`, so every address counts. `record_failure` increments `FailureCount` and sets `Paused` once `count >= threshold`.

**Why it matters.** After init, anyone can pause the breaker. `set_pause` and `set_admin` do authenticate the stored admin. Who wins `initialize` is `F-004`.

### F-051 — High — Bridge validator count is the constant 3

**Path.** `examples/advanced/24-cross-chain-bridge/src/lib.rs`, `count_validators` (345–348), used by `remove_validator` (274–278) and `set_threshold` (293–295).

**What is wrong.** The function always returns `3`. `initialize` does not store a count.

**Why it matters.** `remove_validator` does not stop at the real validator set. With threshold 1 it allows every validator to be removed, because `3 - 1 < 1` is false. `set_threshold` compares against 3 rather than the stored validators. Signature checking is `F-016`.

### F-052 — High — Bridge token transfer happens before the processed flag

**Path.** `examples/advanced/24-cross-chain-bridge/src/lib.rs`, `mint_tokens` (148–156), `release_tokens` (230–240), `lock_tokens` (108–115).

**What is wrong.** `token_client.transfer` runs, then `ProcessedTransfer` is written. `release_tokens` also subtracts `LockedBalance` after the transfer. The contract has no reentrancy guard. The token address comes from the caller-supplied `BridgeTransfer`. `lock_tokens` has the same order for the lock accounting.

**Why it matters.** Together with `F-016`, a token that calls back into `mint_tokens` or `release_tokens` can transfer again before the processed key exists.

### F-053 — High — Claimable balance pays out before it reduces the balance

**Path.** `examples/advanced/09-fuzz-testing/src/lib.rs`, `claim` (120–156).

**What is wrong.** `token::Client::transfer` runs at 142–146. `amount` is subtracted and stored at 148–156. No reentrancy guard. `deposit` (104) transfers before writing `DataKey::Balance`. The double-pay is in `claim`.

**Why it matters.** The depositor-chosen token can reenter `claim` while `ClaimableBalance.amount` is still the old value and withdraw twice.

### F-054 — High — A negative batch burn increases balances

**Path.** `examples/advanced/35-gas-optimization/src/lib.rs`, `batch_burn` (294–305).

**What is wrong.** The loop rejects the call only when `current < amount`. A negative `amount` passes that test for a non-negative balance, and `current - amount` adds the absolute value. `batch_mint` (280) skips non-positive amounts. `batch_burn` does not.

**Why it matters.** An admin, including the unauthenticated admin from `F-004`, can mint by burning a negative amount. Overflow checks do not stop this. It is ordinary subtraction of a negative. The fee that disappears on `transfer` is `F-071`.

### F-055 — High — Gasless relayer invents balance and does not authenticate the relayer

**Path.** `examples/advanced/27-gasless-relayer/src/lib.rs`, `fund` (115–129), `relay_transfer` (132–146).

**What is wrong.** `fund` adds `amount` to `DataKey::Balance` and never pulls a token. `relay_transfer` checks `is_trusted_relayer` and never calls `relayer.require_auth()`.

**Why it matters.** Any caller can credit themselves and can pass the admin address as `relayer`, so the trusted-relayer gate in the README does not hold. Signed meta-transactions then move that invented balance. Who is written into the allowlist at init is `F-004`.

### F-056 — High — Trusted forwarder balances are storage, and the signed fee is what moves

**Path.** `examples/advanced/07-trusted-forwarder/src/lib.rs`, `fund` (100–113), `withdraw` (116–132), `forward` (135–220).

**What is wrong.** `fund` and `withdraw` only add or subtract `ForwarderDataKey::Balance`. The stored fee from `set_fee` is never compared to `tx.fee` in `forward`. The signed `tx.fee` is what moves.

**Why it matters.** The README says the forwarder holds balances deposited via `fund` and pays relayer fees from them. Callers can mint spendable fee balance, and `withdraw` does not transfer a token out.

### F-057 — High — Multicall invokes any contract as itself

**Path.** `examples/advanced/42-multicall/src/lib.rs`, `aggregate` (21–28).

**What is wrong.** No authorization. Each `Call` is `invoke_contract`’d with the caller-supplied target, function, and args.

**Why it matters.** Any user can make this contract call other contracts. Targets that treat the multicall address as trusted will accept those calls.

### F-058 — High — Integration-test registry entries can be replaced by anyone

**Path.** `examples/advanced/16-cross-contract-integration-testing/src/lib.rs`, `Registry::register` (20–26). `Factory::create_instance` records deployments at 85–86.

**What is wrong.** The comment says anyone may register. The write does not check that `name` is free and does not call `require_auth`.

**Why it matters.** A later `register` overwrites the name, so lookup returns an attacker address. A different name registry that refuses overwrites is `F-074`.

### F-059 — High — Cross-contract `update_entry` has no authorization

**Path.** `examples/advanced/25-cross-contract-optimization/src/lib.rs`, `update_entry` (34–48).

**What is wrong.** Any caller adds `delta` to `DataKey::Entry` with no `require_auth`. The same file does not build (`F-088`).

**Why it matters.** If the attributes are repaired without adding auth, every key is writable by anyone. The auth gap is visible in the function body as it stands.

### F-060 — High — A partial v1 to v2 migration rewrites profiles to zero

**Path.** `examples/intermediate/storage-migration/src/lib.rs`, `migrate_v1_to_v2` (112–172), `add_user` (75–93).

**What is wrong.** The function always starts `index` at 0 (139). Each step copies `LegacyBalance` into `Profile` and deletes the legacy key (142–154). If `batch_size` stops before the end, version stays 1 and state is `Prepared` (166–169), but the next call starts at 0 again. Users already migrated read `unwrap_or(0)` and their profile balance is overwritten with 0. `add_user` also appends duplicate addresses. One full pass then migrates the second copy after the legacy key is gone and writes 0.

**Why it matters.** Re-running a partial migration zeroes balances that were already copied. This crate is also outside the workspace (`F-084`). The known-issues log already flagged it for review.

### F-061 — High — Access-control `execute` ignores the configured threshold

**Path.** `examples/intermediate/access-control/src/lib.rs`, `initialize` (117–161), `execute` (439–476), `create_proposal` (343–399).

**What is wrong.** `initialize` rejects a bad `threshold` (130–132) and never stores it. `execute` ignores that parameter and instead requires `approvals.len() >= 2` and `approvals.len() >= signers.len() / 2 + 1` (468–469). `create_proposal` does not require the proposer to be a signer, despite the comment on line 342. `initialize` does call `admin.require_auth()` and does not clamp `timelock_delay` (140), so delay `0` makes `execute_at` immediate (372).

**Why it matters.** A configured unanimous set executes at a bare majority. A one-signer set can never execute because of the hard `< 2` check. This crate is outside the workspace (`F-084`).

### F-062 — Medium — Release overflow checks are under a table Cargo ignores

**Path.** Root `Cargo.toml`, `[Profile.release]` and `[Profile.release-with-logs]` (lines 172–184). `[profile.test]` is line 186.

**What is wrong.** During `cargo test -p mint-burn-token --offline`, Cargo warned `unused manifest key: Profile`. `[profile.test]` does set `overflow-checks = true`. The capital-P release tables are not Cargo profile tables.

**Why it matters.** Plain `+` / `-` / `*` on balances in `F-019`, `F-020`, `F-024`, `F-027`, `F-028`, `F-030`, and `F-040` are not checked operations. In a release build they are not saved by this table. Test builds still panic on overflow. This is separate from KI-3, which is a `[profile]` table in a non-root package.

### F-063 — Medium — Lending borrow rate divides by zero at the kink

**Path.** `examples/defi/03-lending-pool/src/lib.rs`, `get_borrow_rate` (223–246).

**What is wrong.** If `utilization <= kink_utilization`, the rate divides by `kink_utilization` (242). `initialize` does not require `kink_utilization > 0`. If `kink_utilization` is 100 and utilization is 100, the else branch divides by `100 - kink_utilization` (245). `get_utilization` can return 100 (219).

**Why it matters.** A zero kink panics the view whenever utilization is 0 (`0 <= 0`). Borrow is then blocked on a panic in the rate view when something calls it.

### F-064 — Medium — Constant-product AMM mints a 1-unit first share with no locked minimum

**Path.** `examples/defi/02-constant-product-amm/src/lib.rs`, `add_liquidity` (104–113 and 138–140).

**What is wrong.** The first deposit mints `integer_sqrt(amount_x * amount_y)` and does not lock a minimum LP amount to the contract. Later deposits that round to 0 are rejected after the token transfers. Those transfers revert with the `Err`.

**Why it matters.** A first depositor can mint a tiny share. The empty-share rejection fails the victim’s transaction. It does not stop inflation of reserves relative to a 1-unit supply before the victim arrives. The reentrancy gap is `F-025`.

### F-065 — Medium — Negative voting power subtracts from delegates

**Path.** `examples/governance/07-delegation/src/lib.rs`, `set_voting_power` (197–216), `effective_power` (456–497).

**What is wrong.** Admin is authenticated (204–205). `power` is not required to be positive. `effective_power` does `base - delegated_out` and adds incoming `delegator_base * bp / 10_000`.

**Why it matters.** A negative base subtracts voting power from delegates who receive it. `initialize` authenticates admin (186). `delegate` and `revoke` authenticate the delegator. This crate is outside the workspace (`F-084`).

### F-066 — Medium — Rented voting power disappears on expiry, and the fee saturates at `i128::MAX`

**Path.** `examples/governance/08-delegation-marketplace/src/lib.rs`, `expire_delegation` (332–341), fee math (238–240).

**What is wrong.** If the offer row is gone (`voting_power` hit 0 and `rent_voting_power` removed it, 268–269), expiry drops `returned_units` on the floor. Those units are not written back to a new offer. `fee` uses `checked_mul(...).unwrap_or(i128::MAX)`.

**Why it matters.** Listed power disappears. Overflow charges `i128::MAX` instead of failing closed. The minted rent balance is `F-015`.

### F-067 — Medium — Registry registration compares a caller-supplied payment and does not transfer it

**Path.** `examples/advanced/33-registry-access-controls/src/lib.rs`, `register` (88–114).

**What is wrong.** `payment` is compared to the stored `fee`. No token transfer runs.

**Why it matters.** A caller authorizes themselves, passes `payment >= fee`, and is marked registered without paying.

### F-068 — Medium — Multi-party escrow stores an amount and never takes a token

**Path.** `examples/advanced/01-multi-party-auth/src/lib.rs`, `sequential_auth_escrow` (223–247). `multi_sig_transfer` is 145–161.

**What is wrong.** Step 0 is documented as the buyer funding. The body stores `amount` under `EscrowBal` and never transfers a token. Step 2 zeros that entry. `multi_sig_transfer` leaves the token transfer commented out. The signer loop itself does call `require_auth`.

**Why it matters.** The example teaches an escrow state machine that does not custody value.

### F-069 — Medium — Virtual-channel deposits are storage writes

**Path.** `examples/advanced/13-virtual-channel/src/lib.rs`, `open_ledger` (156–185).

**What is wrong.** Comments call the arguments a collateral lock-up. The function writes `endpoint_deposit` and `intermediary_deposit` after both parties `require_auth`, and does not transfer a token.

**Why it matters.** Later `materialize` (`F-049`) reassigns those numbers. Nothing is locked.

### F-070 — Medium — A facet passes the interface check when one selector matches

**Paths.** `examples/advanced/19-diamond-security/src/lib.rs` `add_facet` (95–100). `support` (37–45) in `examples/advanced/19-diamond-security/facet-adder/src/lib.rs` and `examples/advanced/19-diamond-security/facet-multiplier/src/lib.rs`.

**What is wrong.** `support` returns true when any requested symbol equals `add` or `multiply`. `add_facet` treats that as “the facet supports this function list” and registers every symbol in the list.

**Why it matters.** Extra selectors are routed to a facet that only implements one of them. The admin who owns the diamond, including the first caller from `F-004`, chooses those selectors.

### F-071 — Medium — The transfer fee is subtracted and never credited

**Path.** `examples/advanced/35-gas-optimization/src/lib.rs`, `transfer` (205–216).

**What is wrong.** `fee` is `amount * fee_bps / 10_000`, subtracted from the recipient credit, and never added to an admin or fee balance. The debit uses `checked_sub`.

**Why it matters.** Recorded supply shrinks on every transfer. The missing credit is the defect.

### F-072 — Medium — Whitelist fee withdrawal transfers before it clears the accumulator

**Path.** `examples/advanced/29-merkle-whitelist/src/lib.rs`, `collect_fees` (819–840). `collect_registration_fee` is 1073–1074.

**What is wrong.** `token::Client::transfer` of `accumulated` runs at 836. `AccumulatedFees` is set to 0 at 838–840. Admin auth at 820–821 is present. Registration fees are transferred in `collect_registration_fee` before the accumulator is increased.

**Why it matters.** A fee token that calls back into `collect_fees` pays the accumulator twice. `initialize` (202–213) does call `admin.require_auth()`.

### F-073 — Medium — Storage-optimization balances are not token balances, and batch deposit skips auth

**Path.** `examples/advanced/09-storage-optimization/src/lib.rs`, `deposit` (72–104), `withdraw` (107–137), `batch_deposit` (163–195).

**What is wrong.** The doc comment is “Deposit with packed user storage”. `deposit` requires `user.require_auth()` and then does `data.balance += amount` with no token transfer. `withdraw` subtracts the same way. `batch_deposit` credits every `(user, amount)` pair with no `require_auth`.

**Why it matters.** This file is a storage-layout example, so it is not filed as a fund drain. Balances used by `withdraw` are not backed by tokens, and `batch_deposit` can credit any address. The unchecked update is a habit a learner can copy, and release builds do not get the misspelled profile’s overflow checks (`F-062`). `set_paused` (221–229) does authenticate the stored admin.

### F-074 — Medium — The contract-name registry can be claimed with no auth

**Path.** `examples/advanced/contract-registry/src/lib.rs`, `register` (41–60).

**What is wrong.** No `require_auth`. The first writer of a name owns that entry. `AlreadyRegistered` blocks overwrite.

**Why it matters.** The directory of contract addresses is set by whoever registers the symbol first. An existing name cannot be replaced. The integration-test registry that does allow replacement is `F-058`.

### F-075 — Medium — The case-study reward balance is storage only

**Path.** `examples/advanced/12-real-world-case-studies/src/lib.rs`, `fund_reward` (78–88), `claim_reward` (98–110).

**What is wrong.** `fund_reward` adds to `RewardBalance` with checked add and admin auth, and does not transfer a token. `claim_reward` zeros the balance and returns the amount. The comment at 93–97 says a production token transfer is not in this function.

**Why it matters.** The claim path cannot pay tokens. The function names are fund and claim. The zero-before-return order matches the comment. `initialize` (66) does call `admin.require_auth()`.

### F-076 — Medium — State-channel “deposits” are fields on the channel record

**Paths.** `examples/advanced/07-state-channels/src/lib.rs` `open` (204–240). `examples/advanced/34-state-channel-disputes/src/lib.rs` `open_channel` (96–140).

**What is wrong.** Both record `deposit_a` / `balance_a` style fields after party auth and do not call a token contract. `07-state-channels` says so in the `open` doc comment. `34` describes the amounts as deposits.

**Why it matters.** Settlement functions only change those fields. The dispute authorization bugs are `F-047` and `F-048`.

### F-077 — Medium — Diamond registration calls the facet before the proxy stores the mapping

**Path.** `examples/advanced/19-diamond-security/src/lib.rs`, `execute` (161–177), `add_facet` (95–100).

**What is wrong.** The proxy prepends its own address and `invoke_contract`s the facet. It does not authenticate an end user. Facet `add` / `multiply` then require the diamond address. `add_facet` calls `support` on the facet (96–97) before any facet mapping is written.

**Why it matters.** A facet chosen by the admin, or by the attacker from `F-004`, runs code during registration, before the proxy’s storage update. The selector check that makes that registration broad is `F-070`.

### F-078 — Medium — SEP-41 allowances have no expiration ledger

**Paths and functions**

| Path | Functions |
| --- | --- |
| `examples/tokens/01-sep41-token/src/lib.rs` | `approve` (112–130), `transfer_from` (134–172), `read_allowance` (337–341) |
| `examples/tokens/03-pausable-token/src/lib.rs` | `approve` (116–135), `transfer_from` (138–177) |
| `examples/tokens/10-custom-token/src/lib.rs` | `approve` (131–150), `transfer_from` (153–192) |

**What is wrong.** SEP-41 `approve` is `(from, spender, amount, expiration_ledger)`. These store a bare `i128`. `transfer_from` never compares a ledger sequence.

**Why it matters.** Allowances live until overwritten. The `01-sep41-token` README claims SEP-41 compatibility. `03-pausable-token` has no README (KI-5). `examples/tokens/05-allowance-pattern` and `templates/token-dapp` do store `expiration_ledger` and are not part of this finding.

### F-079 — Medium — Extension allowances with expiration 0 never expire, and `permit` stores no nonce

**Path.** `examples/tokens/02-sep41-extensions/src/lib.rs`, `transfer_from` (348–353), `allowance` (391–395), `approve` (311–333), `batch_approve` (265–271), `permit` (167–173).

**What is wrong.** Expiry is treated as expired only when `expiry > 0 && expiry < sequence`. Expiration `0` stays fully spendable. `approve` and `batch_approve` do not reject a past or zero ledger. `permit` does reject that (167–168). The README calls `permit` an EIP-2612 off-chain permit that prevents replay. The body is `require_auth_for_args` (173) with no stored nonce.

**Why it matters.** A zero expiration never expires. Argument substitution on `permit` is bound. A classic permit nonce is not implemented.

### F-080 — Medium — “Optimized” token operations edit an internal map, and the non-wasm wrap writes balances after the pull

**Path.** `examples/tokens/03-optimized-operations/src/lib.rs`.

**What is wrong.** `OptimizedTokenOps::batch_transfer` (183–235) and `transfer` (239–274) only edit the balances map. No function credits the map from `underlying`, and no `TokenClient` transfer runs. The module header (lines 1–14) describes batched token transfers against an underlying token. `StandardTokenOps` is compiled only for non-wasm, tests, or `testutils` (lines 23–47), so it is not in the wasm artifact. Its `initialize` (50–57) has no already-initialized check and can reset `TotalSupply` to 0 while balances remain. `wrap` (60–104) calls `TokenClient::transfer` (94–95) before writing the wrapped balance (97–102) and has no reentrancy flag.

**Why it matters.** The wasm path never moves the underlying token. On the comparison type, a callback can pull a second deposit while both calls write `old_balance + amount`, and a second `initialize` can point `Underlying` at a different token. `examples/tokens/06-token-wrapper` `wrap`, `unwrap`, and `transfer` do set and check `DataKey::Entered` (99–102, 141–144, 189–191). That guard matches the comment.

### F-081 — Medium — `publish` invokes every subscriber with no publisher auth

**Path.** `examples/intermediate/event-subscriptions/src/lib.rs`, `publish` (165–204).

**What is wrong.** Any caller chooses `topic`, `entity`, and `payload` and `try_invoke_contract`s `notify` on each active subscriber (184–193).

**Why it matters.** A subscriber that opted in can be called by anyone, with attacker-chosen arguments. Failures are isolated per subscriber. The missing authorization is the defect.

### F-082 — Medium — Savings and Escrow factory deploys are left uninitialized

**Path.** `examples/intermediate/ajo-factory/src/lib.rs`, `create_instance` (312–318).

**What is wrong.** Only the Ajo template is initialized in the deploy transaction. Savings and Escrow contracts are deployed and their own `initialize` is a separate call.

**Why it matters.** Those instances are deployed from WASM that `register_template` can replace (`F-012`), and they sit uninitialized until someone calls `initialize`.

### F-083 — Medium — Primitive-types balance is a global counter with no actor, and `initialize` is repeatable

**Path.** `examples/basics/11-primitive-types/src/lib.rs`, `initialize` (103–113), `transfer` (565–584), `deposit` (588–607).

**What is wrong.** `initialize` has no stored-key guard, so every call resets `Balance` to `1000`. `transfer` and `deposit` update that balance with checked math and no `require_auth`.

**Why it matters.** This is a type demo, not a token. The functions are still state-changing balance updates with no actor. The mechanical scan read this and did not treat it as a fund bug. The later review filed it as Medium.

### F-084 — Medium — Thirteen example packages are outside the workspace build

**What is wrong.** `cargo metadata --no-deps --offline` reported 137 `workspace_members`. These 13 `[package]` crates are on disk, are not in root `members`, are not in root `exclude`, and are not in `workspace_members`. No other workspace `Cargo.toml` path-depends on them. `cargo test --workspace` and `cargo build --workspace` do not build them.

**Paths**

- `examples/advanced/15-oracle-integration` (`oracle-integration`)
- `examples/advanced/15-reentrancy-guard` (`reentrancy-guard`)
- `examples/advanced/20-batch-transfer` (`batch-transfer`)
- `examples/advanced/21-merkle-proofs` (`merkle-proofs`)
- `examples/advanced/24-cross-chain-bridge` (`cross-chain-bridge`)
- `examples/advanced/25-cross-contract-optimization` (`cross-contract-optimization`)
- `examples/advanced/29-merkle-whitelist` (`merkle-whitelist`). Git tracks 11 files under this directory.
- `examples/advanced/37-beacon-management` (`beacon-management`)
- `examples/advanced/40-computation-optimization` (`computation-optimization`). The manifest has no `[lib]` or `[[bin]]`. `src/lib.rs` is absent (`F-089`). The only Rust file is `src/test.rs` (`F-085`).
- `examples/governance/07-delegation` (`delegation`)
- `examples/intermediate/access-control` (`access-control`)
- `examples/intermediate/storage-migration` (`storage-migration`). The known-issues log flags this crate for review. It is still not a workspace member.
- `examples/tokens/10-automatic-snapshot-triggers` (`snapshot-triggers-tokens`). Its `Cargo.toml` starts with `[workspace]` and also has `[package]`, so it is its own workspace root. Dependencies use `soroban-sdk = { workspace = true }`, and this file defines no `[workspace.dependencies]`. Root `exclude` does not list it.

**Why it matters.** Defects in these crates (`F-003`, `F-016`, `F-039`, `F-042`, `F-045`, `F-051`, `F-052`, `F-060`, `F-061`, `F-065`, `F-072`, and the init rows in `F-004`) are outside the workspace test and build. Three further packages are excluded on purpose and match the root comment. Those are `F-095`, not this finding.

### F-085 — Medium — New orphan `src/test.rs` files are never compiled

**What is wrong.** A `src/test.rs` is orphaned when `lib.rs`, after comment stripping, contains neither `mod test` nor `cfg(test)`. These five were not in the KI-2 affect list. The two KI-2 paths that are still orphaned are recorded under KI-2, not here.

| Path | What was read |
| --- | --- |
| `examples/advanced/35-gas-optimization` | `src/lib.rs` has neither `mod test` nor `cfg(test)`. `src/test.rs` is a suite of about 420 lines (`#[cfg(test)] mod tests`) and is never a module of `lib.rs`. Explicit workspace member. |
| `examples/advanced/40-computation-optimization` | No `src/lib.rs`. `src/test.rs` is 34 bytes: `#[test] fn t() yasslert!(trdui; }`. It is not a test module. Also outside the workspace (`F-084`). |
| `examples/basics/03-custom-errors` | The only `mod test` / `cfg(test)` text in `src/lib.rs` is commented out at lines 305–306. After comment stripping, neither declaration remains. Explicit workspace member. |
| `examples/defi/10-swap-liquidity` | `src/lib.rs` has neither declaration. `src/test.rs` has `use soroban_validation::test_events::EventList;` on line 2 and `#![cfg(test)]` on line 3, so the inner attribute is not at the top of the file. Explicit workspace member. |
| `examples/governance/02-token-voting` | `src/lib.rs` has neither declaration. `src/test.rs` starts with `#![cfg(test)]` and `use super::*;` (about 230 lines). Explicit workspace member. |

**Why it matters.** `cargo test -p` for these packages does not run those suites. `examples/advanced/16-hierarchical-access-control` is a different case (`F-101`).

### F-086 — Low — `cargo fmt --check` fails, and two test files do not parse

**Paths.** 58 files under `examples/`, `templates/nft-marketplace-dapp/contracts/marketplace/src/lib.rs`, and `tests/integration/tests/`. The two files rustfmt refused:

- `examples/defi/01-simple-swap/src/test.rs` line 3
- `examples/defi/11-amm-price-oracle/src/test.rs` line 3

**What is wrong.** `cargo fmt --all -- --check` from the repo root exited 1. 303 diff hunks are whitespace, import order, and wrapping. Both unparsable files put `#![cfg(test)]` after a `use` item, so rustfmt reports `an inner attribute is not permitted in this context` and emits no diff. `tests/fuzz` is excluded from the root workspace, so this command did not format it.

**Why it matters.** Formatting only, except those two test files are not valid as rustfmt sees them. The per-file hunk list is omitted here. The 58 files are 28 under `examples/advanced`, 6 under `examples/defi`, 5 under `examples/governance`, 7 under `examples/intermediate`, 4 under `examples/nfts`, 1 template, 3 under `tests/integration`, and 4 under `examples/tokens`.

### F-087 — Low — Advanced timelock `execute` consumes an id and does not run a call

**Path.** `examples/advanced/02-timelock/src/lib.rs`, `execute` (196–235). `queue` stores only `execute_at` (176–177) and does `extend_ttl` on the operation key (180).

**What is wrong.** The README describes delayed execution of operations. `execute` deletes the timestamp and emits an event. It does not invoke a target. Admin auth on `execute` is present.

**Why it matters.** Waiting out the delay only consumes an id. No scheduled call runs. `initialize` is the unauthenticated-admin row in `F-004`. Governance `examples/governance/09-timelock-governance` does invoke, and that is `F-008`.

### F-088 — Low — Cross-contract optimization source does not compile

**Path.** `examples/advanced/25-cross-contract-optimization/src/lib.rs`, lines 1, 8, 10, 15–16, 28, 31, and 39.

**What is wrong.** Attribute text is corrupted (`#f!allow(deprecated)]`, `#[ofg_attr`, `#[ofg_test)]`, `#[ocontracttype]`, `#kderive(Clone)]`, `#kontractimpl]`). `get` at line 39 is missing a closing parenthesis.

**Why it matters.** The example cannot be built. The auth gap inside `update_entry` is `F-059`. The package is also outside the workspace (`F-084`).

### F-089 — Low — Computation-optimization has no contract source

**Path.** `examples/advanced/40-computation-optimization/`.

**What is wrong.** The directory has `README.md`, `Cargo.toml`, and `src/test.rs`, and no `src/lib.rs`. The README says the example includes algorithm, loop, and caching optimization.

**Why it matters.** There is no contract to match that claim. Workspace membership is `F-084`. The test file is `F-085`.

### F-090 — Low — Token-template metadata reads trap if initialization never ran

**Path.** `templates/token-dapp/contracts/token/src/lib.rs`, `name` (341–342), `symbol` (346–347), `decimals` (351–352), `approve` (226–260), `allowance` (106–113).

**What is wrong.** `name` and `symbol` `unwrap` instance storage. `decimals` uses `unwrap_or(7)`. `approve` still accepts an already-past `expiration_ledger`. `allowance` then treats it as zero.

**Why it matters.** A metadata read aborts when the contract was never initialized. The past expiration is a usability gap. It does not spend an expired allowance. Transfer, allowance expiration, and the explicit `from != to` self-transfer check in this template (117–222) are in better shape than the cookbook tokens in `F-001`.

### F-091 — Low — Ordinary calls panic

**Paths**

| Path | Function | Lines | Behavior |
| --- | --- | --- | --- |
| `examples/intermediate/ajo/src/lib.rs` | `get_creator`, `get_amount` | 49–53, 56–60 | `.expect("Not initialized")` |
| `examples/intermediate/storage-pagination/src/lib.rs` | `get_item` | 146–150 | `panic!("Index out of bounds")`. The doc comment states this. `list` returns a page |
| `examples/intermediate/03-priority-queue/src/lib.rs` | `pop_max` | 68–72 | `panic!("Empty priority queue")`. `push` (48–56) and `pop_max` also have no auth. The queue is a single global heap |
| `examples/tokens/token-lock/src/lib.rs` | `lock`, `unlock`, `unlockable_balance` | 57–62, 80–82, 112–126, 165–167 | `panic!` on bad input and on checked-math failure |

**What is wrong.** These abort the calling transaction. They match the panic pointers in the known-issues log.

**Why it matters.** They do not by themselves corrupt balances. The token-lock module docs (lines 7–9 and 44–45) say that contract does not custody SEP-41 tokens. The panics are the pointer, not a hidden transfer. `examples/tokens/04-snapshot-token` `record_balance_snapshot` (420–423) and `record_supply_snapshot` (444–447) call `history.get(history.len() - 1).unwrap()` only in the `else` of `history.is_empty()`. `examples/intermediate/lazy-loading` `insert_cache` (271–272) unwraps `keys.get(0)` only when `keys.len() >= CACHE_CAPACITY`. Those two pointers are safe. `invalidate` (206–208) has no auth and only drops a cache entry.

### F-092 — Low — Fuzz targets do not exercise the token or intermediate trees

**Paths.** `tests/fuzz/fuzz_targets/example_fuzz.rs`, `advanced_claimable_balance.rs`, `advanced_timelock.rs`, `advanced_multi_party_auth.rs`.

**What is wrong.** `example_fuzz.rs` only calls hello-world `hello`. The other three construct advanced-category contracts under `env.mock_all_auths()`. The harness source showed no invariant bug. The targets have no `initialize`, `deposit`, `transfer`, or `require_auth` hits of their own. `tests/fuzz` is excluded from the root workspace (`F-095`).

**Why it matters.** The fuzz crate does not cover `examples/tokens` or `examples/intermediate`.

### F-093 — Low — The pause demo counter can be incremented by anyone

**Path.** `examples/intermediate/03-pause-unpause/src/lib.rs`, `increment` (143–149).

**What is wrong.** Any caller can bump the counter while unpaused. `reset` is admin-authenticated.

**Why it matters.** The impact is the demo counter only. Who wins `initialize` is `F-004`.

### F-094 — Low — Seventeen packages are workspace members without being listed in `members`

**What is wrong.** These paths are not strings in the root `members` array. They are in `cargo metadata` `workspace_members` (137 = 120 explicit + 17). Each is a `path` dependency of `tests/integration/Cargo.toml`. `examples/intermediate/02-role-based-access-control` is also a path dependency of `tests/security/Cargo.toml`. No explicit `members` path is missing a `Cargo.toml`.

**Paths**

- `examples/advanced/04-upgradeable-proxy/implementation-v1` (`upgradeable-proxy-implementation-v1`)
- `examples/advanced/16-cross-contract-integration-testing` (`cross_contract_integration_testing`)
- `examples/advanced/17-bridge-security` (`bridge-security`)
- `examples/advanced/31-proxy-admin` (`proxy-admin`)
- `examples/advanced/39-batch-operations` (`batch-operations`)
- `examples/advanced/contract-registry` (`contract-registry`)
- `examples/basics/02-storage-patterns` (`storage-patterns`)
- `examples/basics/03-authentication` (`authentication`)
- `examples/basics/04-events` (`events`)
- `examples/basics/05-auth-context` (`auth-context`)
- `examples/basics/05-error-handling` (`soroban-error-handling-example`)
- `examples/basics/08-soroban-types` (`soroban_types_example`)
- `examples/basics/09-enum-types` (`enum-types`)
- `examples/basics/10-custom-structs` (`custom-structs`)
- `examples/basics/11-primitive-types` (`primitive-types`)
- `examples/basics/12-data-types` (`data-types`)
- `examples/intermediate/02-role-based-access-control` (`role-based-access-control`)

**Why it matters.** The members list does not match the workspace Cargo actually uses. The 2026-09-29 KI-2 changelog wired `mod test;` in `02-role-based-access-control` and did not add that path to `members`.

### F-095 — Low — Three packages are root-excluded in the way the root comment describes

**Paths**

- `templates/dao-governance-dapp/contracts/governance` (`dao-governance`), child of the virtual workspace `templates/dao-governance-dapp`
- `templates/token-dapp/contracts/token` (`token-contract`), child of the virtual workspace `templates/token-dapp`
- `tests/fuzz` (`soroban-cookbook-fuzz`)

**What is wrong.** None of these are root `workspace_members`. Root `exclude` lists the two virtual workspaces and `tests/fuzz`. That matches the comment at the top of the root `Cargo.toml`. `tests/fuzz` also has `[workspace] members = ["."]` and its own `soroban-sdk = "28.0.0-rc.1"`. The template children pin the same SDK string. `cargo metadata` on the root manifest does not include them.

**Why it matters.** Workspace commands do not build these crates. The exclusion is the layout the root file describes. The nft-marketplace entry that is in both lists is `F-096`.

### F-096 — Low — `templates/nft-marketplace-dapp` is both a member and an exclude

**Path.** Root `Cargo.toml` `members` (line 152) and `exclude` (line 30). Package path `templates/nft-marketplace-dapp`.

**What is wrong.** The comment above `exclude` (lines 19–20) says the nft-marketplace-dapp is excluded from the root workspace. `cargo metadata --no-deps --offline` includes `templates/nft-marketplace-dapp/Cargo.toml` in `workspace_members`. The exclude entry does not drop it. The package points `[lib].path` at `contracts/marketplace/src/lib.rs`, which has `#[cfg(test)] mod test;`.

**Why it matters.** The manifest says the package is excluded and Cargo still builds it as a workspace member. `buy_item` on that contract is `F-010`.

### F-097 — Low — Twenty committed directory prefixes are shared

**What is wrong.** The leading number of a child directory is not an identifier. `examples/defi` and `examples/governance` have no committed duplicate prefixes. `examples/nfts` numbered directories `01-` through `05-` are unique. `nfts/`, `templates/`, and `tests/` have no shared prefix. Twenty committed groups share a prefix:

- `examples/advanced` `02-`: `02-beacon-proxy`, `02-timelock`
- `examples/advanced` `04-`: `04-circuit-breaker`, `04-upgradeable-proxy`
- `examples/advanced` `07-`: `07-state-channels`, `07-trusted-forwarder`, `07-upgrade-patterns`
- `examples/advanced` `09-`: `09-fuzz-testing`, `09-storage-layout-validator`, `09-storage-optimization`
- `examples/advanced` `12-`: `12-oracle-consumer` (no `Cargo.toml` at this level; tracked children `aggregating`, `common`, `guarded`, `settlement`), `12-real-world-case-studies`
- `examples/advanced` `14-`: `14-bridge-monitor`, `14-bridge-validators`
- `examples/advanced` `15-`: `15-oracle-integration`, `15-reentrancy-guard` (both also `F-084`)
- `examples/advanced` `16-`: `16-cross-contract-integration-testing`, `16-hierarchical-access-control`
- `examples/basics` `03-`: `03-authentication`, `03-custom-errors`
- `examples/basics` `05-`: `05-auth-context`, `05-error-handling`
- `examples/basics` `13-`: `13-collection-types`, `13-compressed-storage`, `13-queue-variants`
- `examples/basics` `14-`: `14-event-filtering`, `14-fifo-queue`
- `examples/intermediate` `03-`: `03-pause-unpause`, `03-priority-queue`
- `examples/tokens` `01-`: `01-sep41-token`, `01-vesting-management`
- `examples/tokens` `02-`: `02-minting-strategies`, `02-sep41-extensions`
- `examples/tokens` `03-`: `03-optimized-operations`, `03-pausable-token`
- `examples/tokens` `04-`: `04-mint-burn`, `04-snapshot-token`
- `examples/tokens` `05-`: `05-allowance-pattern`, `05-vesting`
- `examples/tokens` `06-`: `06-reward-token`, `06-token-wrapper`
- `examples/tokens` `10-`: `10-automatic-snapshot-triggers`, `10-custom-token`, `10-pausable-permissions`

**Why it matters.** A number in a path does not pick out one example. Snapshot-only directories that make a local listing look like a second example are `F-098`.

### F-098 — Low — Ten snapshot-only directories share a prefix with a committed example

**What is wrong.** Each directory’s only child is gitignored `test_snapshots/` (`.gitignore` line 32). `git ls-files` is 0. They are local working-tree directories, not committed examples, and they share a numeric prefix with a committed sibling.

**Paths**

- `examples/advanced/03-proxy-admin`, `examples/advanced/03-rbac-modifiers`, and `examples/advanced/03-registry-access-controls` share `03-` with committed `examples/advanced/03-oracle-pattern`
- `examples/advanced/04-cross-contract-integration-testing` shares `04-` with `04-circuit-breaker` and `04-upgradeable-proxy`
- `examples/advanced/05-diamond-security` shares `05-` with committed `examples/advanced/05-rate-limiting`
- `examples/defi/01-vault-strategies` shares `01-` with committed `examples/defi/01-simple-swap`
- `examples/governance/01-proposal-validation` and `examples/governance/01-vote-delegation` share `01-` with committed `examples/governance/01-simple-voting`
- `examples/governance/02-voting-time-constraints` shares `02-` with committed `examples/governance/02-token-voting`
- `examples/governance/03-dao-treasury` shares `03-` with committed `examples/governance/03-voting-time-constraints`

**Why it matters.** A local `ls` looks like a second example. Separate working-tree directories that contain only `test_snapshots/` and do not share a prefix are `examples/advanced/08-batch-operations`, `examples/tokens/allowance-pattern`, `examples/tokens/mint-burn`, `examples/tokens/multi-token-balance-manager`, `examples/tokens/token-metadata`, and `examples/tokens/token-wrapper`. `git ls-files` is 0 for each. They are not packages.

### F-099 — Low — Example folders beyond KI-5 have no README

**What is wrong.** `book/src/docs/style-guide.md` §2 says every example folder must contain a `README.md` with Overview, Key Soroban Patterns, Usage, and Code Deep Dive. KI-5’s three paths are still missing and are not repeated here. Additional gaps:

- `examples/advanced/16-cross-contract-integration-testing` has `Cargo.toml` and `src/` and no `README.md`. It is also absent from the `members` array and present in `workspace_members` (`F-094`).
- Nested package directories with no README of their own. The parent example folder does have a `README.md`:
  - `examples/advanced/04-upgradeable-proxy/implementation-v1`
  - `examples/advanced/12-oracle-consumer/aggregating`
  - `examples/advanced/12-oracle-consumer/common`
  - `examples/advanced/12-oracle-consumer/guarded`
  - `examples/advanced/12-oracle-consumer/settlement`
  - `examples/advanced/19-diamond-security/facet-adder`
  - `examples/advanced/19-diamond-security/facet-multiplier`

**Why it matters.** 134 package directories under `examples/` have a `README.md`. The 11 package directories without one are the 3 KI-5 paths, `16-cross-contract-integration-testing`, and these 7 nested paths. `examples/basics/events` is not a package (`F-100`).

### F-100 — Low — `examples/basics/events` is a tracked file tree, not the events package

**Path.** `examples/basics/events/src/lib.rs` (60 lines, git-tracked). No `Cargo.toml`. No `README.md`.

**What is wrong.** The package for events is `examples/basics/04-events` (`name = "events"`, `src/lib.rs` 399 lines). `diff -q` on the two `lib.rs` files exits 1. `examples/storage/` is also not a package. It contains only `README.md`.

**Why it matters.** The tracked `events` tree is easy to mistake for the example that has a README and a manifest.

### F-101 — Low — Hierarchical access control does not compile `src/test.rs`

**Path.** `examples/advanced/16-hierarchical-access-control`. `src/lib.rs` lines 460–461 are an inline `#[cfg(test)] mod test { ... }`. `src/test.rs` starts with `#![allow(deprecated)]` and `use super::*;` (about 200 lines).

**What is wrong.** The KI-2 predicate does not match, because `lib.rs` has an inline `#[cfg(test)]` module. An inline `mod test { }` does not load `src/test.rs`.

**Why it matters.** The suite in `src/test.rs` is a different file and is not part of the crate.

## Known issues

Status below uses only this pass’s notes. The log file was not edited. A KI stays open unless a note says that path is fixed.

### KI-1 — Test and lint builds fail on `soroban-sdk` testutils

**Log status:** Open. **This pass:** the described failure did not reproduce. Not re-filed as a vulnerability.

`cargo test -p mint-burn-token --offline` passed. `soroban-sdk` 28.0.0 compiled. Six unit tests passed and none failed (`test_burn_fails_with_insufficient_balance`, `test_burn_requires_owner_auth`, `test_initialize_and_mint_burn_flow`, `test_mint_and_burn_reject_non_positive_amounts`, `test_mint_requires_admin_auth`, `test_mint_respects_supply_cap`). Doc-tests: 0. The log’s evidence is a `soroban-sdk` 26.0.x `testutils` failure to link `serde_json` and `rand`. That error did not appear. The workspace `Cargo.toml` now depends on `soroban-sdk = "28.0.0-rc.1"`. Clippy was not run. The same command printed the unused-manifest warning recorded as `F-062`.

### KI-2 — Orphaned `src/test.rs`

**Log status:** Open. **This pass:** two listed paths are still orphaned, three are wired, and five new orphans exist (`F-085`).

Still true:

- `examples/tokens/01-sep41-token`. `src/test.rs` starts with `#![cfg(test)]` and `use super::*;`. `src/lib.rs` has no `mod test` and no `cfg(test)`.
- `examples/tokens/09-optimized-token-ops`. `src/test.rs` exists. `src/lib.rs` has no `mod test` and no `cfg(test)`.

Wired on disk, so these are no longer orphans:

- `examples/intermediate/03-priority-queue`. `src/lib.rs` lines 259–260 are `#[cfg(test)]` and `mod test;`. The log’s Affects list still names this path. The change log does not mention wiring it.
- `examples/defi/11-amm-price-oracle`. `src/lib.rs` lines 352–353. The 2026-09-29 change log says this was wired. It is still wired.
- `examples/nfts/02-nft-metadata`. `src/lib.rs` lines 235–236. The log strikes this path through, and the 2026-09-29 change log records the wiring. It is still wired.

The tokens review said it did not re-verify the log’s list and still named `03-priority-queue`. The inventory read the `lib.rs` lines. This status follows the inventory.

New orphans, not in the KI-2 affect list, are `F-085`: `examples/advanced/35-gas-optimization`, `examples/advanced/40-computation-optimization`, `examples/basics/03-custom-errors`, `examples/defi/10-swap-liquidity`, and `examples/governance/02-token-voting`. `examples/advanced/16-hierarchical-access-control` does not match the KI-2 predicate (`F-101`).

### KI-3 — Non-root `[profile]` table is ignored

**Log status:** Open. **This pass:** not re-filed. The advanced review says this still covers `examples/advanced/16-cross-contract-integration-testing/Cargo.toml`. The root `[Profile.release]` typo is a different defect (`F-062`).

### KI-4 — No committed `Cargo.lock`

**Log status:** Open. **This pass:** still true as a policy gap. A local `Cargo.lock` exists and is gitignored. The log’s SDK string `soroban-sdk = "26.0.0-rc.1"` is not the string in the workspace manifest now, which is `soroban-sdk = "28.0.0-rc.1"`. The lockfile is still not committed, so two builds can still resolve different versions.

### KI-5 — Missing example README

**Log status:** Open. **This pass:** all three paths are still missing. None has gained a README.

- `examples/tokens/03-pausable-token`
- `examples/tokens/06-reward-token`
- `examples/intermediate/ajo`

Further missing READMEs that are not in this KI are `F-099`.

### KI-6 — `examples/tokens/README.md` does not match the tree

**Log status:** Open. **This pass:** the tokens review restates that the category README links do not match the tree. The inventory did not re-check each link. Left open.

### Pointers in the known-issues log

These were pointers, not KI entries.

- `storage-migration` ordering and authorization: partial migration zeroes profiles (`F-060`). `initialize` is in `F-004`.
- `ajo` `.expect("Not initialized")`, `storage-pagination` out-of-range panic, `03-priority-queue` empty-queue panic, and `token-lock` panics: `F-091`. They abort the caller. They were not found to corrupt balances.
- `lazy-loading` cache-key `unwrap` and `04-snapshot-token` history `unwrap`: guarded by a non-empty check. Dismissed in `F-091`.
- `02-role-based-access-control`: `initialize` authenticates. `grant_role` / `revoke_role` compare caller rank to the current role and the new role. Owner cannot be demoted by Admin. No separate finding.
- `multi-sig-patterns`: after init, `approve` and `cancel` require a stored signer and `require_auth`. `execute` does not require the executor to be a signer and only flips `executed`. Noted on the `F-004` row. No funds move.
- `ajo-factory` and `event-history` `unwrap` / discarded admin: `F-012` and the `F-004` row for `event-history`.
- `iterable-mappings`: the pagination `unwrap` is an internal invariant of that map. No balance finding.
