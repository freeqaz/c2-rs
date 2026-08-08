#!/bin/sh
# bytegrade.sh -- GRID L through the SOLE JUDGE.
#
# `c2rs diff` is the project's own differential: it captures the IL, replays it
# through the real c2.dll under wibo, runs the port on the same IL, and compares
# the two objs BYTE FOR BYTE with the COFF TimeDateStamp zeroed.  A cell is
# `Port=Match` (byte-exact), `Port=NotImplemented` (an honest refusal) or
# `Port=Mismatch` -- and a single Mismatch anywhere reverts this whole rung.
set -u
R="$(cd "$(dirname "$0")/../.." && pwd)"
C="$R/target/release/c2rs"; F="$R/work/dc3-workload/flags.txt"
for d in "$R"/work/w-lineage/gridL/*/; do
  n=$(basename "$d")
  s="work/w-lineage/gridL/$n/$n.cpp"
  v=$("$C" diff "$s" --flags-file "$F" 2>&1 \
      | grep -oE 'Port=[A-Za-z]+|ReferenceReplay=[A-Za-z]+' | tr '\n' ' ')
  printf '%s\t%s\n' "$n" "$v"
done
