#!/usr/bin/env bash
# w-corpushealth — per-TU emitted `.text` COMDAT symbol census over the whole
# 878-TU workload. Compiles each TU with real c2 under wibo at the workload's
# own flags, extracts the obj's `.text` symbol names, and DELETES the obj.
#
# This is what turns the per-TU Frechet bound into a NAME-SPACE bound: an
# unfinished symbol attributed by objdiff to unit j can still be emitted into
# TU i's obj, and a per-TU min() cannot see that. The symbol names can.
#
# Output: work/w-corpushealth/syms/<slug>.txt  (first line "OK n" or "BAD ...")
set -u
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
dc3="${C2RS_DC3:-}"
if [ -z "$dc3" ]; then d="$root"; while [ "$d" != "/" ]; do
    [ -d "$d/dc3-decomp" ] && { dc3="$d/dc3-decomp"; break; }; d="$(dirname "$d")"; done; fi
[ -n "$dc3" ] || { echo "dc3-decomp not found; set C2RS_DC3" >&2; exit 2; }
out="$here/syms"; mkdir -p "$out" "$here/objs"
c2rs="$root/target/release/c2rs"
jobs="${JOBS:-8}"

one() {
  src="$1"
  slug="$(printf '%s' "$src" | tr '/' '_')"
  [ -s "$out/$slug.txt" ] && return 0
  obj="$here/objs/$slug.obj"
  if ! "$c2rs" compile "$src" --flags-file "$root/work/dc3-workload/flags.txt" \
       --cwd "$dc3" --keep-obj "$obj" >/dev/null 2>"$out/$slug.err"; then
    echo "COMPILE-FAIL" > "$out/$slug.txt"
    rm -f "$obj"
    return 0
  fi
  python3 "$here/objsyms.py" "$obj" > "$out/$slug.txt" 2>>"$out/$slug.err"
  rm -f "$obj"
}
export -f one
export here root dc3 out c2rs

xargs -a "$root/work/dc3-workload/files.txt" -P "$jobs" -I{} bash -c 'one "$@"' _ {}
echo "done: $(ls "$out" | grep -c '\.txt$') files"
