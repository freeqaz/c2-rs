#!/bin/sh
# w-front3 — the NULL-LIFT CONTROL.
#
# The positive question, asked before any lift is trusted: *would this have
# reported differently if the clause I lifted were not the binding one?*
#
# Every one of the six hatches is applied ALONE to every one of the seventeen
# frontier TUs, and the round-0 first-refusal key is read. A hatch that is not
# the TU's binding clause must leave the key BYTE-IDENTICAL to the unhatched
# baseline; a hatch that IS must move it. The DISCRIMINATING CELLS — the pairs
# where the key moves — are counted and printed, because "no cell disagreed"
# over zero discriminating cells is a vacuous negative, not a narrow one.
#
#   sh work/w-front3/control.sh > work/w-front3/control.txt
set -u
R="$(cd "$(dirname "$0")/../.." && pwd)"
C="$R/target/release/c2rs"
F="$R/work/dc3-workload/flags.txt"
# The dc3 tree is DERIVED, never hard-coded (CLAUDE.md) — walk up from the repo
# root looking for a sibling, exactly as `work/w-mrslot/ladder.sh` does.
sib() {
  d="$R"
  while [ "$d" != "/" ]; do
    [ -d "$d/../$1" ] && { (cd "$d/../$1" && pwd); return 0; }
    d="$(dirname "$d")"
  done
  return 1
}
D="${C2RS_DC3:-$(sib dc3-decomp)}"
[ -d "$D" ] || { echo "SKIP: no dc3 tree (set C2RS_DC3)"; exit 3; }
key() { "$C" census "$1" --flags-file "$F" --cwd "$D" 2>&1 \
        | sed -nE 's/^ *\[ *[0-9]+\] GAP ([^ ]+).*/\1/p' | sort | tr '\n' ',' ; }
for t in $(cat "$R/work/w-front3/tus.txt"); do
  base="$(key "$t")"
  for h in param-width assign-store-type call-arg-lit-permuted expr-shr-mixed-sign store-run-bind-mixed-kind; do
    got="$(W_FRONT3_LIFT=$h key "$t")"
    if [ "$got" = "$base" ]; then v=SAME; else v=MOVED; fi
    printf '%-46s %-28s %s\n' "$t" "$h" "$v"
  done
done
