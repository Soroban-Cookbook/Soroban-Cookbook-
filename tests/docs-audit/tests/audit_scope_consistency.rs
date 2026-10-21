//! Regression tests for the security-audit-prep documentation in
//! `docs/security-audit/`. These do not touch any contract code; they check
//! that the audit-prep documents stay in sync with the actual
//! `examples/intermediate` and `examples/tokens` directory trees, which is
//! exactly the kind of drift that made the original scope/checklist stale.

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // tests/
        .and_then(Path::parent) // repo root
        .expect("tests/docs-audit is expected to be two levels below the repo root")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

/// Directory names under `examples/<category>/` that contain a `Cargo.toml`,
/// i.e. are buildable example crates rather than plain markdown/asset dirs.
fn example_dir_names(category: &str) -> Vec<String> {
    let root = repo_root().join("examples").join(category);
    let mut names: Vec<String> = fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", root.display()))
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter(|entry| entry.path().join("Cargo.toml").exists())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn intermediate_examples_are_listed_in_audit_scope() {
    let scope = read("docs/security-audit/audit-scope.md");
    let missing: Vec<String> = example_dir_names("intermediate")
        .into_iter()
        .filter(|name| !scope.contains(&format!("`{name}`")))
        .collect();
    assert!(
        missing.is_empty(),
        "examples/intermediate/* not referenced in docs/security-audit/audit-scope.md: \
         {missing:?}. Add them to the in-scope table so the audit scope stays current."
    );
}

#[test]
fn token_examples_are_listed_in_audit_scope() {
    let scope = read("docs/security-audit/audit-scope.md");
    let missing: Vec<String> = example_dir_names("tokens")
        .into_iter()
        .filter(|name| !scope.contains(&format!("`{name}`")))
        .collect();
    assert!(
        missing.is_empty(),
        "examples/tokens/* not referenced in docs/security-audit/audit-scope.md: \
         {missing:?}. Add them to the in-scope table so the audit scope stays current."
    );
}

#[test]
fn intermediate_and_token_examples_are_listed_in_prep_checklist() {
    let checklist = read("docs/security-audit/audit-prep-checklist.md");
    let mut missing = Vec::new();
    for category in ["intermediate", "tokens"] {
        for name in example_dir_names(category) {
            if !checklist.contains(&format!("`{name}`")) {
                missing.push(format!("{category}/{name}"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "examples not referenced in docs/security-audit/audit-prep-checklist.md: {missing:?}"
    );
}

#[test]
fn examples_missing_a_readme_are_recorded_in_known_issues_log() {
    let known_issues = read("docs/security-audit/known-issues-log.md");
    let mut undocumented_gaps = Vec::new();
    for category in ["intermediate", "tokens"] {
        let category_root = repo_root().join("examples").join(category);
        for name in example_dir_names(category) {
            let has_readme = category_root.join(&name).join("README.md").exists();
            let logged = known_issues.contains(&format!("examples/{category}/{name}"));
            if !has_readme && !logged {
                undocumented_gaps.push(format!("{category}/{name}"));
            }
        }
    }
    assert!(
        undocumented_gaps.is_empty(),
        "in-scope examples with no README.md are not tracked in \
         docs/security-audit/known-issues-log.md: {undocumented_gaps:?}. \
         Either add a README or record the gap as a known issue."
    );
}
