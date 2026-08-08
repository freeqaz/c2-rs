#!/bin/sh
# lit.sh -- ask the READER's census key of every cell of the allocation-key
# literature.  Prints the key ONLY; no obj is disassembled and no register is
# read, so this cannot compromise a frozen grid.
set -u
R="$(cd "$(dirname "$0")/../../.." && pwd)"
C="$R/target/release/c2rs"; F="$R/work/dc3-workload/flags.txt"
for d in w-self2b/gridZ w-prod/gridP w-mixed/gridM w-mixkind/gridX w-spell/gridB \
         w-alloc2 w-refbind w-seam w-heap; do
  [ -d "$R/work/$d" ] || continue
  find "$R/work/$d" -maxdepth 2 -name '*.cpp' | sort | while read -r s; do
    k=$("$C" census "${s#$R/}" --flags-file "$F" 2>&1 \
        | awk '/functions in class/{c=$NF" "$(NF-3)} / GAP /{for(i=1;i<=NF;i++)if($i=="GAP"){print $(i+1);f=1}} END{if(!f)print "IN-CLASS"}' | head -1)
    printf '%s\t%s\t%s\n' "$d" "$(basename "$s")" "$k"
  done
done
