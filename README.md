# rust-dbg-fmt

[![CI](https://github.com/poi2/rust-dbg-fmt/actions/workflows/ci.yml/badge.svg)](https://github.com/poi2/rust-dbg-fmt/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

A zero-dependency Rust library and CLI tool that pretty-prints Rust `Debug` trait output with proper indentation and newlines.

## Example

**Input:**

```text
Foo { bar: 1, baz: Vec { items: [1, 2, 3] }, name: "hello" }
```

**Output:**

```text
Foo {
  bar: 1,
  baz: Vec {
    items: [
      1,
      2,
      3,
    ],
  },
  name: "hello",
}
```

## As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-dbg-fmt = "0.1"
```

Use in your code:

```rust
use rust_dbg_fmt::format_debug;

let input = format!("{:?}", my_struct);
let pretty = format_debug(&input, 2);
println!("{pretty}");
```

## As a CLI

### Installation

```bash
cargo install rust-dbg-fmt
```

### Usage

```bash
# Pass as argument
rust-dbg-fmt 'Foo { bar: 1, baz: [2, 3] }'

# Pipe from stdin
echo 'Foo { bar: 1, baz: [2, 3] }' | rust-dbg-fmt
```

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.
