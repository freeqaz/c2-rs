#!/bin/bash
# w-vocab burner sweep: capture each cell, print N and the `.gl` field of the
# trailing `?f@@YAHH@Z` record under BOTH framings.
#   usage: sweep.sh <cell-dir> <out-dir> [extra c2rs capture args...]
set -uo pipefail
cd "$(dirname "$0")/../.."
cells="$1"; outd="$2"; shift 2
mkdir -p "$outd"
for f in "$cells"/*.cpp; do
    n="$(basename "$f" .cpp)"
    d="$outd/$n"; mkdir -p "$d"
    cargo run --release -q -p c2-harness --bin c2rs -- capture "$f" --keep-il "$d" "$@" >/dev/null 2>&1
    gl="$(ls "$d"/*.gl 2>/dev/null | head -1)"
    if [ -z "$gl" ]; then echo "$n CAPTURE-FAIL"; continue; fi
    wide="$(python3 work/w-vocab/glframe.py "$gl"        | grep -c '?f@@YAHH@Z' || true)"
    gate="$(python3 work/w-vocab/glframe.py "$gl" --gate | grep -c '?f@@YAHH@Z' || true)"
    fld="$(python3 work/w-vocab/glframe.py "$gl" | awk '/\?f@@YAHH@Z/{print $1}' | head -1)"
    nrec_w="$(python3 work/w-vocab/glframe.py "$gl"        | head -1 | awk '{print $1}')"
    nrec_g="$(python3 work/w-vocab/glframe.py "$gl" --gate | head -1 | awk '{print $1}')"
    echo "$n $fld wide=$wide gate=$gate records_wide=$nrec_w records_gate=$nrec_g"
done
