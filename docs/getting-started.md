# Getting started with LTKoi

LTKoi is this fork's command, package, and GitHub Action. Its release assets
come from [Akshith-AI/git-koi](https://github.com/Akshith-AI/git-koi), with
original upstream credit preserved. Install from GitHub releases or the Git
repository; this project does not publish the original author's crates.io
package.

## Profile README setup

Your profile repository is the public repository named after your GitHub user,
such as `yourname/yourname`. Copy
[the complete workflow](../examples/profile-workflow.yml) into it as
`.github/workflows/refresh-fish.yml`, then add repository variables under
Settings → Secrets and variables → Actions → Variables:

- `LEETCODE_USERNAME`: your public username, without a profile URL.
- `LTKOI_MODE`: `pond` (default), `betta`, or `koi`.
- `LTKOI_GITHUB_USERNAME`: optional; defaults to the repository owner.
- `LTKOI_PUBLISH_WIDGET`: optional `true` to commit website assets too.

Run Refresh fish manually once, then embed the corresponding file:

```html
<img src="fish/betta.gif" width="512" alt="My LeetCode Halfmoon betta" />
```

For pond mode use `fish/pond.gif`; for koi mode use `fish/koi.gif`. Koi mode
does not require a LeetCode username. A workflow without a configured LeetCode
username skips betta/pond rather than collecting an invented account. The
installed Action reports a useful error if called directly without the required
username.

The example grants `contents: write` to its commit step. Repositories with
branch rules may require a different publication branch or an approved pull
request; adapt the commit step to your repository's rules. The workflow writes
only the selected GIF unless full bundle publishing is enabled. Manual mode
choices apply to that run; the repository variable controls scheduled runs.

## Action settings

```yaml
- uses: Akshith-AI/git-koi@v0.1.0
  id: fish
  with:
    mode: pond
    leetcode-username: YOUR_LEETCODE_USERNAME
    github-username: YOUR_GITHUB_USERNAME
    output-dir: fish
```

| Input               | Default          | Meaning                                               |
| ------------------- | ---------------- | ----------------------------------------------------- |
| `mode`              | `betta`          | `betta`, `pond`, `koi`                                |
| `leetcode-username` | Unset            | Required for betta/pond unless a snapshot is supplied |
| `github-username`   | Repository owner | GitHub account for pond/koi                           |
| `body`              | `white`          | GitHub-only koi body; pond uses Asagi                 |
| `output-dir`        | `fish`           | Safe relative workspace folder                        |
| `leetcode-snapshot` | Unset            | Optional cached LeetCode JSON instead of a live query |
| `github-snapshot`   | Unset            | Optional cached GitHub JSON instead of a live query   |
| `token`             | `github.token`   | Public GitHub collection and release download token   |

The example daily workflow defaults to `pond`; the reusable Action defaults to
`betta` so a minimal call needs only a LeetCode username. Snapshot inputs are
useful for reproducible offline rendering and tests. They must follow the
current JSON schemas; a LeetCode snapshot is not interchangeable with a GitHub
snapshot.

Outputs are `mode`, `gif-path`, and `preview-path`. For example,
`${{ steps.fish.outputs.gif-path }}` is `fish/pond.gif` in pond mode. The Action
requires Bash, curl, and the GitHub CLI, all available on supported Linux/macOS
GitHub-hosted runners. Windows CLI binaries are available separately; the
composite Action does not support Windows runners.

## Install locally

Choose your platform archive from
[the releases page](https://github.com/Akshith-AI/git-koi/releases):

| Platform            | Target                       |
| ------------------- | ---------------------------- |
| Linux x64           | `x86_64-unknown-linux-musl`  |
| Linux ARM64         | `aarch64-unknown-linux-musl` |
| macOS Intel         | `x86_64-apple-darwin`        |
| macOS Apple Silicon | `aarch64-apple-darwin`       |
| Windows x64         | `x86_64-pc-windows-msvc`     |

Verify the accompanying checksum. On macOS use
`shasum -a 256 -c ARCHIVE.sha256`; on Linux use `sha256sum -c ARCHIVE.sha256`.
On Windows use `Get-FileHash ARCHIVE.zip -Algorithm SHA256` in PowerShell and
compare it with the checksum text, then use `Expand-Archive`. Add the extracted
binary to your PATH.

Or install from the tagged Git source with Rust 1.88 or newer:

```sh
cargo install --git https://github.com/Akshith-AI/git-koi --tag v0.1.0 --locked
```

The command is now `ltkoi`; the inherited `git-koi` executable name is not
installed by this fork. Commands in old upstream documentation refer to the
original project.

## Collect and render

```sh
ltkoi collect-leetcode YOUR_LEETCODE_USERNAME --output leetcode.json
ltkoi animate-betta leetcode.json --output-dir fish
```

LeetCode collection uses curl without a login. For the GitHub account, install
and authenticate `gh`, then collect and compose:

```sh
gh auth login
ltkoi collect YOUR_GITHUB_USERNAME --output github.json
ltkoi animate-pond github.json leetcode.json --output-dir fish
```

To render the original GitHub koi alone:

```sh
ltkoi render github.json --output-dir koi --body asagi
```

Supported body names are `white`, `black`, `asagi`, `tea`, `gold`, and
`platinum`. Use `ltkoi --help` and `ltkoi --version` for the local build's
details.

[LeetCode fields](leetcode.md) · [Portraits](betta.md) ·
[Animation and website embeds](animation.md)

## Rust library

Use this fork as a Git dependency rather than the upstream registry crate:

```toml
[dependencies]
ltkoi = { git = "https://github.com/Akshith-AI/git-koi", tag = "v0.1.0" }
```

The Rust namespace is `ltkoi`; public modules include `leetcode`, `betta`,
`history`, `koi`, `render`, and `output`. Run `cargo doc --no-deps --open` from
a checkout for local API documentation. Collection owns network access;
rendering saved snapshots is deterministic and does not read the wall clock.
