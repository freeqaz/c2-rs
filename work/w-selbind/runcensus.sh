#!/bin/sh
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
for t in "$@"; do
    g=$(ls "$here/il/$t"/*.gl)
    e=$(ls "$here/il/$t"/*.ex)
    echo "### $t"
    python3 "$here/runcensus.py" "$g" "$e"
    echo
done
