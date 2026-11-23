#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

case "$(uname -m)" in
  arm64) rust_target="aarch64-apple-darwin" ;;
  x86_64) rust_target="x86_64-apple-darwin" ;;
  *) echo "Unsupported macOS architecture: $(uname -m)" >&2; exit 2 ;;
esac

version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)"
app_name="Arch Notebook"
app_bundle="$repo_root/dist/$app_name.app"
distribution_status="unsigned"
if [[ -n "${APPLE_SIGNING_IDENTITY:-}" ]]; then
  distribution_status="developer-id-signed"
fi
if [[ -n "${APPLE_NOTARY_PROFILE:-}" ]]; then
  distribution_status="notarized"
fi
archive="$repo_root/dist/Arch-Notebook-macos-$(uname -m)-$distribution_status.zip"

binary_path="target/$rust_target/release/arch-notebook"
if [[ "${ARCH_NOTEBOOK_SKIP_BUILD:-0}" != "1" ]]; then
  cargo build --release --locked --target "$rust_target"
fi
if [[ ! -x "$binary_path" ]]; then
  echo "Missing executable: $binary_path" >&2
  echo 'Build the release target first or unset ARCH_NOTEBOOK_SKIP_BUILD.' >&2
  exit 1
fi

mkdir -p "$repo_root/dist"
rm -rf "$app_bundle"
mkdir -p "$app_bundle/Contents/MacOS" "$app_bundle/Contents/Resources"
install -m 755 "$binary_path" "$app_bundle/Contents/MacOS/arch-notebook"
install -m 755 "$repo_root/scripts/install-macos-automation.sh" \
  "$app_bundle/Contents/Resources/install-macos-automation.sh"
install -m 755 "$repo_root/scripts/run-macos-automation.sh" \
  "$app_bundle/Contents/Resources/run-macos-automation.sh"

cat > "$app_bundle/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>$app_name</string>
  <key>CFBundleDisplayName</key><string>$app_name</string>
  <key>CFBundleIdentifier</key><string>com.archnotebook.desktop</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleExecutable</key><string>arch-notebook</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSMinimumSystemVersion</key><string>12.0</string>
</dict>
</plist>
PLIST

if [[ -n "${APPLE_SIGNING_IDENTITY:-}" ]]; then
  codesign --force --options runtime --timestamp --sign "$APPLE_SIGNING_IDENTITY" "$app_bundle"
  codesign --verify --deep --strict --verbose=2 "$app_bundle"
fi

rm -f "$archive"
ditto -c -k --sequesterRsrc --keepParent "$app_bundle" "$archive"
if [[ -n "${APPLE_NOTARY_PROFILE:-}" ]]; then
  [[ -n "${APPLE_SIGNING_IDENTITY:-}" ]] || { echo 'Set APPLE_SIGNING_IDENTITY before notarization.' >&2; exit 2; }
  xcrun notarytool submit "$archive" --keychain-profile "$APPLE_NOTARY_PROFILE" --wait
  xcrun stapler staple "$app_bundle"
  rm -f "$archive"
  ditto -c -k --sequesterRsrc --keepParent "$app_bundle" "$archive"
fi
printf 'Packaged %s\n' "$archive"
