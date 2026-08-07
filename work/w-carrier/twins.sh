#!/bin/sh
# twins.sh — cross GRID K's PAIRED cells against each other in EMITTED BYTES.
#
# Board #1174: 1,576 generated cases were at 0 mismatch through two wrong emits,
# and what caught them was a hand-written cross-product, not the corpus. This is
# that cross-product for board #1199: every `_c` control is the same body with
# the bind removed, and the pair is compared on real `c2.dll`'s own `.text`
# words rather than on a verdict label.
#
# A pair that is TEXT IDENTICAL is a cell where the bind makes no body — the
# zero-offset and dead cases — and a pair that DIFFERS is a cell where a reader
# collapsing the two spellings would emit the other one's words (#1128/#232).
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
grid="$repo_root/work/w-carrier/grid"

txt() { grep -E '^   [0-9a-f]{4}  ' "$grid/$1/dis.txt" | sed 's/  *$//'; }

pairs="$(cd "$grid" && ls -d *_c 2>/dev/null | sed 's/_c$//') k_target"

for a in $pairs; do
    case "$a" in
        k_target) b=k_target_direct ;;
        *)        b="${a}_c" ;;
    esac
    printf '== %-16s vs %-18s ' "$a" "$b"
    if [ ! -f "$grid/$a/dis.txt" ] || [ ! -f "$grid/$b/dis.txt" ]; then
        echo "NO DISASM"
        continue
    fi
    txt "$a" > "$grid/$a/.text.tmp"
    txt "$b" > "$grid/$b/.text.tmp"
    if diff "$grid/$a/.text.tmp" "$grid/$b/.text.tmp" > /dev/null; then
        echo "TEXT IDENTICAL"
    else
        n=$(diff "$grid/$a/.text.tmp" "$grid/$b/.text.tmp" | grep -c '^[<>]' || true)
        echo "TEXT DIFFERS — $n lines"
        diff "$grid/$a/.text.tmp" "$grid/$b/.text.tmp" | sed 's/^/     /'
    fi
done
