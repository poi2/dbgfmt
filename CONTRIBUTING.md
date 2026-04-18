# Contributing

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.85+)
- [markdownlint-cli2](https://github.com/DavidAnson/markdownlint-cli2) — `npm install -g markdownlint-cli2`
- [prek](https://github.com/j178/prek) — `cargo install prek`

## Setup

Clone your fork or the repository, then install the pre-commit hooks:

```bash
prek install
```

This runs the same checks as CI on every commit:

- Markdown Lint
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-targets`
