# Lane w-xtea3 toolchain environment.
#
# A worktree under `.claude/worktrees/` has no `./compilers` and no `../wibo`
# sibling, so `Toolchain::locate`'s relative-to-repo-root defaults miss both.
# Everything below is DERIVED from the main checkout git already knows about —
# no absolute machine path is written down (CLAUDE.md: toolchain location is
# env-driven by design).  Copied from `work/w-front5/env.sh`, which did it right.
#
#     . work/w-xtea3/env.sh
#
# `--git-common-dir` is the MAIN checkout's `.git`, whatever worktree we are in.
_wx3_main="$(cd "$(git rev-parse --git-common-dir)/.." && pwd)"
export C2RS_COMPILERS="${C2RS_COMPILERS:-$_wx3_main/compilers}"
export C2RS_WIBO="${C2RS_WIBO:-$_wx3_main/../wibo/build/release/wibo}"
export C2RS_DC3="${C2RS_DC3:-$_wx3_main/../dc3-decomp}"
unset _wx3_main
