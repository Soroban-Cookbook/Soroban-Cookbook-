# CHANGELOG Entry for Issue #1099

## [Unreleased]

### Changed

#### Advanced Examples Reorganization (#1099)
Resolved prefix collision in advanced examples directory where 8 unrelated examples shared the "05-" prefix. Each example now has a unique identifier for clearer navigation and learning progression.

**Directory Renames:**
- `05-batch-transfer` → `20-batch-transfer`
- `05-bridge-security` → `17-bridge-security`
- `05-diamond-facets` → `18-diamond-facets`
- `05-diamond-security` → `19-diamond-security`
- `05-hierarchical-access-control` → `16-hierarchical-access-control`
- `05-merkle-proofs` → `21-merkle-proofs`
- `05-reentrancy-guard` → `15-reentrancy-guard`

**Impact:**
- All documentation and code references updated
- Cargo workspace configuration updated
- Added numbering explanation section to `examples/advanced/README.md`

**Breaking Change:** If you reference these examples in external projects, update your paths according to the mapping above.
