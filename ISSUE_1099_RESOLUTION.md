# Issue #1099: Advanced Example Prefix Collision Resolution

## Problem Summary

Eight unrelated advanced example directories share the "05-" prefix:
- `05-batch-transfer`
- `05-bridge-security`
- `05-diamond-facets`
- `05-diamond-security`
- `05-hierarchical-access-control`
- `05-merkle-proofs`
- `05-rate-limiting`
- `05-reentrancy-guard`

This creates ambiguity for learners following numbered guides, as they cannot determine which example logically follows another.

## Root Cause Analysis

The numbering system appears to have broken down as examples were added over time without coordinating the prefix numbers. There's no clear documentation stating whether prefixes are meant to be sequential learning paths or topical groupings.

## Proposed Solution

### Option 1: Unique Sequential Numbering (Recommended)

Establish unique prefixes within `examples/advanced/` and document the intended learning progression. This involves:

1. **Renumbering the conflicting directories** to establish a logical progression
2. **Updating all references** in documentation and links
3. **Adding a note in README.md** explaining the numbering scheme

#### Suggested Renumbering Based on Topic Grouping:

**Security & Access Control Group:**
- `05-rate-limiting` (foundational security primitive)
- `15-reentrancy-guard` (security primitive)
- `16-hierarchical-access-control` (advanced RBAC building on simpler auth patterns)

**Bridge Security Group:**
- `17-bridge-security` (comprehensive bridge security controls)

**Diamond Pattern Group:**
- `18-diamond-facets` (introduction to diamond pattern)
- `19-diamond-security` (securing diamond implementations)

**Batch & Optimization Group:**
- `20-batch-transfer` (optimization technique)

**Cryptographic Primitives:**
- `21-merkle-proofs` (foundational crypto primitive)

### Option 2: Topic-Based Prefixes (Alternative)

Replace numeric prefixes with topic-based prefixes:
- `security-rate-limiting`
- `security-reentrancy-guard`
- `access-hierarchical-control`
- `bridge-security`
- `diamond-facets`
- `diamond-security`
- `batch-transfer`
- `crypto-merkle-proofs`

**Pros:** More descriptive, eliminates ordering confusion
**Cons:** Breaks existing links, changes established patterns, larger migration

### Option 3: Document Non-Sequential Nature (Minimal Change)

Add clear documentation stating that prefixes are not sequential and represent topical groupings rather than learning paths.

**Update `examples/advanced/README.md`:**
```markdown
## Directory Organization

⚠️ **Note on Numbering:** Directory prefixes in this section are **not sequential**. 
Multiple examples may share the same prefix number, indicating they are related topics 
or were developed in parallel. The prefix does not imply a required learning order.

Use the table below and individual README files to determine dependencies and 
recommended progression paths.
```

## Recommended Approach

**Implement Option 1 (Unique Sequential Numbering)** for the following reasons:

1. **Clearer learning paths** - Learners can follow a logical progression
2. **Reduced confusion** - Each example has a unique identifier
3. **Better organization** - Related topics can be grouped numerically
4. **Searchability** - Easier to reference specific examples
5. **Consistency** - Aligns with basic and intermediate examples

## Implementation Steps

1. Create mapping of old → new names
2. Use `smart_relocate` to rename directories (auto-updates imports)
3. Update all documentation references:
   - `examples/advanced/README.md`
   - `book/src/examples/advanced.md`
   - Any internal cross-references
4. Update CI/CD workflows if they reference specific directories
5. Add migration note to CHANGELOG

## Files Requiring Updates

### Documentation Files
- `examples/advanced/README.md` - Update all directory links
- `book/src/examples/advanced.md` - Update example listings and links
- Any steering files or guides referencing these examples

### Potential Code References
- Test configurations
- Build scripts
- CI/CD workflows (`.github/workflows/*.yml`)

## Migration Commands (Example)

```powershell
# Example renumbering (Option 1)
# Security primitives
git mv examples/advanced/05-rate-limiting examples/advanced/05-rate-limiting-temp
git mv examples/advanced/05-reentrancy-guard examples/advanced/15-reentrancy-guard
git mv examples/advanced/05-hierarchical-access-control examples/advanced/16-hierarchical-access-control

# Bridge security
git mv examples/advanced/05-bridge-security examples/advanced/17-bridge-security

# Diamond pattern
git mv examples/advanced/05-diamond-facets examples/advanced/18-diamond-facets
git mv examples/advanced/05-diamond-security examples/advanced/19-diamond-security

# Optimization
git mv examples/advanced/05-batch-transfer examples/advanced/20-batch-transfer

# Crypto primitives
git mv examples/advanced/05-merkle-proofs examples/advanced/21-merkle-proofs

# Finalize rate-limiting
git mv examples/advanced/05-rate-limiting-temp examples/advanced/05-rate-limiting
```

## Testing Checklist

- [ ] All build scripts run successfully
- [ ] All tests pass with new directory names
- [ ] Documentation links are valid (no 404s)
- [ ] GitHub Actions workflows complete
- [ ] Book builds and renders correctly
- [ ] No broken internal references

## Communication Plan

1. **PR Description** - Explain the rationale and migration
2. **CHANGELOG Entry** - Document the directory renames
3. **Migration Guide** - For external projects referencing these examples
4. **Deprecation Notice** - Consider keeping redirects or notes about old paths

## Related Issues

This issue overlaps with:
- Diamond pattern topic-group issues (mentions two of these directories)
- RBAC/access control organization
- Overall advanced examples documentation structure

Resolving #1099 will establish a foundation for addressing those related issues.
