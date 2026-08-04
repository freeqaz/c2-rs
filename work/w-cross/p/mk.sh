#!/bin/sh
# mk.sh — compile a probe .cpp in this directory at the WORKLOAD's own flags.
# The workload's /I paths are dc3-relative and resolve to nothing here, which is
# harmless: the probes include no headers. Everything else in flags.txt (/O1
# /Oi /EHsc /GR) is what the TU-match metric is graded at, and board #195 is why
# `c2rs compile` cannot be used.
set -eu
repo_root="$(cd "$(dirname "$0")/../../.." && pwd)"
sib() { d="$repo_root"; while [ "$d" != "/" ]; do [ -d "$d/../$1" ] && { (cd "$d/../$1" && pwd); return 0; }; d="$(dirname "$d")"; done; return 1; }
wibo="${C2RS_WIBO:-$(sib wibo)/build/release/wibo}"
cl="$repo_root/compilers/X360/16.00.11886.00/cl.exe"
[ -x "$wibo" ] || { echo "SKIP: toolchain absent (wibo)"; exit 3; }
here="$(cd "$(dirname "$0")" && pwd)"
src="$1"; out="${2:-${src%.cpp}.obj}"
set -- $(cat "$repo_root/work/dc3-workload/flags.txt")
cd "$here"
TMP="$here" TEMP="$here" WIBO_FS_CACHE=1 "$wibo" "$cl" "$@" ${EXTRA_FLAGS:-} \
    "/FoZ:$(printf '%s' "$here/$(basename "$out")" | tr '/' '\\')" "$(basename "$src")" 2>&1 | tail -5
[ -s "$here/$(basename "$out")" ] || { echo "FAIL: no obj"; exit 1; }
