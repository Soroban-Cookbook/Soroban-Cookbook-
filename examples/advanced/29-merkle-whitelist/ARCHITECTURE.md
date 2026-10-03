# Architecture: Merkle Whitelist Contract

## System Architecture

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Merkle Whitelist System                          │
└────────────────────────────────────────────────────────────────────────┘
                                    │
                ┌───────────────────┼───────────────────┐
                │                   │                   │
                ▼                   ▼                   ▼
        ┌───────────────┐   ┌───────────────┐  ┌──────────────┐
        │   Off-Chain   │   │   On-Chain    │  │  Frontend    │
        │   Services    │   │   Contract    │  │  Integration │
        └───────────────┘   └───────────────┘  └──────────────┘
```

## Component Breakdown

### 1. Off-Chain Services

```
┌─────────────────────────────────────────┐
│         Off-Chain Services              │
├─────────────────────────────────────────┤
│                                         │
│  ┌────────────────────────────────┐   │
│  │   Merkle Tree Generator        │   │
│  │   - Build tree from whitelist  │   │
│  │   - Generate proofs            │   │
│  │   - Distribute to users        │   │
│  └────────────────────────────────┘   │
│                                         │
│  ┌────────────────────────────────┐   │
│  │   Proof Distribution API       │   │
│  │   - REST/GraphQL endpoint      │   │
│  │   - User authentication        │   │
│  │   - Rate limiting              │   │
│  └────────────────────────────────┘   │
│                                         │
│  ┌────────────────────────────────┐   │
│  │   Monitoring & Alerts          │   │
│  │   - Dispute notifications      │   │
│  │   - Governance events          │   │
│  │   - Fee collection tracking    │   │
│  └────────────────────────────────┘   │
│                                         │
└─────────────────────────────────────────┘
```

### 2. On-Chain Contract

```
┌─────────────────────────────────────────────────────────────────┐
│                    Merkle Whitelist Contract                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────┐  ┌──────────────────┐  ┌─────────────┐  │
│  │  Core Whitelist  │  │   Governance     │  │  Disputes   │  │
│  │  - verify()      │  │   - propose()    │  │  - submit() │  │
│  │  - is_listed()   │  │   - vote()       │  │  - vote()   │  │
│  │  - get_entry()   │  │   - execute()    │  │  - resolve()│  │
│  └──────────────────┘  └──────────────────┘  └─────────────┘  │
│                                                                 │
│  ┌──────────────────┐  ┌──────────────────┐  ┌─────────────┐  │
│  │   Fee Manager    │  │  Role Manager    │  │  Security   │  │
│  │  - collect()     │  │   - add_role()   │  │  - pause()  │  │
│  │  - set_fee()     │  │   - has_role()   │  │  - limit()  │  │
│  │  - waiver()      │  │   - remove()     │  │  - blacklist│  │
│  └──────────────────┘  └──────────────────┘  └─────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 3. Storage Architecture

```
┌───────────────────────────────────────────────────────────────┐
│                      Storage Layers                            │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  Instance Storage (Persistent, Never Expires)       │    │
│  │  • Admin address                                     │    │
│  │  • Merkle root hash                                  │    │
│  │  • Fee configuration                                 │    │
│  │  • Governance config                                 │    │
│  │  • Role mappings                                     │    │
│  │  • Blacklist                                         │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  Persistent Storage (User-Managed TTL)              │    │
│  │  • Whitelist entries (with metadata)                │    │
│  │  • Nonces (replay protection)                       │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  Temporary Storage (Auto-Expires)                   │    │
│  │  • Votes (proposal & dispute)                       │    │
│  │  • Rate limit counters                              │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

## Data Flow Diagrams

### Whitelist Verification Flow

```
┌─────────┐                                           ┌──────────┐
│  User   │                                           │ Contract │
└────┬────┘                                           └────┬─────┘
     │                                                      │
     │  1. Request proof from API                          │
     │ ─────────────────────────────────────────────────▶  │
     │                                                      │
     │  2. Receive proof                                   │
     │ ◀─────────────────────────────────────────────────  │
     │                                                      │
     │  3. Submit verify_whitelist(proof, metadata)        │
     │ ─────────────────────────────────────────────────▶  │
     │                                                      │
     │                          4. Check blacklist ────┐   │
     │                                                  │   │
     │                          5. Check rate limit ◀──┘   │
     │                                                  │   │
     │                          6. Verify proof ◀───────┘   │
     │                                                  │   │
     │                          7. Collect fee ◀────────┘   │
     │                                                  │   │
     │                          8. Store entry ◀────────┘   │
     │                                                      │
     │  9. Success response                                │
     │ ◀─────────────────────────────────────────────────  │
     │                                                      │
```

### Governance Flow

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│Governor 1│    │Governor 2│    │Governor 3│    │ Contract │
└─────┬────┘    └─────┬────┘    └─────┬────┘    └────┬─────┘
      │               │               │               │
      │  1. Propose root update                       │
      │ ─────────────────────────────────────────────▶│
      │                                                │
      │               │               │  2. Create proposal
      │               │               │               │
      │  3. Vote FOR                                  │
      │ ─────────────────────────────────────────────▶│
      │               │               │               │
      │               │  4. Vote FOR                  │
      │               │ ─────────────────────────────▶│
      │               │               │               │
      │               │               │  5. Vote FOR  │
      │               │               │ ─────────────▶│
      │               │               │               │
      │               │               │  6. Check quorum
      │               │               │               │
      │               │               │  7. Wait timelock
      │               │               │               │
      │  8. Execute proposal                          │
      │ ─────────────────────────────────────────────▶│
      │               │               │               │
      │               │               │  9. Update root
      │               │               │               │
      │  10. Success                                  │
      │ ◀─────────────────────────────────────────────│
      │               │               │               │
```

### Dispute Resolution Flow

```
┌───────────┐    ┌───────────┐    ┌───────────┐    ┌──────────┐
│Validator 1│    │Validator 2│    │Validator 3│    │ Contract │
└─────┬─────┘    └─────┬─────┘    └─────┬─────┘    └────┬─────┘
      │                │                │                │
      │  1. Submit dispute (target + evidence)          │
      │ ───────────────────────────────────────────────▶│
      │                │                │                │
      │                │                │  2. Collect fee
      │                │                │                │
      │                │                │  3. Create dispute
      │                │                │                │
      │  4. Vote: Invalid                               │
      │ ───────────────────────────────────────────────▶│
      │                │                │                │
      │                │  5. Vote: Invalid              │
      │                │ ───────────────────────────────▶│
      │                │                │                │
      │                │                │  6. Vote: Valid│
      │                │                │ ───────────────▶│
      │                │                │                │
      │                │                │  7. Wait period│
      │                │                │                │
      │  8. Resolve dispute                             │
      │ ───────────────────────────────────────────────▶│
      │                │                │                │
      │                │                │  9. Count votes
      │                │                │     (2 invalid,
      │                │                │      1 valid)  │
      │                │                │                │
      │                │                │  10. Revoke entry
      │                │                │                │
      │                │                │  11. Blacklist
      │                │                │                │
      │  12. Dispute resolved                           │
      │ ◀───────────────────────────────────────────────│
      │                │                │                │
```

## Security Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Security Layer Stack                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Layer 7: Application Security                                  │
│  ┌───────────────────────────────────────────────────────┐    │
│  │ • Input validation    • Error handling                │    │
│  │ • Business logic      • State management              │    │
│  └───────────────────────────────────────────────────────┘    │
│                                                                 │
│  Layer 6: Access Control                                        │
│  ┌───────────────────────────────────────────────────────┐    │
│  │ • Role-based permissions  • Multi-sig support         │    │
│  │ • Authorization checks    • Admin key protection      │    │
│  └───────────────────────────────────────────────────────┘    │
│                                                                 │
│  Layer 5: Governance Security                                   │
│  ┌───────────────────────────────────────────────────────┐    │
│  │ • Timelock enforcement    • Quorum requirements       │    │
│  │ • Vote validation         • Proposal integrity        │    │
│  └───────────────────────────────────────────────────────┘    │
│                                                                 │
│  Layer 4: Economic Security                                     │
│  ┌───────────────────────────────────────────────────────┐    │
│  │ • Fee mechanisms          • Dispute bonds             │    │
│  │ • Rate limiting           • Spam prevention           │    │
│  └───────────────────────────────────────────────────────┘    │
│                                                                 │
│  Layer 3: Cryptographic Security                                │
│  ┌───────────────────────────────────────────────────────┐    │
│  │ • Merkle proof verification  • SHA-256 hashing        │    │
│  │ • Nonce-based replay protection                       │    │
│  └───────────────────────────────────────────────────────┘    │
│                                                                 │
│  Layer 2: Storage Security                                      │
│  ┌───────────────────────────────────────────────────────┐    │
│  │ • TTL management          • Data persistence          │    │
│  │ • Storage quotas          • Backup strategies         │    │
│  └───────────────────────────────────────────────────────┘    │
│                                                                 │
│  Layer 1: Emergency Controls                                    │
│  ┌───────────────────────────────────────────────────────┐    │
│  │ • Circuit breaker (pause)  • Blacklist               │    │
│  │ • Admin override           • Recovery procedures      │    │
│  └───────────────────────────────────────────────────────┘    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Integration Patterns

### Pattern 1: Token Gating

```
Application Layer
       │
       ▼
┌──────────────────┐
│  Check Whitelist │──── No ────▶ Reject Access
│     Contract     │
└──────────────────┘
       │
       │ Yes
       ▼
┌──────────────────┐
│  Grant Access to │
│  Token Features  │
└──────────────────┘
```

### Pattern 2: Tiered Service

```
User Request
       │
       ▼
┌──────────────────┐
│  Query Whitelist │
│  Get Metadata    │
└──────────────────┘
       │
       ▼
┌──────────────────┐
│  Parse Tier Info │
└──────────────────┘
       │
       ├──── Tier 1 ───▶ Basic Features
       ├──── Tier 2 ───▶ Premium Features
       └──── Tier 3 ───▶ Enterprise Features
```

### Pattern 3: Dynamic Updates

```
Off-Chain Process
       │
       ▼
┌──────────────────┐
│  Build New Tree  │
└──────────────────┘
       │
       ▼
┌──────────────────┐
│ Propose Update   │──┐
└──────────────────┘  │
                      │
    ┌─────────────────┘
    │
    ▼
┌──────────────────┐
│  Governors Vote  │
└──────────────────┘
    │
    ▼
┌──────────────────┐
│ Execute After    │
│    Timelock      │
└──────────────────┘
    │
    ▼
┌──────────────────┐
│  Root Updated    │
│ Notify Users     │
└──────────────────┘
```

## Merkle Tree Structure

```
                        Root Hash
                      /           \
                    /               \
                  H12                H34
                /    \              /    \
              /        \          /        \
            H1          H2      H3          H4
           /  \        /  \    /  \        /  \
          L1  L2      L3  L4  L5  L6      L7  L8

L1-L8: Leaf hashes (whitelist entries)
H1-H4: Intermediate hashes
H12, H34: Parent hashes
Root: Final Merkle root stored on-chain

Proof for L1: [L2, H2, H34]
Verification: hash(hash(hash(L1, L2), H2), H34) == Root
```

## Deployment Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Production Deployment                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌────────────────────────────────────────────────────┐   │
│  │              Frontend (React/Next.js)              │   │
│  │  • User interface   • Proof display                │   │
│  │  • Wallet connect   • Transaction signing          │   │
│  └────────────────────────────────────────────────────┘   │
│                           │                                 │
│                           │ HTTPS                           │
│                           ▼                                 │
│  ┌────────────────────────────────────────────────────┐   │
│  │              Backend API (Node.js/Rust)            │   │
│  │  • Proof generation  • Authentication              │   │
│  │  • Rate limiting     • Caching                     │   │
│  └────────────────────────────────────────────────────┘   │
│                           │                                 │
│                           │ RPC                             │
│                           ▼                                 │
│  ┌────────────────────────────────────────────────────┐   │
│  │           Stellar/Soroban Network                  │   │
│  │  • Contract instance  • State storage              │   │
│  │  • Event emission     • Transaction execution      │   │
│  └────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌────────────────────────────────────────────────────┐   │
│  │              Monitoring & Logging                  │   │
│  │  • Transaction tracking  • Alert system            │   │
│  │  • Analytics dashboard   • Error logging           │   │
│  └────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Scalability Considerations

### Horizontal Scaling

```
Load Balancer
      │
      ├──▶ API Server 1 ──┐
      ├──▶ API Server 2 ──┤
      ├──▶ API Server 3 ──┼──▶ Shared Cache (Redis)
      └──▶ API Server N ──┘         │
                                     │
                                     ▼
                              Blockchain Network
```

### Vertical Optimization

- **Proof Caching**: Cache generated proofs with root version
- **Batch Processing**: Group operations to amortize costs
- **Lazy Loading**: Load whitelist data on-demand
- **CDN Distribution**: Serve static proofs via CDN

## Monitoring Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Monitoring Stack                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Event Listeners                                            │
│  ┌──────────────────────────────────────────────────┐     │
│  │  • Proposal events   • Dispute events            │     │
│  │  • Vote events       • Admin actions             │     │
│  └──────────────────────────────────────────────────┘     │
│                           │                                 │
│                           ▼                                 │
│  Metrics Collection                                         │
│  ┌──────────────────────────────────────────────────┐     │
│  │  • Gas usage         • Transaction success rate  │     │
│  │  • Active users      • Fee collection            │     │
│  └──────────────────────────────────────────────────┘     │
│                           │                                 │
│                           ▼                                 │
│  Alerting System                                            │
│  ┌──────────────────────────────────────────────────┐     │
│  │  • Anomaly detection  • Threshold alerts         │     │
│  │  • Email/SMS/Slack    • Incident tracking        │     │
│  └──────────────────────────────────────────────────┘     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Technology Stack

### Smart Contract Layer
- **Language**: Rust
- **Framework**: Soroban SDK
- **Cryptography**: SHA-256
- **Storage**: Soroban storage primitives

### Off-Chain Layer
- **Tree Generation**: Rust / JavaScript
- **API**: Node.js / Rust (Actix)
- **Database**: PostgreSQL / MongoDB
- **Cache**: Redis

### Frontend Layer
- **Framework**: React / Next.js
- **Web3**: Stellar SDK / Freighter
- **State**: Redux / Zustand
- **UI**: TailwindCSS

### Infrastructure
- **Hosting**: AWS / GCP / Azure
- **CDN**: Cloudflare
- **Monitoring**: Datadog / Grafana
- **Logging**: ELK Stack

## Performance Metrics

| Metric | Target | Actual |
|--------|--------|--------|
| Proof Verification | <15K CPU | ~11K CPU |
| Initialization | <10K CPU | ~5K CPU |
| Governance Vote | <5K CPU | ~3K CPU |
| Role Check | <2K CPU | <2K CPU |
| API Response Time | <500ms | ~200ms |
| Proof Generation | <1s | ~300ms |

---

**Document Version**: 1.0  
**Last Updated**: 2026-08-27  
**Maintained By**: Core Development Team
