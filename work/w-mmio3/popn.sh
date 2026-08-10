#!/bin/sh
# w-mmio3 — CEILING.md §11.4 item 5's SECOND half: grep the WORKLOAD for the
# construct before pricing the class.
#
# The construct is an INDIRECT CALL THROUGH A LOADED MEMBER FUNCTION POINTER —
# `info->pIOProc(info, 4, fuClose, 0)` — which is mechanism 2 of the six and the
# only one of them that could plausibly have a population beyond this TU.
#
# Instrument, stated with its bound: every `(*name)(` function-pointer member
# declared in a workload HEADER is collected, then every workload `.cpp` that
# spells `->name(` for one of those names is listed. It CANNOT see a member
# whose function-pointer type is spelled through a typedef with no `(*` in the
# header, and it CANNOT tell a call from a mere reference. It over-counts on the
# second axis and under-counts on the first, so the number it prints is neither
# a bound nor an estimate — it is a list to read.
set -eu
dc3="${C2RS_DC3:?set C2RS_DC3 to the dc3-decomp checkout}"
out="$(cd "$(dirname "$0")" && pwd)"

grep -rhoE '\(\s*\*\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)\s*\(' \
    --include='*.h' --include='*.hpp' "$dc3/src" 2>/dev/null \
  | grep -oE '\*\s*[A-Za-z_][A-Za-z0-9_]*' \
  | sed 's/\*[[:space:]]*//' | sort -u > "$out/popn_fpnames.txt"

pat=$(sed 's/^/->[[:space:]]*/; s/$/[[:space:]]*(/' "$out/popn_fpnames.txt" | paste -sd'|')
( cd "$dc3" && grep -rlE "$pat" --include='*.cpp' src 2>/dev/null | sort ) > "$out/popn.txt" || true

echo "function-pointer member names in headers: $(wc -l < "$out/popn_fpnames.txt")"
echo "workload .cpp spelling '->name(' for one of them:"
sed 's/^/  /' "$out/popn.txt"

# THE INSTRUMENT'S OWN FALSE NEGATIVE, RUN AS A CONTROL. `mmio.cpp` is the TU
# this lane converts and it is NOT in the list above: `pIOProc` is declared
# `LPMMIOPROC pIOProc;` — a typedef with no `(*` at the member — so the first
# grep never sees the name. A population instrument that cannot see its own
# known positive reports absence as evidence of absence, which is the failure
# `docs/GAPS.md` and `CEILING.md` §11.4 item 5 both name. Printed here so the
# zero above is never read as a bound.
echo
echo "CONTROL — the known positive, which the instrument above MISSES:"
( cd "$dc3" && grep -rlE '[A-Za-z_][A-Za-z0-9_]*->p[A-Z][A-Za-z0-9_]*[[:space:]]*\(' \
    --include='*.cpp' src 2>/dev/null | sort ) | sed 's/^/  /'
