# Release process

The build workflow creates Linux x86_64 and ARM64 archives, an Arch Linux x86_64 package, and macOS Apple Silicon and Intel packages. It does not create a public release from a branch push.

## Release readiness — 2026-11-25

- Current Cargo version: 0.1.0.
- Implementation baseline [2dadb09](https://github.com/pskeffington/arch_notebook/commit/2dadb09570e2a871489b2146f9ad1e1410b70929) passed [CI run 45269810987](https://github.com/pskeffington/arch_notebook/actions/runs/45269810987) on all five platform jobs. Release attachment was skipped as expected on a branch build.
- The later feature-branch candidate `986f3e8` passed [CI run 45723413554](https://github.com/pskeffington/arch_notebook/actions/runs/45723413554), including SQL/ANX acceptance, isolated Linux and macOS scheduler installer checks, Linux x86_64/ARM64, Arch packaging, and macOS Intel/Apple Silicon. The LaunchAgent check stubs host commands and is not native macOS scheduler evidence.
- No published GitHub Release was listed at review time. Native installation, interactive acceptance, scheduler execution and external IBM i2 import still require recorded evidence; macOS notarization is not established.
- Keep the release unpublished until owner approval. Verify the actual release candidate's CI and checksums rather than relying on the historical baseline.

## Local user install evidence — 2026-11-28

The optimized Rust candidate from product source `5bb4593` is installed for the current Linux user; its binary and synchronized Desktop/menu/icon assets pass local installation checks. This is an installation acceptance result, not authorization to publish a GitHub Release. Native interactive review, live scheduler dispatch, exact-head CI for this candidate, and IBM application/schema validation remain open. See [LOCAL-INSTALL-12](docs/evidence/deployment/LOCAL-INSTALL-12-linux.md) and [current-source SQL/build acceptance](docs/evidence/sql-automation/SQL-AUTO-11-current-source-clinton-20261128.md).

## Publish a version

1. Set the new semantic version in `Cargo.toml` and update user-facing release notes.
2. Run the formatting, locked offline check/build, release build, Arch metadata, and macOS packaging-script syntax checks from `AGENTS.md`.
3. Commit and push the version change to `main`. Confirm the branch workflow passes and local `main` matches `origin/main`.
4. Create and push an annotated version tag, for example:

   ```sh
   git tag -a v0.1.1 -m "Arch Notebook v0.1.1"
   git push origin v0.1.1
   ```

5. Wait for the tag build to pass. Create a draft GitHub Release for that tag and review the generated notes. Publishing the draft triggers a fresh build that attaches both Linux archives, the Arch package, macOS zips, and `SHA256SUMS` to the release.
6. Check the release page and verify its asset names, checksums, and workflow run before sharing it.

The macOS bundles produced by CI are explicitly labeled `unsigned`. For direct distribution, sign and notarize a bundle on a Mac with the required Apple credentials, as described in [Build and deploy](docs/build-and-deploy.md), then attach the additional archive and its checksum to the release:

```sh
gh release upload v0.1.1 \
  "dist/Arch-Notebook-macos-$(uname -m)-notarized.zip" \
  --clobber
(
  cd dist
  shasum -a 256 "Arch-Notebook-macos-$(uname -m)-notarized.zip" \
    > SHA256SUMS-macos-notarized
)
gh release upload v0.1.1 dist/SHA256SUMS-macos-notarized --clobber
```

Keep the unsigned artifact available when adding the signed archive so users can distinguish the two builds. Do not commit certificates, API keys, or notarization credentials.
