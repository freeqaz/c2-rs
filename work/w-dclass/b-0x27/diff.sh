#!/bin/sh
# diff.sh — DELIBERATELY NOT THE JUDGE. Use `judge.sh` / `judge_sink.sh`.
#
# This lane tried `c2rs diff --flags-file …` first and got
# `diff: unknown option: --flags-file`. That is not a typo — `DIFF_SPEC` is
# literally `Spec::new("diff", &[])` (`crates/c2-harness/src/cli/reference.rs`),
# so `diff` takes NO options at all and compiles at its hardcoded `/Ox /GS- /c`.
# Boards #194/#195: an obj built at `/Ox` is not the obj the TU-match metric is
# graded against, so every number read off it is about a different compilation.
#
# `c2rs prefilter` is the one CLI arm that reads `--flags-file` AND emits an obj
# AND byte-compares it, which is why `judge.sh` uses it. Kept as a signpost so
# the next lane does not spend the same twenty minutes.
#
# `diff` IS still the right tool at its own flags, e.g. for the fixtures.
set -eu
root=<repo>/.claude/worktrees/agent-a90821e906953b0fd
export C2RS_COMPILERS=<repo>/compilers
export C2RS_WIBO=<home>/code/milohax/wibo/build/wibo
DIR="$root/work/w-dclass/b-0x27/p"
echo "NOTE: c2rs diff has no --flags-file and hardcodes /Ox (#194/#195)." >&2
echo "      For the WORKLOAD profile use judge.sh / judge_sink.sh." >&2
for f in "$@"; do
    printf '########## %s (at /Ox, NOT the workload profile)\n' "$f"
    (cd "$DIR" && "$root/target/release/c2rs" diff "$f" 2>&1)
done
