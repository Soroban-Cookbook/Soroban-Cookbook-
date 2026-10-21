# NFT Best Practices on Soroban

This guide summarizes practical choices for NFT contracts and integrations on
Soroban. The examples in the [NFT patterns reference](./nft-patterns.md) and
[NFT examples](../examples/nfts.md) provide implementation details.

## Metadata Standards

Soroban does not impose one universal NFT metadata schema. Choose a schema that
your target wallets and marketplaces support, and keep its fields consistent
across the collection. A common JSON shape includes `name`, `description`,
`image`, and `attributes`; use optional fields such as `external_url` and
`animation_url` only when consumers understand them.

- Validate required fields, URI schemes, and attribute types when metadata is
  created or updated.
- Prefer content-addressed storage such as IPFS for metadata that should not
  change. HTTPS endpoints are useful when updates are intentional, but their
  content can change independently of the contract.
- Document whether metadata is immutable, who can update it, and how clients
  discover the canonical URI.
- Avoid putting large images or JSON blobs in contract storage unless the
  verifiability benefit justifies the storage and rent costs.

## Gas and Storage Efficiency

- Store only the ownership and metadata fields the contract needs to enforce or
  query. Keep large descriptive content off-chain where possible.
- Choose storage tiers and TTL policies deliberately. Ownership and other
  durable records need persistent storage and appropriate TTL extension.
- Avoid redundant indexes unless on-chain enumeration is required. If using
  per-owner indexes, use a dense removal strategy such as swap-remove.
- Batch related reads and writes where the interface permits, and measure
  resource usage with representative collection sizes before optimizing.
- Emit compact, consistent events so indexers can track mints, transfers, and
  marketplace activity without repeatedly querying contract state.

## Marketplace Integration

- Confirm the marketplace's expected token interface, metadata fields, event
  topics, and supported URI schemes before deploying.
- Require authorization from the acting owner or approved operator for every
  transfer, and clear single-token approvals after ownership changes.
- Make listing, payment, and NFT transfer settlement atomic where possible; a
  failed leg must not leave funds or assets in an inconsistent state.
- Validate listing state and price at purchase time rather than trusting cached
  marketplace data. Emit events for listing creation, cancellation, bids, and
  completed sales.
- Test against the actual marketplace or a compatible test integration,
  including expired listings, failed transfers, and insufficient payment.

## Royalties

Royalties are marketplace behavior, not a universal guarantee of NFT transfers.
A buyer can transfer a token outside a marketplace that honors royalty terms.

- Store a clearly defined recipient and rate, expressed in basis points, and
  enforce a documented maximum (for example, no more than 10,000 basis points).
- Calculate the royalty from the sale price using integer arithmetic, then
  transfer the remainder to the seller. Test rounding and boundary values.
- Apply the same rule on every marketplace settlement path, including bundles
  and auctions, and make the royalty visible to buyers before purchase.
- Do not describe royalties as enforced on all transfers unless the design
  actually prevents transfers that bypass the royalty logic.

## Review Checklist

- [ ] Metadata schema and mutability policy are documented.
- [ ] Durable ownership state uses suitable storage and TTL handling.
- [ ] Authorization, approval cleanup, and transfer events are tested.
- [ ] Marketplace settlement handles failure atomically.
- [ ] Royalty limits, rounding, and bypass limitations are explicit.