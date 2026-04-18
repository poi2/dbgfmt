# Contributing

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.85+)
- [markdownlint-cli2](https://github.com/DavidAnson/markdownlint-cli2)
- [prek](https://github.com/j178/prek)

## Setup

```bash
git clone https://github.com/poi2/dbgfmt.git
cd dbgfmt
prek install
```

This installs pre-commit hooks that run the same checks as CI:

- Markdown Lint
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-targets`
