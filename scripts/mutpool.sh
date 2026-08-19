#!/bin/bash
#
# mutpool.sh — run a registered mutant campaign N mutants at a time.
#
# ## The cost this exists to cut
#
# A mutation campaign is `M` full workspace suites plus their rebuilds, and it
# is run **serially in one tree** because the mutation is a real edit to
# `crates/`: apply, build, `cargo test --workspace`, revert, repeat
# (`work/w-calleeguard/run_mutant.sh` is the reference implementation and this
# script does not replace it — it schedules it). At master `e82c9ede6` one suite
# is ~200 s of wall on an idle box and the whole run uses **~7 of 32 cores**, so
# a 21-mutant campaign spends most of its hours with the machine idle.
#
# Mutants are independent by construction: each one is applied to a pristine
# tree, measured, and reverted, and no mutant reads another's result. The only
# thing making them serial is that they share **one working tree and one
# `target/`**. Give each concurrent slot its own worktree and they stop sharing.
#
# ## What a slot is
#
# One `scripts/setup_worktree.sh` worktree: `compilers/` symlinked (read-only),
# `target/` **reflinked** from the main repo so the slot starts on a warm cargo
# cache instead of a cold build, `work/dc3-workload/` reflinked. Slots are
# created once and reused across every mutant they process, so the reflink and
# the first rebuild are paid `N` times, not `M` times.
#
# The **capture cache is deliberately shared**: `provenance::main_repo_root()`
# resolves every linked worktree to the main repo's `work/capture-cache`, and
# concurrent same-key captures there are guarded by `capture_cache`'s `O_EXCL`
# lockfile. Concurrent slots therefore *help* each other — the second slot to
# want a capture gets the first slot's.
#
# ## What it refuses to do
#
# Scheduling is the only thing this script does. It does **not** classify a run,
# does not aggregate, and does not print a colour: the results table is derived
# from the logs by the campaign's own re-deriver (`docs/rungs/README.md` rule 2,
# and the three retroactive classifier corrections that rule comes from).
#
# It carries the two probe rules that a campaign runner must carry:
#
#   * `C2RS_REQUIRE_TOOLCHAIN=1` is exported into every slot. A slot worktree is
#     created by `setup_worktree.sh`, which refuses to run without `compilers/`
#     — but the variable is what turns a *silent* skip into a failure if the
#     symlink ever breaks mid-campaign (#3219 / #3231).
#   * Every run's `census_gate` **duration** is recorded next to its log. A run
#     whose differential took ~0 s graded nothing, and its colour is void
#     regardless of its exit code. This script writes the number; it does not
#     judge it.
#
# ## Usage
#
#   scripts/mutpool.sh --list <tsv> --slots N [--out DIR] [--suite-jobs J]
#                      [--keep-slots] [--base <ref>]
#
# `<tsv>` is one mutant per line, tab-separated:
#
#   <id>  <file-relative-to-repo>  <exact-from-string>  <exact-to-string>
#
# `NONE` for both strings means an unmutated baseline run. The `from` string
# must occur **exactly once** in the file or the mutant aborts — a patch that
# matched zero sites reads GREEN and is the whole reason mutation campaigns get
# re-run (`w-mutcensus`'s enumeration went stale twice inside one lane's wall
# clock, which is why this matches strings and never line numbers).
#
# Outputs under `--out` (default `work/mutpool`):
#
#   <id>.log        the whole `partest.sh` run for that mutant
#   <id>.meta       id, slot, rc, wall, build-wall, census_gate seconds, patch stat
#   <id>.names      `<target> :: <test> :: <verdict>`, sorted — so a campaign can
#                   diff a mutant against the baseline BY NAME instead of by count
#   pool.tsv        one row per mutant, appended as it finishes

set -uo pipefail

MAIN="$(cd "$(dirname "$0")/.." && pwd)"
cd "$MAIN"

LIST=""
SLOTS=2
OUT="work/mutpool"
SUITE_JOBS=8
KEEP=0
BASE="HEAD"

while [ $# -gt 0 ]; do
    case "$1" in
        --list) LIST="$2"; shift 2 ;;
        --slots) SLOTS="$2"; shift 2 ;;
        --out) OUT="$2"; shift 2 ;;
        --suite-jobs) SUITE_JOBS="$2"; shift 2 ;;
        --keep-slots) KEEP=1; shift ;;
        --base) BASE="$2"; shift 2 ;;
        -h|--help) sed -n '3,75p' "$0"; exit 0 ;;
        *) echo "mutpool.sh: unknown option $1" >&2; exit 2 ;;
    esac
done

[ -n "$LIST" ] || { echo "mutpool.sh: --list <tsv> is required" >&2; exit 2; }
[ -f "$LIST" ] || { echo "mutpool.sh: no such list: $LIST" >&2; exit 2; }
case "$SLOTS" in ''|*[!0-9]*) echo "mutpool.sh: --slots wants a number" >&2; exit 2 ;; esac
[ "$SLOTS" -ge 1 ] || { echo "mutpool.sh: --slots must be >= 1" >&2; exit 2; }

mkdir -p "$OUT"; OUT="$(cd "$OUT" && pwd)"
BASE_SHA="$(git rev-parse "$BASE")" || exit 2

# A campaign taken off a dirty tree measures the dirt (`w-bind16` §8.1's
# discarded run). Refuse before spending an hour.
if ! git diff --quiet -- crates/; then
    echo "mutpool.sh: ABORT — crates/ is dirty before the campaign" >&2
    exit 2
fi

# ---------------------------------------------------------------------------
# Slots
# ---------------------------------------------------------------------------
SLOTDIR="$MAIN/.claude/worktrees"
declare -a SLOTPATH
for i in $(seq 1 "$SLOTS"); do
    p="$SLOTDIR/mutpool-$i"
    if [ ! -e "$p/.git" ]; then
        echo "== provisioning slot $i at $p"
        "$MAIN/scripts/setup_worktree.sh" "$p" "mutpool-slot-$i" "$BASE_SHA" \
            >"$OUT/slot-$i.setup.log" 2>&1 || {
            echo "mutpool.sh: slot $i setup FAILED — see $OUT/slot-$i.setup.log" >&2
            tail -20 "$OUT/slot-$i.setup.log" >&2
            exit 1
        }
    else
        echo "== reusing slot $i at $p"
        git -C "$p" checkout -q --detach "$BASE_SHA" 2>/dev/null
        git -C "$p" checkout -q -- crates/ 2>/dev/null
    fi
    SLOTPATH[$i]="$p"
done

# ---------------------------------------------------------------------------
# One mutant in one slot
# ---------------------------------------------------------------------------
run_mutant() {
    local slot="$1" id="$2" file="$3" from="$4" to="$5"
    local wt="${SLOTPATH[$slot]}"
    local meta="$OUT/$id.meta"

    : >"$meta"
    printf 'id\t%s\nslot\t%s\nworktree\t%s\nbase\t%s\n' "$id" "$slot" "$wt" "$BASE_SHA" >>"$meta"

    if ! git -C "$wt" diff --quiet -- crates/; then
        printf 'status\tABORT-DIRTY-BEFORE\n' >>"$meta"
        return 2
    fi

    if [ "$from" != "NONE" ]; then
        local n
        n=$(grep -Fc -- "$from" "$wt/$file" 2>/dev/null || true)
        printf 'sites\t%s\n' "${n:-0}" >>"$meta"
        if [ "${n:-0}" != "1" ]; then
            printf 'status\tABORT-SITE-COUNT\n' >>"$meta"
            return 3
        fi
        python3 - "$wt/$file" "$from" "$to" <<'PY' || { printf 'status\tABORT-PATCH\n' >>"$meta"; return 3; }
import sys
p, a, b = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p).read()
assert s.count(a) == 1, (p, s.count(a))
open(p, 'w').write(s.replace(a, b, 1))
PY
        git -C "$wt" --no-pager diff --stat -- crates/ >>"$meta"
    else
        printf 'sites\tN/A-baseline\n' >>"$meta"
    fi

    # Build and suite are timed separately: the rebuild is cargo-parallel and
    # saturates the box on its own, so it is the leg that decides where slot
    # concurrency stops paying. Reporting one number for both hides that.
    local b0 b1 t0 t1 rc
    b0=$(date +%s.%N)
    ( cd "$wt" && cargo test --workspace --release --no-run ) >"$OUT/$id.build.log" 2>&1
    b1=$(date +%s.%N)

    t0=$(date +%s.%N)
    ( cd "$wt" && C2RS_REQUIRE_TOOLCHAIN=1 scripts/partest.sh --jobs "$SUITE_JOBS" \
        --out "$wt/work/mutpool-run" ) >"$OUT/$id.log" 2>&1
    rc=$?
    t1=$(date +%s.%N)

    cp "$wt/work/mutpool-run/names.txt" "$OUT/$id.names" 2>/dev/null
    local census
    census=$(awk '$1=="census_gate"{print $2}' "$wt/work/mutpool-run/durations.tsv" 2>/dev/null)

    printf 'rc\t%s\nbuild_s\t%s\nsuite_s\t%s\ncensus_gate_s\t%s\nnames\t%s\n' \
        "$rc" \
        "$(awk -v a="$b0" -v b="$b1" 'BEGIN{printf "%.1f", b-a}')" \
        "$(awk -v a="$t0" -v b="$t1" 'BEGIN{printf "%.1f", b-a}')" \
        "${census:-MISSING}" \
        "$(wc -l <"$OUT/$id.names" 2>/dev/null || echo 0)" >>"$meta"

    if [ "$from" != "NONE" ]; then
        git -C "$wt" checkout -- "$file"
    fi
    if ! git -C "$wt" diff --quiet -- crates/; then
        printf 'status\tABORT-DIRTY-AFTER\n' >>"$meta"
        return 4
    fi
    printf 'status\tRAN\n' >>"$meta"
    return "$rc"
}

# ---------------------------------------------------------------------------
# The pool: one worker per slot, all pulling from one queue.
# ---------------------------------------------------------------------------
# A worker-per-slot queue, not a batch barrier. `scripts/gate.sh` and
# `scripts/expr_sweep.sh` both use `if running >= jobs; then wait; running=0`,
# which is the only pool a POSIX `#!/bin/sh` can express — and it stalls every
# slot until the slowest member of each batch finishes. Mutant runs are minutes
# long and unequal, so that barrier is expensive here; this file is bash for
# exactly that reason.
QUEUE="$OUT/queue"
grep -v '^[[:space:]]*#' "$LIST" | grep -v '^[[:space:]]*$' >"$QUEUE"
NMUT=$(wc -l <"$QUEUE")
[ "$NMUT" -gt 0 ] || { echo "mutpool.sh: the list is empty — refusing to report a campaign" >&2; exit 1; }

echo "== $NMUT mutant(s), $SLOTS slot(s), suite jobs $SUITE_JOBS, base $BASE_SHA"
printf 'id\tslot\trc\tbuild_s\tsuite_s\tcensus_gate_s\tnames\n' >"$OUT/pool.tsv"

LOCK="$OUT/queue.lock"
NEXT="$OUT/queue.next"
echo 1 >"$NEXT"

take() {
    # Serialise the queue read with an O_EXCL directory lock — the same
    # primitive `capture_cache` uses, and the only one that is atomic on every
    # filesystem this repo runs on.
    local i
    for i in $(seq 1 2000); do
        if mkdir "$LOCK" 2>/dev/null; then
            local n; n=$(cat "$NEXT")
            echo $((n + 1)) >"$NEXT"
            rmdir "$LOCK"
            echo "$n"
            return 0
        fi
        sleep 0.05
    done
    echo 0
}

worker() {
    local slot="$1"
    while :; do
        local n; n=$(take)
        [ "$n" -ge 1 ] || return 0
        [ "$n" -le "$NMUT" ] || return 0
        local line; line=$(sed -n "${n}p" "$QUEUE")
        [ -n "$line" ] || continue
        local id file from to
        IFS=$'\t' read -r id file from to <<<"$line"
        echo "-- slot $slot -> $id"
        run_mutant "$slot" "$id" "$file" "${from:-NONE}" "${to:-NONE}"
        local rc=$?
        # Append atomically under the same lock so two slots cannot interleave
        # a partial line into pool.tsv.
        local i
        for i in $(seq 1 2000); do
            if mkdir "$LOCK" 2>/dev/null; then
                printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$id" "$slot" "$rc" \
                    "$(awk -F'\t' '$1=="build_s"{print $2}' "$OUT/$id.meta")" \
                    "$(awk -F'\t' '$1=="suite_s"{print $2}' "$OUT/$id.meta")" \
                    "$(awk -F'\t' '$1=="census_gate_s"{print $2}' "$OUT/$id.meta")" \
                    "$(awk -F'\t' '$1=="names"{print $2}' "$OUT/$id.meta")" >>"$OUT/pool.tsv"
                rmdir "$LOCK"
                break
            fi
            sleep 0.05
        done
    done
}

WALL0=$(date +%s.%N)
for i in $(seq 1 "$SLOTS"); do worker "$i" & done
wait
WALL1=$(date +%s.%N)

echo
echo "mutpool: $NMUT mutant(s) at $SLOTS slot(s) in $(awk -v a="$WALL0" -v b="$WALL1" 'BEGIN{printf "%.1f", b-a}')s"
echo "mutpool: load $(uptime | sed 's/.*load average: //')"
echo "mutpool: rows in $OUT/pool.tsv — colours are DERIVED FROM THE LOGS, not from this script"

if [ "$KEEP" -eq 0 ]; then
    echo "mutpool: slots kept at $SLOTDIR/mutpool-* (remove with git worktree remove)"
fi
