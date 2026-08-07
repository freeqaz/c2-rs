#!/bin/sh
# nicmp.sh — is `__declspec(noinline)` READABLE in the IL at all?
#
# Lane w-target. GRID-W cell `w04a` is the chain c2 does NOT close: c2 obeys
# `noinline`, keeps ?f's branch against ?g, and R-CLOSE would rename it to ?ext.
# The rule can only be narrowed if the port can SEE the attribute. So the two
# sources are compiled with the SAME FILENAME LENGTH — the `.gl` embeds the
# source path, and an unmatched pair would show a difference that is the path
# and not the attribute.
#
# Usage:  work/w-target/nicmp.sh
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
for e in ex gl sy in db; do
    a=$(ls work/w-target/il/a/*."$e")
    b=$(ls work/w-target/il/b/*."$e")
    if cmp -s "$a" "$b"; then
        echo ".$e  BYTE-IDENTICAL  ($(stat -c%s "$a") B)"
    else
        n=$(cmp -l "$a" "$b" | wc -l)
        echo ".$e  DIFFERS: $n bytes, sizes $(stat -c%s "$a") / $(stat -c%s "$b")"
    fi
done
