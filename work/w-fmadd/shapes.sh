#!/bin/sh
# lane w-fmadd — grade one-function TUs against real c2, one line each.
# usage: sh work/w-fmadd/shapes.sh <file-with-one-C++-expression-per-line>
set -e
cd "$(dirname "$0")/../.."
d=work/w-fmadd/probe/shapes
mkdir -p "$d"
i=0
while IFS= read -r line; do
  [ -z "$line" ] && continue
  case "$line" in \#*) continue ;; esac
  i=$((i+1))
  printf '%s\n' "$line" > "$d/s$i.cpp"
  v=$(cargo run -q -p c2-harness --bin c2rs -- diff "$d/s$i.cpp" 2>&1 | sed 's/.*Port=//; s/ .*//')
  printf '%-12s %s\n' "$v" "$line"
done < "$1"
