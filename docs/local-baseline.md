# Local foundation baseline (historical stage 1)

This records the inherited stage 1 baseline. Stage 5 renames the command and
package to `ltkoi`; use [the current setup guide](getting-started.md) for
installation and releases.

This guide exercises the inherited GitHub koi generator from this checkout. It
does not yet generate a LeetCode betta. The sample profile is synthetic test
data, not the project owner's live account.

## Requirements

Install Rust 1.88 or newer (the minimum is recorded in Cargo.toml), including
Cargo, rustfmt, and Clippy. No GitHub login is needed for the fixture render.
The first build needs internet access to download the locked dependencies.

## Run the existing checks

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --release --locked
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
npx --yes prettier@3.9.10 --check .
```

The documentation formatting check also requires Node.js and npm. Existing CI
runs these checks; its separate minimum-version job checks all targets on the
Rust version recorded in Cargo.toml.

## Generate a sample koi

```sh
cargo run --release --locked -- render tests/fixtures/account-with-history.json --output-dir target/ltkoi-baseline --body asagi
```

Open `target/ltkoi-baseline/preview.html` in a browser. It shows the animated
GIF and still image against light and dark backgrounds. The output folder
contains:

- `koi.gif`: looping animation.
- `koi.png`: still image.
- `preview.html`: local preview.

Generated files stay under the ignored `target/` folder and are not committed.
The upstream CLI name and package name remain `git-koi` during this stage.

## Collect a real GitHub profile later

Install the GitHub CLI and authenticate with `gh auth login`, then replace
`YOUR_GITHUB_USERNAME` below:

```sh
cargo run --release --locked -- collect YOUR_GITHUB_USERNAME --output target/github-snapshot.json
cargo run --release --locked -- render target/github-snapshot.json --output-dir target/my-github-koi --body asagi
```

This reads public GitHub activity and needs authenticated API access. The local
foundation verification uses fixtures instead; it does not validate live account
collection or publish a release.

## Fork release boundary

The inherited Action reads its release repository from Cargo.toml, which still
points to the original project. The inherited profile workflow also references
the original Action. Use the local source build above to verify this fork;
release metadata and workflow distribution will be adapted in stage 5.

## Stage 1 verification record

Verified on November 11, 2026, on Apple Silicon macOS with Rust 1.88.0, starting
from inherited commit `7305e9b1fadee6f43d3b4fa543a25d4cde79856e`:

- Rust formatting and Clippy with warnings denied: passed.
- Existing test suite: 93 tests passed, none failed.
- Locked release build: passed.
- API documentation with warnings denied: passed.
- Repository documentation/configuration formatting: passed.
- Fixture render with the Asagi body: GIF, PNG, and HTML preview generated.
- GIF: 512 by 512 pixels, 120 frames, continuous looping; every frame decoded.
- PNG: decoded successfully and visually inspected. The fixture produces a small
  juvenile koi, consistent with its low contribution total.
- Original LICENSE: unchanged from the inherited commit.

Rust was installed in an isolated temporary location for this verification,
without changing the user's shell setup. While that installation remains on this
machine, the same commands can be run after setting:

```sh
export CARGO_HOME=/tmp/ltkoi-cargo
export RUSTUP_HOME=/tmp/ltkoi-rustup
export PATH="/tmp/ltkoi-cargo/bin:$PATH"
```

Temporary files can be removed by the operating system; install Rust normally
for a persistent development environment. Live GitHub collection and hosted
GitHub Actions were not exercised in this local verification.
