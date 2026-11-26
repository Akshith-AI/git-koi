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
into fabricated zero activity. The release smoke jobs need `contents: write`
because GitHub hides draft assets from read-only tokens; normal Action consumers
need only read access after publication. Only after all smoke jobs pass does the
workflow publish the draft and update the `v0` convenience tag. The full
`v0.1.0` tag is preferred in copyable examples. No crates.io publication occurs.

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

To retry a draft release after changing workflow infrastructure, use the Release
workflow's manual `tag` input, for example `v0.1.0`. It checks out that existing
tag for every build and smoke test, while using the current workflow definition.
This preserves the source of the release and avoids moving an existing tag.

## First release verification

[Release run](https://github.com/Akshith-AI/git-koi/actions/runs/45729108511):
all five platform archives built, all three downloaded Action modes passed, and
the release was published as v0.1.0. Installation from a clean source checkout
and checksum verification/execution of the downloaded Apple Silicon release
binary passed. The original MIT license and attribution are included in every
archive.

[First live daily refresh](https://github.com/Akshith-AI/git-koi/actions/runs/45747131169)
passed for Hecker2026 + Akshith-AI in pond mode, committed `fish/pond.gif`, and
attached the complete website bundle. The committed GIF was decoded and verified
as a 60-frame, four-second continuous loop. Scheduled refresh is active at 05:23
UTC; the source repository variables hold the configured accounts and mode.
