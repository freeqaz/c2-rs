#!/bin/sh
# judge_sink.sh — judge.sh with the off-add sink PROMOTED. Same sole judge:
# real c2.dll under wibo, byte-exact, at the workload's own flags.
set -eu
root=<repo>/.claude/worktrees/agent-a90821e906953b0fd
export C2RS_SINK_OFF_ADD_ARG=expr
exec sh "$root/work/w-dclass/b-0x27/judge.sh" "$@"
