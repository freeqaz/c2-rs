#!/bin/sh
# wstest.sh — the workspace test bar, with the TARGET COUNT printed.
#
# `cargo test --workspace` STOPS at the first failing target, so a truncated run
# reports fewer passes AND fewer targets and reads as a smaller passing run.
# The count of `test result:` lines is the only thing that catches that, so it
# is printed positively rather than inferred from the absence of "FAILED".
#
# $1 = output tag. $2 (optional) = C2RS_SINK_OFF_ADD_ARG value.
set -eu
root=/home/free/code/milohax/c2-rs/.claude/worktrees/agent-a90821e906953b0fd
export C2RS_COMPILERS=/home/free/code/milohax/c2-rs/compilers
export C2RS_WIBO=/home/free/code/milohax/wibo/build/wibo
[ $# -ge 2 ] && export C2RS_SINK_OFF_ADD_ARG="$2"
out="$root/work/w-dclass/b-0x27/test-$1.txt"
cd "$root"
cargo test --workspace --release > "$out" 2>&1 || true
printf 'targets      : %s\n' "$(grep -c '^test result:' "$out")"
printf 'targets ok   : %s\n' "$(grep -c '^test result: ok' "$out")"
printf 'targets FAILED: %s\n' "$(grep -c '^test result: FAILED' "$out")"
printf 'passed       : %s\n' "$(grep '^test result:' "$out" | awk '{s+=$4} END{print s+0}')"
printf 'failed       : %s\n' "$(grep '^test result:' "$out" | awk '{s+=$6} END{print s+0}')"
grep -E '^(error|failures:)' "$out" | head -10 || true
