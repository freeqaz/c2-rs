#!/bin/bash
# spellall.sh — the byte table, printed off the captured streams.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MAIN="$(cd "$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir)/.." && pwd)"
export C2RS_LANEROOT="$MAIN"
cd "$ROOT"
while read -r cell want; do
    echo "## $cell"
    python3 work/w-inread/spell.py "$cell" "$want" < /dev/null
done <<'EOF'
z01_partial_struct ?s@@3
z02_partial_array ?arr@@3
z05_fill_124 ?arr@@3
z04_fill_128 ?arr@@3
z03_fill_252 ?arr@@3
z15_fill_1196 ?arr@@3
z18_fill_7_bytes ?cs@@3
z19_fill_3_bytes ?cs@@3
z23_short_fill ?ss@@3
z24_bool_fill ?s@@3
z17_two_fills ?s@@3
z06_fill_then_ptr ?s@@3
z16_ptr_then_fill ?s@@3
z26_fill_only ?s@@3
z09_null_data_ptr ?s@@3
z10_null_fn_ptr ?s@@3
z11_data_ptr_4 ?s@@3
z12_data_ptr_big ?s@@3
z13_fn_ptr_4 ?s@@3
z22_fn_ptr_big ?s@@3
z20_null_fn_ptr_only ?s@@3
z21_null_data_ptr_only ?s@@3
z25_pmd_shape ?c@@3
z14_typedesc ??_R0
z14_typedesc ??_R2
z07_bases ??_R2
z08_throw _TI1
z08_throw _CT??_R0
EOF
