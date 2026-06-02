# Contract Registry Example

This example implements a small on-chain contract registry for discovery and metadata lookup.

Features:
- Store `name`, `category`, `version`, and `address` for registered contracts.
- Index names by category and expose query APIs for off-chain tooling.

Example usage (client calls):

- `register(name, category, version, address)` — register an entry.
- `get_by_name(name)` — retrieve metadata for a name.
- `list_by_category(category)` — returns a `Vec<Symbol>` of names in that category.
- `list_categories()` — list known categories.

See `src/lib.rs` and `src/test.rs` for implementation and tests.
