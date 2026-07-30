#!/bin/sh
# gt_capture.sh — compile ONE source with arbitrary flags and print the obj.
#
# The ground-truth capture lane. `c2rs capture`/`compile` hardcode the fixture
# profile and drive the full IL-capture path (strace + bundle scrape); this is
# the thin version for *measurement*: run the real `cl.exe` under wibo with
# whatever flags the claim is about, keep the obj, and dump it with
# `scripts/gt_dump.py`.
#
# Usage:
#   scripts/gt_capture.sh <src.cpp> [flags...]        # default flags: /O1 /GS- /c
#   scripts/gt_capture.sh /tmp/gt/src/a.cpp /Ox /GS- /c
#
# Env:
#   C2RS_WIBO       wibo binary (default: the sibling ../wibo/build/release/wibo,
#                   then PATH). NOTE: ../wibo/build/wibo is a *stale* 1.0.1-7
#                   build that produces wrong objs — do not point this at it.
#   C2RS_COMPILERS  toolchain root (default <repo>/compilers)
#   GT_OUT          obj output path (default alongside the source, .obj)
#
# Prints the obj path on stdout; compiler diagnostics go to stderr.
set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"

src="${1:?usage: gt_capture.sh <src.cpp> [flags...]}"
shift
[ $# -gt 0 ] || set -- /O1 /GS- /c

wibo="${C2RS_WIBO:-}"
if [ -z "$wibo" ]; then
    if [ -x "$repo_root/../wibo/build/release/wibo" ]; then
        wibo="$repo_root/../wibo/build/release/wibo"
    else
        wibo="$(command -v wibo || true)"
    fi
fi
compilers="${C2RS_COMPILERS:-$repo_root/compilers}"
cl="$compilers/X360/16.00.11886.00/cl.exe"

if [ ! -x "$wibo" ] || [ ! -f "$cl" ]; then
    echo "SKIP: toolchain absent (wibo=$wibo cl=$cl)" >&2
    exit 0
fi

src_abs="$(cd "$(dirname "$src")" && pwd)/$(basename "$src")"
out="${GT_OUT:-${src_abs%.*}.obj}"
rm -f "$out"

# wibo maps the host root at Z:\ — cl.exe needs backslashed Z: paths.
zpath() { printf 'z:%s' "$(printf '%s' "$1" | tr '/' '\\')"; }

"$wibo" "$cl" "$@" "/Fo$(zpath "$out")" "$(zpath "$src_abs")" >&2

[ -f "$out" ] || { echo "capture failed: no obj at $out" >&2; exit 1; }
printf '%s\n' "$out"
