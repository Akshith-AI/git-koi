#!/usr/bin/env bash
set -euo pipefail
version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$GITHUB_ACTION_PATH/Cargo.toml" | head -1)
repository=$(sed -n 's/^repository = "\([^"]*\)"/\1/p' "$GITHUB_ACTION_PATH/Cargo.toml")
case "$RUNNER_OS-$RUNNER_ARCH" in
  Linux-X64) target=x86_64-unknown-linux-musl ;;
  Linux-ARM64) target=aarch64-unknown-linux-musl ;;
  macOS-X64) target=x86_64-apple-darwin ;;
  macOS-ARM64) target=aarch64-apple-darwin ;;
  *) echo "LTKoi Action supports Linux and macOS runners; use ubuntu-latest." >&2; exit 1 ;;
esac
archive="ltkoi-$version-$target.tar.gz"
install_dir=$(mktemp -d "$RUNNER_TEMP/ltkoi-release-XXXXXX")
gh release download "v$version" --repo "$repository" --dir "$install_dir" \
  --pattern "$archive" --pattern "$archive.sha256"
(
  cd "$install_dir"
  if command -v sha256sum >/dev/null; then sha256sum -c "$archive.sha256";
  else shasum -a 256 -c "$archive.sha256"; fi
  tar -xzf "$archive"
)
echo "$install_dir/ltkoi-$version-$target" >> "$GITHUB_PATH"
