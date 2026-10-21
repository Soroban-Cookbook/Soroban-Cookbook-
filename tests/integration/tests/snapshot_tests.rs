#[test]
fn snapshot_contract_registry_summary() {
    let summary = "contract_registry_summary:\n  examples: 81\n  snapshot_suite: integration-tests\n  target: wasm32v1-none\n  state_model: ledger + contract storage\n";

    insta::assert_snapshot!(summary, @r###"
    contract_registry_summary:
      examples: 81
      snapshot_suite: integration-tests
      target: wasm32v1-none
      state_model: ledger + contract storage
    "###);
}

#[test]
fn snapshot_wasm_output_stub() {
    let wasm_summary = "wasm_snapshot:\n  artifact: contract.wasm\n  format: soroban-wasm\n  verification: cargo build --target wasm32v1-none --release\n";

    insta::assert_snapshot!(wasm_summary, @r###"
    wasm_snapshot:
      artifact: contract.wasm
      format: soroban-wasm
      verification: cargo build --target wasm32v1-none --release
    "###);
}

#[test]
fn snapshot_state_layout_stub() {
    let state_summary = "state_snapshot:\n  store: instance + persistent + temporary\n  validation: env.dump_state()\n  ci_gate: cargo test -p integration-tests\n";

    insta::assert_snapshot!(state_summary, @r###"
    state_snapshot:
      store: instance + persistent + temporary
      validation: env.dump_state()
      ci_gate: cargo test -p integration-tests
    "###);
}
