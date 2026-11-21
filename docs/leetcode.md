# LeetCode profile collection

Stage 2 adds a public LeetCode data source for the upcoming betta renderer. It
collects JSON; it does not draw the betta yet. The GitHub koi generator
continues to work independently.

## Collect a profile

From this checkout, install Rust 1.88 or newer and curl. No LeetCode password,
session cookie, API key, or GitHub login is required for this command.

```sh
cargo run --release --locked -- collect-leetcode YOUR_LEETCODE_USERNAME --output target/leetcode.json
```

The output's parent folder must exist. Cargo creates `target/` during the build.
To write JSON to standard output instead, omit `--output`:

```sh
cargo run --release --locked -- collect-leetcode YOUR_LEETCODE_USERNAME
```

The command exits 0 if required profile data is valid, even if optional fields
are unavailable. Inspect each optional field's `status`. A required request,
unknown account, malformed required response, or output failure exits 1. Invalid
command syntax exits 2. Existing output files are replaced only after successful
collection and serialization, using a sibling temporary file.

The `render` command still accepts GitHub snapshots only. Do not feed this JSON
to it; betta geometry and animation are stages 3 and 4.

## Snapshot fields and their meaning

| Field             | Meaning                                                          | Intended visual role                      |
| ----------------- | ---------------------------------------------------------------- | ----------------------------------------- |
| `schema_version`  | Format version, currently `1`                                    | Reader compatibility                      |
| `username`        | Canonical endpoint username, matched case-insensitively to input | Stable fish identity                      |
| `observed_on`     | Actual UTC collection date                                       | Observation window                        |
| `solved`          | Unique accepted problems: All, Easy, Medium, Hard                | Growth and difficulty palette             |
| `topics`          | Overlapping accepted-problem counts by topic slug                | Koi-marble pattern influences             |
| `calendar`        | Requested current UTC year's submission activity                 | Activity signals, subject to availability |
| `recent_accepted` | Up to 20 recent accepted submissions                             | Recent accepted activity                  |

`All` must equal Easy + Medium + Hard. Missing counts, negative counts,
duplicate difficulties, and inconsistent totals are rejected instead of becoming
zero. The snapshot is an observation across four requests, not an atomic
transaction; an account changing during collection may make an optional section
unavailable.

Optional sections have one of two explicit forms:

```json
{ "status": "available", "data": [] }
```

```json
{
  "status": "unavailable",
  "reason": "LeetCode request failed: endpoint returned GraphQL errors"
}
```

An available empty list means confirmed empty observations. An unavailable
calendar means its activity is unknown; it must not make the fish appear
inactive. Raw server errors are not copied into diagnostics.

## Topic overlap

LeetCode groups topic statistics into fundamental, intermediate, and advanced
skills. We flatten all groups, order topics by slug, deduplicate identical
records, and reject conflicting duplicate counts. A topic cannot contain more
accepted problems than the account's total solved count.

Problems can have multiple tags. Topic counts therefore need not sum to the
unique solved total. The `topic_weights` library helper divides each positive
count by the sum of all positive tag incidences. This supplies stable visual
weights, not a disjoint breakdown of problems. It cannot remove multi-tag bias
without per-problem tag membership, which this collector does not obtain.

## Calendar and recent history limits

- The calendar is explicitly requested for the observation's UTC year. It is not
  lifetime activity; earlier years are not fetched in stage 2.
- `submissionCalendar` is itself a JSON string. It is decoded into UTC dates and
  unsigned submission counts. Dates outside the requested window, future dates,
  and inconsistent active-day totals make this optional section unavailable.
- Calendar submissions may include failed attempts and repeated submissions.
  They are not unique accepted solves.
- `reported_streak` preserves the endpoint's streak value. We do not claim that
  it is the currently active daily streak or reuse the GitHub weekly-streak
  rule.
- Recent accepts preserve different submissions of the same problem. Duplicate
  submission IDs are collapsed; conflicting copies are rejected. The window is
  sorted newest first. It is not sufficient to reconstruct lifetime solves.

## Endpoint and failures

The collector queries `https://leetcode.com/graphql/` directly over HTTPS using
curl, with no third-party proxy. The website endpoint is not a versioned public
API contract and may change. Live requests are kept out of automated tests.

Each request has a 10-second connection limit, a 30-second overall timeout, and
a 2 MiB response limit. There are four sequential queries for a valid account:
required identity/solved counts, then optional topics, calendar, and recent
accepts. Invalid usernames are rejected before any request. A failed required
profile request stops collection. Optional failures remain explicit unavailable
sections; they are not automatically retried.

Run this occasionally, such as once daily, rather than fetching on every website
hover. Website animation should use already collected data locally. Hover,
keyboard focus, and touch flaring belong to stage 4, not to data collection.

## Reproduce the tests

```sh
cargo test --locked --test leetcode_parsing --test leetcode_collection --test leetcode_command
cargo clippy --locked --all-targets -- -D warnings
```

The fixtures in `tests/fixtures/leetcode/` are synthetic. Parsing and collection
tests cover empty accounts, unknown users, restricted calendars, transport
failures, malformed payloads, overlapping tags, and mismatched identities. On
Unix, CLI tests replace curl with a per-test local stub to exercise stdout, file
replacement, unavailable optional fields, and errors without network access.

## Verification record

Implementation and verification were performed on November 25, 2026. At the
project owner's request, this stage's 15 commits have assigned author and
committer timestamps on November 16–20, 2026, between 8 a.m. and 10 p.m.
America/Detroit. Those timestamps do not represent the actual execution dates.

Live checks on November 25 used `lee215` (all optional sections available) and
`leetcode` (calendar unavailable, solved counts/topics/recent accepts retained).
Live snapshots are local ignored artifacts under `target/`, not test fixtures.

Validation on Rust 1.88.0 / Apple Silicon macOS:

- All 115 tests passed in the release profile, including 22 new LeetCode tests.
- The 22 LeetCode tests also passed in the development profile.
- Rust formatting, Clippy with warnings denied, and all-target checking passed.
- Locked release build and API documentation with warnings denied passed.
- Repository Markdown/JSON/configuration formatting passed.
- The inherited Asagi koi fixture still generated its GIF, PNG, and preview.
- The original upstream LICENSE remains unchanged.
