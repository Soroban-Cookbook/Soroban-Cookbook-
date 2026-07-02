#641 [Cookbook] Create Batch Execution Pattern
Repo Avatar
Soroban-Cookbook/Soroban-Cookbook-
Summary
Execute multiple operations in batch

Current State
examples/advanced/ has 01-multi-party-auth and 02-timelock only; planned Phase 5 directories absent.

Implementation Hints
Add new crate under examples/advanced/; follow patterns in existing advanced examples.

Acceptance Criteria
 Project in examples/advanced/08-batch-operations/
 Batch call interface
 Atomic execution
 Partial execution option
 Revert handling
 10+ tests
Verification
`cargo clippy --workspace --all-targets -- -D warnings`; `cargo build --target wasm32-unknown-unknown --release -p <crate>`.
Source
Phase issue #227 from phase 5 issues.md (line 562)
Priority: High | Scope: L

#639 [Cookbook] Create Bug Bounty Documentation
Repo Avatar
Soroban-Cookbook/Soroban-Cookbook-
Summary
Documentation for bug bounty program

Current State
No in-repo deliverables; operational/community work tracked for planning.

Implementation Hints
Document outcomes in README or CONTRIBUTING.md when completed; link external resources.

Acceptance Criteria
 How to participate
 Submission guidelines
 Reward structure
 Hall of fame
 FAQ
Verification
Manual verification against acceptance criteria; update phase file when done.
Source
Phase issue #388 from phase 8 issues.md (line 182)
Priority: Medium | Scope: M

#640 [Cookbook] Add Proposal Validation
Repo Avatar
Soroban-Cookbook/Soroban-Cookbook-
Summary
Validate proposals before execution

Current State
examples/{defi,nfts,governance,tokens}/ contain README stubs only — no numbered example crates.

Implementation Hints
Place at examples//01-/; reuse 04-events and 03-authentication patterns from basics.

Acceptance Criteria
 Validation rules
 Parameter checks
 Conflict detection
 Tests and docs
 Best practices
Verification
`cargo test -p <crate>`, WASM release build, category README updated.
Source
Phase issue #173 from phase 4 issues.md (line 717)
Priority: Medium | Scope: L

#642 [Cookbook] Internal Security Review - Advanced
Repo Avatar
Soroban-Cookbook/Soroban-Cookbook-
Summary
Internal security review of advanced patterns

Current State
Basic integration tests in tests/integration/ (12 tests); coverage via tarpaulin.toml and CI.

Implementation Hints
Extend tests/integration/Cargo.toml path deps; follow integration_tests.rs cross-contract patterns.

Acceptance Criteria
 Complex pattern security validated
 Upgrade safety verified
 Bridge security checked
 Review report
 Issues addressed
Verification
`cargo test -p integration-tests`; CI `test.yml` green.
Source
Phase issue #273 from phase 6 issues.md (line 374)
Priority: High | Scope: M