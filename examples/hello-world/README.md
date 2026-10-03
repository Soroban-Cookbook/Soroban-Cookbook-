# Hello World Contract — Alternate Simplified Layout

> ⚠️ **This is not the canonical hello-world example.**
>
> The canonical, fully-documented introduction to Soroban smart contracts lives at:
>
> **→ [examples/basics/01-hello-world/](../basics/01-hello-world/)**
>
> Start there. It includes a code walkthrough, common pitfalls, comprehensive tests,
> a video walkthrough, and "next steps" that link onward to the rest of the basics sequence.

This directory holds an alternate, minimal layout of the same `hello(to: Symbol) -> Vec<Symbol>` contract. It is kept as a workspace member to verify that the simplest-possible standalone crate layout still compiles and works independently of the numbered-basics folder structure.

## When to use this variant

Only reach for this layout if you specifically want:
- A single, flat `examples/hello-world/` directory (not nested under `basics/`)
- A crate named `soroban-hello-world-contract` (vs. the canonical `hello-world`)
- The absolute minimum README surface with no learning commentary

For every other case — especially if you are new to Soroban — use the **canonical example** linked above.

## Building

```bash
cargo build --target wasm32-unknown-unknown --release
```

## Testing

```bash
cargo test
```

## See also

- [Canonical 01-hello-world (with full walkthrough)](../basics/01-hello-world/)
- [Basics overview](../basics/)
- [All examples index](../../book/src/examples-index.md)
