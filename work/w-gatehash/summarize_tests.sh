#!/bin/sh
# Reduce a `cargo test --workspace` log to its 63 per-target result lines plus a
# summed total, and say which tests FAILED by name. The full log is 272 KB of
# mostly `ok` and is not evidence of anything the summary is not — but the
# per-target lines ARE the denominator (#3470: a total with no target count
# cannot tell a short run from a clean one).
#
# Usage: summarize_tests.sh <raw log> <out>
set -eu
raw="$1"; out="$2"
{
    echo "# cargo test --workspace --release --no-fail-fast   (C2RS_REQUIRE_TOOLCHAIN=1)"
    echo
    grep '^test result:' "$raw" \
      | awk '{p+=$4; f+=$6; i+=$8}
             END{printf "TOTAL: %d targets, %d passed, %d failed, %d ignored\n", NR, p, f, i}'
    echo
    echo "## tests that FAILED, by name"
    grep '^test .* FAILED$' "$raw" || echo "  (none)"
    echo
    echo "## per-target result lines (the denominator)"
    grep '^test result:' "$raw"
} > "$out"
cat "$out"
