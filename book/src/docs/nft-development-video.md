# NFT Development Video Walkthrough

This page is the production script and companion guide for a 15–20 minute video that builds an NFT on Soroban, adds metadata, connects it to a marketplace, and reviews the security and interoperability decisions behind the implementation.

## Video

**Status:** Ready to record. Replace the placeholder below with the hosted video URL after upload.

**Watch:** [NFT Development on Soroban](https://www.youtube.com/watch?v=VIDEO_ID) (15–20 minutes)

## Tutorial Outline

### 1. Contract and Metadata Setup (0:00–5:00)

- Start with [`examples/nfts/01-basic-nft/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/nfts/01-basic-nft).
- Show the owner, token identifier, approval, and enumeration model.
- Mint one token using authenticated admin and recipient addresses.
- Move to [`examples/nfts/02-nft-metadata/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/nfts/02-nft-metadata).
- Explain `name`, `symbol`, `token_uri`, `base_uri`, and the choice between on-chain and off-chain metadata.

### 2. Create and Inspect an NFT (5:00–9:00)

Demonstrate the test-driven flow:

```bash
cd examples/nfts/02-nft-metadata
cargo test
```

Then show a metadata document containing a stable name, description, image URL, and attributes. Explain that an off-chain URI should be immutable or content-addressed when the token promises durable provenance, and that the contract should reject malformed or unauthorized updates.

### 3. Marketplace Integration (9:00–14:00)

- Open [`examples/nfts/04-nft-marketplace/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/nfts/04-nft-marketplace).
- Create a listing with an asking price and expiration.
- Verify ownership and approval before a sale.
- Buy the NFT and show the atomic transfer of the asset and payment.
- Demonstrate auction, bundle, and royalty paths where supported by the example.
- Explain why a marketplace should never trust a cached owner, stale approval, or client-supplied seller address.

### 4. Best Practices and Testing (14:00–18:00)

- Require authorization for minting, transfers, approvals, listings, and administrative metadata changes.
- Check ownership and approval immediately before settlement.
- Use checked arithmetic for prices, royalties, and quantities; reject zero or expired listings.
- Prevent replay by making a listing single-use and recording its final state before external calls where the contract design requires it.
- Test unauthorized minting, duplicate token IDs, invalid metadata, expired listings, insufficient payment, royalty limits, and failed transfers.
- Extend storage TTL for long-lived token and listing records.

### 5. Recap and Resources (18:00–20:00)

Summarize the path from minting to marketplace settlement, then point viewers to the [NFT Patterns Reference](./nft-patterns.md), [NFT examples overview](../examples/nfts.md), and the repository test suites.

## Recording Checklist

- [ ] Run the three NFT example test suites on the commit being recorded.
- [ ] Record at 1080p with readable terminal text and captions enabled.
- [ ] Upload the final 15–20 minute video to the Soroban Cookbook channel.
- [ ] Replace `VIDEO_ID` above with the published URL and verify it opens.
- [ ] Check every source link and timestamp before publishing.