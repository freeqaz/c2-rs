#!/bin/sh
# refobj_ob0.sh — the same TU, the same workload flags, with `/Ob0` appended.
#
# Lane w-inline measurement tooling. Read-only with respect to `crates/`.
#
# WHY THIS EXISTS — the site enumerator
# ------------------------------------
# An obj compiled at the workload's own flags shows the inline DECLINES (a
# surviving REL24) and cannot show the INLINES, so a predicate graded on it
# alone is graded in one direction only — and a one-sided rate is trivially
# improved by making the rule more conservative. `work/w-inline/PREREG.md` P4
# asks for both directions, which needs the set of call SITES the source wrote.
#
# `/Ob0` is inline expansion OFF. Compile the identical TU with it and **every
# source-level call to a same-TU function leaves exactly one REL24**, so the
# `/Ob0` obj is the site enumerator and the `/O1` obj is the verdict. The
# enumerator is a real compilation by the real compiler — not a model of the
# source, and not the port's own opinion of what the IL says.
#
# `s` and every other INLINE-P input is still read from the `/O1` obj: `/Ob0`
# changes what c2 emits, and reading a size off it would be reading a different
# compilation (board #195's shape).
#
# Usage:  refobj_ob0.sh <src-relative-to-dc3> <out.obj>
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

dc3="${C2RS_DC3:-$(sib dc3-decomp)}"
wibo="${C2RS_WIBO:-$(sib wibo)/build/release/wibo}"
cl="$repo_root/compilers/X360/16.00.11886.00/cl.exe"

[ -x "$wibo" ] || { echo "SKIP: toolchain absent (wibo)"; exit 3; }
[ -f "$cl" ]   || { echo "SKIP: toolchain absent (cl.exe)"; exit 3; }

src="$1"
out="$(cd "$(dirname "$2")" && pwd)/$(basename "$2")"
zout="Z:$(printf '%s' "$out" | tr '/' '\\')"

# The workload profile, verbatim and read from the file, plus /Ob0 LAST so it
# wins over the /Ob2 that /O1 implies.
set -- $(cat "$repo_root/work/dc3-workload/flags.txt") /Ob0

cd "$dc3"
TMP="$(dirname "$out")" TEMP="$(dirname "$out")" WIBO_FS_CACHE=1 \
    "$wibo" "$cl" "$@" "/Fo$zout" "$src" >/dev/null 2>&1 || true
[ -s "$out" ] || { echo "FAIL: no obj for $src"; exit 1; }
echo "OK: $out"
