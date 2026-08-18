#!/usr/bin/env bash
# w-corpushealth — reading (3), tested on the RAW captured bytes.
#
# `--replay-every 1` already proved all 870 bundles reproduce c2's obj byte-
# exactly, which no truncated container could. This is the independent check on
# the bytes themselves: re-capture a stratified sample FRESH from source and
# compare the `.ex` length against the length the cached scan recorded. Two
# captures taken minutes apart through different code paths agreeing on the
# byte count is a truncation test the replay does not duplicate.
set -u
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
dc3="${C2RS_DC3:-}"
if [ -z "$dc3" ]; then d="$root"; while [ "$d" != "/" ]; do
    [ -d "$d/dc3-decomp" ] && { dc3="$d/dc3-decomp"; break; }; d="$(dirname "$d")"; done; fi
[ -n "$dc3" ] || { echo "dc3-decomp not found; set C2RS_DC3" >&2; exit 2; }
c2rs="$root/target/release/c2rs"
out="$here/ilcheck"; rm -rf "$out"; mkdir -p "$out"

while read -r src; do
  slug="$(printf '%s' "$src" | tr '/' '_')"
  d="$out/$slug"; mkdir -p "$d"
  "$c2rs" capture "$src" --flags-file "$root/work/dc3-workload/flags.txt" \
      --cwd "$dc3" --keep-il "$d" > "$d/capture.log" 2>&1
  n=$(ls "$d" | grep -c '^_CL_' || true)
  ex=$(stat -c%s "$d"/_CL_*.ex 2>/dev/null || echo 0)
  gl=$(stat -c%s "$d"/_CL_*.gl 2>/dev/null || echo 0)
  sy=$(stat -c%s "$d"/_CL_*.sy 2>/dev/null || echo 0)
  in_=$(stat -c%s "$d"/_CL_*.in 2>/dev/null || echo 0)
  db=$(stat -c%s "$d"/_CL_*.db 2>/dev/null || echo 0)
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$src" "$n" "$ex" "$gl" "$sy" "$in_" "$db"
  rm -rf "$d"
done < "$here/ilcheck.list"
