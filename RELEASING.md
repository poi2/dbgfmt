# Releasing

## Steps

1. Update the version in `Cargo.toml`
2. Update `CHANGELOG.md`: move `[Unreleased]` entries to `[X.Y.Z] - YYYY-MM-DD` and update links
3. Create a PR, get it merged into main
4. Create a tag and push

   ```bash
   git checkout main && git pull
   git tag vX.Y.Z
   git push origin --tags
   ```

Alternatively, you can create a release with a tag from the GitHub UI.

## What happens automatically

The [Release workflow](.github/workflows/release.yml) (powered by [cargo-dist](https://opensource.axo.dev/cargo-dist/)) will:

- Build binaries for all target platforms (macOS, Linux, Windows)
- Create a GitHub Release with the built artifacts
- Publish to [crates.io](https://crates.io/crates/dbgfmt)
- Publish the Homebrew formula to [poi2/homebrew-tap](https://github.com/poi2/homebrew-tap)
- Generate a shell installer script
