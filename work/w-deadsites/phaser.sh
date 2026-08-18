#!/bin/bash
# `w-deadsites` phase R — `w-fence163`'s rule applied to the strlit witness:
# a guard that catches a mutation INCIDENTALLY is not a guard. The four
# incumbent guards of `data-sym-strlit-fenced` are skipped BY NAME (never
# deleted — `w-readphase`'s runner defect), so MS1 and MS2 are graded against
# this lane's new per-SITE table ALONE.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
S=(-- --skip the_string_literal_admission_is_narrow_only_and_leaves_cell_b_alone
      --skip only_the_narrow_string_literal_is_admitted_and_the_wide_twin_still_refuses
      --skip the_strlit_fence_turns_on_the_local_callees_eh_state_and_nothing_else
      --skip the_older_inline_fence_shadows_this_one_on_a_walkable_tu)
./work/w-deadsites/suite.sh N0R "${S[@]}"
for m in MS1 MS2; do
    python3 work/w-deadsites/mutants.py apply "$m" || exit 1
    cargo build --release -p c2-harness > /dev/null 2>&1
    ./work/w-deadsites/suite.sh "${m}R" "${S[@]}"
    python3 work/w-deadsites/mutants.py revert
done
echo "phaser done"
