#!/bin/sh
# w-sect — regenerate the two coverage profiles `sweep.py` subtracts, pointed at
# THIS lane's seam (`crates/c2-core/src/coff/`) instead of at codegen.
#
# **The profiles are built from the lane registry, never from a remembered list
# of runs.** `work/w-frame/README.md` records both of this instrument's own
# errors as exactly that mistake, and this lane made the SECOND one on its first
# attempt: a REACHED profile built from `c2rs perf` alone reported
# `emit_dyninit_obj` as NEVER EXECUTED. It is not — `perf` runs at `/Ox`, `/GF`
# is implied by `/O1`/`/O2` and NOT by `/Ox`, so the dyninit fixtures refuse at
# `/Ox` and the emitter is only reached on the `/O1`-family lanes. A profile
# without the 12 registered lanes cannot see it.
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
RUSTFLAGS="-C instrument-coverage" cargo build --release -p c2-harness --bin c2rs \
    --target-dir target-cov >/dev/null 2>&1
BIN="$PWD/target-cov/release/c2rs"
rm -rf work/w-sect/cov work/w-sect/cov2
mkdir -p work/w-sect/cov work/w-sect/cov2

list="$PWD/work/w-sect/cov-fixtures.txt"
: > "$list"
for f in "$PWD"/fixtures/cpp/*.cpp; do
    printf 'z:%s\n' "$(printf '%s' "$f" | tr '/' '\\')" >> "$list"
done

# ---- B = REACHED: every fixture at every registered lane, refusals included --
grep -v '^#' scripts/lanes.txt | grep -v '^$' | while read -r name flags; do
    echo "$flags /GS- /c" > "work/w-sect/cov2/$name.flags"
    LLVM_PROFILE_FILE="$PWD/work/w-sect/cov2/$name-%p.profraw" \
        "$BIN" gap --list "$list" --flags-file "work/w-sect/cov2/$name.flags" \
        --jsonl "$PWD/work/w-sect/cov2/$name.jsonl" --jobs 8 >/dev/null 2>&1 || true
done
LLVM_PROFILE_FILE="$PWD/work/w-sect/cov2/perf-%p.profraw" "$BIN" perf >/dev/null 2>&1
for c in work/w-sect/sweep-w/*.cpp; do
    LLVM_PROFILE_FILE="$PWD/work/w-sect/cov2/case-%p.profraw" "$BIN" diff "$c" >/dev/null 2>&1
done

# ---- A = GRADED: only runs whose obj was byte-compared AND MATCHED -----------
# Each lane restricted to ITS OWN match list, read back out of that lane's
# JSONL — not a global list, which would credit a fixture at a lane where it
# refused.
grep -v '^#' scripts/lanes.txt | grep -v '^$' | while read -r name flags; do
    m="work/w-sect/cov/$name.list"
    python3 -c '
import json,sys
out=[]
for l in open(sys.argv[1]):
    r=json.loads(l)
    if r.get("class")=="match" or r.get("verdict")=="match": out.append(r.get("src") or r.get("path"))
sys.stdout.write("\n".join(x for x in out if x)+"\n")
' "work/w-sect/cov2/$name.jsonl" > "$m" 2>/dev/null || : > "$m"
    [ -s "$m" ] || continue
    echo "$flags /GS- /c" > "work/w-sect/cov/$name.flags"
    LLVM_PROFILE_FILE="$PWD/work/w-sect/cov/$name-%p.profraw" \
        "$BIN" gap --list "$m" --flags-file "work/w-sect/cov/$name.flags" --jobs 8 \
        >/dev/null 2>&1 || true
done
"$BIN" perf 2>/dev/null | awk '$NF=="Match"{print $1}' > work/w-sect/match_fixtures.txt
awk '$1=="Port=Match"{print $2}' work/w-sect/tally.txt | while read -r c; do
    LLVM_PROFILE_FILE="$PWD/work/w-sect/cov/case-%p.profraw" "$BIN" diff "$c" >/dev/null 2>&1
done

for d in cov cov2; do
    llvm-profdata merge -sparse work/w-sect/$d/*.profraw -o work/w-sect/$d/c.profdata
    llvm-cov export -instr-profile=work/w-sect/$d/c.profdata "$BIN" > work/w-sect/$d/export.json
done
echo "profiles written: $(ls work/w-sect/cov/*.profraw | wc -l) graded, $(ls work/w-sect/cov2/*.profraw | wc -l) reached"
