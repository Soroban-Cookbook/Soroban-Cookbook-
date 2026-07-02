# Site Search

The Soroban Cookbook has full-text search built in via [mdBook's search feature](https://rust-lang.github.io/mdBook/guide/reading.html#search).

## How to Search

Press **`S`** or click the **magnifying glass** icon in the top-left toolbar to open the search bar.

Type any keyword, function name, concept, or code snippet — results appear instantly as you type.

## Search Features

### Full-Text Search

Every page, heading, and paragraph in the cookbook is indexed at build time. Queries match anywhere in the content.

### Code Search

Code blocks are included in the search index. You can search for:

- Soroban macros: `contractimpl`, `contracttype`, `symbol_short`
- Function names: `invoke_contract`, `get_ledger_timestamp`
- Error types: `ContractError`, `AuthError`
- Crate names: `soroban-sdk`, `stellar-xdr`

Example searches:

```
token transfer
invoke_contract auth
storage get put
```

### Search Suggestions (Auto-Expand)

The search engine automatically expands partial words. Searching for `token` also matches `tokenize`, `tokenomics`, etc. This is powered by the `expand = true` setting in `book.toml`.

### Boolean AND Mode

By default, all search terms must appear in a result (AND logic). A search for `storage persistent` only returns pages that contain **both** words.

### Result Ranking

Results are ranked by relevance using boost weights:

| Match location | Boost |
|---|---|
| Page title | 2× |
| Section heading | 1× |
| Paragraph text | 1× |

### Keyboard Navigation

| Key | Action |
|---|---|
| `S` | Open search |
| `↑` / `↓` | Move through results |
| `Enter` | Open highlighted result |
| `Escape` | Close search |

## Search Index

The search index (`searchindex.json`) is generated automatically during `mdbook build` and is stored alongside the HTML output. It contains:

- All page titles and headings
- Full paragraph text
- Code block contents
- Hierarchical section paths (used for breadcrumb display in results)

No external service is required — search runs entirely in the browser using the bundled JavaScript.

## Tips for Better Results

- Use **specific terms** — `pause_contract` finds more targeted results than `pause`
- Search for **error messages** you encounter to jump directly to troubleshooting content
- Search by **pattern name** — `factory`, `proxy`, `oracle`, `vault`
- Combine terms — `defi vault fee` narrows results across DeFi vault examples

## Configuration

Search is configured in `book/book.toml`:

```toml
[output.html.search]
enable = true
limit-results = 30
teaser-word-count = 30
use-boolean-and = true
boost-title = 2
boost-hierarchy = 1
boost-paragraph = 1
expand = true
heading-split-level = 3
copy-js = true
```

To rebuild the search index after adding new content:

```bash
cd book && mdbook build
```
