#!/bin/bash
# `w-grammarscreen` — run the corpus, STAGE BY STAGE, each with its own probe log.
#
#     stages.sh <tag> [stage ...]
#
# Stages (default: all): suite bench sweep cross debug gate scan
#
# `w-deadsites` F2 is the reason this is per stage rather than per run: that lane
# recorded THAT a site fired and not WHERE, so all its reachable rows were priced
# the same. A site reached by the unit suite is a cheap witness; one reached only
# by the 878-TU workload is an expensive one.
#
# `C2RS_GRAMMARPROBE_LOG` is a FRESH EMPTY FILE per stage, so a stage can never
# inherit another's hits. Every log is kept.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TAG="${1:?usage: stages.sh <tag> [stage ...]}"; shift
STAGES=("$@")
[ ${#STAGES[@]} -eq 0 ] && STAGES=(suite bench sweep cross debug gate scan)

OUT="$ROOT/work/w-grammarscreen/logs"
mkdir -p "$OUT"

export C2RS_REQUIRE_TOOLCHAIN=1
MAIN_REPO="$(cd "$(git -C "$ROOT" rev-parse --git-common-dir)/.." && pwd)"
export C2RS_DC3="${C2RS_DC3:-$MAIN_REPO/../dc3-decomp}"

run_stage() {
    local st="$1"; shift
    local probe="$OUT/$TAG.$st.hits"
    : > "$probe"
    export C2RS_GRAMMARPROBE_LOG="$probe"
    echo "=== $TAG/$st ==="
    local t0=$SECONDS
    "$@" > "$OUT/$TAG.$st.log" 2>&1
    local rc=$?
    local n
    n=$(sort -u "$probe" 2>/dev/null | wc -l)
    echo "$TAG/$st exit=$rc wall=$((SECONDS - t0))s distinct-sites=$n"
}

for st in "${STAGES[@]}"; do
  case "$st" in
    suite) run_stage suite cargo test --workspace --release --no-fail-fast ;;
    bench) run_stage bench ./target/release/c2rs bench ;;
    sweep) run_stage sweep scripts/expr_sweep.sh ;;
    cross) run_stage cross scripts/mode_cross.sh ;;
    debug) run_stage debug scripts/debug_lane.sh ;;
    gate)  run_stage gate  scripts/gate.sh --jobs 16 --require-graded ${GATE_EXTRA:-} ;;
    scan)  run_stage scan  ./target/release/c2rs gap --list work/dc3-workload/files.txt \
                --flags-file work/dc3-workload/flags.txt --cwd "$C2RS_DC3" --jobs 16 ;;
    *) echo "unknown stage $st" >&2; exit 2 ;;
  esac
done
