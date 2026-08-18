#!/bin/sh
# Generated differential sweep over the shapes that claim byte-exactness.
#
# Enumerates small translation units over a set of axes, compiles each against the
# real toolchain, and reports every byte MISMATCH. This is the thing that found the
# reassociation and repeated-leaf mis-emits: ~20 wrong-bytes bugs in the
# straight-line class that the hand-written corpus had never separated, because
# every fixture in it happened to use distinct operands in ascending order.
#
# The lesson is in `docs/GAPS.md`: a green fixture run is only as strong as the
# corpus's ability to *separate* the candidate rules, and a hand-picked corpus is
# systematically biased toward the shapes whoever wrote it was already thinking
# about. Enumeration has no such bias.
#
# A MISMATCH is an alarm, not a gap — the port emitted bytes and they were wrong.
# Either fix the lowering or tighten the gate until it refuses. NotImplemented is
# fine and expected for most cases here.
#
# ---- the fragment contract (docs/ARCHITECTURE_SEAMS.md §2.4) -------------------
#
# The generator lives in `scripts/sweep.d/`, ONE FILE PER AXIS. A rung adds a NEW
# file; two rungs adding fragments never conflict, and two rungs claiming the same
# fragment name is an add/add conflict git flags loudly.
#
# Each fragment defines exactly:
#
#     def cases(emit):
#         emit("int f(int a) { return a + 1; }\n")   # one .cpp case
#
# `emit` is supplied by the LOADER (`scripts/sweep_gen.py`), which owns the
# counter and namespaces the output by fragment (`10-int-chains-0007.cpp`). A
# fragment therefore cannot see or touch another fragment's counter: the
# `n`-shadowing trap that silently rewound the file counter and overwrote 1,233
# already-written cases is now *unrepresentable*, not merely fixed. The loader
# prints a per-fragment count, **fails if any fragment emits zero cases** — the
# observable symptom of that bug is now a hard error — and **fails if what it
# counted is not what is on disk**, which is that bug's actual damage.
#
# The loader is a module rather than an inline block because it has a second
# consumer: `scripts/cross_sweep.py` grades the CROSS PRODUCT of the shape
# families these cases exercise, and two copies of the enumeration is the
# "one rule, two implementations" shape `docs/GAPS.md` §6 keeps recording.
#
# Usage:  scripts/expr_sweep.sh [outdir] [max-cases]
#         scripts/expr_sweep.sh /tmp/sweep 400     # a STRIDED subset, see below
#         C2RS_SWEEP_ONLY=fp scripts/expr_sweep.sh # only fragments matching "fp"
#         C2RS_SWEEP_JOBS=8 scripts/expr_sweep.sh  # grade 8 cases at a time
#
# `C2RS_SWEEP_ONLY` is for iterating on one axis; it makes the total meaningless
# by design, so the driver says so out loud and any gate run must be unfiltered.
#
# ---- `max-cases` is a STRIDE, not a prefix (changed 2026-08-04) ----------------
#
# It used to be `head -n N` over `cases.txt`, which is sorted by fragment name — so
# a "quick subset" was **the alphabetically first fragments and nothing else**.
# Measured on today's corpus: `head -400` covers **1 of the 47 fragments**, and the
# case that carried board #232 — the live `Port=Mismatch` this sweep found — is
# **line 9,538 of 14,484**. So every prefix under 66 % of the corpus, i.e. every
# subset small enough to be worth taking, was STRUCTURALLY BLIND to it. A biased
# sample of an enumeration defeats the only property the enumeration has
# (`docs/GAPS.md`: a hand-picked corpus is biased toward the shapes whoever picked
# it was thinking about — and a prefix of a sorted list is hand-picked by the sort).
#
# So `N` now selects every `ceil(total/N)`-th case, which keeps every fragment
# represented in proportion: the same budget of 400 reaches **46 of 47** fragments
# (the missing one, `52-callee-name`, is smaller than the stride — a sample is
# still a sample, and this is the honest count, not "all of them"). It cannot
# establish what a full run establishes; `scripts/gate.sh` therefore refuses to
# print an unqualified PASS over one.
#
# ---- grading is PARALLEL --------------------------------------------------------
#
# Each case is an independent `c2rs diff`; nothing is shared but the capture cache,
# which has been cross-process safe since board #181 (an `O_EXCL` lockfile per key,
# fail-open). MEASURED here 2026-08-04, 14,484 cases, warm cache, 32-core host —
# **9 min 51 s serial, 1 min 26 s at `C2RS_SWEEP_JOBS=8`**, with `checked=14484
# mismatches=0` from both. That cost is the whole reason the biased `max-cases`
# knob existed and the whole reason this sweep was not in the merge gate while
# #232 survived 241 commits; at 8 jobs it is affordable unconditionally, so the
# argument for leaving it out is spent.
#
# Workers write per-worker count and mismatch files and the driver SUMS THE COUNTS:
# a worker that dies contributes a short count and the reconciliation below fails,
# rather than contributing a silence that reads as zero.
#
# Needs the toolchain (see CLAUDE.md); without it every case reports SKIP and the
# sweep is vacuous, so it checks for that up front.
#
# ---- the CAPTURE CACHE, and why the case directory is now STABLE ---------------
# (lane `w-gateperf`, 2026-08-18)
#
# This row was the dominant leg of `scripts/gate.sh` — 300 s of a 446 s run on a
# quiet box, 747 s of 1,368 s on a loaded one — and PROFILED, its cost is one
# `cl.exe` process tree per case:
#
#     one warm `c2rs diff` on a generated case, serial, load 12-16, MEASURED
#       capture (strace + wibo + cl.exe -> c1xx.dll -> c2.dll)   43 ms   75 %
#       replay  (wibo + c2host.exe + c2.dll)                      6 ms   11 %
#       everything else (c2rs start, scratch, port, obj diff)     8 ms   14 %
#       TOTAL                                                    57 ms
#
# `c2rs` process startup is **under 1 ms**, so batching cases into one process —
# the obvious guess — is worth under 2 %. The 43 ms is the whole story, and it is
# spent recomputing bytes that are a pure function of the case's source, the
# flags and the toolchain binaries. `scripts/mode_cross.sh` has consumed
# `work/capture-cache` for exactly this since 2026-08-04 (its header: cold
# 5 min 45 s, warm 13.8 s over 61,539 cells) and its header also names why this
# file could not: *"`expr_sweep.sh` cannot take this path: it drives `c2rs
# diff`, which does not consult the cache at all."* `c2rs diff` consults it now.
#
# **Two things had to change together, and only one of them is the cache.** The
# key includes the SOURCE PATH (it is baked verbatim into `.gl` and `.debug$S`),
# so with the corpus regenerated into a per-run `$out` — which is
# `/tmp/c2rs-gate-$$/sweep` under the gate — every key would be new on every run
# and the cache would serve nothing while costing an extra write. The case
# directory is therefore stable and lives outside `$out`, exactly as
# `mode_cross.sh` does it, with the run artifacts (lists, parts, reports) still
# in the caller's outdir so two runs never read each other's numbers.
#
# **What is graded does not move.** The port is recomputed per case per run; the
# standalone-c2 replay check runs per case per run; only c2's own obj and IL
# bundle are served from disk, keyed over the source bytes, the source argument,
# the flags, the cwd, the `cl.exe`/`c1xx.dll`/`c2.dll` contents, the wibo version
# and the cache root. What DOES move is that this row now depends on cache
# integrity, and that dependency is not left implicit:
#
#   * every case's outcome word (`cache=hit|miss|validated|poisoned|foreign|
#     bypass|off`) is counted and PRINTED in the count line, so an all-miss run
#     or a cold worktree is legible instead of being a mystery in the wall clock;
#   * `poisoned` and `foreign` are HARD FAILURES, never tolerances;
#   * **every run bypass-and-compares a strided sample of its own hits**
#     (`C2RS_SWEEP_VALIDATE`, default every 100th case): the entry is
#     re-captured through the real toolchain and byte-compared against what the
#     cache served. Nothing in this repo ran that validator on a schedule before.
#     At ~196 re-captures it costs ~8 s serial, ~2 s at 4 jobs.
#
# Set `C2RS_SWEEP_NO_CACHE=1` to grade the old way (every case captured for
# real). That is the control, and it is what the A/B in the rung was taken with.
set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-/tmp/c2rs-expr-sweep}"
limit="${2:-0}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"

# Refuse to share an output directory with another live sweep. This driver
# `rm -f`s the whole case set before regenerating it, so two concurrent runs
# against the default `/tmp/c2rs-expr-sweep` delete each other's cases mid-grade
# — one run's cleanup lands in the middle of the other's grading, and BOTH
# results are then meaningless while looking perfectly ordinary. That happened:
# an agent ran two sweeps at once, spotted it, killed both and re-ran with a
# private outdir. Nothing in the output would have said so.
#
# `mkdir` is the lock because it is atomic on every filesystem this runs on.
# The default stays shared on purpose — the generated cases are worth inspecting
# after a run — so the collision is made IMPOSSIBLE rather than made unlikely by
# a per-PID default that would leave litter nobody reads.
_lock="$out/.sweep.lock"
if ! mkdir "$_lock" 2>/dev/null; then
    echo "REFUSING: another sweep holds $out (lock: $_lock)." >&2
    echo "  Two sweeps in one outdir delete each other's cases and both results" >&2
    echo "  are silently wrong. Pass a private outdir:" >&2
    echo "      scripts/expr_sweep.sh /tmp/c2rs-sweep-\$\$" >&2
    echo "  If no sweep is running, the previous one was killed: rmdir '$_lock'" >&2
    exit 2
fi
trap 'rmdir "$_lock" 2>/dev/null || true' EXIT INT TERM

# ---- the STABLE case directory -------------------------------------------------
#
# See the block at the top. `mode_cross.sh` establishes the pattern and its
# reasoning applies here unchanged, including the ABSOLUTE requirement: `cl.exe`
# runs under wibo and is handed `Z:<path>`, so a relative case dir yields
# `Z:work\…`, which it cannot open, and the whole run comes back capture-fail
# while every check reads green.
#
# It stays PER WORKTREE (`$repo_root/work/…`, like `mode_cross.sh`'s) rather
# than being pointed at the main repo. Sharing one case directory across
# worktrees would warm a fresh worktree's first run, and it would also make two
# lanes with different `scripts/sweep.d` overwrite each other's corpus between
# runs — board #3249's hazard, taken deliberately for a cold-start saving that
# only ever costs one run per worktree. Not worth it.
cases="${C2RS_SWEEP_CASES:-$repo_root/work/expr-sweep/cases}"
mkdir -p "$cases"
cases="$(cd "$cases" && pwd)"

# On contention FALL BACK to a private (cold) case set rather than refusing —
# `mode_cross.sh`'s rule, and its argument is the one that matters here too: a
# gate row that reports NO-RESULT because a sibling was running is a RED GATE
# FROM AN ABSENCE, which is the failure this file's rules exist to forbid,
# arriving from the other direction. A private set is genuinely private; its
# only cost is a cold cache.
_clock="$cases/../.cases.lock"
_clock_held=1
if ! mkdir "$_clock" 2>/dev/null; then
    echo "NOTE: another sweep holds $cases (lock: $_clock)."
    echo "  Falling back to a PRIVATE case set for this run, which starts COLD."
    echo "  The result is exactly as valid; only the capture cache misses. If no"
    echo "  sweep is running, the previous one was killed: rmdir '$_clock'"
    cases="$out/cases"
    mkdir -p "$cases"
    cases="$(cd "$cases" && pwd)"
    _clock_held=0
fi
[ "$_clock_held" -eq 1 ] && trap 'rmdir "$_lock" 2>/dev/null || true; rmdir "$_clock" 2>/dev/null || true' EXIT INT TERM

rm -f "$cases"/*.cpp "$out"/*.cpp "$out"/cases.txt 2>/dev/null || true

# Build unconditionally and run a RUN-PRIVATE COPY of the binary — never
# `target/release/c2rs` directly. `scripts/harness_bin.sh` has the two failures
# this closes: the `if [ ! -x ]` guard that let a sweep grade today's cases with
# yesterday's code (47 phantom mismatches, false-green in the other direction),
# and a gate reading a file the rest of the tree may rewrite mid-run (one sweep
# did die that way at 6,225 cases; the mechanism is an unproven hypothesis and
# `harness_bin.sh` says so — the fix rests on the structural property, not on
# that observation). The identity line it prints — build
# time, content sha, tree HEAD — is what makes "which code produced this number"
# answerable from the log.
. "$repo_root/scripts/harness_bin.sh"
pin_harness "$repo_root" "$out"
c2rs="$C2RS_PINNED"

python3 "$repo_root/scripts/sweep_gen.py" "$cases" "$repo_root/scripts/sweep.d"

ls "$cases"/*.cpp | sort > "$out/cases.txt"
total=$(wc -l < "$out/cases.txt")
stride=1
if [ "$limit" -gt 0 ] 2>/dev/null && [ "$limit" -lt "$total" ]; then
    stride=$(( (total + limit - 1) / limit ))
    awk -v k="$stride" 'NR % k == 1 || k == 1' "$out/cases.txt" > "$out/cases.run"
else
    cp "$out/cases.txt" "$out/cases.run"
fi
run=$(wc -l < "$out/cases.run")

# Bail out loudly rather than reporting a vacuous pass.
first=$(head -1 "$out/cases.run")
if "$c2rs" diff "$first" 2>&1 | grep -q "SKIP"; then
    echo "SKIP: toolchain absent — the sweep would be vacuous"
    exit 0
fi

jobs="${C2RS_SWEEP_JOBS:-4}"
case "$jobs" in ''|*[!0-9]*) jobs=4 ;; esac
[ "$jobs" -ge 1 ] || jobs=1

# ---- the cache knobs, resolved here so the run's own output states them --------
#
# `C2RS_SWEEP_NO_CACHE=1` is the control path: every case captured for real,
# which is exactly what this driver did before 2026-08-18. Compared against the
# exact string `1` — a half-set variable (`=`, `=no`, `=0`) must not silently
# disarm a speedup NOR silently arm one, the rule `gate.sh` already applies to
# `C2RS_GATE_REQUIRE_GRADED`.
nocache=""
if [ "${C2RS_SWEEP_NO_CACHE:-0}" = 1 ]; then nocache="--no-cache"; fi
# Every Nth case in each worker's chunk is re-captured through the real
# toolchain and byte-compared against what the cache served. Chunks are assigned
# round-robin by line number, so "every Nth of each chunk" is a stride across the
# whole corpus, never a prefix — the same property `max-cases` was rewritten for.
# 0 disables. NOT disabled by default: a cache trusted without a sampling check
# is the instrument failure `capture_cache`'s own module docs are about.
validate="${C2RS_SWEEP_VALIDATE:-100}"
case "$validate" in ''|*[!0-9]*) validate=100 ;; esac
[ -z "$nocache" ] || validate=0

if [ "$stride" -eq 1 ]; then
    echo "sweeping $run of $total generated cases"
else
    echo "sweeping $run of $total generated cases (STRIDE $stride — a SAMPLE, not the corpus)"
fi
echo "  grading at $jobs job(s)"

# Split into `jobs` chunks by line number, so every case lands in exactly one
# worker and the chunk sizes are derivable from the case count alone.
part="$out/parts"
rm -rf "$part"; mkdir -p "$part"
awk -v j="$jobs" -v d="$part" '{ print > (d "/chunk." ((NR - 1) % j)) }' "$out/cases.run"

w=0
while [ "$w" -lt "$jobs" ]; do
    (
        _n=0; _m=0; _u=0; _x=0
        _hit=0; _miss=0; _val=0; _bad=0
        if [ -f "$part/chunk.$w" ]; then
            while read -r f; do
                _n=$((_n + 1))
                # The strided bypass-and-compare sample. `--validate-cache 1`
                # makes THIS invocation re-capture its hit and byte-compare;
                # `c2rs diff` runs one capture per process, so the sampling has
                # to be done out here — an in-process `--validate-cache N` would
                # test `1 % N`, which is never 0 for N > 1 and would validate
                # exactly nothing while printing that it was validating.
                if [ "$validate" -gt 0 ] && [ $((_n % validate)) -eq 0 ]; then
                    verdict=$("$c2rs" diff --validate-cache 1 "$f" 2>&1 | tail -1)
                else
                    # shellcheck disable=SC2086
                    verdict=$("$c2rs" diff $nocache "$f" 2>&1 | tail -1)
                fi
                # ---- the CACHE outcome, counted positively ------------------
                # Same discipline as the verdict classifier below: enumerate the
                # words that are OK and name everything else, so the next word
                # nobody foresaw is not the next silence.
                case "$verdict" in
                    *"cache=hit"*)       _hit=$((_hit + 1)) ;;
                    *"cache=validated"*) _val=$((_val + 1)); _hit=$((_hit + 1)) ;;
                    *"cache=miss"*)      _miss=$((_miss + 1)) ;;
                    *"cache=off"*|*"cache=bypass"*) : ;;
                    *"cache=poisoned"*|*"cache=foreign"*)
                        _bad=$((_bad + 1))
                        echo "CACHE-BAD $f  |  $verdict" >> "$part/cachebad.$w"
                        ;;
                    *) : ;;
                esac
                # ---- classify POSITIVELY. ------------------------------------
                # This used to be one arm, `*Mismatch*)`, and `case` in `sh` is
                # case-SENSITIVE, so it recognized exactly ONE of the four
                # verdicts `c2rs diff` can print. The other three —
                # `ReferenceError:` (the reference could not be captured or
                # replayed at all), `ReferenceReplay=MISMATCH` (the P0.1 ORACLE
                # itself failing, spelled in capitals) and anything unforeseen —
                # fell out of the `case` and were counted in `checked` as though
                # they had been graded clean.
                #
                # MEASURED 2026-08-04 on the whole corpus: **96 of 14,635 cases**
                # print `ReferenceError` — `cl.exe` rejects the generated source
                # (`error C2662`, `C4716`, and one `intstruct` typo) — and every
                # one of them was inside `checked=14635 mismatches=0`. The count
                # the gate re-derives was real; the grading behind 96 of it was
                # not. Eleven fragments are affected.
                #
                # So the OK set is now enumerated and everything else is named.
                # An unrecognized verdict is `unknown`, which fails: the next
                # verdict string nobody foresaw must not be the next silence.
                case "$verdict" in
                    *"Port=Match"*|*"Port=NotImplemented"*)
                        : ;;
                    *"Port=Mismatch"*)
                        _m=$((_m + 1))
                        # The FILE NAME first, then the source line. #232 took an
                        # extra investigation because only the source line was
                        # printed and ten cases share it — a mismatch you cannot
                        # re-run is a mismatch somebody calls unreproducible.
                        echo "MISMATCH  $f  |  $(head -1 "$f")" >> "$part/mismatch.$w"
                        ;;
                    *ReferenceError*|*"ReferenceReplay=MISMATCH"*|*ToolchainAbsent*|*SKIP*)
                        _u=$((_u + 1))
                        echo "UNGRADED  $f  |  $verdict" >> "$part/ungraded.$w"
                        ;;
                    *)
                        _x=$((_x + 1))
                        echo "UNKNOWN   $f  |  $verdict" >> "$part/unknown.$w"
                        ;;
                esac
            done < "$part/chunk.$w"
        fi
        echo "$_n" > "$part/checked.$w"
        echo "$_m" > "$part/mism.$w"
        echo "$_u" > "$part/ungr.$w"
        echo "$_x" > "$part/unk.$w"
        echo "$_hit $_miss $_val $_bad" > "$part/cache.$w"
    ) &
    w=$((w + 1))
done
wait

# Sum the workers' own counts. A worker killed mid-chunk writes no `checked.N` at
# all, so the sum comes up short and the reconciliation below fails — the count
# is the evidence the work happened, never the exit status (STATUS.md trap 5).
checked=0; mismatch=0; ungraded=0; unknown=0; reported=0
c_hit=0; c_miss=0; c_val=0; c_bad=0
w=0
while [ "$w" -lt "$jobs" ]; do
    if [ -f "$part/checked.$w" ]; then
        checked=$((checked + $(cat "$part/checked.$w")))
        reported=$((reported + 1))
    fi
    [ -f "$part/mism.$w" ] && mismatch=$((mismatch + $(cat "$part/mism.$w")))
    [ -f "$part/ungr.$w" ] && ungraded=$((ungraded + $(cat "$part/ungr.$w")))
    [ -f "$part/unk.$w" ]  && unknown=$((unknown + $(cat "$part/unk.$w")))
    if [ -f "$part/cache.$w" ]; then
        set -- $(cat "$part/cache.$w")
        c_hit=$((c_hit + $1)); c_miss=$((c_miss + $2))
        c_val=$((c_val + $3)); c_bad=$((c_bad + $4))
    fi
    w=$((w + 1))
done
cat "$part"/mismatch.* 2>/dev/null || true
cat "$part"/unknown.* 2>/dev/null || true
cat "$part"/cachebad.* 2>/dev/null || true

# `graded` is the count that carries evidence: cases the oracle actually ruled
# on. `checked` is only "cases the loop reached". They were the same number for
# this sweep's whole existence because three of the four verdicts were invisible.
graded=$((checked - ungraded - unknown))

count_line() {
    echo "checked=$checked mismatches=$mismatch graded=$graded ungraded=$ungraded unknown=$unknown"
}

# The cache's own accounting, on EVERY run, whether or not it did anything. A
# denominator nobody prints on both sides of a change is a denominator that
# grows unwatched (board #1002), and the thing being watched here is *how much
# of this row's evidence came off a disk this run*. `cache-bad` is the poisoned
# + refused population and is expected to be 0 forever; `validated` is the part
# of the hits that was bypass-and-compared against the real toolchain in THIS
# run, which is the number that says the speedup is still checking itself.
cache_line() {
    echo "cache: hit=$c_hit miss=$c_miss validated=$c_val cache-bad=$c_bad (of $checked cases)"
}

if [ "$checked" -ne "$run" ]; then
    count_line
    cache_line
    echo "FATAL: selected $run cases and only $checked were graded" >&2
    echo "  $reported of $jobs workers reported a count. A short count is a worker" >&2
    echo "  that died; the cases it held were never graded and this run establishes" >&2
    echo "  nothing about them." >&2
    exit 3
fi

count_line
cache_line

# ---- the UNGRADED baseline -----------------------------------------------------
#
# 96 generated cases do not compile: `cl.exe` rejects them, so no reference obj
# exists and the differential never runs. They are named here rather than
# silently absorbed, and the baseline is a NUMBER WITH A REASON next to it, the
# same discipline `sweep_mode.sh`'s `C2RS_SWEEP_MODE_MAX_DISAGREE` carries.
#
# Measured 2026-08-04 over all 14,635 cases at `/Ox /GS- /c`: 96 ungraded across
# 11 fragments — 99-chain-tail-fp-load 17, 98-cmp-order 16, 93-virtual-byval 15,
# 34-volatile-formal 12, 99-chain-tail-load 10, 96-cmp-two-calls 8,
# 98-chain-link-arg 4, 97-chained-call 4, 72-member-call 4,
# 73-framed-member-call 3, 45-offset-run 3. Two causes: a `volatile` receiver
# calling a non-`volatile`-qualified member (`error C2662`), and a handful of
# outright generator typos (`intstruct S`, an `int f(...)` with no `return`).
#
# Driving this to 0 means fixing the generators, which moves the corpus total
# every lane quotes; it is filed rather than done here. RAISING it needs a reason
# written next to the number, never just a passing run.
max_ungraded="${C2RS_SWEEP_MAX_UNGRADED:-96}"
if [ "$unknown" -ne 0 ]; then
    echo
    echo "UNRECOGNIZED VERDICT on $unknown case(s) — listed above."
    echo "  \`c2rs diff\` printed something this classifier does not enumerate. That"
    echo "  is how 96 cases spent this sweep's whole existence inside a clean count;"
    echo "  it is a hard failure, never a default."
    exit 1
fi
if [ "$ungraded" -gt "$max_ungraded" ]; then
    echo
    echo "UNGRADED $ungraded exceeds the carried baseline $max_ungraded."
    echo "  These cases produced NO reference obj, so the oracle never ruled on them"
    echo "  and nothing in this run establishes anything about their shapes."
    cat "$part"/ungraded.* 2>/dev/null | head -40
    exit 1
fi
if [ "$graded" -eq 0 ]; then
    echo
    echo "VACUOUS: $checked cases reached and NONE graded."
    exit 1
fi

# ---- the cache's own alarm -----------------------------------------------------
#
# `poisoned` = an entry was served, re-captured through the real toolchain, and
# the two DIFFERED. `foreign` = an entry was refused because the path it records
# having been captured at is not the path it was about to be served from. Both
# are expected to read 0 forever (`crates/c2-harness/src/capture_cache.rs`, board
# #1388), and both mean this row's oracle bytes did not come from this toolchain
# at this path. There is no tolerance and no baseline: the whole argument for
# serving this row from a cache is that the cache is checked, so an unchecked
# cache failing its check is a hard red.
if [ "$c_bad" -ne 0 ]; then
    echo
    echo "CACHE POISONED/REFUSED on $c_bad case(s) — listed above."
    echo "  The capture cache served bytes that are not what this toolchain"
    echo "  produces for this source at this path, or held entries it would not"
    echo "  serve. This row's oracle side is not trustworthy on this run."
    echo "  Re-run with C2RS_SWEEP_NO_CACHE=1 to grade every case for real."
    exit 1
fi
[ "$mismatch" -eq 0 ] || exit 1
