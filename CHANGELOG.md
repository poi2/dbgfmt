# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `dbg!()` macro output support: strip `[file:line:col] expr =` prefix and preserve it in formatted output.
- Multi-value support: handle multiple Debug values in a single input.
- Bracket validation with line/column error messages.
- `--recover` (`-r`) option to best-effort format broken/truncated input.
- Hint message suggesting `--recover` when bracket validation fails.
- Security audit workflow for automated dependency vulnerability checking.

## [0.2.0] - 2025-05-01

### Added

- `--indent` option to customize indentation width.
- `--color` option to enable colored output.

### Changed

- Refactored output formatting with the `Emitter` trait.

## [0.1.0] - 2025-04-01

### Added

- CLI tool (`dbgfmt`) to pretty-print Rust `Debug` output.
- Library crate for programmatic usage.
- Read from file or stdin.

[Unreleased]: https://github.com/poi2/dbgfmt/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/poi2/dbgfmt/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/poi2/dbgfmt/releases/tag/v0.1.0
