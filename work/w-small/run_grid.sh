#!/bin/sh
# Grade the w-small Lead 2 grid at one mode, through `c2rs gap` (the only entry
# point that takes --flags-file).  $1 = mode string.
set -eu
mode="$1"
work="$(cd "$(dirname "$0")" && pwd)"
c2rs="$(cd "$work/../.." && pwd)/target/release/c2rs"
echo "$mode /GS- /c" > "$work/grid/flags.txt"
res="$work/grid/res_$(echo "$mode" | tr -d '/ ').txt"
"$c2rs" gap --list "$work/grid/list.txt" --flags-file "$work/grid/flags.txt" \
    --jobs "${JOBS:-6}" 2>&1 | grep -E '^\s+\[[0-9]+/' | sed 's/.*\] *//' \
    | awk '{print $1, $2}' | sort > "$res"
pm=0; pb=0; nn=0; nb=0; mm=0
while read -r kind f; do
  v=$(awk -v f="$f" '$2==f {print $1}' "$res")
  [ "$v" = mismatch ] && mm=$((mm+1))
  if [ "$kind" = POS ]; then
    if [ "$v" = match ]; then pm=$((pm+1)); else pb=$((pb+1)); echo "  POS-NOT-MATCH $v $f"; fi
  else
    if [ "$v" = match ]; then nb=$((nb+1)); echo "  NEG-MATCHED $f"; else nn=$((nn+1)); fi
  fi
done < "$work/grid/manifest.txt"
echo "mode=[$mode /GS- /c]  POS match=$pm/$((pm+pb))  NEG refused=$nn/$((nn+nb))  MISMATCH=$mm"
