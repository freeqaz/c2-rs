#!/bin/sh
# tally.sh — the `88-store-run-call` / `89-store-run-live-arg` PORT SPLIT.
#
# The sweep driver (`scripts/expr_sweep.sh`) reports `checked/mismatches/graded`
# and nothing per fragment, so a lane that widens exactly these two families
# cannot read its own payment off it. This regenerates the two fragments'
# cases into a PRIVATE directory (never the shared `$out`, which the sweep
# locks) and grades each with `c2rs diff`, printing one `Port=...` line per case.
#
#   sh work/w-midrun/tally.sh <tag> [jobs]
#
# Writes work/w-midrun/out/tally88.<tag>.txt and tally89.<tag>.txt.
# A case whose verdict is neither Match nor NotImplemented is printed as-is and
# counted separately — a `Mismatch` here is an ALARM, not a row.
set -u
tag="${1:?usage: tally.sh <tag> [jobs]}"
jobs="${2:-8}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
cases="$root/work/w-midrun/sweepcases"
outd="$root/work/w-midrun/out"
mkdir -p "$outd"
rm -rf "$cases"; mkdir -p "$cases"

python3 "$root/scripts/sweep_gen.py" "$cases" "$root/scripts/sweep.d" >/dev/null
# keep only the two families this lane can move
find "$cases" -name '*.cpp' ! -name '88-store-run-call-*' \
     ! -name '89-store-run-live-arg-*' -delete

grade() {
    pre="$1"; shift
    ls "$cases"/$pre-*.cpp 2>/dev/null | sort > "$outd/list.$pre.txt"
    n=$(wc -l < "$outd/list.$pre.txt")
    [ "$n" -gt 0 ] || { echo "FATAL: $pre generated 0 cases" >&2; exit 2; }
    : > "$outd/tally.$pre.$tag.txt"
    xargs -P "$jobs" -I{} sh -c \
        '"$0" diff "$1" 2>&1 | tail -1 | sed "s|^|$1 |"' \
        "$root/target/release/c2rs" {} \
        < "$outd/list.$pre.txt" \
        | sed 's/.*\(Port=[A-Za-z]*\).*/\1/;t;s/^/UNKNOWN /' \
        > "$outd/tally.$pre.$tag.txt"
    printf '%s %s: ' "$pre" "$tag"
    sort "$outd/tally.$pre.$tag.txt" | uniq -c | tr '\n' ' '
    printf ' (of %s)\n' "$n"
}

grade 88-store-run-call
grade 89-store-run-live-arg
