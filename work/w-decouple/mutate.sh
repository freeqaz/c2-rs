#!/bin/sh
# The MUTATION GRID. Each cell deletes exactly one clause this lane shipped,
# rebuilds, re-grades the cells that clause is supposed to fence, and restores
# the tree. Committed BEFORE it is run (#2668: `hatch_red.py` discards
# uncommitted `crates/` edits while printing "final crates/ diff: EMPTY", and
# #2699: a lane's own restore trap did the same).
#
#     work/w-decouple/mutate.sh <M1|M3|M4>
#
# Restore is `git checkout -- crates/`, which is safe ONLY because every
# `crates/` change is already committed; the script refuses if it is not.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
cd "$repo"

if [ -n "$(git status --porcelain -- crates/ fixtures/)" ]; then
    echo "REFUSED: uncommitted crates/ or fixtures/ edits — commit first (#2668)" >&2
    exit 2
fi

cell="$1"
gl=crates/c2-il/src/func/gl.rs
bind=crates/c2-il/src/func/bind.rs

case "$cell" in
  M1)
    # Delete the varargs clause the WIDE walk pays for its widening with.
    python3 - "$gl" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = """            if fit == NameFit::InlineOrStringTable
                && !looks_mangled(&runs[k].2)
                && record_is_varargs(gl, runs[k].1) != Some(false)
            {
                return Err(GlBindStop::VariadicRecord);
            }
"""
assert s.count(old) == 1, "M1 anchor not unique"
open(p, "w").write(s.replace(old, "            #[allow(clippy::no_effect)]\n            {}\n"))
PY
    graded="wdec_ec_varargs_neg wdec_ec_varargs_long_neg wdec_ecshort_leaf wdec_ecshort_eight wdec_ecshort_mix"
    ;;
  M3)
    # Give the gate's FENCE EXEMPTION the wide walk — the build this lane
    # measured and did not ship.
    python3 - "$gl" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = "pub(crate) fn plain_external_defined_names(gl: &[u8]) -> std::collections::BTreeSet<String> {\n    let (bound, _) = gl_defined_names(gl);"
new = "pub(crate) fn plain_external_defined_names(gl: &[u8]) -> std::collections::BTreeSet<String> {\n    let (bound, _) = gl_bound_names(gl);"
assert s.count(old) == 1, "M3 anchor not unique"
open(p, "w").write(s.replace(old, new))
PY
    graded="wdec_ec_localcall_neg wdec_ecshort_leaf"
    ;;
  M4)
    # Re-couple: give the BINDING the fence's narrow walk. The must-fail for
    # the whole lane.
    python3 - "$bind" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = "        let (bound, unclaimed) = super::gl::gl_bound_names(gl);"
new = "        let (bound, unclaimed) = gl_defined_names(gl);"
assert s.count(old) == 1, "M4 anchor not unique"
open(p, "w").write(s.replace(old, new))
PY
    graded="wdec_ecshort_leaf wdec_ecshort_eight wdec_ecshort_mix wdec_ec_varargs_neg wdec_ec_localcall_neg"
    ;;
  *) echo "usage: mutate.sh <M1|M3|M4>" >&2; exit 2 ;;
esac

echo "== $cell — clause deleted, rebuilding"
if ! cargo build --release > "$here/mut_$cell.build" 2>&1; then
    echo "BUILD FAILED"
    tail -20 "$here/mut_$cell.build"
    git checkout -- crates/
    exit 1
fi
# `Port=` AND the gate CAUSE. A `_neg` cell whose clause only ever moves a
# refusal from one gate to a later one is graded by the cause and by nothing
# else — `Port=NotImplemented` on both sides of the mutation is not evidence the
# cell grades nothing (that is M3's whole shape, and reading only `Port=` is how
# this lane first concluded it did).
cp "$repo/target/release/c2rs" "$here/c2rs-mut"
: > "$here/mut_list.txt"
for f in $graded; do
    printf '  %-28s ' "$f"
    "$repo/target/release/c2rs" diff "fixtures/cpp/$f.cpp" 2>&1 | tail -1 | sed 's/.*  //'
    printf 'fixtures/cpp/%s.cpp\n' "$f" >> "$here/mut_list.txt"
done
# BOTH modes. `c2rs diff` above is `/Ox`, and W-FENCE2's exemption is gated on
# every segment being `/O1` (mode gate in the parser, board #1638) — so a cell
# fencing that clause is a NO-OP at `/Ox` by construction, and grading only the
# mode the CLI defaults to reads as "this cell grades nothing".
for m in o1 ox; do
    case "$m" in
        o1) printf '/O1 /GS- /c\n' > "$here/mut_flags.txt" ;;
        ox) printf '/Ox /GS- /c\n' > "$here/mut_flags.txt" ;;
    esac
    echo "  -- gate causes at /$m"
    # `|| true`: `gap` exits nonzero when a TU MISMATCHES, which is the whole
    # point of a must-fail cell — under `set -e` the grid would abort before its
    # own restore and leave the tree mutated. It did, once.
    "$here/c2rs-mut" gap --list "$here/mut_list.txt" --flags-file "$here/mut_flags.txt" \
        --cwd "$repo" --jsonl "$here/mut_${cell}_$m.jsonl" > "$here/mut_${cell}_$m.gap" 2>&1 \
        || true
    python3 "$here/rowfields.py" "$here/mut_${cell}_$m.jsonl" \
        | grep -E '^===|class |gate_causes' | sed 's/^/    /'
done
git checkout -- crates/
cargo build --release > /dev/null 2>&1
echo "== $cell restored; crates/ diff:"
git status --porcelain -- crates/ | sed 's/^/   /'
echo "   (empty above == restored)"
