#!/bin/sh
# build_refs.sh — one reference obj per FRONTIER TU, at the workload's own flags.
#
# Lane w-front2 measurement tooling. Read-only with respect to `crates/`.
#
# A DIRECTORY PER TU, never a shared scratch: `refobj.sh` sets TMP/TEMP to the
# obj's own directory and `cl.exe` writes intermediates there, so two TUs
# sharing one directory race (the standing rule, board #1045).
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
out_root="$repo_root/work/w-front2/ref"
mkdir -p "$out_root"

while read -r src; do
    [ -n "$src" ] || continue
    key="$(printf '%s' "$src" | tr '/' '_')"
    d="$out_root/$key"
    mkdir -p "$d"
    if sh "$repo_root/work/w-frame/refobj.sh" "$src" "$d/out.obj" >/dev/null 2>&1; then
        echo "OK   $(stat -c%s "$d/out.obj")	$src"
    else
        echo "FAIL	$src"
    fi
done < "$repo_root/work/w-front2/tus.txt"
