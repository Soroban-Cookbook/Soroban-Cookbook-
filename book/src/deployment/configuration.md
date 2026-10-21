# Deployment Configuration

## GitHub Pages Settings

The documentation is deployed to GitHub Pages with the following configuration:

- **Source**: GitHub Actions
- **Custom domain**: Configured in the repository settings
- **SSL**: Enforce HTTPS enabled

## Environment Variables

The workflow uses the following secrets and variables:

- `GITHUB_TOKEN`: Automatically provided by GitHub Actions
- No additional secrets required for basic deployment

## Build Configuration

The mdBook is built with the following settings:

- **Output directory**: `book/book`
- **Build command**: `mdbook build`
- **Source directory**: `book/src`

## Troubleshooting

### Build Failures

If the build fails, check:
1. All markdown files are properly formatted
2. All internal links are valid
3. The SUMMARY.md file is correctly structured

### Deployment Issues

If deployment fails:
1. Verify GitHub Pages is enabled in repository settings
2. Check that the workflow has write permissions
3. Ensure the custom domain DNS is properly configured