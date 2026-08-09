#!/bin/sh
# `c2rs diff` on one cell with the lane's toolchain env and a NAMED binary.
#
#     work/w-decouple/diff.sh <base|tip> <cell.cpp> [flags-file]
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
. "$here/env.sh"
which="$1"
src="$2"
if [ $# -ge 3 ]; then
    "$here/c2rs-$which" diff "$src" --flags-file "$3" 2>&1 | tail -2
else
    "$here/c2rs-$which" diff "$src" 2>&1 | tail -2
fi
