# Translation Infrastructure

The Soroban Cookbook uses a structured translation process so that the English documentation remains the source of truth while localized content can be introduced safely over time.

## Selected Translation System

We recommend using Crowdin as the translation management platform for this repository.

Why Crowdin fits the project:

- GitHub-friendly workflows for pull requests and review
- Support for glossary and terminology consistency
- Clear translation status tracking for each locale
- A low-friction path for contributors who want to localize docs without changing the main build process

## i18n Framework Configuration

The repository now includes a Crowdin configuration file at [.crowdin.yml](../../.crowdin.yml) to define the translation source and target layout for the mdBook content.

The current configuration targets the Markdown sources under the mdBook tree so that translations can be managed independently from the build output.

## Translation Workflow

1. Update the English source content in the mdBook source tree under `book/src/`.
2. Open or update a pull request for the change.
3. Upload the new or modified strings to Crowdin for translation.
4. Review translated content and resolve any terminology or formatting issues.
5. Merge approved translations back into the repository and run the documentation build.

## Quality Assurance Process

All translation work should follow the same quality checks as regular documentation updates:

- Run `cd book && mdbook build` after content changes.
- Verify that links and relative references still resolve correctly.
- Review translated strings for technical accuracy and tone.
- Ensure terminology remains consistent with the glossary and existing docs.

## Documentation Ownership

- English content remains maintained by repository contributors.
- Locale maintainers review translations for their language.
- New pages should be added to the mdBook summary so they remain discoverable in both source and translated builds.

This setup gives the repository a practical foundation for translation work while keeping the existing mdBook workflow intact.
