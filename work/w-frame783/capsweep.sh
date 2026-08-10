#!/bin/bash
# w-frame783 — capture a list of workload TUs' IL into per-TU dirs, keeping only
# `.gl` and `.ex` (the two streams the framing question needs). Scratch only;
# nothing here is ever committed (`_CL_*` / `*.il` are gitignored by rule).
#
#   capsweep.sh <list-file> <out-dir> [jobs]
set -uo pipefail
LIST="$1"; OUT="$2"; JOBS="${3:-24}"
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
DC3="${C2RS_DC3:-$REPO/../../../dc3-decomp}"
DC3="$(cd "$DC3" && pwd)"
mkdir -p "$OUT"

one() {
    local src="$1"
    local key; key="$(echo -n "$src" | tr '/.' '__')"
    local d="$OUT/$key"
    [ -f "$d/.done" ] && return 0
    mkdir -p "$d"
    if "$REPO/target/release/c2rs" capture "$src" --keep-il "$d" \
         --flags-file "$REPO/work/dc3-workload/flags.txt" --cwd "$DC3" \
         > "$d/cap.log" 2>&1; then
        rm -f "$d"/*.db "$d"/*.sy "$d"/*.in
        echo "$src" > "$d/.done"
    else
        echo "CAPFAIL $src"
    fi
}
export -f one
export OUT REPO DC3

xargs -a "$LIST" -P "$JOBS" -I{} bash -c 'one "$@"' _ {}
echo "captured: $(find "$OUT" -name .done | wc -l) of $(wc -l < "$LIST")"
