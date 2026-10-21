# CI/CD Setup for Documentation

This guide explains how the automated documentation deployment pipeline works.

## Overview

The documentation deployment uses GitHub Actions to automatically build and deploy the mdBook documentation whenever changes are pushed to the repository.

## Workflow Triggers

The deployment workflow is triggered by:
- Pushes to the `main` branch (production deployment)
- Pull requests targeting the `main` branch (preview deployment)

## Workflow Steps

1. **Checkout**: Clones the repository
2. **Install mdBook**: Sets up the mdBook tool
3. **Build Documentation**: Runs `mdbook build` in the `book/` directory
4. **Deploy Preview**: For PRs, deploys to a preview URL
5. **Deploy Production**: For main branch, deploys to the production URL

## Preview Deployments

Pull request previews allow you to review documentation changes before they go live. Each PR gets a unique preview URL that updates as you push changes.

## Production Deployment

When changes are merged to the `main` branch, the documentation is automatically deployed to the production URL with a custom domain and SSL certificate.

## Custom Domain Configuration

The documentation is served from a custom domain configured in the GitHub Pages settings. The SSL certificate is automatically provisioned and managed by GitHub Pages.

## Verification

To verify the deployment works locally:
