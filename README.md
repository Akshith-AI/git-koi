# LTKoi

LTKoi is a work-in-progress adaptation of
[git-koi](https://github.com/yalongwastaken/git-koi) that will turn LeetCode
progress into a male Halfmoon betta with koi marbling.

Two output modes are planned:

- **Betta:** a LeetCode betta whose growth, colors, and patterns reflect solving
  progress.
- **Pond:** the LeetCode betta alongside a GitHub koi, each developing from its
  own account's activity.

**Current status:** GitHub koi generation, LeetCode collection, betta portraits,
animated betta and shared-pond GIFs, and interactive website embeds are
available from this checkout. Release packaging and automated refresh setup
remain stage 5. The executable and Rust package are still named `git-koi`.

Collect a public LeetCode profile from this checkout:

```sh
cargo run --release --locked -- collect-leetcode YOUR_LEETCODE_USERNAME --output target/leetcode.json
```

See the [LeetCode guide](docs/leetcode.md) for requirements, field meanings,
topic overlap, and unavailable-data handling. Draw a still portrait from that
data:

```sh
cargo run --release --locked -- render-betta target/leetcode.json --output-dir target/my-betta
```

See the [betta guide and gallery](docs/betta.md) for the visual legend and
sample portraits. This generates PNG/SVG stills, not an animated betta GIF.

Generate an animated betta and hover-flare website widget:

```sh
cargo run --release --locked -- animate-betta target/leetcode.json --output-dir target/animated-betta
```

See the [animation and embedding guide](docs/animation.md) for the shared pond,
website integration, keyboard/touch controls, and reduced-motion support.

See the [roadmap](docs/roadmap.md) for the five implementation stages and the
[local baseline guide](docs/local-baseline.md) for a sample render without
GitHub authentication.

## Attribution

LTKoi is maintained in
[Akshith-AI/git-koi](https://github.com/Akshith-AI/git-koi) and is based on
[git-koi](https://github.com/yalongwastaken/git-koi) by
[Anthony Yalong (@yalongwastaken)](https://github.com/yalongwastaken). The
existing koi geometry, coloring, movement, rendering, GIF output, and supporting
code originate from his project. The images below are upstream koi examples, not
previews of the planned betta.

The original MIT license and copyright notice are preserved in
[LICENSE](LICENSE). See [NOTICE.md](NOTICE.md) for provenance and the scope of
this adaptation.

## Original GitHub koi functionality

Git koi draws a pixel-art koi from a GitHub account's public activity. Its
languages color it, its contributions grow it, its streaks mark it, and the day
that just ended decides how it swims. A nightly workflow puts the koi on a
profile README as a looping GIF, the same tool runs from the command line, and
the Rust library underneath is published on crates.io.

<p align="center">
  <img src="https://raw.githubusercontent.com/yalongwastaken/yalongwastaken/main/koi/koi.gif" width="256" alt="The author's koi, redrawn every night from the day's activity" />
</p>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/bodies-dark.png" />
  <img src="docs/bodies-light.png" alt="The six bodies, white, black, Asagi, tea, gold, and platinum, at each growth stage" />
</picture>

## Repository Structure

```text
.
├── .github/      workflows, issue forms, and Dependabot
├── docs/         the getting-started guide and the render sheets
├── examples/     the profile workflow and the koi library generator
├── src/          the library and the git-koi command
├── tests/        the public-boundary tests and their fixtures
└── action.yml    the GitHub Action
```

## Usage

### Requirements

- For a koi on your profile README: a GitHub account. The Action runs in your
  profile repository's workflow with its built-in token.
- For the command line: a binary from the
  [releases page](https://github.com/yalongwastaken/git-koi/releases), or Rust
  at or above the `rust-version` in [Cargo.toml](Cargo.toml), plus the
  [GitHub CLI](https://cli.github.com), authenticated.
- For the library: Rust at or above that same version.

[docs/getting-started.md](docs/getting-started.md) walks through all three: the
Action in a profile workflow, the command line, and the library. The API is
documented on [docs.rs](https://docs.rs/git-koi).

## Original project author

**Anthony Yalong**

- Email: yalong.anthony123@gmail.com
- GitHub: [@yalongwastaken](https://github.com/yalongwastaken)

## License

MIT. See [LICENSE](LICENSE).
