#!/bin/bash
# w-seclayout — for a probe source: obj `.text` COMDATs vs `.gl` framed records
# vs `.ex` segments, at a named flag profile.  The shape being hunted is
#   records == segments  AND  obj `.text` < segments
# i.e. the 1:1 acceptance path would emit a function c2 discarded.
#   probe.sh <file.cpp> <tag> [flags-file]
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
SRC="$1"; TAG="$2"; FLAGS="${3:-}"
mkdir -p work/w-seclayout/probe/out
ARGS=()
[ -n "$FLAGS" ] && ARGS=(--flags-file "$FLAGS")
./work/w-seclayout/c2rs-base compile "$SRC" "${ARGS[@]}" \
    --keep-obj "work/w-seclayout/probe/out/$TAG.obj" >/dev/null 2>&1 || {
        echo "$TAG: compile failed"; exit 0; }
./work/w-seclayout/c2rs-base capture "$SRC" "${ARGS[@]}" \
    --keep-il "work/w-seclayout/probe/out/$TAG" >/dev/null 2>&1 || {
        echo "$TAG: capture failed"; exit 0; }
GL=$(ls "work/w-seclayout/probe/out/$TAG"/*.gl)
EX=$(ls "work/w-seclayout/probe/out/$TAG"/*.ex)
echo "### $TAG  ($SRC${FLAGS:+, $FLAGS})"
python3 work/w-seclayout/glwalk26.py "$GL" "$EX" \
    --tsv "work/w-seclayout/probe/out/$TAG/walk.tsv" \
    | grep -E "framed defined records|segments|clause 3/4|STOPS AT"
python3 work/w-seclayout/seclayout.py "work/w-seclayout/probe/out/$TAG.obj" \
    | grep -E "\.text:|aux Selection|section names"
