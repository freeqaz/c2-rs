#!/bin/sh
# w-mmioclose — run a command with this worktree's toolchain overrides in the
# environment.  `sh work/w-mmioclose/run.sh <cmd> [args...]`
#
# The worktree's own root has no ./compilers and no sibling ../wibo, so the
# documented C2RS_* overrides point at the main checkout's copies.  Nothing
# absolute is baked in: every path derives from this file's own location.
set -eu
here=$(cd "$(dirname "$0")/../.." && pwd)   # the worktree root
main=$(cd "$here/../../.." && pwd)          # the main c2-rs checkout
C2RS_COMPILERS="$main/compilers";           export C2RS_COMPILERS
C2RS_WIBO="$main/../wibo/build/release/wibo";  export C2RS_WIBO
C2RS_WIBO_DEBUG="$main/../wibo/build/debug/wibo"; export C2RS_WIBO_DEBUG
C2RS_DC3="$main/../dc3-decomp";             export C2RS_DC3
exec "$@"
