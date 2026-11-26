# Releasing LTKoi

LTKoi releases belong to `Akshith-AI/git-koi`. The original git-koi crate and
its registry publishing credentials are not used. `publish = false` in
Cargo.toml prevents accidental registry publication; GitHub binary releases and
installation from tagged Git source remain supported.

## Before tagging

1. Update Cargo.toml's version and the corresponding root package in Cargo.lock.
2. Add that version to CHANGELOG.md and update pinned Action examples as needed.
3. Run formatting, lint, all Rust tests, and the browser/widget checks.
4. Merge a reviewed, passing change into main without rewriting the prior stage
   commits. Tag the tested commit, for example `v0.1.0`, and push the tag.

## Automated release workflow

The Release workflow checks that the tag matches the manifest and changelog. It
builds Linux x64/ARM64, macOS Intel/Apple Silicon, and Windows x64 binaries.
Every archive contains the binary, LICENSE, NOTICE.md, README.md, and
CHANGELOG.md, alongside a SHA-256 checksum file.

The workflow stages a draft release, then smoke-tests the reusable Action in all
three modes by downloading and verifying the actual release binary. These tests
use committed snapshots so public API outages cannot turn release verification
into fabricated zero activity. Only after all smoke jobs pass does the workflow
publish the draft and update the `v0` convenience tag. The full `v0.1.0` tag is
preferred in copyable examples. No crates.io publication occurs.

Rerunning a failed workflow reuses its existing draft release and replaces that
run's assets; the release is not public until its smoke tests succeed. An
already-published release can also be inspected or rerun, so do not move full
version tags to different commits. Make a new version for code changes.

## Verify the first daily refresh

Set repository variables `LEETCODE_USERNAME`, `LTKOI_GITHUB_USERNAME`, and
`LTKOI_MODE`, then manually run Refresh fish. Confirm:

- The selected GIF is committed under `fish/` and can be decoded.
- The Actions run attaches a complete `daily-fish` preview bundle.
- Public account totals match the source observations and restricted optional
  data stays unavailable rather than becoming zero.
- A repeat with identical snapshots does not create a new GIF commit.

The daily workflow uses the checked-out Action in this source repository. The
profile example pins the published Action tag, so it works in another repository
without installing Rust. `LTKOI_PUBLISH_WIDGET=true` commits the complete
bundle; it does not enable or configure website hosting.
