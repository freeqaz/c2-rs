#!/bin/sh
# ladder.sh — the CHAIN-SINK LADDER for one TU, re-derived per lane rather than
# inherited (`w-bd` #1316: a ladder terminal published on one tree was the
# terminal of ZERO of seventeen ladders on the next one).
#
#   sh work/w-pool/ladder.sh <c2rs-binary> <src-relative-to-dc3> [out-prefix]
#
# Each rung ADDS the token the census reported as the blocker and re-runs the
# per-function census. The chain sink is POISONED — a body that reaches the end
# having used one refuses under `expr-chain-sink-poison` — so no rung of this
# ladder can move an obj byte, and the whole walk is a measurement.
#
# The walk stops at:
#   READER-CLEAR   no function reports a blocker (the census is empty)
#   EXIT:<key>     the key is not a sinkable token (`noform`, `cflow-*`, a
#                  shape key) — the ladder's terminal, and the thing to quote
#   BADTOKEN       a typo; the sink reports it rather than shortening the chain
#
# The dc3 tree is DERIVED (`C2RS_DC3`, else the nearest sibling) — CLAUDE.md
# forbids absolute machine paths in source.
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
sib() {
    d="$repo_root"
    while [ "$d" != "/" ]; do
        [ -d "$d/../$1" ] && { (cd "$d/../$1" && pwd); return 0; }
        d="$(dirname "$d")"
    done
    return 1
}
dc3="${C2RS_DC3:-$(sib dc3-decomp)}"
[ -d "$dc3" ] || { echo "SKIP: no dc3 tree (set C2RS_DC3)"; exit 3; }

c2rs="${1:?usage: ladder.sh <c2rs> <src> [out-prefix]}"
src="${2:?usage: ladder.sh <c2rs> <src> [out-prefix]}"
out="${3:-}"

flags="$repo_root/work/dc3-workload/flags.txt"
[ -f "$flags" ] || { echo "SKIP: no $flags (run scripts/gen_dc3_workload.sh)"; exit 3; }

spec=""
rung=0
echo "== ladder $src"
while [ "$rung" -lt 40 ]; do
    if [ -n "$out" ]; then log="$out.rung$rung.txt"; else log="$(mktemp)"; fi
    C2RS_SINK_CHAIN="$spec" "$c2rs" census "$src" \
        --flags-file "$flags" --cwd "$dc3" > "$log" 2>&1 || true

    inclass=$(sed -n 's/.*-> \([0-9]*\)\/\([0-9]*\) functions in class.*/\1 \2/p' "$log")

    # Every reported blocking key, most frequent first. The census prints them
    # under "blocking features"; the per-function lines carry them too, and the
    # per-function lines are used because they cannot be confused with the
    # ">" byte-context lines beneath the histogram.
    keys=$(sed -n 's/^  \[ *[0-9]*\] GAP \([^ ]*\) .*/\1/p' "$log" | sort | uniq -c | sort -rn)

    if [ -z "$keys" ]; then
        echo "  rung $rung  [$spec]"
        echo "  -> READER-CLEAR after $rung rungs (in class: $inclass)"
        exit 0
    fi
    # `expr-chain-sink-poison` is not a blocker — it is the sink announcing that
    # THIS body walked to its end on sunk tokens. Counting it as the terminal
    # would stop the ladder at the first body that finished and leave the others
    # unwalked, which is the shape `w-bd` #1316 caught: a terminal that is an
    # artefact of the stopping rule rather than of the TU.
    live=$(echo "$keys" | grep -v 'expr-chain-sink-poison' || true)
    if [ -z "$live" ]; then
        echo "  rung $rung  [$spec]"
        echo "  -> ALL-POISON after $rung rungs — every body walks to its end on sunk tokens (in class: $inclass)"
        exit 0
    fi
    first=$(echo "$live" | head -1 | sed 's/^ *[0-9]* *//')
    echo "  rung $rung  in-class $inclass  first=$first   all=$(echo "$keys" | sed 's/^ *//' | tr '\n' ' ')"

    case "$first" in
        expr-chain-badtoken*)  echo "  -> BADTOKEN in [$spec]"; exit 1 ;;
        expr-op-*)   tok="op:$(echo "$first" | sed 's/^expr-op-0x//')" ;;
        expr-brtrue) tok="op:39" ;;
        expr-brfalse) tok="op:38" ;;
        expr-load-type-*|expr-lit-type-*) tok="type" ;;
        expr-convert-target*) tok="convert" ;;
        expr-intrinsic-*) tok="intrinsic" ;;
        *) echo "  -> EXIT:$first  after $rung rungs (in class: $inclass)"; exit 0 ;;
    esac

    case ",$spec," in
        *",$tok,"*) echo "  -> STALL: $first maps to $tok which is already in [$spec]"; exit 0 ;;
    esac
    if [ -z "$spec" ]; then spec="$tok"; else spec="$spec,$tok"; fi
    rung=$((rung + 1))
done
echo "  -> CAP: 40 rungs without a terminal"
