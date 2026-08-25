#!/bin/sh
# w-tu: census every TU in the distance-<=10 band, one capture each, at the
# workload's own /O1 /Oi /EHsc. Prints the per-function verdict, the blocking
# bytes, and both class axes for each.
#
# Usage: scripts/w_tu_census.sh <out-file>
set -eu

root="$(cd "$(dirname "$0")/.." && pwd)"

# Two absolute machine paths were baked in here: `main=` and a literal `--cwd`,
# both spelled /home/<user>/… (written with a placeholder here ON PURPOSE — the
# audit below is content-based, so quoting the removed path verbatim in the
# comment re-introduces the violation, and it did on the guard's first run).
# CLAUDE.md forbids committing those —
# "use C2RS_* env / relative-to-repo defaults; toolchain location is env-driven
# by design" — and this file was one of only two tracked files under
# crates/scripts/fixtures carrying one. Found by the tracked-artifact audit
# scripts/tracked_artifact_audit.sh on its FIRST run (board #3545, #3156).
#
# `main` is the MAIN repo, not this worktree: the workload lives there and does
# not follow `git worktree add`. Ask git for it rather than doing arithmetic on
# $0, which is #3500's defect one directory over.
main="${C2RS_MAIN_REPO:-$(cd "$(git -C "$(dirname "$0")" rev-parse \
        --path-format=absolute --git-common-dir)/.." && pwd)}"
# The dc3 checkout the workload's TUs are relative to. Sibling of the repo by
# default, overridable — the same shape every other path in this tree uses.
dc3="${C2RS_DC3_ROOT:-$main/../dc3-decomp}"
out="${1:-/tmp/w-tu-census.txt}"

: > "$out"
for tu in \
    src/system/utl/Spew.cpp \
    src/Main.cpp \
    src/system/math/Primes.cpp \
    src/system/math/Sort.cpp \
    src/xdk/LIBCMT/osfinfo.cpp \
    src/xdk/LIBCMT/undname.cpp \
    src/xdk/LIBCMT/vswprnc.cpp \
    src/xdk/nuispeech/xboxheap.cpp \
    src/xdk/xjson/jsonwriter.cpp \
    src/xdk/xlrc/xlrcimpl.cpp \
    src/ChecksumData_xbox.cpp \
    src/system/negate_test.cpp \
    src/system/synth_xbox/Biquad.cpp \
    src/xdk/LIBCMT/vsnprnc.cpp \
    src/system/rndobj/wordwrap.cpp \
    src/system/utl/Pool.cpp \
    src/xdk/nuiapi/nuidetroit.cpp \
    src/xdk/nuispeech/mmio.cpp \
    src/system/synth_xbox/IPP_basicmath_xbox.cpp \
    src/system/utl/EncryptXTEA.cpp \
    src/xdk/nuispeech/xboxmem.cpp \
    src/system/net/JsonMemory.cpp \
    src/system/math/Rand2.cpp \
    src/system/oggvorbis/VorbisMem.cpp \
    src/system/synth_xbox/MeterEffect.cpp
do
    echo "===== $tu" >> "$out"
    "$root/target/release/c2rs" census "$tu" \
        --flags-file "$main/work/dc3-workload/flags.txt" \
        --cwd "$dc3" >> "$out" 2>&1 || \
        echo "  CENSUS-FAILED" >> "$out"
done
echo "wrote $out"
