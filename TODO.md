# Documentation Site Setup TODO

## Status: In Progress (Steps 1-2 ✅)

### 1. [✅] Migrate top-level duplicates to book/src/
   - Content already synced/identical (docs/, guides/)
   - No migration needed

### 2. [✅] Populate example summaries in book/src/examples/*.md
   - basics.md: Detailed 11-example listing with code snippets
   - intermediate.md: Multisig focus + placeholders
   - advanced.md: 2 examples (multiparty, timelock) + planned
   - defi/nfts/governance/tokens.md: Forward-looking placeholders matching SUMMARY

### 3. [ ] Update book.toml (step 3)
   - Add table-of-contents, copy-button, theme tweaks

### 4. [ ] Refine SUMMARY.md (step 4)
   - Expand examples/ sections with subdirs
   - Verify all links resolve

### 5. [ ] Enhance scripts/ (step 5)
   - Extend build.sh for `mdbook build/serve`
   - Create DOCS_SETUP.md at root

### 6. [ ] Test locally (step 6)
   - `cd book && mdbook serve`

### 7. [ ] Deploy/Validate (step 7)

### 8. [ ] Cleanup/Finalize (step 8)

