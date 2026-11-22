# Soroban Development Workshop

> Reusable, self-paced workshop materials for learning Soroban smart contract development on the Stellar network.

## Overview

This workshop takes developers from zero to building, testing, and deploying Soroban smart contracts. It uses the examples and patterns from the [Soroban Cookbook](../../examples/) as hands-on lab material.

**Duration:** 2 days (or 4 half-day sessions)
**Audience:** Developers with basic programming experience. No prior Rust or blockchain knowledge required, though both help.
**Format:** Instructor-led with hands-on coding exercises.

## Workshop Modules

| Module | Title | Duration | Slides | Exercise |
|--------|-------|----------|--------|----------|
| 1 | Introduction & Environment Setup | 2h | [Slides](slides/module-1-introduction.md) | [Exercise](exercises/module-1-exercise.md) |
| 2 | Storage & Data Types | 3h | [Slides](slides/module-2-storage.md) | [Exercise](exercises/module-2-exercise.md) |
| 3 | Authentication & Access Control | 3h | [Slides](slides/module-3-auth.md) | [Exercise](exercises/module-3-exercise.md) |
| 4 | Tokens & SEP-41 Standard | 3h | [Slides](slides/module-4-tokens.md) | [Exercise](exercises/module-4-exercise.md) |
| 5 | Advanced Patterns & Deployment | 3h | [Slides](slides/module-5-advanced.md) | [Exercise](exercises/module-5-exercise.md) |

## Learning Objectives

By the end of this workshop, participants will be able to:

1. Set up a complete Soroban development environment
2. Write, test, and build Soroban smart contracts in Rust
3. Understand and use all three Soroban storage types
4. Implement authentication and authorization patterns
5. Create SEP-41-compliant fungible tokens
6. Deploy contracts to the Stellar testnet
7. Apply common patterns from the Soroban Cookbook

## Prerequisites

- A Unix-like terminal (macOS, Linux, or WSL2 on Windows)
- Basic command-line familiarity
- A code editor (VS Code recommended with [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer))
- No prior Rust experience required, but the [Rust Book](https://doc.rust-lang.org/book/) is a helpful companion

## Setup Instructions

See the [Participant Guide](participant-guide.md) for detailed setup instructions. The quick version:

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 2. Add the WebAssembly target
rustup target add wasm32-unknown-unknown

# 3. Install the Stellar CLI (includes soroban)
cargo install --locked stellar-cli --features opt

# 4. Clone this repository
git clone https://github.com/Stellar-Cookbook/Soroban-Cookbook.git
cd Soroban-Cookbook

# 5. Verify the build
cargo build --target wasm32-unknown-unknown --release
```

## Materials

- [Facilitator Guide](facilitator-guide.md) — Timing, talking points, and common pitfalls for each module
- [Participant Guide](participant-guide.md) — Setup instructions, step-by-step workbook, and reference links
- [Slide Decks](slides/) — Marp-compatible Markdown slides for each module
- [Exercises](exercises/) — Hands-on lab exercises with starter code and solutions

## How to Run the Slides

The slide decks are written in [Marp](https://marp.app/) Markdown format. You can present them in several ways:

```bash
# Option 1: VS Code Marp extension (recommended)
# Install the "Marp for VS Code" extension, then open any .md file in slides/

# Option 2: Marp CLI tool
npm install -g @marp-team/marp-cli
marp slides/module-1-introduction.md --html -o module-1.html

# Option 3: Just read them as Markdown — they render fine in any viewer
```

## How to Use the Exercises

Each exercise file contains:

1. **Objective** — what you'll build
2. **Setup** — which cookbook example to start from
3. **Tasks** — step-by-step instructions
4. **Solution** — reference solution with explanations
5. **Verification** — how to test your work

Exercises reference existing [cookbook examples](../../examples/) so you always have working code to study.

## Repository Structure

```
docs/workshop/
├── README.md                  # This file
├── facilitator-guide.md        # Instructor notes for each module
├── participant-guide.md        # Participant setup and workbook
├── slides/
│   ├── module-1-introduction.md
│   ├── module-2-storage.md
│   ├── module-3-auth.md
│   ├── module-4-tokens.md
│   └── module-5-advanced.md
└── exercises/
    ├── module-1-exercise.md
    ├── module-2-exercise.md
    ├── module-3-exercise.md
    ├── module-4-exercise.md
    └── module-5-exercise.md
```

## Adapting This Workshop

These materials are designed to be reused and adapted:

- **Self-paced study:** Participants can work through modules independently
- **Instructor-led:** Use the facilitator guide for timing and talking points
- **Hackathon prep:** Run modules 1-3 as a crash course, then let teams build
- **University course:** Spread modules across a semester with deeper Rust exercises

## Contributing

Found a typo or want to improve the workshop? See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.
