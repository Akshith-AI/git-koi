#!/usr/bin/env bash
set -euo pipefail
mode=${LTKOI_MODE:-betta}
output=${LTKOI_OUTPUT_DIR:-fish}
case "$mode" in betta|pond|koi) ;; *) echo "mode must be betta, pond, or koi" >&2; exit 2 ;; esac
case "$output" in
  ''|/*|.|..|../*|*/../*|*/..|.git|.git/*|*/.git|*/.git/*|*$'\n'*|*$'\r'*)
    echo "output-dir must be a safe relative folder in the workspace" >&2; exit 2 ;;
esac
if [[ "$mode" != koi && -z "${LTKOI_LEETCODE_SNAPSHOT:-}" && -z "${LTKOI_LEETCODE_USERNAME:-}" ]]; then
  echo "leetcode-username is required for $mode mode (or supply leetcode-snapshot)" >&2; exit 2
fi
scratch=$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/ltkoi-profiles-XXXXXX")
trap 'rm -rf "$scratch"' EXIT
leetcode=${LTKOI_LEETCODE_SNAPSHOT:-$scratch/leetcode.json}
github=${LTKOI_GITHUB_SNAPSHOT:-$scratch/github.json}
if [[ "$mode" != koi && -z "${LTKOI_LEETCODE_SNAPSHOT:-}" ]]; then
  ltkoi collect-leetcode "$LTKOI_LEETCODE_USERNAME" --output "$leetcode"
fi
if [[ "$mode" != betta && -z "${LTKOI_GITHUB_SNAPSHOT:-}" ]]; then
  ltkoi collect "${LTKOI_GITHUB_USERNAME:?github-username is required}" --output "$github"
fi
case "$mode" in
  betta) ltkoi animate-betta "$leetcode" --output-dir "$output" ;;
  pond) ltkoi animate-pond "$github" "$leetcode" --output-dir "$output" ;;
  koi) ltkoi render "$github" --output-dir "$output" --body "${LTKOI_BODY:-white}" ;;
esac
if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  printf 'mode=%s\ngif-path=%s/%s.gif\npreview-path=%s/preview.html\n' "$mode" "$output" "$mode" "$output" >> "$GITHUB_OUTPUT"
fi
