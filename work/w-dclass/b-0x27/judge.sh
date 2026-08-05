#!/bin/sh
# judge.sh — THE SOLE JUDGE, at the WORKLOAD's own flags.
#
# `c2rs diff` hardcodes /Ox and has no --flags-file (boards #194/#195), so it
# CANNOT grade at the /O1 /Oi /EHsc /GR profile the TU-match metric is graded
# against. `c2rs prefilter` reads flags.txt verbatim, runs the port, and
# byte-compares its obj against the REAL c2 obj for the same TU.
#
# Prints one JSON verdict per probe. `port_obj_match: true` is the only green.
set -eu
root=/home/free/code/milohax/c2-rs/.claude/worktrees/agent-a90821e906953b0fd
export C2RS_COMPILERS=/home/free/code/milohax/c2-rs/compilers
export C2RS_WIBO=/home/free/code/milohax/wibo/build/wibo
FL=/home/free/code/milohax/c2-rs/work/dc3-workload/flags.txt
DIR="$root/work/w-dclass/b-0x27/p"
for f in "$@"; do
    printf '########## %s\n' "$f"
    b="$(basename "$f" .cpp)"
    # The REAL c2 obj for the same TU at the same flags, rebuilt every time so
    # the comparison can never be against a stale artifact.
    sh "$root/work/w-dclass/b-0x27/refobj_probe.sh" "$f" >/dev/null
    # S_OBJNAME is baked into .debug$S, so the port must be told the REFERENCE
    # obj's own /Fo path or the compare reports a path divergence as a codegen
    # one. refobj_probe.sh writes `Z:<dir>\<b>.obj`, spelled the same way here.
    zname="Z:$(printf '%s' "$DIR/$b.obj" | tr '/' '\\')"
    "$root/target/release/c2rs" prefilter --source "$f" --flags-file "$FL" \
        --cwd "$DIR" --emit-obj "$DIR/$b.port.obj" --compare-obj "$DIR/$b.obj" \
        --obj-name "$zname" 2>&1
done
