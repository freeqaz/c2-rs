#!/bin/sh
# scan_pair.sh — THE REQUIRED-ZERO SCAN PAIR, committed (board #3451's pattern,
# applied to the second instrument four lanes have retyped).
#
#   scripts/scan_pair.sh --base work/<lane>/c2rs-base --tip work/<lane>/c2rs-tip \
#                        --out work/<lane> [--cwd ../dc3-decomp] [--jobs 8]
#
# A **construct rung**'s success criterion is a required-zero byte delta, graded
# by a line-for-line identity diff of per-lane counts before and after
# (`docs/rungs/README.md` kind 2). Two instruments carry that grade: `gate.sh`'s
# count table and the 878-TU `c2rs gap` scan's `gap-metric` keys. This script
# runs the second one as a PAIR and refuses to print a comparison it cannot
# stand behind.
#
# `scripts/cost_arms.py` was committed by `w-s1c3` for exactly this reason
# (#3451): a protocol retyped every lane is a protocol whose corrections cannot
# accumulate. The pair harness was NOT committed by that lane, and it is the one
# that had already produced two live defects. Both are below, as assertions:
#
# ---- WHY EVERY GUARD HERE EXISTS -------------------------------------------
#
# * **The workload stamp, read before AND after each arm** — decision 5, as
#   sharpened by board **#3426**. `../dc3-decomp` is a live repo other agents
#   merge into; it took FOUR distinct values during one lane (`w-s1c2` §3), and
#   a threshold and its corpus are one fact. A stamp that moves between the two
#   arms means the two arms did not scan the same workload and the pair is void.
#
# * **…and the DIRTY FLAG beside it, which no rung has ever quoted.** The stamp
#   is `git rev-parse HEAD`, 12 chars. A tracked file modified in the working
#   tree changes what `cl.exe` compiles WITHOUT moving one bit of the stamp.
#   `GitInfo::probe` (`crates/c2-harness/src/provenance.rs:150`) already computes
#   the flag and `dirty_label()` already prints it; nothing ever asserted on it.
#   This script digests `git status --porcelain -uno` at every stamp read, so a
#   dirty-tree edit mid-pair voids the pair exactly as a commit would.
#
# * **An arm that graded NOTHING is not an arm that agreed** — board **#3470**.
#   `w-s1c3`'s first pair passed every stamp check with `PAIR_EXIT=0` and the
#   base arm's entire log was one line: `SKIP: toolchain absent`. `repo_root()`
#   is `CARGO_MANIFEST_DIR/../..` **baked at compile time**, so a binary built in
#   a scratch tree resolves `compilers/` relative to THAT tree, finds none, and
#   degrades cleanly — which `CLAUDE.md` requires — and **exits 0**. What caught
#   it was the key denominator: `base: 0 keys` beside `tip: 399`. So this script
#   asserts, per arm: no `SKIP: toolchain absent`, exit 0, AND a nonzero key
#   count. It exits 4 rather than printing a comparison.
#
# * **The DENOMINATOR prints beside the diff, at BOTH ends.** A zero difference
#   between two empty sets is this project's most-repeated defect (`w-s1c2` §3.2
#   caught it live at `0 lines over 0 keys`). "0 lines" alone is not a result.
#
# * **`C2RS_COMPILERS` and `C2RS_WIBO` are exported for both arms**, from one
#   resolution done once here. Two arms resolving the toolchain by two different
#   mechanisms are two arms that can silently disagree about what they compared
#   against — which is the same defect #3470 found, one layer up.
#
# Exit codes:  0 identical · 1 the keys differ · 2 usage/setup · 3 the workload
#              moved (VOID) · 4 an arm graded nothing (VOID)
set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"

base=''; tip=''; out=''; jobs=8; limit=''
dc3="${C2RS_DC3:-$repo_root/../dc3-decomp}"
list="$repo_root/work/dc3-workload/files.txt"
flags="$repo_root/work/dc3-workload/flags.txt"
tag=''

while [ $# -gt 0 ]; do
    case "$1" in
        --base)  base="$2"; shift 2 ;;
        --tip)   tip="$2";  shift 2 ;;
        --out)   out="$2";  shift 2 ;;
        --cwd)   dc3="$2";  shift 2 ;;
        --list)  list="$2"; shift 2 ;;
        --flags) flags="$2"; shift 2 ;;
        --jobs)  jobs="$2"; shift 2 ;;
        --limit) limit="$2"; shift 2 ;;
        --tag)   tag="$2";  shift 2 ;;
        *) echo "scan_pair.sh: unknown argument '$1'" >&2; exit 2 ;;
    esac
done

[ -n "$base" ] && [ -n "$tip" ] || { echo "scan_pair.sh: --base and --tip are required" >&2; exit 2; }
[ -x "$base" ] || { echo "scan_pair.sh: --base '$base' is not executable" >&2; exit 2; }
[ -x "$tip" ]  || { echo "scan_pair.sh: --tip '$tip' is not executable" >&2; exit 2; }
[ -f "$list" ] || { echo "scan_pair.sh: no TU list at $list (scripts/gen_dc3_workload.sh)" >&2; exit 2; }
[ -f "$flags" ] || { echo "scan_pair.sh: no flags at $flags (scripts/gen_dc3_workload.sh)" >&2; exit 2; }
[ -d "$dc3" ]  || { echo "scan_pair.sh: no workload checkout at $dc3" >&2; exit 2; }
out="${out:-$repo_root/work/scan_pair}"
mkdir -p "$out"
sfx="${tag:+_$tag}"

# ---- the toolchain, resolved ONCE and exported to both arms (#3470) ---------
if [ -z "${C2RS_COMPILERS:-}" ]; then
    if [ -d "$repo_root/compilers" ]; then C2RS_COMPILERS="$repo_root/compilers"
    else echo "scan_pair.sh: no compilers/ and C2RS_COMPILERS unset — both arms would SKIP" >&2; exit 2; fi
fi
if [ -z "${C2RS_WIBO:-}" ]; then
    for cand in "$repo_root/../wibo/build/release/wibo" "$repo_root/../wibo/build/wibo"; do
        [ -x "$cand" ] && { C2RS_WIBO="$cand"; break; }
    done
    [ -n "${C2RS_WIBO:-}" ] || { C2RS_WIBO="$(command -v wibo || true)"; }
    [ -n "${C2RS_WIBO:-}" ] || { echo "scan_pair.sh: wibo not found — both arms would SKIP" >&2; exit 2; }
fi
export C2RS_COMPILERS C2RS_WIBO

# ---- the workload stamp: HEAD, the path set, **and** the CONTENT ------------
#
# Three components, and the third exists because the second is not enough.
#
# `git status --porcelain -uno` is the SET OF MODIFIED PATHS. It moves on
# clean -> dirty and on one dirty file set -> another. It does **not** move when
# a file that is ALREADY listed as modified is edited again: the porcelain line
# is byte-identical either way. That is exactly lane `w-3475`'s hazard — HEAD
# stable across all four reads while the bytes being compiled changed — and a
# path-set digest cannot see it.
#
# Demonstrated on `../dc3-decomp`, two appends to one already-modified file:
#
#     after edit 1   porcelain 183619428431   content  344768490326
#     after edit 2   porcelain 183619428431   content 2974227181337
#                              ^^ UNCHANGED            ^^ MOVED
#
# `git diff HEAD | cksum` costs **0.004 s** on this workload, so there is no
# reason to run the weak version. Untracked files stay excluded, deliberately
# and for `-uno`'s own reason: they are build scratch and cannot change a
# compile.
#
# STRONGER STILL, and NOT built here: pin the corpus instead of observing it —
# `cp --reflink=auto` the workload into `work/<lane>/dc3-pin` and scan that, so
# a mid-pair edit is impossible rather than merely detected (`w-3475`'s valid
# pair). Priced at ~14 GB CoW-shared and a reap step; the detection below is
# what this script offers, and a lane that wants immutability should pin.
stamp() {
    _h=$(git -C "$dc3" rev-parse HEAD 2>/dev/null | cut -c1-12 || echo UNVERSIONED)
    _p=$(git -C "$dc3" status --porcelain -uno 2>/dev/null | sort | cksum | tr -d ' ' || echo '?')
    _c=$(git -C "$dc3" diff HEAD 2>/dev/null | cksum | tr -d ' ' || echo '?')
    printf '%s+%s+%s' "$_h" "$_p" "$_c"
}

# ---- one arm ---------------------------------------------------------------
# Prints nothing a caller can read as a number unless every guard passed.
run_arm() {
    _label="$1"; _bin="$2"
    _log="$out/scan_${_label}${sfx}.log"
    _keys="$out/keys_${_label}${sfx}.txt"

    _before=$(stamp)
    echo "  [$_label] stamp BEFORE $_before"
    set +e
    # shellcheck disable=SC2086
    "$_bin" gap --list "$list" --flags-file "$flags" --cwd "$dc3" \
        --jobs "$jobs" ${limit:+--limit "$limit"} > "$_log" 2>&1
    _exit=$?
    set -e
    _after=$(stamp)
    echo "  [$_label] stamp AFTER  $_after   (exit $_exit, log $_log)"

    # -- the three void checks, in the order they were learned ---------------
    if grep -q 'SKIP: toolchain absent' "$_log"; then
        echo "VOID: [$_label] logged 'SKIP: toolchain absent' — it graded NOTHING and exited $_exit (#3470)" >&2
        mv "$_log" "$out/scan_${_label}${sfx}_VOID.log"
        return 4
    fi
    if [ "$_exit" -ne 0 ]; then
        echo "VOID: [$_label] exited $_exit" >&2
        return 4
    fi
    grep -E '^ *gap-metric ' "$_log" | sed -E 's/^ *gap-metric //' | sort > "$_keys"
    _n=$(wc -l < "$_keys" | tr -d ' ')
    if [ "$_n" -eq 0 ]; then
        echo "VOID: [$_label] produced 0 gap-metric keys — a zero difference between two empty sets is not a result (#3470, w-s1c2 §3.2)" >&2
        return 4
    fi
    echo "  [$_label] gap-metric keys: $_n"
    if [ "$_before" != "$_after" ]; then
        echo "VOID: [$_label] the workload MOVED across this arm: $_before -> $_after (#3426)" >&2
        return 3
    fi
    printf '%s\n' "$_before" > "$out/stamp_${_label}${sfx}.txt"
    printf '%s\n' "$_n" > "$out/n_${_label}${sfx}.txt"
    return 0
}

echo "scan_pair.sh — required-zero scan pair"
echo "  workload  $dc3"
echo "  list      $list ($(wc -l < "$list" | tr -d ' ') TUs)"
echo "  compilers $C2RS_COMPILERS"
echo "  wibo      $C2RS_WIBO"
echo "  base      $base"
echo "  tip       $tip"
echo "  stamp at start: $(stamp)"
echo

# The two arms run BACK TO BACK against one corpus, pinned binaries, base first.
run_arm base "$base" || exit $?
run_arm tip  "$tip"  || exit $?

sb=$(cat "$out/stamp_base$sfx.txt"); st=$(cat "$out/stamp_tip$sfx.txt")
nb=$(cat "$out/n_base$sfx.txt");     nt=$(cat "$out/n_tip$sfx.txt")

echo
if [ "$sb" != "$st" ]; then
    echo "VOID: the workload moved BETWEEN the arms: $sb -> $st (#3426)" >&2
    exit 3
fi
echo "STAMP HELD: $sb  (HEAD+dirty-digest, read before and after each arm)"

diff -u "$out/keys_base$sfx.txt" "$out/keys_tip$sfx.txt" > "$out/keys_diff$sfx.txt" || true
d=$(grep -cE '^[+-][^+-]' "$out/keys_diff$sfx.txt" || true)

# The denominator prints beside the diff, at BOTH ends, always.
echo "IDENTITY DIFF: $d lines over $nb keys (base) / $nt keys (tip)"
if [ "$nb" -ne "$nt" ]; then
    echo "  NOTE: the key COUNT itself moved, $nb -> $nt — a new or removed metric is a result, not noise (#1002)"
fi
if [ "$d" -eq 0 ] && [ "$nb" -eq "$nt" ]; then
    echo "PAIR: IDENTICAL"
    exit 0
fi
echo "PAIR: DIFFERS — see $out/keys_diff$sfx.txt"
exit 1
