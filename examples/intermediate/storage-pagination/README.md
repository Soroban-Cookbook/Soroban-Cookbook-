# Storage Pagination

Cursor-based pagination for large on-chain collections, enabling scalable reads without exceeding instruction limits.

## What It Demonstrates

- Cursor-based pagination pattern (more gas-efficient than offset-based)
- Stable cursor encoding using indices
- Page boundaries and consistent ordering
- Batch retrieval with configurable page size
- No instruction limit exceeded scenarios
- Empty page and edge case handling

## Use Cases

- Paginating large collections of tokens, users, or items
- Providing efficient batch data retrieval
- Supporting frontend pagination UI without loading all items
- Working with collections that exceed single-transaction limits

## Key Concepts

The contract stores items sequentially and uses index-based cursors:
- **Cursor**: Base64-encoded index pointing to the next item to fetch
- **Page Size**: Maximum items returned per call (prevents exceeding gas limits)
- **Stable Cursors**: Cursors remain valid across contract executions
- **Empty Cursor**: `None` or empty string indicates end of collection

This pattern avoids the inefficiency of offset-based pagination (which requires skipping items) and provides stable, resumable cursors.
