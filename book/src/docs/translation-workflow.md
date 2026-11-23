# Translation Workflow

This page defines the end-to-end process for maintaining translations in the Soroban Cookbook. English is the source of truth; all localized content derives from it.

For the high-level platform overview, see [Translation Infrastructure](./translation-infrastructure.md).

---

## Translation Guidelines

### Source of Truth

- English content under `book/src/` is canonical.
- Translations live in locale-specific subdirectories (e.g., `book/src/zh/` for Chinese).
- Never edit English source files from a translation PR — open a separate PR for source changes.

### What to Translate

| Translate | Do Not Translate |
|---|---|
| Prose, headings, and descriptions | Code blocks and inline code |
| Table cell text (descriptions, labels) | Code identifiers, function names, struct names |
| Frontmatter `title` and `description` | Frontmatter keys, YAML keys |
| Navigation labels in `SUMMARY.md` | File paths and link targets |
| Glossary definitions (translate the definition, keep the term) | Glossary terms themselves |

### Terminology Consistency

- Maintain a per-locale glossary so technical terms are translated consistently.
- Keep blockchain-specific terms stable across pages (e.g., "ledger", "entry", "instance" — decide whether to translate or transliterate, then apply everywhere).
- Cross-reference the English [Glossary](./glossary.md) when unsure about a term's meaning.

### Formatting Rules

- Preserve all Markdown structure: headings, lists, tables, blockquotes.
- Keep relative links pointing to the same file paths. If a translated page links to another page, the link target stays in English unless a translated version of that target exists.
- Preserve code fences and their language annotations.
- Do not add or remove `---` frontmatter delimiters.

---

## Update Process

The translation cycle mirrors the documentation release cycle:

```
English source change → PR merged → Crowdin sync → Translators notified → Translation PR → Review → Merge → Build
```

### Step-by-Step

1. **Source update.** A contributor updates English content under `book/src/` and merges the PR via the normal review process.

2. **String upload.** After merge, updated strings are pushed to Crowdin. This happens automatically if the Crowdin GitHub integration is configured, or manually via the Crowdin CLI:
   ```bash
   crowdin upload sources
   ```

3. **Translator notification.** Crowdin notifies assigned translators for each locale that new or changed strings are available.

4. **Translation work.** Translators complete translations in the Crowdin editor or locally. Local workflow:
   ```bash
   # Download current translation files
   crowdin download --language zh

   # Edit the .md files in book/src/zh/

   # Upload completed translations
   crowdin upload translations --language zh
   ```

5. **PR creation.** Approved translations are exported from Crowdin and a PR is opened against the main repository. The PR should:
   - Scope changes to locale subdirectories only (e.g., `book/src/zh/`).
   - Include a summary of which pages were translated or updated.
   - Link to the source PR that introduced the changes.

6. **Review.** A locale maintainer reviews the PR for:
   - Technical accuracy
   - Terminology consistency with the locale glossary
   - Markdown structure preservation
   - Link validity

7. **Merge and build.** Once merged, the `deploy-docs.yml` GitHub Action rebuilds the mdBook site automatically.

### Handling Source Changes to Already-Translated Pages

When English source content changes after a page has been translated:

1. Crowdin marks the affected strings as "needs update" in the locale.
2. The translator reviews the diff and updates the translation.
3. The translator verifies that surrounding context still makes sense — a small source change can shift meaning in translation.

### Adding a New Language

1. Create a locale directory under `book/src/` using the ISO 639-1 code (e.g., `book/src/es/` for Spanish).
2. Copy `SUMMARY.md` from the English tree and translate the navigation labels. Keep all link paths unchanged.
3. Copy `README.md` and translate the introduction.
4. Configure the locale in `.crowdin.yml` if it is not already mapped.
5. Add the locale section to the English `SUMMARY.md` so the translated table of contents is discoverable:
   ```markdown
   # Español

   - [Introducción](./es/README.md)
   - [Índice](./es/SUMMARY.md)
   ```

---

## Translator Onboarding

### Prerequisites

- Fluency in the target language and English.
- Familiarity with Markdown.
- Basic understanding of Soroban / Stellar concepts (enough to translate technical docs accurately).
- A GitHub account and a Crowdin account (free tier is sufficient).

### Onboarding Steps

1. **Read the guidelines.** Review this page and the [Translation Infrastructure](./translation-infrastructure.md) overview.

2. **Join the Crowdin project.** Request access to the project on Crowdin. A maintainer will assign you to your locale.

3. **Set up the repository locally.**
   ```bash
   git clone https://github.com/Soroban-Cookbook/Soroban-Cookbook-.git
   cd Soroban-Cookbook-
   ```

4. **Install mdBook for local preview.**
   ```bash
   cargo install mdbook --locked
   cd book
   mdbook serve   # Preview at http://localhost:3000
   ```

5. **Install the Crowdin CLI (optional).**
   ```bash
   # Via npm
   npm install -g @crowdin/cli
   # Or download the standalone binary from https://crowdin.github.io/crowdin-cli/
   ```

6. **Start with a small page.** Pick a short page (e.g., `guides/getting-started.md`) and translate it as a test run. Submit via Crowdin or open a PR.

7. **Coordinate with your locale maintainer.** Ask questions about terminology decisions and review expectations before tackling larger pages.

### Translator Roles

| Role | Responsibilities |
|---|---|
| **Translator** | Translate strings, follow guidelines, maintain terminology consistency. |
| **Reviewer** | Review translation PRs for accuracy, tone, and formatting. |
| **Locale Maintainer** | Owns a locale: manages translators, approves terminology, merges translation PRs. |
| **Project Maintainer** | Owns the Crowdin project configuration, manages locale maintainers, oversees the overall translation process. |

---

## Quality Checks

Every translation PR must pass these checks before merging:

### Automated Checks

- **mdBook build.** The documentation must build without errors:
  ```bash
  cd book && mdbook build
  ```
  This catches broken links, malformed Markdown, and missing referenced files.

- **Link validation.** Verify that all relative links in translated pages resolve to existing files. mdBook reports broken links as warnings during build.

- **Typos / spell check.** Run the project's spell checker on translated content where applicable:
  ```bash
  typos
  ```

### Manual Review Checklist

- [ ] Technical terms are translated consistently with the locale glossary.
- [ ] Code blocks are untouched — no translated code, identifiers, or comments within code fences.
- [ ] Markdown structure (headings, lists, tables) matches the source page.
- [ ] Relative links point to valid paths.
- [ ] Frontmatter is preserved (keys in English, values translated where appropriate).
- [ ] No content added or removed — translation only, no editorial changes.
- [ ] Tone matches the source: instructional, concise, developer-focused.
- [ ] Page renders correctly in `mdbook serve` preview.

### Handling Ambiguity

If a source string is ambiguous or could be translated multiple ways:

1. Leave a comment in the Crowdin editor or PR discussion.
2. Tag the locale maintainer for a decision.
3. Record the decision in the locale glossary to prevent future recurrence.

---

## Documentation

### Key Files

| File | Purpose |
|---|---|
| `.crowdin.yml` | Crowdin project configuration: source patterns, target layout, file type |
| `book/src/docs/translation-infrastructure.md` | High-level platform overview and rationale |
| `book/src/docs/translation-workflow.md` | This page: detailed process, guidelines, onboarding |
| `book/src/zh/` | Example locale: Chinese translation tree |
| `book/src/docs/glossary.md` | English glossary — reference for terminology consistency |

### Crowdin Configuration

The current `.crowdin.yml` maps all Markdown files under `book/src/` as source content and writes translations to locale subdirectories:

```yaml
project_id: ""
api_token: ""
base_url: "https://api.crowdin.com"

files:
  - source: "/book/src/**"
    translation: "/book/src/%two_letters_code%/**"
    type: "md"
```

The `%two_letters_code%` placeholder uses the ISO 639-1 code for each locale (e.g., `zh` for Chinese, `es` for Spanish, `ja` for Japanese).

### mdBook Integration

Translated content is included in the mdBook build via locale sections in the root `SUMMARY.md`. Each locale has its own `SUMMARY.md` with translated navigation labels. The build output includes all locales in a single site, accessible from the sidebar.

### Deploy Pipeline

Translation PRs merged to `main` trigger the `deploy-docs.yml` GitHub Action, which rebuilds the site and deploys to GitHub Pages. No separate deployment step is needed for translations.
