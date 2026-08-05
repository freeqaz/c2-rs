#!/bin/sh
# chainwalk.sh — walk one TU's IL refusal chain with C2RS_SINK_CHAIN (board
# #660/#661), one opcode at a time, printing the SUCCESSOR key at each step.
#
# The sink decodes and POISONS: it pushes no IlOp and every walk that used one
# refuses under `expr-chain-sink-poison`, so this instrument cannot move an obj
# byte. It measures the LENGTH of the chain, which is what a price is.
#
# Usage: chainwalk.sh <src-relative-to-dc3> [max-steps]
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
root="$here/../.."
# The dc3 tree: `C2RS_DC3`, else the sibling checkout found by walking UP from
# the repo root — this tree may be the main repo or a worktree under
# `.claude/worktrees/<lane>/`, and those differ by three levels. No absolute
# path lives in this file (CLAUDE.md). Same locator as `work/w-frame/refobj.sh`.
sib() {
    d="$root"
    while [ "$d" != "/" ]; do
        [ -d "$d/../$1" ] && { (cd "$d/../$1" && pwd); return 0; }
        d="$(dirname "$d")"
    done
    return 1
}
dc3="${C2RS_DC3:-$(sib dc3-decomp)}"
src="$1"
max="${2:-40}"
n=$(basename "$src" .cpp)
printf '%s\n' "$src" > "$here/one_$n.txt"

spec=""
i=0
while [ "$i" -lt "$max" ]; do
    if [ -z "$spec" ]; then unset C2RS_SINK_CHAIN || true; else export C2RS_SINK_CHAIN="$spec"; fi
    "$root/target/release/c2rs" gap \
        --list "$here/one_$n.txt" \
        --flags-file "$root/work/dc3-workload/flags.txt" \
        --cwd "$dc3" \
        --jsonl "$here/walk_$n.jsonl" \
        --jobs 1 > "$here/walk_$n.log" 2>&1 || true
    key=$(python3 - "$here/walk_$n.jsonl" <<'PY'
import json,sys
for line in open(sys.argv[1]):
    d=json.loads(line)
    if d.get('record')=='provenance': continue
    b=d.get('fn_blockers') or {}
    print(';'.join(sorted(b)) if b else 'NONE')
PY
)
    echo "step $i  spec=[${spec:-<none>}]  -> $key"
    case "$key" in
        expr-op-0x*)
            hex=${key#expr-op-0x}
            if [ -z "$spec" ]; then spec="op:$hex"; else spec="$spec,op:$hex"; fi
            ;;
        *) echo "TERMINAL: $key"; break ;;
    esac
    i=$((i+1))
done
