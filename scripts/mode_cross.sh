#!/bin/sh
# THE PRODUCT — the generated corpus x the mode-lane registry, with the provably
# redundant cells removed.
#
# ---- why this exists -----------------------------------------------------------
#
# This repo has two correctness instruments and until now nobody graded their
# product:
#
#   * `scripts/gate.sh` runs the 12 lanes of `scripts/lanes.txt` over the 228
#     hand-written `fixtures/cpp/*.cpp`. **Broad in flags, narrow in shapes.**
#   * `scripts/expr_sweep.sh` runs 14,635 generated cases through `c2rs diff`,
#     which hardcodes **one** profile (`/Ox /GS- /c`, `c2-reference/src/lib.rs`).
#     **Broad in shapes, one flag profile.**
#
# Every wrong-emit family found on 2026-08-04 needed a conjunction:
#
#   * board **#232** — an implicit destructor **x** the packed path.
#   * w-order **Y-a** — an empty-bodied locally-defined unwind target **x**
#     `/EHsc`, live at `/O1 /EHsc` (the dc3 workload's own profile) and invisible
#     at `/Ox`, which is the only profile the sweep runs.
#
# The naive product is 14,635 x 12 = **175,620** gradings. That is not
# affordable, and the answer is not to make the check optional — this project has
# twelve recorded instances of an absence reading as a success. The answer is to
# find out how much of the product is *not redundant*, which turns out to be
# **61,539 cells, 2.85x smaller**, and to grade all of that.
#
# ---- what makes a cell removable, and why it is a proof -------------------------
#
# `scripts/mode_invariance.py` measures, per fragment, which of the 12 lanes are
# distinguishable at all. Two lanes are merged only when, at every sampled case,
#
#     the IL bundle is byte-identical, the reference obj is byte-identical, and
#     `flags_imply_function_level_linking` agrees
#
# — and those three are the port's entire input plus the oracle's entire output,
# so the two gradings are the same computation. **Not** "the verdicts matched":
# `scripts/lanes.txt` already records that `/O1 /EHsc` and `/O1` agree on 0
# verdict rows while emitting genuinely different objs.
#
# Measured 2026-08-04, 48 fragments x 24 strided cases x 12 lanes, 13,824 cells,
# every 7th re-captured and reproduced:
#
#     42 of 48 fragments collapse 12 lanes to 4    (O1, O2, Od, Ox)
#      6 of 48 collapse to 8                       (+ their /EHsc twins)
#      1 of 48 to 5                                (53-data-symbol-addr splits Ox-Gy)
#
#     /O1 vs /O1 /Oi        differ on  0 of 48 fragments — IL AND obj identical
#     /Ox vs /Ox /Gy        differ on 48 of 48 — but their IL is identical on ALL
#                           48, so an IL-only redundancy test would be WRONG
#     /EHsc anywhere        differs on  6 of 48 — and those 6 are exactly the
#                           fragments that carry a destructor or a vtable
#
# `scripts/mode_classes.txt` is that table. It is generated, every row carries a
# digest of the fragment's own case set, and a row whose digest no longer matches
# — or a fragment with no row at all — is graded at **all 12 lanes**. The
# fail-safe direction is deliberate: a fragment excluded as invariant is a
# fragment nothing will ever grade again at the excluded lanes.
#
# ---- cost, measured ------------------------------------------------------------
#
# See `docs/rungs/2026-08-04-w-modes.md` §3. Quote the number from a run, not
# from here.
#
# ---- usage ---------------------------------------------------------------------
#
#   scripts/mode_cross.sh [outdir] [max-cells]
#   scripts/mode_cross.sh /tmp/cross-$$ 4000     # a STRIDED subset; never a PASS
#   C2RS_JOBS=16 scripts/mode_cross.sh           # grading concurrency
#   C2RS_CROSS_CASES=DIR scripts/mode_cross.sh   # a private (COLD) case set
#
# `max-cells` is a STRIDE across the assigned, fragment-sorted case list, never a
# prefix — `head -n` over a name-sorted corpus reaches one fragment (see
# `expr_sweep.sh`, and `sweep_mode.sh` still has the prefix bug).
#
# A MISMATCH is an ALARM, not a gap: the port emitted bytes and they were wrong.
# Needs the toolchain; without it this prints `SKIP: toolchain absent` and exits
# 0 — never a vacuous pass.
set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-/tmp/c2rs-mode-cross}"
limit="${2:-0}"
registry="${C2RS_LANES:-$repo_root/scripts/lanes.txt}"
classes="${C2RS_MODE_CLASSES:-$repo_root/scripts/mode_classes.txt}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"

# ---- the case directory is STABLE, and that is the whole cost argument ---------
#
# `c2rs gap` reads the capture cache, whose key includes the SOURCE PATH (it is
# baked verbatim into `.gl` and `.debug$S`). So a per-run case directory means a
# cold cache on every single run, and the difference is not marginal:
#
#     MEASURED 2026-08-04, 61,539 cells, 8 jobs, 32-core host
#         cold (a fresh case directory)      5 min 45 s
#         warm (the same paths again)            13.8 s
#     both printing `checked=61539 mismatches=0 graded=61151 ungraded=388`.
#
# That is 25x, and it is what makes an unconditional gate row affordable — the
# same move `expr_sweep.sh` made by parallelising: **remove the cost, do not make
# the check optional.** The RUN ARTIFACTS (lists, reports, logs) still go in the
# caller's outdir, so two runs never read each other's numbers.
#
# `expr_sweep.sh` cannot take this path: it drives `c2rs diff`, which does not
# consult the cache at all.
cases="${C2RS_CROSS_CASES:-$repo_root/work/mode-cross/cases}"
mkdir -p "$cases"
cases="$(cd "$cases" && pwd)"

# ABSOLUTE, always: `cl.exe` runs under wibo and is handed `z:<path>`, so a
# relative case dir yields `z:work\...`, which it cannot open, and the whole run
# comes back capture-fail while every check reads green. `sweep_mode.sh` records
# that happening on its first real use.

# The lock is on the CASE directory, not the outdir, because the case set is the
# shared mutable thing: this driver deletes and regenerates it, so two concurrent
# runs delete each other's cases mid-grade and BOTH results are meaningless while
# looking perfectly ordinary (`expr_sweep.sh` records that happening). `mkdir` is
# the lock because it is atomic on every filesystem this runs on.
#
# On contention this FALLS BACK to a private case set rather than refusing.
# Refusing would be the safe-looking choice and it is the wrong one here: this is
# a gate row, two gate runs in one worktree is an ordinary thing to do, and a row
# that reports NO-RESULT because a sibling was running is a RED GATE FROM AN
# ABSENCE — the exact failure this file's rules exist to forbid, arriving from the
# other direction. The fallback is correct (a private set is genuinely private),
# it is loud, and its only cost is a cold cache.
_lock="$cases/../.cross.lock"
_lock_held=1
if ! mkdir "$_lock" 2>/dev/null; then
    echo "NOTE: another cross holds $cases (lock: $_lock)."
    echo "  Falling back to a PRIVATE case set for this run, which starts COLD"
    echo "  (~5-6 min instead of ~15 s). The result is exactly as valid; only the"
    echo "  capture cache misses. If no cross is running, the previous one was"
    echo "  killed and the lock is stale: rmdir '$_lock'"
    cases="$out/cases"
    mkdir -p "$cases"
    cases="$(cd "$cases" && pwd)"
    _lock_held=0
fi
[ "$_lock_held" -eq 1 ] && trap 'rmdir "$_lock" 2>/dev/null || true' EXIT INT TERM

rm -f "$cases"/*.cpp 2>/dev/null || true

# Build unconditionally and run a RUN-PRIVATE COPY — never `target/release/c2rs`
# directly. `scripts/harness_bin.sh` carries the two failures this closes: a
# sweep grading today's cases with yesterday's code, and a gate reading a file
# the rest of the tree may rewrite mid-run.
. "$repo_root/scripts/harness_bin.sh"
pin_harness "$repo_root" "$out"
c2rs="$C2RS_PINNED"

# The SAME loader `expr_sweep.sh`, `sweep_mode.sh` and `cross_sweep.py` use.
python3 "$repo_root/scripts/sweep_gen.py" "$cases" "$repo_root/scripts/sweep.d" \
    > "$out/gen.log" 2>&1 || { cat "$out/gen.log" >&2; exit 3; }
total_cases=$(ls "$cases"/*.cpp | wc -l)

# Assign every case to the lanes its fragment can actually be distinguished at.
# One reader for `mode_classes.txt`, in `mode_invariance.py`; this script never
# parses it.
python3 "$repo_root/scripts/mode_invariance.py" \
    --assign "$cases" --assign-out "$out/lists" \
    --classes "$classes" --registry "$registry" > "$out/assign.log" 2>&1 \
    || { cat "$out/assign.log" >&2; exit 3; }
sed -n '1,200p' "$out/assign.log"

cells_total=$(sed -n 's/^assigned [0-9]* cases over [0-9]* lanes = \([0-9]*\) cells.*/\1/p' \
    "$out/assign.log" | head -1)
cells_full=$(sed -n 's/^assigned [0-9]* cases over [0-9]* lanes = [0-9]* cells (full cross would be \([0-9]*\)).*/\1/p' \
    "$out/assign.log" | head -1)
if [ -z "$cells_total" ] || [ "$cells_total" -eq 0 ] 2>/dev/null; then
    echo "FATAL: the assignment produced no cells at all." >&2
    exit 3
fi

# ---- the stride ----------------------------------------------------------------
# Applied per lane list, so a budget keeps every lane AND every fragment
# represented in proportion. A prefix would keep the alphabetically first
# fragments of the alphabetically first lane and nothing else.
stride=1
if [ "$limit" -gt 0 ] 2>/dev/null && [ "$limit" -lt "$cells_total" ]; then
    stride=$(( (cells_total + limit - 1) / limit ))
fi

run=0
for lf in "$out"/lists/*.list; do
    if [ "$stride" -gt 1 ]; then
        awk -v k="$stride" 'NR % k == 1 || k == 1' "$lf" > "$lf.run"
    else
        cp "$lf" "$lf.run"
    fi
    run=$((run + $(wc -l < "$lf.run")))
done

# Toolchain probe on the first non-empty lane. Absent -> SKIP, exit 0.
probe=""
for lf in "$out"/lists/*.list.run; do
    [ -s "$lf" ] || continue
    probe="$lf"; break
done
[ -n "$probe" ] || { echo "FATAL: every lane list is empty." >&2; exit 3; }
probe_slug=$(basename "$probe" .list.run)
probe_flags=$(sed 's/#.*//' "$registry" | awk -v s="$probe_slug" '$1==s{$1="";sub(/^[ \t]+/,"");print;exit}')
echo "$probe_flags /GS- /c" > "$out/probe.flags"
if "$c2rs" gap --list "$probe" --flags-file "$out/probe.flags" --limit 1 --jobs 1 2>&1 \
        | grep -q "SKIP"; then
    echo "SKIP: toolchain absent — the cross would be vacuous"
    exit 0
fi

jobs="${C2RS_JOBS:-8}"
if [ "$stride" -eq 1 ]; then
    echo "sweeping $run of $cells_total case-lane cells (cross of $total_cases cases x $registry; the full 12x product is $cells_full)"
else
    echo "sweeping $run of $cells_total case-lane cells (STRIDE $stride — a SAMPLE, not the product)"
fi
echo "  grading at $jobs job(s)"

# ---- grade, one `c2rs gap` batch per lane --------------------------------------
#
# Each lane writes its OWN bucket counts and the driver SUMS THEM, exactly as
# `expr_sweep.sh` sums its workers: a lane that dies contributes a short count
# and the reconciliation below fails, rather than contributing a silence that
# reads as zero.
res="$out/lane-results"
rm -rf "$res"; mkdir -p "$res"
sed 's/#.*//' "$registry" | awk 'NF >= 2 {print $1}' | while read -r slug; do
    lf="$out/lists/$slug.list.run"
    [ -s "$lf" ] || { echo "0 0 0 0" > "$res/$slug"; continue; }
    flags=$(sed 's/#.*//' "$registry" | awk -v s="$slug" '$1==s{$1="";sub(/^[ \t]+/,"");print;exit}')
    echo "$flags /GS- /c" > "$out/$slug.flags"
    rep="$out/$slug.report"
    "$c2rs" gap --list "$lf" --flags-file "$out/$slug.flags" --jobs "$jobs" \
        > "$rep" 2>&1 || true
    b() { sed -n "s|^  $1  *\([0-9]*\) .*|\1|p" "$rep" | head -1; }
    _mm=$(b mismatch);     _mm=${_mm:-0}
    _ma=$(b match);        _ma=${_ma:-0}
    _cg=$(b codegen-gap);  _cg=${_cg:-0}
    _vg=$(b vocab-gap);    _vg=${_vg:-0}
    _pe=$(b port-error);   _pe=${_pe:-0}
    _cf=$(b capture-fail); _cf=${_cf:-0}
    _n=$(wc -l < "$lf")
    _g=$((_ma + _mm + _cg + _vg + _pe))
    # A report with NO census line is a run that died mid-way; every `sed` above
    # would then read a missing number as 0 and this lane would contribute a
    # clean zero. Refuse to write a count for it — the sum then comes up short.
    if ! grep -q "FUNCTION CENSUS" "$rep"; then
        echo "  lane $slug produced no census line — its report is truncated" >&2
        continue
    fi
    echo "$_n $_g $_mm $_cf" > "$res/$slug"
    if [ "$_mm" -ne 0 ]; then
        grep -F "mismatch" "$rep" | grep -v "^  mismatch" \
            | sed "s|^|MISMATCH  [$flags]  |" >> "$out/mismatches.txt" || true
    fi
done

checked=0; graded=0; mismatch=0; ungraded=0; reported=0; lanes_n=0
for slug in $(sed 's/#.*//' "$registry" | awk 'NF >= 2 {print $1}'); do
    lanes_n=$((lanes_n + 1))
    [ -f "$res/$slug" ] || continue
    reported=$((reported + 1))
    set -- $(cat "$res/$slug")
    checked=$((checked + $1)); graded=$((graded + $2))
    mismatch=$((mismatch + $3)); ungraded=$((ungraded + $4))
done
[ -f "$out/mismatches.txt" ] && cat "$out/mismatches.txt"

echo "checked=$checked mismatches=$mismatch graded=$graded ungraded=$ungraded unknown=0"

if [ "$checked" -ne "$run" ]; then
    echo "FATAL: selected $run cells and only $checked were reached" >&2
    echo "  $reported of $lanes_n lanes reported a count. A short count is a lane" >&2
    echo "  whose report died; the cells it held were never graded and this run" >&2
    echo "  establishes nothing about them." >&2
    exit 3
fi
if [ "$graded" -eq 0 ]; then
    echo "VACUOUS: $checked cells reached and NONE graded. Every count above would" >&2
    echo "  otherwise read 0 and pass." >&2
    exit 1
fi

# ---- the UNGRADED baseline -----------------------------------------------------
#
# `capture-fail` here is overwhelmingly the same 96 generated cases `cl.exe`
# refuses to compile that `expr_sweep.sh` carries (see its baseline comment),
# multiplied by the lanes each is assigned. It is a NUMBER WITH A REASON, not a
# tolerance: raising it needs the reason written next to it.
max_ungraded="${C2RS_CROSS_MAX_UNGRADED:-400}"
if [ "$ungraded" -gt "$max_ungraded" ]; then
    echo
    echo "UNGRADED $ungraded exceeds the carried baseline $max_ungraded."
    echo "  Those cells produced no reference obj, so the oracle never ruled on them."
    exit 1
fi
[ "$mismatch" -eq 0 ] || exit 1
