# Lane w-selbind toolchain environment.
#
# A worktree under `.claude/worktrees/` has no `./compilers`, no `../wibo`
# sibling and no `work/dc3-workload/` (untracked, lives in the MAIN checkout).
# Everything below is DERIVED from the main checkout git already knows about —
# no absolute machine path is written down (CLAUDE.md: toolchain location is
# env-driven by design, and three lanes in a row left `/home/<user>/...` in a
# committed file).
#
#     . work/w-selbind/env.sh
#
# `--git-common-dir` is the MAIN checkout's `.git`, whatever worktree we are in.
_wsb_main="$(cd "$(git rev-parse --git-common-dir)/.." && pwd)"
export C2RS_COMPILERS="${C2RS_COMPILERS:-$_wsb_main/compilers}"
export C2RS_WIBO="${C2RS_WIBO:-$_wsb_main/../wibo/build/release/wibo}"
export C2RS_DC3="${C2RS_DC3:-$_wsb_main/../dc3-decomp}"
# The workload list and flags, USED AS COMMITTED-BY-USE and never regenerated
# (#2700: the generator's include mapping is broken against today's dc3 and
# yields capture-fail 851/878). They live untracked in the main checkout.
export WD_FILES="${WD_FILES:-$_wsb_main/work/dc3-workload/files.txt}"
export WD_FLAGS="${WD_FLAGS:-$_wsb_main/work/dc3-workload/flags.txt}"
unset _wsb_main
