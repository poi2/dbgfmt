# Releasing

## Steps

1. Update the version in `Cargo.toml`
2. Update `CHANGELOG.md`: move `[Unreleased]` entries to `[X.Y.Z] - YYYY-MM-DD` and update links
3. Create a PR, get it merged into main

## What happens automatically

1. The [Auto Tag workflow](.github/workflows/auto-tag.yml) detects the version change in `Cargo.toml` on main and creates a `vX.Y.Z` tag.
2. The [Release workflow](.github/workflows/release.yml) (powered by [cargo-dist](https://opensource.axo.dev/cargo-dist/)) is triggered by the tag and will:
   - Build binaries for all target platforms (macOS, Linux, Windows)
   - Create a GitHub Release with the built artifacts
   - Publish to [crates.io](https://crates.io/crates/dbgfmt)
   - Publish the Homebrew formula to [poi2/homebrew-tap](https://github.com/poi2/homebrew-tap)
   - Generate a shell installer script
