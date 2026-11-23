# Build and deploy

Commands below run from the repository root unless stated otherwise.

The supported desktop targets are Arch Linux on x86_64, generic Linux archives on x86_64 and ARM64, and macOS on Apple Silicon or Intel. Rust 1.89 or newer is required to build. macOS bundles SQLite so it does not depend on the older system SQLite library; Linux builds link to the system SQLite library, which is installed as an application dependency. On Linux, the application stores its database, CSV example, and default case under `$XDG_DATA_HOME/arch-notebook`, or `~/.local/share/arch-notebook` when `XDG_DATA_HOME` is unset. On macOS, it uses `~/Library/Application Support/Arch Notebook`. Existing sample files are left in place on later launches.

Check an installed build with `arch-notebook --version`; scripts can use `arch-notebook --version --json` for machine-readable package name and version. In a macOS app bundle, invoke `"/Applications/Arch Notebook.app/Contents/MacOS/arch-notebook" --version`.

### Arch Linux

Build and install the current `main` branch as a native package with `makepkg`:

```sh
git clone https://github.com/pskeffington/arch_notebook.git
cd arch_notebook/packaging/arch
makepkg -si
```

The package includes the desktop menu entry and declares its shared-library dependencies. `makepkg` downloads the locked Cargo dependencies and builds the release binary for the host architecture. After opening the app and configuring enabled automation jobs, install the hourly user timer with:

```sh
arch-notebook-install-automation
```

Pass a case file as the first argument to schedule a different case. The installer writes user-owned systemd units under `${XDG_CONFIG_HOME:-~/.config}/systemd/user`, enables the timer, and validates the case before installation.

### Generic Linux desktop archive

For a Linux distribution without an Arch package, download the matching `arch-notebook-linux-x86_64.tar.gz` or `arch-notebook-linux-aarch64.tar.gz` workflow/release artifact, then install it for the current user without `sudo`:

```sh
case "$(uname -m)" in
  x86_64) archive_arch=x86_64 ;;
  aarch64|arm64) archive_arch=aarch64 ;;
  *) echo "Unsupported Linux architecture: $(uname -m)" >&2; exit 1 ;;
esac
archive_dir=arch-notebook-linux-$archive_arch
tar -xzf "$archive_dir.tar.gz"
"$archive_dir/install-linux-user.sh" "$archive_dir"
```

The installer puts the executable and its automation installer in `~/.local/bin`, and the desktop launcher under `${XDG_DATA_HOME:-~/.local/share}/applications`. Add `~/.local/bin` to `PATH` to launch it from a terminal. The companion installer lets the app’s **Install/update schedule** button configure systemd after the extracted archive has been removed. To remove this user installation, delete `~/.local/bin/arch-notebook`, `~/.local/bin/arch-notebook-install-automation`, and `${XDG_DATA_HOME:-$HOME/.local/share}/applications/arch-notebook.desktop`; case files and SQL databases are kept separately.

To schedule hourly SQL ingestion, first launch the app and enable the desired jobs in the case. Then run the bundled installer with the case path and binary path if `~/.local/bin` is not on `PATH`:

```sh
"$archive_dir/install-linux-automation.sh" \
  "${XDG_DATA_HOME:-$HOME/.local/share}/arch-notebook/harbor-light.case.json" \
  "$HOME/.local/bin/arch-notebook"
```

The installer creates and enables a user-level systemd timer. Inspect it with `systemctl --user status arch-notebook-automation.timer`; job output is available through `journalctl --user -u arch-notebook-automation.service`.

### macOS

Install Xcode Command Line Tools and Rust, then build a self-contained `.app` bundle and zip archive for the current Mac architecture:

```sh
./scripts/package-macos.sh
open "dist/Arch Notebook.app"
```

The script supports native Apple Silicon (`arm64`) and Intel (`x86_64`) hosts. For direct distribution outside the Mac App Store, sign with a Developer ID certificate and notarize with Apple. Configure a `notarytool` keychain profile, then run:

```sh
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
export APPLE_NOTARY_PROFILE="arch-notebook-notary"
./scripts/package-macos.sh
```

The script builds the native release binary by default. Set `ARCH_NOTEBOOK_SKIP_BUILD=1` only when the matching `aarch64-apple-darwin` or `x86_64-apple-darwin` release binary has already been built; CI uses this mode to package each matrix build without compiling it a second time.

The profile is created with `xcrun notarytool store-credentials`; the signing identity and credentials are not stored in this repository. See [Apple's notarization guide](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution) and [Developer ID signing overview](https://developer.apple.com/developer-id/).

### Continuous builds

GitHub Actions checks formatting and builds Linux x86_64 and ARM64 archives, macOS Apple Silicon and Intel apps, and an Arch Linux x86_64 package. Pull requests and pushes build without publishing a release; a newer push to `main` supersedes an older `main` build so CI prioritizes the latest commit. Generated workflow artifacts are available for 14 days. Publishing a GitHub Release triggers the workflow to attach the Linux archives, Arch package, macOS zips, and a `SHA256SUMS` file. macOS zip names state whether they are `unsigned`, `developer-id-signed`, or `notarized`; CI artifacts are unsigned because signing credentials are not stored in the repository.

To attach a locally signed and notarized macOS archive to a published release, build it with the credentials described above and upload it with:

```sh
gh release upload v0.1.0 "dist/Arch-Notebook-macos-$(uname -m)-notarized.zip" --clobber
```

The CI runner labels follow [GitHub's current hosted runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners). Arch package metadata follows the [Arch package guidelines](https://wiki.archlinux.org/title/Arch_package_guidelines).

### Install a published release

Published releases include a `SHA256SUMS` file for the attached CI artifacts. Download the release files and verify them before extracting or installing:

```sh
version=v0.1.0
release_dir="$PWD/arch-notebook-$version"
mkdir -p "$release_dir"
gh release download "$version" --dir "$release_dir"
if command -v sha256sum >/dev/null 2>&1; then
  (cd "$release_dir" && sha256sum --check SHA256SUMS)
else
  (cd "$release_dir" && shasum -a 256 --check SHA256SUMS)
fi
```

On Linux, continue with the generic archive steps above or install the Arch package with `pacman -U` on an Arch system. Keep the downloaded checksum file with the artifacts for later verification.

macOS CI bundles are unsigned and may be blocked by Gatekeeper. For normal direct installation, use a Developer ID signed and notarized bundle; the release process describes how to prepare and attach it. Extract a verified notarized zip into Applications with:

```sh
ditto -x -k "$release_dir/Arch-Notebook-macos-$(uname -m)-notarized.zip" "$release_dir"
ditto "$release_dir/Arch Notebook.app" "/Applications/Arch Notebook.app"
```

The CI-provided `SHA256SUMS` covers the CI assets. A separately attached notarized macOS zip has its own `SHA256SUMS-macos-notarized`; verify it with `(cd "$release_dir" && shasum -a 256 --check SHA256SUMS-macos-notarized)` before extracting it.

See [RELEASING.md](../RELEASING.md) for the version, tag, and signed macOS release process.
