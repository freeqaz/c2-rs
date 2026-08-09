#!/bin/sh
# tests.sh — the workspace test run, with #2262's correction applied.
#
# `--no-fail-fast` is MANDATORY: a plain `cargo test --workspace` stops at the
# first failing TARGET and its `N passed` is then a truncation rather than a
# count. The TARGET count is extracted beside the test count for the same
# reason — a run that silently lost a target reports a smaller, greener number.
#
# Usage:  tests.sh <stem>        writes work/w-mmioclose/<stem>.txt
set -eu
here=$(cd "$(dirname "$0")/../.." && pwd)
stem="$1"
out="$here/work/w-mmioclose/$stem.txt"
cd "$here"
cargo test --workspace --release --no-fail-fast > "$out" 2>&1 || true
awk '
  /^test result:/ {
    n = 0
    for (i = 1; i <= NF; i++) if ($i == "passed;") n = $(i-1)
    for (i = 1; i <= NF; i++) if ($i == "failed;") f = $(i-1)
    for (i = 1; i <= NF; i++) if ($i == "ignored;") g = $(i-1)
    P += n; F += f; G += g; T += 1
  }
  END { printf "%d passed, %d failed, %d ignored, %d targets\n", P, F, G, T }
' "$out"
