#!/bin/sh
# cov_sweep.sh — build the two coverage profiles `work/w-frame/sweep.py` reads.
#
# Lane w-cross, honouring w-frame row **F-c** as a standing rule: *a rung that
# adds a code path with no coverage under the GRADED profile is adding a first
# witness and should say so.* `sweep.py` consumes `work/w-frame/cov/export.json`
# (profile A) and `cov2/export.json` (profile B) and does not build them; this
# is that half, written down so the sweep is reproducible rather than folklore.
#
#   A = GRADED   — only runs whose obj was byte-compared AND matched:
#                  `perf` restricted to the matched fixtures, plus each of the
#                  12 `scripts/lanes.txt` lanes restricted to ITS OWN matching
#                  set, plus the 8 matching workload TUs at the workload flags.
#   B = REACHED  — `gap` over ALL fixtures at ALL 12 lanes. Over-credits on
#                  purpose: a NotImplemented fixture's codegen runs as far as
#                  the refusal and is never byte-compared.
#
# The workload TUs are NOT optional. w-frame recorded that its first profile A
# omitted them and falsely accused `dyninit_thunk_text`; leaving them out is a
# known way to make this instrument lie in the accusing direction.
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
out="$repo_root/work/w-frame"
cov="$out/cov"; cov2="$out/cov2"
rm -rf "$cov" "$cov2"; mkdir -p "$cov" "$cov2"

echo "==> instrumented build"
RUSTFLAGS="-C instrument-coverage" cargo build --release -p c2-harness --bin c2rs \
    --target-dir "$repo_root/target-cov" >/dev/null
bin="$repo_root/target-cov/release/c2rs"

# The matched set, from an UNINSTRUMENTED run: profile A must contain only runs
# that matched, and asking the instrumented binary which ones matched would put
# the query itself into the profile.
matched=$("$repo_root/target/release/c2rs" perf | awk '$NF=="Match"{print $1}' | paste -sd,)
echo "==> A: perf over $(printf '%s' "$matched" | tr ',' '\n' | wc -l) matched fixtures"
LLVM_PROFILE_FILE="$cov/a-%p-%m.profraw" "$bin" perf --fixtures "$matched" \
    --port-iters 1 --ref-iters 1 >/dev/null 2>&1 || true

# Each lane, restricted to that lane's own matching set. A fixture can match at
# /O1 and refuse at /Ox, so a single-lane A under-credits every /O1-only region
# — w-frame's second correction, which falsely accused 24 of them.
lanes="$repo_root/scripts/lanes.txt"
work="$repo_root/work/w-cross/covwork"; rm -rf "$work"; mkdir -p "$work"
: > "$work/all.txt"
for f in "$repo_root"/fixtures/cpp/*.cpp; do
    printf 'z:%s\n' "$(printf '%s' "$f" | tr '/' '\\')" >> "$work/all.txt"
done
grep -v '^ *#' "$lanes" | grep -v '^ *$' | while read -r slug flags; do
    echo "$flags /GS- /c" > "$work/$slug.flags"
    # which fixtures MATCH at this lane, measured with the uninstrumented binary
    "$repo_root/target/release/c2rs" gap --list "$work/all.txt" \
        --flags-file "$work/$slug.flags" --jobs 8 --jsonl "$work/$slug.jsonl" \
        >/dev/null 2>&1 || true
    python3 - "$work/$slug.jsonl" "$work/$slug.match.txt" <<'PY'
import json, sys
src, dst = sys.argv[1], sys.argv[2]
keep = []
for line in open(src):
    try:
        r = json.loads(line)
    except ValueError:
        continue
    if r.get("class") == "match":
        keep.append(r.get("src") or "")
open(dst, "w").write("".join(k + "\n" for k in keep if k))
PY
    if [ -s "$work/$slug.match.txt" ]; then
        LLVM_PROFILE_FILE="$cov/a-$slug-%p-%m.profraw" "$bin" gap \
            --list "$work/$slug.match.txt" --flags-file "$work/$slug.flags" \
            --jobs 4 >/dev/null 2>&1 || true
    fi
    # B: the same lane over EVERY fixture, refusals included.
    LLVM_PROFILE_FILE="$cov2/b-$slug-%p-%m.profraw" "$bin" gap \
        --list "$work/all.txt" --flags-file "$work/$slug.flags" \
        --jobs 4 >/dev/null 2>&1 || true
done

# The 8 matching workload TUs, at the workload's own flags, into A.
echo "==> A: the matching workload TUs"
"$repo_root/target/release/c2rs" gap --list "$repo_root/work/dc3-workload/files.txt" \
    --flags-file "$repo_root/work/dc3-workload/flags.txt" --cwd "$repo_root/../../../../dc3-decomp" \
    --jobs 16 --jsonl "$work/wl.jsonl" >/dev/null 2>&1 || true
python3 - "$work/wl.jsonl" "$work/wl.match.txt" <<'PY'
import json, sys
keep = []
for line in open(sys.argv[1]):
    try:
        r = json.loads(line)
    except ValueError:
        continue
    if r.get("class") == "match":
        keep.append(r.get("src") or "")
open(sys.argv[2], "w").write("".join(k + "\n" for k in keep if k))
PY
if [ -s "$work/wl.match.txt" ]; then
    LLVM_PROFILE_FILE="$cov/a-wl-%p-%m.profraw" "$bin" gap \
        --list "$work/wl.match.txt" --flags-file "$repo_root/work/dc3-workload/flags.txt" \
        --cwd "$repo_root/../../../../dc3-decomp" --jobs 8 >/dev/null 2>&1 || true
fi

for d in "$cov" "$cov2"; do
    llvm-profdata merge -sparse "$d"/*.profraw -o "$d/merged.profdata"
    llvm-cov export --instr-profile "$d/merged.profdata" "$bin" > "$d/export.json"
    echo "==> $(basename "$d"): $(ls "$d"/*.profraw | wc -l) raw profiles"
done
echo "==> now: work/w-frame/sweep.py"
