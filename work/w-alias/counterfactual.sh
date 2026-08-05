#!/bin/bash
# counterfactual.sh — break the alias decode transiently, measure, revert.
#
# "A breaker that breaks everything proves nothing."  Both breakers below are a
# ONE-TOKEN edit and each has a prediction registered in this file BEFORE it was
# run, naming which numbers must move and — the load-bearing half — which must
# NOT.  The script reverts in the same run and asserts the tree is clean after.
#
#   usage: work/w-alias/counterfactual.sh
#
# REGISTERED, before running:
#
#   B1 — read the target field one byte late (`gl_alias_table` -> shift 1).
#        MUST move : bound 95820 -> 2449 ; shape 95818 -> 0
#                    JFP_ALIAS 308 / 0.94413 -> exactly JFP, 132 / 0.92655
#                    ALIAS_IN  472 / 0.99243 -> exactly ORACLE, 151 / 0.97888
#        MUST NOT  : tag10 96220 (the RECORD is still found; only the FIELD moved)
#                    RGL / INIT / SKIP / ORACLE / JFP unchanged, every digit
#                    dom_with_body 0
#
#   B2 — accept every kind-4 tag as an alias (0x04 / 0x0E / 0x10), the
#        one-character relaxation that RAISES a count.
#        MUST move : tag10 far above 96220 ; the corpus test's
#                    `dom_with_body == 0` assertion FIRES, because a bodied name
#                    lands in dom(alias) and rule 4 would suppress a symbol that
#                    must be emitted
#        MUST fail : the unit test `a_body_record_is_never_an_alias`
#
set -uo pipefail
cd "$(dirname "$0")/../.."
WT="$PWD"
SRC=crates/c2-il/src/func/glalias.rs
OUT=work/w-alias/cf
mkdir -p "$OUT"

# The main repo, which holds work/w-emit/truth and work/w-emitp/. A worktree
# sits three directories down; override with C2RS_LANEROOT if it does not.
export C2RS_LANEROOT="${C2RS_LANEROOT:-$(cd "$WT/../../.." && pwd)}"
TRUTH="$C2RS_LANEROOT/work/w-emit/truth"

clean_or_die() {
    if [ -n "$(git status --porcelain crates/)" ]; then
        echo "FATAL: crates/ is dirty at step '$1'"; git status --porcelain crates/; exit 1
    fi
    echo "  crates/ clean at '$1'"
}

measure() {   # $1 = tag
    local tag="$1"
    C2RS_ALIAS_CACHEIDX="$WT/work/w-alias/cacheidx.tsv" \
    C2RS_ALIAS_OUT="$WT/$OUT/$tag.jsonl" C2RS_ALIAS_JOBS=6 \
        cargo test -p c2-il --release --test gl_alias_corpus -- --nocapture \
        > "$OUT/$tag.corpus.log" 2>&1
    grep -E "^alias-corpus|panicked at|dom\(alias\)" "$OUT/$tag.corpus.log" | head -5
}

model() {     # $1 = tag
    local tag="$1"
    C2RS_ALIAS_JSONL="$WT/$OUT/$tag.jsonl" \
        python3 work/w-alias/scan_rust.py work/w-alias/cacheidx.tsv \
            work/w-alias/dtruth "$TRUTH" "$OUT/$tag.scan.jsonl" 6 \
        > "$OUT/$tag.scan.log" 2>&1
    tail -1 "$OUT/$tag.scan.log"
    python3 "$C2RS_LANEROOT/work/w-emitp/score.py" "$OUT/$tag.scan.jsonl" \
        > "$OUT/$tag.score.txt" 2>&1
    sed -n '/THE MODELS/,/^$/p' "$OUT/$tag.score.txt"
}

echo "=============================== BASELINE"
clean_or_die "start"
git rev-parse --short HEAD

echo "=============================== B1 — the field, one byte late"
sed -i 's|    gl_alias_table_shifted(gl, 0)|    gl_alias_table_shifted(gl, 1)|' "$SRC"
git diff --stat -- "$SRC"
[ -n "$(git diff -- "$SRC")" ] || { echo "FATAL: B1 did not apply"; exit 1; }
measure b1
model b1
git checkout -- "$SRC"
clean_or_die "after B1"

echo "=============================== B2 — accept every kind-4 tag"
sed -i 's|        if tag != ALIAS_TAG {|        if !KIND4_TAGS.contains(\&tag) {|' "$SRC"
[ -n "$(git diff -- "$SRC")" ] || { echo "FATAL: B2 did not apply"; exit 1; }
echo "-- the unit tests, which must FAIL:"
cargo test -p c2-il --lib glalias 2>&1 | grep -E "^test func::glalias|test result" | tail -20
echo "-- the corpus assertion, which must FIRE:"
measure b2
git checkout -- "$SRC"
clean_or_die "after B2"

echo "=============================== RESTORED"
cargo test -p c2-il --lib glalias 2>&1 | grep -E "test result" | tail -2
git status --porcelain crates/ | sed 's/^/  /'
echo "  (empty above == the tree is exactly as it was)"
