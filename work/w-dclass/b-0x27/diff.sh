#!/bin/sh
# diff.sh — the SOLE judge: real c2.dll under wibo vs the port, byte-exact.
# Probe .cpp basenames, at the WORKLOAD's own flags.
set -eu
root=/home/free/code/milohax/c2-rs/.claude/worktrees/agent-a90821e906953b0fd
export C2RS_COMPILERS=/home/free/code/milohax/c2-rs/compilers
export C2RS_WIBO=/home/free/code/milohax/wibo/build/wibo
FL=/home/free/code/milohax/c2-rs/work/dc3-workload/flags.txt
DIR="$root/work/w-dclass/b-0x27/p"
for f in "$@"; do
    printf '########## %s\n' "$f"
    "$root/target/release/c2rs" diff "$f" --flags-file "$FL" --cwd "$DIR" 2>&1 \
        | grep -vE '^  profile:|^  cwd:' || true
done
