#!/bin/sh
# listing.sh -- produce c2's OWN `/FAcs` assembly listing (`.cod`) for one TU,
# at the dc3 workload's own flags plus `/FAcs`.
#
# The `.cod` machine-code column is the ORACLE this lane verifies its VMX128
# decoder against: it is Microsoft's own listing writer printing the exact
# instruction word beside its own mnemonic and operands. Reading the compiler's
# published output is black-box observation -- no `docs/whitebox/DISCLOSURE.md`
# row is implied by anything derived from a `.cod`.
#
# Read-only with respect to `crates/`. Never a gate.
#
# Usage:
#   listing.sh <src-relative-to-dc3>   <out-dir>     # a workload TU
#   listing.sh /abs/path/to/file.cpp   <out-dir>     # a probe TU
#
# Env: C2RS_DC3 (dc3 tree), C2RS_WIBO (wibo binary). Prints `SKIP: toolchain
# absent` and exits 3 when the toolchain is not there, per the project rule.
set -eu

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"

# Walk UP for sibling checkouts rather than hardcoding a depth -- this tree may
# be the main repo or a worktree. No `find`, no glob: the standing rule forbids
# recursive walks from the repo root.
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
outdir="$(cd "$2" && pwd)"
zo="Z:$(printf '%s' "$outdir" | tr '/' '\\')\\"

# The workload profile, read from the file rather than transcribed, so this
# cannot drift from what `c2rs gap` grades.
set -- $(cat "$repo_root/work/dc3-workload/flags.txt")

case "$src" in
    /*) cd "$(dirname "$src")"; src="$(basename "$src")" ;;
    *)  cd "$dc3" ;;
esac

TMP="$outdir" TEMP="$outdir" WIBO_FS_CACHE=1 \
    "$wibo" "$cl" "$@" /FAcs "/Fa$zo" "/Fo$zo" "$src" >"$outdir/cl.log" 2>&1 || true

base="$(basename "$src")"
cod="$outdir/${base%.*}.cod"
[ -s "$cod" ] || { echo "FAIL: no listing for $src (see $outdir/cl.log)"; exit 1; }
echo "$cod"
