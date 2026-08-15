#!/bin/sh
# w-counted — MUST-FAIL MUTATIONS ON THE /Ox CODEGEN, so the accepted-set cross
# is a measurement and not an absence.
#
# W2 reports 20/20 in-class cells `match` at four /Ox-family profiles and at /O2,
# 120 gradings, mismatch 0. A green grid is only evidence if the grid can go red,
# so each mutant below breaks ONE word of the eight this class emits and the grid
# is re-run AT /Ox -- the mode the lane was dispatched to doubt.
#
#   G1  the guard COMPARE and BRANCH BIT ignore counter_unsigned (board #1788's
#       four bytes: cmpwi/bclr 4,25 where the obj has cmplwi/bclr 12,26).
#       SEPARATING CONTROL: the ten signed cells must stay `match` -- a mutant
#       that reddens both columns is measuring something else.
#   G2  the bdnz latch displacement is +1 word.  Every cell must redden.
#   G3  mtctr takes the wrong source register.   Every cell must redden.
#
# Usage: work/w-counted/codegen_mutants.sh   (run from the worktree root)
set -eu
E=crates/c2-core/src/codegen/counted_accum_loop.rs
W=work/w-counted
cp "$E" "$W/emit.rs.bak"
trap 'cp "$W/emit.rs.bak" "$E"' EXIT INT TERM

grid() {
    cargo build --release -p c2-harness >/dev/null 2>&1
    for m in O1 Ox; do
        ./target/release/c2rs gap --list "$W/cells/list.txt" \
            --flags-file "$W/probe/flags_$m.txt" --jobs 4 2>&1 \
            | grep -E '^\s+\[[0-9]' | sed 's|z:.*cells.||' \
            | awk -v M="$m" '{v=$2; n=$3; sub(/\.cpp/,"",n)
                              if (n ~ /_u$/) u[v]++; else if (n ~ /^x_/) s[v]++; else neg[v]++}
                 END{printf "    %-3s  unsigned:", M; for(k in u) printf " %s=%d", k, u[k]
                     printf "   signed:"; for(k in s) printf " %s=%d", k, s[k]
                     printf "   +=ctl:"; for(k in neg) printf " %s=%d", k, neg[k]; print ""}'
    done
}

printf '=== BASE — the shipped emitter\n'; grid

printf '\n=== G1 — the guard ignores counter_unsigned\n'
sed 's|    let (guard_cmp, guard_bo, guard_bit) = if l.counter_unsigned {|    let (guard_cmp, guard_bo, guard_bit) = if false {|' \
    "$W/emit.rs.bak" > "$E"
grep -c 'if false {' "$E" >/dev/null
grid

printf '\n=== G2 — the bdnz latch displacement is one word off (-4 -> -8)\n'
cp "$W/emit.rs.bak" "$E"
python3 - "$E" <<'PY'
import re, sys
p = sys.argv[1]
src = open("work/w-counted/emit.rs.bak").read()
m = re.search(r'encode_bdnz\(([^)]*)\)', src)
assert m, "no encode_bdnz call site"
src = src[:m.start(1)] + m.group(1) + " - 4" + src[m.end(1):]
open(p, "w").write(src)
PY
grid

printf '\n=== G3 — mtctr takes the wrong source register\n'
cp "$W/emit.rs.bak" "$E"
python3 - "$E" <<'PY'
import re, sys
p = sys.argv[1]
src = open("work/w-counted/emit.rs.bak").read()
m = re.search(r'encode_mtctr\((\w+)\)', src)
assert m, "no encode_mtctr call site"
src = src[:m.start(1)] + "3" + src[m.end(1):]
open(p, "w").write(src)
PY
grid

printf '\n=== restored\n'
