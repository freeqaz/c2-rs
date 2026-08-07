#!/bin/sh
# bisect.sh — grade each of the positive fixture's structs ALONE, so a
# `Port=Mismatch` in the whole TU names the body that produced it.
#
# One directory per cell (#1045). The header is repeated verbatim in each cell so
# the only difference between two cells is which constructor is defined.
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
out="$repo_root/work/w-seam2/bisect"
c2rs="$repo_root/target/release/c2rs"
mkdir -p "$out"

for s in C0 C1 C2 C3 W1 W3 W5 U2 H NL WL BW LF; do
    d="$out/$s"
    mkdir -p "$d"
    python3 - "$s" "$d/$s.cpp" <<'PY'
import re, sys
name, dst = sys.argv[1], sys.argv[2]
src = open('fixtures/cpp/w844_store_run_call.cpp').read()
body = src[src.index('struct BE {'):]
# keep the shared declarations, then only the blocks that mention this struct
head = body[:body.index('// ---- the producer-count axis')]
rest = body[body.index('// ---- the producer-count axis'):]
blocks = re.split(r'\n(?=(?:struct|// ---- ))', rest)
keep = [b for b in blocks if re.search(r'\b%s\b' % name, b)]
open(dst, 'w').write(head + '\n'.join(keep))
PY
    "$c2rs" diff "$d/$s.cpp" > "$d/diff.txt" 2>&1 || true
    printf '%-4s %s\n' "$s" \
        "$(grep -oE '(Port=[A-Za-z]+( @ offset [0-9]+)?)' "$d/diff.txt" | head -1)"
done
