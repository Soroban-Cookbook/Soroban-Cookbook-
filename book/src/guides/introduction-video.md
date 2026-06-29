# Introduction to Soroban Cookbook (Video)

A 5-7 minute introductory video providing a complete overview of the Soroban Cookbook project, how to navigate the examples, and getting started.

---

## Video

> **Status:** Planned — script and outline finalized below.
>
> Once recorded, the video will be embedded here and uploaded to the
> [Soroban Cookbook YouTube channel](https://www.youtube.com/@SorobanCookbook).
>
> Duration target: 5-7 minutes

<!-- Embed placeholder — replace with YouTube iframe after upload:
<iframe width="560" height="315" src="https://www.youtube.com/embed/VIDEO_ID"
  title="Introduction to Soroban Cookbook" frameborder="0"
  allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
  allowfullscreen></iframe>
-->

---

## Video Script & Outline

### 1. Introduction (0:00 - 0:45)

**Visual:** Cookbook landing page + animated logo

> "Welcome to the Soroban Cookbook — a developer-first guide to building smart
> contracts on Stellar using Soroban. Whether you're just starting with Rust and
> blockchain development, or you're an experienced engineer looking for production
> patterns, this cookbook has something for you."

**Key points:**
- What is the Soroban Cookbook?
- Who is it for? (beginners through advanced Soroban devs)
- What makes it different? (practical, runnable examples with tests)

---

### 2. Project Overview (0:45 - 2:30)

**Visual:** Repository structure walkthrough in terminal/IDE

> "Let's look at how the project is organized..."

**Walkthrough structure:**

```
Soroban-Cookbook/
├── examples/           ← Runnable smart contract examples
│   ├── basics/         ← 14 foundational examples
│   ├── intermediate/   ← Multi-contract patterns
│   ├── advanced/       ← Complex DeFi & governance
│   ├── defi/           ← DeFi use cases
│   ├── governance/     ← Voting & DAO patterns
│   ├── nfts/           ← NFT examples
│   └── tokens/         ← Token implementations
├── book/               ← This documentation (mdBook)
├── tests/integration/  ← Cross-contract integration tests
└── docs/               ← Additional references
```

**Key points:**
- Each example is a self-contained Cargo crate
- Every example has comprehensive tests
- Examples follow a naming convention: `XX-name/`
- Difficulty progression within each category

---

### 3. Navigating the Examples (2:30 - 4:00)

**Visual:** Opening an example, showing code + tests

> "Let me show you how to use an example. Let's open the simple voting
> contract..."

**Live demo steps:**
1. Navigate to `examples/governance/01-simple-voting/`
2. Show `Cargo.toml` — dependencies and SDK version
3. Show `src/lib.rs` — contract structure (storage, types, impl)
4. Show `src/test.rs` — how tests validate behavior
5. Run `cargo test -p simple-voting` — all tests pass

**Key points:**
- Every example follows the same pattern: `Cargo.toml` + `src/lib.rs` + tests
- Tests serve as both validation and documentation
- You can copy any example as a starting point for your own project
- Integration tests show how contracts interact with each other

---

### 4. Getting Started Guide (4:00 - 5:30)

**Visual:** Terminal — environment setup in real time

> "Here's how to get up and running in under 5 minutes..."

**Steps demonstrated:**

```bash
# 1. Clone the repository
git clone https://github.com/Soroban-Cookbook/Soroban-Cookbook.git
cd Soroban-Cookbook

# 2. Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# 3. Install Soroban CLI
cargo install stellar-cli

# 4. Run your first example
cargo test -p hello-world

# 5. Build for deployment
cargo build -p hello-world --target wasm32-unknown-unknown --release

# 6. Run all tests to verify setup
cargo test --workspace
```

**Key points:**
- Prerequisites: Rust, `wasm32-unknown-unknown` target, Soroban CLI
- Start with the basics examples (01 through 14)
- Each example builds on concepts from previous ones
- The mdBook docs explain concepts in depth
- Integration tests show advanced cross-contract patterns

---

### 5. Where to Go Next (5:30 - 6:15)

**Visual:** mdBook documentation site

> "Now that you're set up, here's how I recommend progressing..."

**Suggested learning path:**

| Stage | Focus | Examples |
|-------|-------|----------|
| Week 1 | Basics | 01-hello through 06-data-types |
| Week 2 | Patterns | 07-storage through 11-time |
| Week 3 | Intermediate | Multi-sig, Ajo Factory |
| Week 4 | Use Cases | DeFi vaults, Governance voting |

**Additional resources:**
- Written guides in the mdBook (testing, deployment, local simulation)
- Integration test framework for learning cross-contract patterns
- Reference docs (best practices, common pitfalls, glossary)

---

### 6. Recap & Outro (6:15 - 7:00)

**Visual:** Summary slide with links

> "To recap what we covered today..."

**Summary bullets:**
- The Cookbook is a practical, example-driven learning resource
- Examples are organized by difficulty and use case
- Every example has tests you can run immediately
- Start with basics, progress through intermediate to advanced
- The docs site provides written explanations alongside the code

**Call to action:**
- Star the repository on GitHub
- Join the Stellar Discord for questions
- Subscribe to the YouTube channel for more videos
- Contribute your own examples (see CONTRIBUTING.md)

**End screen:** Links to GitHub, YouTube, Discord

---

## Production Notes

### Recording Setup

Follow the [Video Creation Tools](./video-creation.md) guide for:
- OBS Studio configuration (1080p, 30fps)
- Audio setup (clear narration, -14 LUFS)
- Code theme (One Dark Pro, JetBrains Mono 18px)

### Post-Production

1. Record narration following the script above
2. Capture screen recordings for each section
3. Edit in DaVinci Resolve (see video-creation guide)
4. Add intro bumper and outro card
5. Generate captions
6. Export and upload per the documented process

### Linking

After upload, update this page with the YouTube embed and add the video link to:
- `book/src/README.md` (main introduction)
- Repository root `README.md`
- YouTube playlist: "Getting Started with Soroban"
