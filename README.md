# dbgfmt

[![CI](https://github.com/poi2/dbgfmt/actions/workflows/ci.yml/badge.svg)](https://github.com/poi2/dbgfmt/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/dbgfmt.svg)](https://crates.io/crates/dbgfmt)
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
dbgfmt = "0.1"
```

Use in your code:

```rust
use dbgfmt::format_debug;

let input = format!("{:?}", my_struct);
let pretty = format_debug(&input, 4);
println!("{pretty}");
```

## As a CLI

### Installation

#### Homebrew

```bash
brew install poi2/tap/dbgfmt
```

#### Shell script

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/poi2/dbgfmt/releases/latest/download/dbgfmt-installer.sh | sh
```

#### Cargo

```bash
cargo install dbgfmt
```

### Usage

```bash
# Pass as argument
dbgfmt 'Foo { bar: 1, baz: [2, 3] }'

# Pipe from stdin
echo 'Foo { bar: 1, baz: [2, 3] }' | dbgfmt

# Format dbg!() macro output (prefix is preserved)
echo '[src/main.rs:5:5] config = Config { host: "localhost", port: 8080 }' | dbgfmt

# Multiple values (separate lines or same line)
printf 'Foo { x: 1 }\nBar { y: 2 }' | dbgfmt
```

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.
