# Dependabot Configuration Guide

This document describes the Dependabot setup for the Soroban Cookbook repository.

## Overview

Dependabot automatically checks for outdated dependencies and creates pull requests to update them. This ensures the cookbook stays current with the latest stable versions of Rust crates and GitHub Actions.

## Configuration

### Rust/Cargo Ecosystem

- **Schedule:** Weekly on Mondays at 09:00 UTC
- **Update Types:** All updates (major, minor, patch)
- **Grouping:** Minor and patch updates are grouped into a single PR
- **Labels:** `dependencies`, `rust`
- **Commit Prefix:** `chore`
- **Open PR Limit:** 10

### GitHub Actions

- **Schedule:** Weekly on Mondays at 09:00 UTC
- **Update Types:** All updates
- **Labels:** `dependencies`, `github-actions`
- **Commit Prefix:** `ci`
- **Open PR Limit:** 5

## Auto-Merge Workflow

The `.github/workflows/dependabot-auto-merge.yml` workflow automatically merges:
- Minor version updates (e.g., 1.2.0 → 1.3.0)
- Patch version updates (e.g., 1.2.0 → 1.2.1)

Major version updates require manual review and approval.

## Files

- `.github/dependabot.yml` - Main Dependabot configuration
- `.github/workflows/dependabot-auto-merge.yml` - Auto-merge workflow

## Maintenance

Dependabot PRs are automatically created and merged based on the configuration above. Monitor the repository for any dependency-related issues and adjust the schedule or grouping as needed.
