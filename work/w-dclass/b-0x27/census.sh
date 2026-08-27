#!/bin/sh
# census.sh — census one or more probe .cpp at the WORKLOAD's own flags.
# Lane w-dclass / subagent B (0x27). Read-only wrt crates/.
set -eu
root=<repo>/.claude/worktrees/agent-a90821e906953b0fd
export C2RS_COMPILERS=<repo>/compilers
export C2RS_WIBO=<home>/code/milohax/wibo/build/wibo
FL=<repo>/work/dc3-workload/flags.txt
# A probe .cpp has no #include, so the workload's /I set is inert for it and the
# cwd may be the probe dir. The CODEGEN flags (/O1 /Oi /EHsc /GR) are read from
# flags.txt verbatim either way — boards #194/#195.
DIR="$root/work/w-dclass/b-0x27/p"
for f in "$@"; do
    printf '########## %s\n' "$f"
    "$root/target/release/c2rs" census "$f" --flags-file "$FL" --cwd "$DIR" 2>&1 \
        | grep -E 'functions in class|^  \[|blocking features|^ +[0-9]+ x |^ +[0-9a-f]{2} |^ +>' || true
done
