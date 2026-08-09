#!/bin/sh
# w-inlfence2 — make the committable extracts of the scan logs.
#
# The full `*.fnd.out` are 152 KB each and every one embeds the worktree's own
# absolute path (the `--cwd` and `--list` arguments the run was invoked with).
# CLAUDE.md forbids committing an absolute machine path, so what goes in the
# tree is the `gap-metric` block plus the TU verdict rows — which is every
# number this lane quotes — with `/home/<user>/...` rewritten to `<repo>`.
#
# The `*.fndiff.jsonl` (2-3 MB each) and `witness.fnd.err` (5 MB) are NOT
# committed; they are regenerable from `scan.sh` and the reproduction section
# of the rung says how.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
for stem in pre base tip tip2 cross2 cross3 cross4 witness rebase_base rebase_tip; do
    f="$here/$stem.fnd.out"
    [ -f "$f" ] || continue
    {
        grep -E '^\s*\[[0-9 ]+/878\]' "$f" || true
        grep -E 'gap-metric |EMITTED CENSUS|FUNCTION CENSUS|^  match |^  mismatch ' "$f" || true
    } | sed -e 's#/home/[^ ")]*/c2-rs#<repo>#g' -e 's#/home/[^ ")]*/dc3-decomp#<dc3>#g' \
          -e 's#/home/[^ ")]*#<path>#g' > "$here/$stem.metrics.txt"
done
# The witness is committed as its SUMMARY only, not its 5 MB of rows.
if [ -f "$here/witness.fnd.err" ]; then
    awk -F'\t' '/^XLOCAL\t/ {print $4"\t"$5}' "$here/witness.fnd.err" \
        | sort | uniq -c | sort -rn > "$here/witness.summary.txt"
fi
# The gate transcripts are scrubbed by their own step AFTER the run finishes —
# a gate still writing would fail this check on a line it has not finished. They
# are excluded here and checked explicitly before they are staged.
if grep -rl '/home/' "$here"/*.txt "$here"/*.md 2>/dev/null \
     | grep -v '/gate_' | grep -q .; then
    echo "SCRUB FAILED: an absolute path survived" >&2
    grep -rl '/home/' "$here"/*.txt "$here"/*.md 2>/dev/null | grep -v '/gate_' >&2
    exit 1
fi
echo "scrubbed ok"
