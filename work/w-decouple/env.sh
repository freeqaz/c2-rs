# Lane w-decouple toolchain environment.
#
# A worktree under `.claude/worktrees/` has no `./compilers`, no `../wibo`
# sibling and no `work/dc3-workload/` (that directory is untracked and lives in
# the MAIN checkout). Everything below is DERIVED from the main checkout git
# already knows about — no absolute machine path is written down (CLAUDE.md:
# toolchain location is env-driven by design, and three lanes in a row left
# `/home/<user>/...` in a committed file this week).
#
#     . work/w-decouple/env.sh
#
# `--git-common-dir` is the MAIN checkout's `.git`, whatever worktree we are in.
_wd_main="$(cd "$(git rev-parse --git-common-dir)/.." && pwd)"
export C2RS_COMPILERS="${C2RS_COMPILERS:-$_wd_main/compilers}"
export C2RS_WIBO="${C2RS_WIBO:-$_wd_main/../wibo/build/release/wibo}"
export C2RS_DC3="${C2RS_DC3:-$_wd_main/../dc3-decomp}"
# The workload list and flags, USED AS COMMITTED-BY-USE and never regenerated
# (#2700: the generator's include mapping is broken against today's dc3 and
# yields capture-fail 851/878). They live untracked in the main checkout.
export WD_FILES="${WD_FILES:-$_wd_main/work/dc3-workload/files.txt}"
export WD_FLAGS="${WD_FLAGS:-$_wd_main/work/dc3-workload/flags.txt}"
unset _wd_main
