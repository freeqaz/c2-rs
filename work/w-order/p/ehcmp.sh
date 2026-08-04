#!/bin/sh
# ehcmp.sh — grade one probe at an arbitrary LANE's flags: the real toolchain's
# obj against the port's, both dumped with scripts/gt_dump.py and diffed.
#
# `c2rs compile` takes --flags-file, `c2rs prefilter` takes repeated --flag, so
# the flag string is written once here and rendered both ways. Board #195 is
# about substituting a DIFFERENT flag set, not about naming one explicitly.
set -eu
root="$(cd "$(dirname "$0")/../../.." && pwd)"
c2rs="$root/target/release/c2rs"
src="$1"; shift
b="$(basename "$src" .cpp)"
out="$root/work/w-order/o"; mkdir -p "$out"
echo "$*" > "$out/$b.flags.txt"
fl=""
for f in "$@"; do fl="$fl --flag $f"; done
"$c2rs" compile "$src" --flags-file "$out/$b.flags.txt" --keep-obj "$out/$b.eh.ref.obj" >/dev/null
# shellcheck disable=SC2086
"$c2rs" prefilter --source "$src" $fl --emit-obj "$out/$b.eh.port.obj" >/dev/null
python3 "$root/scripts/gt_dump.py" "$out/$b.eh.ref.obj"  > "$out/$b.eh.ref.txt"
python3 "$root/scripts/gt_dump.py" "$out/$b.eh.port.obj" > "$out/$b.eh.port.txt"
diff -u "$out/$b.eh.ref.txt" "$out/$b.eh.port.txt" || true
