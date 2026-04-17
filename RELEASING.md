# Releasing

## Steps

1. Update the version in `Cargo.toml`
2. Commit the version bump
   ```bash
   git add Cargo.toml
   git commit -m "chore: bump version to X.Y.Z"
   ```
3. Create a tag and push
   ```bash
   git tag vX.Y.Z
   git push origin main --tags
   ```

Alternatively, you can create a release with a tag from the GitHub UI.

## What happens automatically

The [Release workflow](.github/workflows/release.yml) (powered by [cargo-dist](https://opensource.axo.dev/cargo-dist/)) will:

- Build binaries for all target platforms (macOS, Linux, Windows)
- Create a GitHub Release with the built artifacts
- Publish the Homebrew formula to [poi2/homebrew-tap](https://github.com/poi2/homebrew-tap)
- Generate a shell installer script
