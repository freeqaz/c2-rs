#!/bin/sh
# cells.sh — compile every probe cell in `work/w-main2/probe/` at the workload's
# own flags, capture its IL, and print the label table beside the `.gl` seed.
#
# Lane w-main2. Read-only with respect to `crates/`. Re-runs the whole label
# measurement from scratch; nothing here is transcribed by hand.
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"

bin="${1:-./work/w-main2/c2rs-base}"

for f in work/w-main2/probe/*.cpp; do
    n="$(basename "$f" .cpp)"
    sh work/w-main2/refcell.sh "$f" "work/w-main2/obj/$n.obj"
    rm -rf "work/w-main2/il/$n"
    "$bin" capture "$f" --keep-il "work/w-main2/il/$n" \
        --flags-file work/dc3-workload/flags.txt --cwd . >/dev/null 2>&1
    gl="$(ls work/w-main2/il/$n/*.gl)"
    python3 - "$gl" "work/w-main2/obj/$n.obj" <<'PY'
import struct, sys
seed = struct.unpack_from('<I', open(sys.argv[1], 'rb').read(), 7)[0]
open(sys.argv[2].rsplit('.', 1)[0] + '.seed', 'w').write(str(seed))
PY
done

python3 work/w-main2/labels.py work/w-main2/obj/*.obj
