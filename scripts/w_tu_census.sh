#!/bin/sh
# w-tu: census every TU in the distance-<=10 band, one capture each, at the
# workload's own /O1 /Oi /EHsc. Prints the per-function verdict, the blocking
# bytes, and both class axes for each.
#
# Usage: scripts/w_tu_census.sh <out-file>
set -eu

root="$(cd "$(dirname "$0")/.." && pwd)"
main=/home/free/code/milohax/c2-rs
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
        --cwd /home/free/code/milohax/dc3-decomp >> "$out" 2>&1 || \
        echo "  CENSUS-FAILED" >> "$out"
done
echo "wrote $out"
