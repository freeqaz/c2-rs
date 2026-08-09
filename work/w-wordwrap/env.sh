# Lane w-wordwrap toolchain environment.
#
# A worktree under `.claude/worktrees/` has no `./compilers` and no `../wibo`
# sibling, so `Toolchain::locate`'s relative-to-repo-root defaults miss both.
# Everything below is DERIVED from the main checkout git already knows about —
# no absolute machine path is written down (CLAUDE.md: toolchain location is
# env-driven by design).  Copied from `work/w-front5/env.sh`, which did it right.
#
#     . work/w-wordwrap/env.sh
#
# `--git-common-dir` is the MAIN checkout's `.git`, whatever worktree we are in.
# `C2RS_MAIN` is exported as well because `work/dc3-workload/` (the committed
# workload list and its MAPPED flags — #2700 forbids regenerating them) and
# `work/capture-cache/` live in the MAIN checkout only; they are gitignored and
# therefore absent from every worktree.
_ww_main="$(cd "$(git rev-parse --git-common-dir)/.." && pwd)"
export C2RS_MAIN="${C2RS_MAIN:-$_ww_main}"
export C2RS_COMPILERS="${C2RS_COMPILERS:-$_ww_main/compilers}"
export C2RS_WIBO="${C2RS_WIBO:-$_ww_main/../wibo/build/release/wibo}"
export C2RS_DC3="${C2RS_DC3:-$_ww_main/../dc3-decomp}"
unset _ww_main
