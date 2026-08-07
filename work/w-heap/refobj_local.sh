#!/bin/sh
# refobj_local.sh — `work/w-frame/refobj.sh` for a source inside THIS repo.
#
# `refobj.sh` cd's into the dc3 tree and takes a dc3-relative path, which is
# right for a workload TU and wrong for a probe cell. Same flags, same wibo,
# same cl.exe; only the working directory differs. Everything else is copied
# rather than reimplemented so the two cannot drift on the profile — the flags
# are read from `work/dc3-workload/flags.txt` here too, never transcribed.
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"

sib() {
    d="$repo_root"
    while [ "$d" != "/" ]; do
        [ -d "$d/../$1" ] && { (cd "$d/../$1" && pwd); return 0; }
        d="$(dirname "$d")"
    done
    return 1
}

wibo="${C2RS_WIBO:-$(sib wibo)/build/release/wibo}"
cl="$repo_root/compilers/X360/16.00.11886.00/cl.exe"
[ -x "$wibo" ] || { echo "SKIP: toolchain absent (wibo)"; exit 3; }
[ -f "$cl" ]   || { echo "SKIP: toolchain absent (cl.exe)"; exit 3; }

src="$1"
out="$(cd "$(dirname "$2")" && pwd)/$(basename "$2")"
zout="Z:$(printf '%s' "$out" | tr '/' '\\')"
set -- $(cat "$repo_root/work/dc3-workload/flags.txt")

cd "$repo_root"
TMP="$(dirname "$out")" TEMP="$(dirname "$out")" WIBO_FS_CACHE=1 \
    "$wibo" "$cl" "$@" "/Fo$zout" "$src" >/dev/null 2>&1 || true
[ -s "$out" ] || { echo "FAIL: no obj for $src"; exit 1; }
