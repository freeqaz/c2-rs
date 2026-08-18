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
#   ^ TRUE UNTIL 2026-08-18. `expr_sweep.sh` takes it now (`w-gateperf`), and it
#     is the same source-path key that makes both rows cold in a fresh worktree.
#
# STABLE IS NOT ENOUGH, AND THIS DIRECTORY IS ONLY HALF THE FIX (w-coldcross).
# Stable *within a worktree* warms run 2; every lane's run 1 still paid the whole
# cold cost, because the key contains the source path and this path has the
# worktree in it. MEASURED on a fresh `setup_worktree.sh` tree at `--jobs 16`:
# this leg is **347 s cold** against **29 s** warm, 12x. The directory below is
# still generated per worktree exactly as before — and `resolve_corpus` further
# down then compares it against a shared, content-addressed, IMMUTABLE
# generation and grades those paths when all 19,556 files are byte-identical.
# See `scripts/corpus_dir.sh` for what is shared (only the case sources) and
# what is not (every verdict, and the port, which is recomputed per run).
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

# NEVER GLOB THE CASE DIRECTORY. 19,556 paths at ~100 bytes each is ~2 MB of
# argv, over `ARG_MAX`, and this directory has lived inside a worktree since
# this file was written. The `ls` below USED to be a glob and it had ALREADY
# FAILED: every run from a worktree printed `cross of 0 cases x …` in its own
# header, because `ls` errored and `wc -l` read the empty output as the number 0.
# Nothing caught it — `total_cases` is only printed, so an absence read as a
# count sat in a gate row's headline. Found by `scripts/expr_sweep.sh` hitting
# the hard version of the same limit (lane `w-gateperf`, 2026-08-18).
find "$cases" -maxdepth 1 -name '*.cpp' -delete 2>/dev/null || true

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

# ---- the SHARED, CONTENT-ADDRESSED corpus (lane w-coldcross, 2026-08-18) -------
#
# Everything above produced this run's own private copy of the corpus, exactly as
# it always did, and NOTHING BELOW SKIPS THAT — the private generation is what
# the shared one is verified against. `resolve_corpus` adopts the shared paths
# only when `diff -rq` finds all 19,556 files byte-identical, and otherwise says
# so in one line and leaves this run on its own cases, cold and correct.
#
# An explicit `C2RS_CROSS_CASES` keeps its documented meaning — a PRIVATE, COLD
# case set — because that is the A/B control this file's header quotes numbers
# from, and a control that silently went warm would be no control.
if [ -z "${C2RS_CROSS_CASES:-}" ]; then
    . "$repo_root/scripts/corpus_dir.sh"
    resolve_corpus "$repo_root" "$cases"
    cases="$C2RS_CORPUS_DIR"
fi

total_cases=$(find "$cases" -maxdepth 1 -name '*.cpp' | wc -l)
# Positively checked, because the value this replaced was 0 for months and the
# only symptom was a wrong number in a sentence. A case directory that just got
# regenerated and holds nothing is a generator failure, not a small cross.
if [ "${total_cases:-0}" -eq 0 ]; then
    echo "FATAL: $cases holds no .cpp after regeneration — the corpus is empty" >&2
    exit 3
fi

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

# ---- THE CACHE VALIDATOR THIS ROW NEVER HAD (lane w-coldcross, 2026-08-18) -----
#
# This row has consulted `work/capture-cache` since 2026-08-04 and has never
# checked it. `w-gateperf` gave the sweep a standing bypass-and-compare sample
# the day IT acquired the same dependency, and made `poisoned`/`foreign` a hard
# red — and `gate.sh --selftest` carries an explicit case saying an ABSENT cache
# line must not redden, which exists *because this row prints none*.
#
# That asymmetry was survivable while a fresh worktree's cross was ~100 % misses.
# `w-coldcross` takes it to ~100 % hits, so the check is the price of the speed:
# `--validate-cache N` re-captures every Nth HIT through the real toolchain and
# byte-compares the bundle, the obj and c2's own argv against what came off disk.
#
# 0 disables. The default is the sweep's, deliberately — one number, one meaning.
# The cost is small because it rides on `gap`'s own `--jobs`: ~900 re-captures
# over 90,424 graded cells.
#
# `c2rs diff`'s trap does NOT apply here and the difference is worth stating:
# that command performs exactly ONE capture per process, so an in-process
# `--validate-cache N` tests `1 % N` and validates nothing for any N > 1. Each
# `c2rs gap --list` below carries thousands of cases in one process, so its
# counter reaches N. The run PRINTS `validated=` for that reason: a validator
# whose count is not published is a validator nobody can tell from a disabled one.
validate="${C2RS_CROSS_VALIDATE:-100}"

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
        --validate-cache "$validate" > "$rep" 2>&1 || true
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
    # The cache accounting, written to a FILE for the same reason the counts are:
    # this loop is the right-hand side of a pipe, so it is a subshell and every
    # variable it increments is discarded at `done`. A `_bad=$((_bad+1))` here
    # would read 0 on every run, forever, and look exactly like a clean cache.
    _ch=$(sed -n 's|^  capture cache: \([0-9]*\) hit, .*|\1|p' "$rep" | head -1)
    _cm=$(sed -n 's|^  capture cache: [0-9]* hit, \([0-9]*\) miss, .*|\1|p' "$rep" | head -1)
    _cv=$(sed -n 's|.*validator: \([0-9]*\) re-captured.*|\1|p' "$rep" | head -1)
    _cp=$(sed -n 's|.*re-captured and agreed (.*), \([0-9]*\) POISONED.*|\1|p' "$rep" | head -1)
    _cx=$(sed -n 's|^  cache entries REFUSED on provenance: \([0-9]*\) .*|\1|p' "$rep" | head -1)
    echo "${_ch:-0} ${_cm:-0} ${_cv:-0} $(( ${_cp:-0} + ${_cx:-0} ))" > "$res/$slug.cache"
    if [ "$_mm" -ne 0 ]; then
        grep -F "mismatch" "$rep" | grep -v "^  mismatch" \
            | sed "s|^|MISMATCH  [$flags]  |" >> "$out/mismatches.txt" || true
    fi
done

checked=0; graded=0; mismatch=0; ungraded=0; reported=0; lanes_n=0
c_hit=0; c_miss=0; c_val=0; c_bad=0
for slug in $(sed 's/#.*//' "$registry" | awk 'NF >= 2 {print $1}'); do
    lanes_n=$((lanes_n + 1))
    [ -f "$res/$slug" ] || continue
    reported=$((reported + 1))
    set -- $(cat "$res/$slug")
    checked=$((checked + $1)); graded=$((graded + $2))
    mismatch=$((mismatch + $3)); ungraded=$((ungraded + $4))
    if [ -f "$res/$slug.cache" ]; then
        set -- $(cat "$res/$slug.cache")
        c_hit=$((c_hit + $1)); c_miss=$((c_miss + $2))
        c_val=$((c_val + $3)); c_bad=$((c_bad + $4))
    fi
done
[ -f "$out/mismatches.txt" ] && cat "$out/mismatches.txt"

echo "checked=$checked mismatches=$mismatch graded=$graded ungraded=$ungraded unknown=0"

# THE SAME SPELLING `expr_sweep.sh` USES, ON PURPOSE. `gate.sh`'s `sweep_verdict`
# rules both rows, already parses `^cache: .*cache-bad=N`, and already has four
# selftested cases for the clean / poisoned / cold / ABSENT states. Printing this
# line in that exact shape makes a poisoned cross a HARD RED with no change to
# `gate.sh`'s decision logic at all — and it retires the absent case for this row,
# which existed only because this row printed nothing.
#
# `cache-bad` sums POISONED (the validator re-captured and the bytes disagreed)
# and REFUSED-on-provenance (an entry whose recorded capture path is not where it
# is being served from). Both are expected to be 0 forever; a non-zero says this
# row's ORACLE side is untrustworthy on this run, which is a different statement
# from a mismatch and has to stay distinguishable from one.
echo "cache: hit=$c_hit miss=$c_miss validated=$c_val cache-bad=$c_bad (of $checked cells)"
# A validator that never fires is a validator nobody can tell from a disabled
# one, so the case is NAMED rather than inferred from a 0 sitting next to a 0.
if [ "$validate" -gt 0 ] 2>/dev/null && [ "$c_hit" -ge "$validate" ] && [ "$c_val" -eq 0 ]; then
    echo "NOTE: $c_hit cache hits at --validate-cache $validate re-captured NOTHING."
    echo "  Every hit above was served unchecked. This is not a wrong answer and it"
    echo "  is not a pass either — it means the sampling did not run."
fi

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
