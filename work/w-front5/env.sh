# Lane w-front5 toolchain environment.
#
# A worktree under `.claude/worktrees/` has no `./compilers` and no `../wibo`
# sibling, so `Toolchain::locate`'s relative-to-repo-root defaults miss both.
# Everything below is DERIVED from the main checkout git already knows about —
# no absolute machine path is written down (CLAUDE.md: toolchain location is
# env-driven by design, and three lanes in a row left `/home/<user>/…` in a
# committed file this week).
#
#     . work/w-front5/env.sh
#
# `--git-common-dir` is the MAIN checkout's `.git`, whatever worktree we are in.
_w5_main="$(cd "$(git rev-parse --git-common-dir)/.." && pwd)"
export C2RS_COMPILERS="${C2RS_COMPILERS:-$_w5_main/compilers}"
export C2RS_WIBO="${C2RS_WIBO:-$_w5_main/../wibo/build/release/wibo}"
export C2RS_DC3="${C2RS_DC3:-$_w5_main/../dc3-decomp}"
unset _w5_main
