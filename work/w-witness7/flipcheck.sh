#!/usr/bin/env bash
# `w-witness7` — the CHEAP half of the GREEN -> RED demonstration.
#
# A suite run is ~4 minutes; a `c2rs census` of one cell against a mutated
# `c2-il` is ~15 seconds. This script asks, per mutant, the question the guard
# is built on — *does the cell this row asserts about actually MOVE?* — before
# any suite run pays for the answer.
#
# It is not a substitute for the tip suite runs. It is the design validation,
# and its log is quoted as such.
set -u
cd "$(dirname "$0")/../.." || exit 1
C=work/w-witness7/cells
F="--flags-file $C/flags.txt"
FD="--flags-file $C/flags_od.txt"

say() { printf '%-14s %-16s ' "$1" "$2"; }
cen() { ./target/release/c2rs census "$1" $2 2>&1 \
        | sed -nE 's/^ *\[ *[0-9]+\] +(ok|GAP) +([^ ]+).*/\1 \2/p' | tr '\n' ' '; echo; }

run() {
  local mut=$1; shift
  python3 work/w-witness7/patch.py apply "$mut" >/dev/null || return 1
  cargo build --release -p c2-harness --bin c2rs >/dev/null 2>&1 || {
      echo "$mut BUILD FAILED"; python3 work/w-witness7/patch.py revert >/dev/null; return 1; }
  for spec in "$@"; do
    cell=${spec%%:*}; fl=${spec##*:}
    say "$mut" "$cell"
    if [ "$fl" = od ]; then cen "$C/$cell.cpp" "$FD"; else cen "$C/$cell.cpp" "$F"; fi
  done
  python3 work/w-witness7/patch.py revert >/dev/null
}

echo "=== BASE (clean tree)"
cargo build --release -p c2-harness --bin c2rs >/dev/null 2>&1
for spec in ssl_ns:o1 ssl_local:o1 ssl_bss:o1 gsl_init:o1 gsl_bss:o1 gsl_comdat:o1 \
            ca6_nonformal:o1 ca6_formal:o1 ca8_computed:o1 cs4:o1 ca6_formal:od; do
  cell=${spec%%:*}; fl=${spec##*:}
  say "BASE" "$cell.$fl"
  if [ "$fl" = od ]; then cen "$C/$cell.cpp" "$FD"; else cen "$C/$cell.cpp" "$F"; fi
done

echo "=== MUTANTS"
run M-CS3  ssl_ns:o1
run M-CS3B ssl_ns:o1
run M-CS4  cs4:o1
run M-CS9  ca6_formal:od
run M-CA6  ca6_nonformal:o1
run M-CA8  ca8_computed:o1
run M-B2   ssl_ns:o1 ssl_bss:o1
run M-B7   gsl_init:o1 gsl_comdat:o1
echo "=== FLIPCHECK DONE"
