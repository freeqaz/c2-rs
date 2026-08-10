#!/bin/sh
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
. "$here/env.sh"
echo "C2RS_COMPILERS=$C2RS_COMPILERS"; ls "$C2RS_COMPILERS"
echo "C2RS_WIBO=$C2RS_WIBO"; ls -la "$C2RS_WIBO"
echo "C2RS_DC3=$C2RS_DC3"; git -C "$C2RS_DC3" rev-parse HEAD
echo "WD_FILES=$WD_FILES  ($(wc -l < "$WD_FILES") lines)"
echo "WD_FLAGS=$WD_FLAGS"; cat "$WD_FLAGS"
echo "dc3 src/ dirty:"; git -C "$C2RS_DC3" status --porcelain -- src | head
echo "dc3 dirty (all tracked):"; git -C "$C2RS_DC3" status --porcelain | head
