# Governance Security Checklist

Use this checklist before deploying a Soroban governance contract or accepting a proposal into production. The examples in this cookbook demonstrate individual controls; a production system should verify every item against its own threat model.

## Voting Security

- [ ] Every voting action requires authorization from the voter, and the contract records votes against the authenticated address.
- [ ] A voter can vote only once per proposal, unless an explicitly documented vote-change mechanism is used.
- [ ] Voting is rejected before `vote_start` and after `vote_end`.
- [ ] Vote weights are snapshotted or otherwise resistant to transfers, flash-loan-style acquisition, and delegation changes during a vote.
- [ ] `For`, `Against`, and `Abstain` are represented distinctly, and tally arithmetic uses checked operations.
- [ ] Quorum and approval thresholds are defined before voting starts and cannot be changed retroactively.

## Proposal Validation

- [ ] Proposal creation is restricted to the intended proposer role or threshold.
- [ ] Proposal identifiers cannot collide, and proposal descriptions or hashes are stored consistently with the off-chain proposal record.
- [ ] Every target contract, function, argument, amount, and expiration is validated before a proposal becomes active.
- [ ] Empty actions, unsupported selectors, malformed arguments, and duplicate actions are rejected.
- [ ] Validation does not depend on mutable off-chain data that voters cannot inspect.
- [ ] Proposal creation, cancellation, and state changes emit events containing enough data for an indexer to reconstruct history.

## Execution Safety

- [ ] Execution is allowed only for a proposal that passed its quorum and approval checks.
- [ ] Execution is blocked until the voting period and any required grace period or timelock have elapsed.
- [ ] A proposal is marked as executed before or atomically with its external calls so it cannot be replayed.
- [ ] The executor cannot replace validated targets or arguments at execution time.
- [ ] Multi-action proposals define failure behavior: atomic rollback, explicit partial completion, or a compensating action.
- [ ] External calls use the narrowest required authorization and do not rely on reentrant callbacks.
- [ ] Cancellation, veto, emergency pause, and expired-proposal paths are tested and emit auditable events.

## Timelock Checks

- [ ] The delay is enforced by ledger time in the contract, not only by a UI countdown.
- [ ] Queueing records the proposal identifier, action hash, queue time, and earliest execution time.
- [ ] The delay cannot be shortened by the proposer, executor, or a single privileged account without an approved governance action.
- [ ] Queued actions expire after a documented window and cannot be executed after cancellation.
- [ ] Any timelock administrator is protected by multisignature or equivalent independent authorization.
- [ ] Users and monitoring systems can observe queue, cancellation, and execution events before funds or permissions change.

## Testing Requirements

- [ ] Unit tests cover authorization failures, duplicate votes, invalid time windows, quorum misses, and threshold boundaries.
- [ ] Tests cover proposal state transitions: Draft, Active, Queued, Executed, Defeated, Cancelled, and Expired where applicable.
- [ ] Tests advance ledger time across every boundary, including `vote_start`, `vote_end`, grace period, timelock, and expiry.
- [ ] Tests verify replay protection, action validation, external-call failures, and multi-action failure behavior.
- [ ] Fuzz or property tests exercise proposal inputs, vote weights, arithmetic limits, and action ordering.
- [ ] Integration tests run the governance contract against every target contract it can call.
- [ ] A review confirms that storage TTL extension, event fields, and migration behavior are safe for long-lived governance.

## Review Record

Record the contract version, network, reviewers, threat model, and unresolved risks alongside the deployment. Re-run this checklist whenever governance parameters, target contracts, authorization roles, or execution logic change.