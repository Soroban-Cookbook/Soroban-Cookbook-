# Documentation Style Guide

This guide defines the standards for writing and maintaining documentation in this project. Following these guidelines keeps our documentation consistent, accurate, and approachable for all readers.

## Writing Standards

### Tone and Voice

- Write in a **clear, direct, and friendly** tone.
- Prefer the **active voice**: "The compiler checks the type" rather than "The type is checked by the compiler."
- Address the reader as **"you"**. Refer to the project as **"we"** when describing team decisions or conventions.
- Keep sentences short. Aim for one idea per sentence.
- Avoid jargon unless it is defined on first use. Link to a glossary entry or explain inline.

### Terminology

Use the following terms consistently throughout the documentation:

| Preferred term | Avoid |
| -------------- | ----- |
| function | method (unless referring to a trait method) |
| crate | package, library |
| command | instruction |
| configuration | config (in prose) |
| directory | folder |

- Capitalize product names exactly as the project does.
- Use code formatting for identifiers, file names, commands, and literal values (for example, `Cargo.toml`, `cargo build`).
- Spell out acronyms on first use, for example, "Application Programming Interface (API)".

### Inclusive Language

- Use gender-neutral language. Prefer "they" over "he/she".
- Avoid idioms, slang, and culture-specific references that may not translate well.
- Use "allowlist"/"denylist" instead of "whitelist"/"blacklist".
- Avoid ableist terms such as "crazy", "insane", or "dummy" when describing code or behavior. Prefer "unexpected", "placeholder", or "stub".
- Refer to people by their roles rather than descriptors where possible.

## Code Example Standards

### Complete and Runnable

- Every code example should be **complete enough to compile and run** unless it is explicitly marked as a fragment.
- Include necessary imports, `fn main()`, and any required setup.
- If an example is intentionally partial, indicate this with a comment such as `// ...` and explain what is omitted.

```rust
use std::collections::HashMap;

fn main() {
    let mut counts: HashMap<&str, u32> = HashMap::new();
    for word in ["apple", "banana", "apple"] {
        *counts.entry(word).or_insert(0) += 1;
    }
    println!("{counts:?}");
}
```

### `no_std` Compatibility

- When an example is intended for `no_std` environments, state this explicitly and avoid the standard library.
- Use `#![no_std]` at the crate root and document any required `extern crate alloc;` usage.

```rust
#![no_std]

pub fn add(a: u32, b: u32) -> u32 {
    a + b
}
```

### Error Handling

- Demonstrate idiomatic error handling. Prefer `Result` and the `?` operator over `unwrap()` or `expect()` in examples unless the example specifically illustrates panics.
- When `unwrap()` is used for brevity, add a comment explaining that real code should handle the error.

```rust
use std::fs;

fn read_config(path: &str) -> Result<String, std::io::Error> {
    let contents = fs::read_to_string(path)?;
    Ok(contents)
}
```

### Comments

- Use comments to explain **why**, not **what**. The code itself should make the "what" clear.
- Keep comments up to date with the code they describe.
- Use `//` for inline comments and `///` for documentation comments on public items.

## Formatting Guidelines

### Headings

- Use **sentence case** for headings: "Getting started", not "Getting Started".
- Do not skip heading levels. A document should start with a single `#` title, followed by `##` sections, then `###` subsections.
- Keep headings short and descriptive.

### Lists

- Use `-` for unordered lists and `1.` for ordered lists.
- Keep list items parallel in structure.
- End list items with a period only when they are complete sentences.

### Code Fences

- Always specify a language tag on fenced code blocks so syntax highlighting works correctly.

````markdown
```rust
fn main() {
    println!("Hello, world!");
}
```
````

- Use `text` for plain output or non-code content.
- Keep lines within code blocks reasonably short to avoid horizontal scrolling.

### Links

- Use descriptive link text. Prefer "[the configuration guide](configuration.md)" over "[click here](configuration.md)".
- Use relative links for pages within the book so they work in all deployment targets.
- Verify that links resolve before submitting changes.

## Tool Recommendations

The following tools help maintain consistent, high-quality documentation:

- **mdBook** — builds this documentation site. Run `mdbook build` in the `book/` directory to verify changes compile without errors, and `mdbook serve` for a live preview.
- **rustfmt** — formats Rust code examples consistently. Run `cargo fmt` before committing.
- **typos** — catches spelling mistakes in prose and code. Run `typos` across the repository.
- **markdownlint** — enforces Markdown formatting rules such as heading structure and list style. Run `markdownlint 'book/src/**/*.md'`.

### Pre-submission Checklist

Before opening a pull request that changes documentation:

1. Run `cd book && mdbook build` and confirm it succeeds without warnings.
2. Run `typos` and fix any reported issues.
3. Run `markdownlint` on changed files.
4. Confirm all code examples compile and all links resolve.