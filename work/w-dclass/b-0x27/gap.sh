#!/bin/sh
# gap.sh — full-workload gap scan. $1 = output tag, $2 (optional) = sink value.
set -eu
root=<repo>/.claude/worktrees/agent-a90821e906953b0fd
export C2RS_COMPILERS=<repo>/compilers
export C2RS_WIBO=<home>/code/milohax/wibo/build/wibo
[ $# -ge 2 ] && export C2RS_SINK_OFF_ADD_ARG="$2"
out="$root/work/w-dclass/b-0x27/gap-$1"
"$root/target/release/c2rs" gap \
    --list <repo>/work/dc3-workload/files.txt \
    --flags-file <repo>/work/dc3-workload/flags.txt \
    --cwd <home>/code/milohax/dc3-decomp \
    --jobs 6 --jsonl "$out.jsonl" > "$out.log" 2>&1
tail -40 "$out.log"
