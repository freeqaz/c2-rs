#!/bin/sh
# judge.sh — THE SOLE JUDGE.  Compile the 21 quarantined TUs with the real
# toolchain (cl.exe + c1xx.dll + c2.dll under wibo) at the workload's own
# flags.txt, unmodified, and keep each reference obj.
#
# Run ONLY after the predictions are a git object.
#
#     usage: judge.sh <lane-root> <main-root> <dc3-root> <out-dir>
set -e
LANE="$1"; MAIN="$2"; DC3="$3"; OUT="$4"
mkdir -p "$OUT"
: "${C2RS_WIBO:?set C2RS_WIBO to the wibo binary}"
export C2RS_WIBO
export C2RS_COMPILERS="${C2RS_COMPILERS:-$MAIN/compilers}"
n=0
while IFS= read -r src; do
  [ -n "$src" ] || continue
  slug=$(printf '%s' "$src" | sed 's#/#__#g')
  "$LANE/target/release/c2rs" compile "$src" \
      --keep-obj "$OUT/$slug.obj" \
      --flags-file "$MAIN/work/dc3-workload/flags.txt" \
      --cwd "$DC3" > "$OUT/$slug.log" 2>&1 \
    || { echo "FAIL $src"; cat "$OUT/$slug.log"; exit 1; }
  n=$((n+1))
  printf '%3d  %-64s %s\n' "$n" "$src" "$(grep -o 'bytes' "$OUT/$slug.log" | head -1)"
done < "$LANE/work/w-quar/quar21.txt"
echo "compiled $n reference objs into $OUT"
