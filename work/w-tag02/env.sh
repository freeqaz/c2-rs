#!/bin/bash
# w-tag02 — toolchain env for a WORKTREE.
#
# A worktree sits three directories below the main repo, so `<repo>/compilers`
# and the `../wibo` sibling do not resolve from one. Both are pointed at the
# main repo's copies here; nothing absolute is committed in `crates/`.
#
# Source it: `. work/w-tag02/env.sh`
MAIN="${C2RS_MAIN_REPO:-$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --path-format=absolute --git-common-dir)/..}"
MAIN="$(cd "$MAIN" && pwd)"
export C2RS_COMPILERS="$MAIN/compilers"
export C2RS_WIBO="$(cd "$MAIN/../wibo" && pwd)/build/release/wibo"
export C2RS_DC3="$(cd "$MAIN/../dc3-decomp" && pwd)"
