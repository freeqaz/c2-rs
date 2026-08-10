#!/bin/bash
# w-frame783 — MUST-FAIL mutations (#2698/#2699).
#
# Each mutation deletes the WHOLE conjunction a cell grades, not a corner of it,
# and the cell must go RED. A cell that stays green under its own mutation
# grades nothing and is named-not-counted in the rung.
#
# THE TREE IS COMMITTED BEFORE THIS RUNS (#2668, #2699: a lane's own restore
# trap discarded uncommitted crates/ edits while reporting success). The restore
# is `git checkout -- crates/` against a committed HEAD, and the script asserts
# a clean tree on entry and on exit.
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"

if [ -n "$(git status --porcelain crates/)" ]; then
    echo "REFUSING: crates/ is dirty. Commit first (#2668)." >&2
    exit 1
fi
HEAD0="$(git rev-parse HEAD)"

CODEC=crates/c2-il/src/codec.rs
GL=crates/c2-il/src/func/gl.rs

run_cell() {  # run_cell <test-name>  -> prints PASS/FAIL
    if cargo test --release -p c2-il --lib "$1" 2>&1 | grep -q "test result: ok. 1 passed"; then
        echo PASS
    else
        echo FAIL
    fi
}

restore() {
    git checkout -- crates/
    if [ -n "$(git status --porcelain crates/)" ] || \
       [ "$(git rev-parse HEAD)" != "$HEAD0" ]; then
        echo "RESTORE BROKEN — stop and inspect" >&2
        exit 2
    fi
}

mutate() {  # mutate <id> <description> <cells...>  (mutation already applied)
    local id="$1"; shift
    local desc="$1"; shift
    echo "--- $id: $desc"
    if ! cargo build --release -p c2-il > /dev/null 2>&1; then
        echo "    (mutation does not compile — that is itself a refusal; skipping)"
        restore
        return
    fi
    for c in "$@"; do
        printf '    %-62s %s\n' "$c" "$(run_cell "$c")"
    done
    restore
}

echo "=== baseline (unmutated) — every cell must PASS"
for c in the_relaxed_framing_reaches_the_binding_and_neither_fence \
         the_relaxed_framing_refuses_an_offset_past_the_bound \
         the_name_lookup_rewrite_keeps_the_distance_bound \
         the_three_walk_free_readers_differ_by_exactly_one_byte_each; do
    printf '    %-62s %s\n' "$c" "$(run_cell "$c")"
done

# M1 — the ship itself: point the gate's binding back at the incumbent framing.
python3 - "$GL" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = ("pub(crate) const GATE_BIND_FRAME: fn(&[u8], usize) -> bool =\n"
       "    crate::codec::gl_offset_framed_relaxed;")
new = ("pub(crate) const GATE_BIND_FRAME: fn(&[u8], usize) -> bool =\n"
       "    crate::codec::gl_offset_framed;")
assert old in s, "M1 target not found — the mutation would have graded nothing"
open(p, "w").write(s.replace(old, new))
PY
mutate M1 "GATE_BIND_FRAME reverted to codec::gl_offset_framed (the ship, deleted)" \
    the_relaxed_framing_reaches_the_binding_and_neither_fence \
    the_three_walk_free_readers_differ_by_exactly_one_byte_each

# M2 — delete the offset bound, i.e. #2783 exactly as filed.
sed -i 's|^        && u32::from_le_bytes(\[gl\[o + 1\], gl\[o + 2\], gl\[o + 3\], gl\[o + 4\]\]) < GL_OFFSET_MAX$|        \&\& true|' "$CODEC"
mutate M2 "GL_OFFSET_MAX clause deleted (#2783 as filed)" \
    the_relaxed_framing_refuses_an_offset_past_the_bound \
    the_three_walk_free_readers_differ_by_exactly_one_byte_each

# M3 — THE HAZARD: give the FENCE ground set the widened framing too. This is
# the change #2622/#2623 measured at -1 fnbyte-exact, and the decoupling cell
# exists to stop it landing silently.
sed -i 's|^        crate::codec::gl_offset_framed,\n        NameFit::StringTableOnly,|XX|' "$GL"
python3 - "$GL" <<'PY'
import sys, re
p = sys.argv[1]
s = open(p).read()
s = s.replace(
    "    gl_defined_names_framed(\n        gl,\n        sep26,\n"
    "        crate::codec::gl_offset_framed,\n        NameFit::StringTableOnly,\n    )",
    "    gl_defined_names_framed(gl, sep26, GATE_BIND_FRAME, NameFit::StringTableOnly)")
open(p, "w").write(s)
PY
mutate M3 "the FENCE ground set widened to GATE_BIND_FRAME (#2622/#2623's hazard)" \
    the_relaxed_framing_reaches_the_binding_and_neither_fence

# M4 — delete the name-distance bound the partition_point rewrite carries.
python3 - "$GL" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
s = s.replace(
    "                k @ 1.. if p - runs[k - 1].1 <= MAX_NAME_TO_OFFSET => k - 1,",
    "                k @ 1.. => k - 1,")
open(p, "w").write(s)
PY
mutate M4 "MAX_NAME_TO_OFFSET deleted from the rewritten name lookup" \
    the_name_lookup_rewrite_keeps_the_distance_bound

echo "=== final: crates/ diff must be EMPTY and HEAD unmoved"
echo "    crates/ diff: $( [ -z "$(git status --porcelain crates/)" ] && echo EMPTY || echo DIRTY )"
echo "    HEAD: $(git rev-parse --short HEAD)  (entered at ${HEAD0:0:7})"
cargo build --release -p c2-harness > /dev/null 2>&1
