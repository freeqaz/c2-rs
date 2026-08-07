# Toolchain env for lane w-seed. Source it; do not run it.
#
# `compilers/` is gitignored and does NOT follow a worktree, so a lane that
# forgets this gets `GATE: SKIPPED — NOTHING WAS GRADED` and **exit 0**. Every
# command in this lane runs through this file, and every number is checked
# against a non-zero GRADED count rather than against an exit status.
#
# Nothing absolute is written here (CLAUDE.md: machine paths are never
# committed). A worktree lives at `<main>/.claude/worktrees/<name>`, so the main
# repo is three levels up from it; a plain checkout is its own main repo. Every
# value is `${VAR:-default}` so the caller can override any of them.
_wt=$(cd "$(dirname "$0")/../.." 2>/dev/null && pwd)
case "$_wt" in
    */.claude/worktrees/*) _main=$(cd "$_wt/../../.." && pwd) ;;
    *)                     _main="$_wt" ;;
esac
export C2RS_COMPILERS="${C2RS_COMPILERS:-$_main/compilers}"
export C2RS_WIBO="${C2RS_WIBO:-$_main/../wibo/build/wibo}"
export C2RS_DC3="${C2RS_DC3:-$_main/../dc3-decomp}"
export C2RS_WORKLOAD="${C2RS_WORKLOAD:-$_main/work/dc3-workload}"
unset _wt _main
