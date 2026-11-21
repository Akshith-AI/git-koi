# LTKoi roadmap

## 1. Foundation — complete

Establish a reproducible build and fixture-based GitHub koi render. Retain the
upstream license, document attribution, and distinguish existing features from
planned work. Record validation in local-baseline.md.

Acceptance: the existing checks pass and a sample GIF, PNG, and preview render
from the committed account fixture without credentials.

## 2. LeetCode profile collection — complete

Fetch unique solved counts by difficulty, available topic statistics, and
activity history. Validate endpoint fields before committing to visual mappings.
Represent missing or restricted activity separately from confirmed inactivity.
Recent accepted submissions are a limited window, not a complete solve history.
Calendar submissions must not be described as unique accepted problems.
Normalize aggregate tag incidences into visual weights while documenting that
per-problem multi-tag bias cannot be removed from aggregate counts alone.

Acceptance: save validated profiles for normal, empty, and calendar-restricted
accounts; reject unknown accounts without overwriting existing output. Document
the limits of each metric. See [the collection guide](leetcode.md) for commands,
coverage, and the verification record.

## 3. Male Halfmoon betta renderer

Create a distinct compact betta body, broad 180-degree fan-shaped caudal fin,
flowing dorsal and anal fins, and slender paired ventral fins. Use irregular koi
marbling inspired by the red, white, dark blue, and black reference palette.
Preserve recognizable anatomy at every growth stage.

Map unique solves to growth, difficulty proportions to color balance, and the
dominant DSA topics to marbling. Keep a stable identity seed and gradual visual
changes. Set thresholds for realistic LeetCode totals rather than inheriting
GitHub's contribution scale. Draw original graphics; do not redistribute the
reference photograph.

Acceptance: deterministic still images across representative progress levels,
with a documented legend for the visual traits.

## 4. Animation and two output modes

Animate swimming and fin movement. Provide betta-only output for a LeetCode
username and a shared pond for separate GitHub and LeetCode usernames. Each fish
retains independent progression. Compose the two fish and coordinate their paths
in the shared scene. Handle unavailable activity without fabricating a streak.

Also provide an interactive website embed: hovering over the betta spreads its
Halfmoon tail and fins and flares its gill covers; leaving the fish eases back
to normal swimming. Tapping triggers a brief flare on touch devices. Respect
reduced motion preferences and offer keyboard focus activation. GIFs remain the
portable README format and cannot react to pointer events.

Acceptance: seamless looping GIFs for both modes, with no clipped fins and with
both fish readable in the pond; an interactive website preview supports hover,
keyboard focus, touch, and reduced motion.

## 5. Release and onboarding

Adapt the Action and release packaging for this fork. Document username inputs,
mode selection, daily refresh, examples, attribution, and the trait legend.
Verify that release downloads use this fork's assets before publishing. The
inherited Action and profile example currently refer to upstream releases and
must not be presented as an LTKoi release.

Acceptance: a new user can follow the instructions and generate either mode from
a published release. Publishing is a later stage, not part of the foundation.
