#!/bin/sh
# Regenerate every artifact under docs/whitebox/grids/w-sizetest/ from the
# pinned image + the flat Ghidra export.  Bind every published count to a
# recipe: this file IS the recipe (WAVE21_BRIEF §5).
#
# Inputs (neither committed):
#   compilers/X360/16.00.11886.00/c2.dll   sha256 c80981c0…a66258
#   ~/ghidra-projects/export/c2/objdump_intel.asm   (C2_MAP_METHOD.md §2–§4)
#
# Run from the repo root:  sh work/w-sizetest/regen.sh
set -eu

D=docs/whitebox/grids/w-sizetest
W=work/w-sizetest
mkdir -p "$D"

# The candidacy function, end to end: 0x10b5fb5f + 377 = 0x10b5fcd8 (ADDR.tsv).
python3 "$W/listing.py" 10b5fb5f 10b5fcd8            > "$D/FUN_10b5fb5f.asm"
python3 "$W/cfg.py"     10b5fb5f 10b5fcd8            > "$D/cfg_FUN_10b5fb5f.txt"
python3 "$W/cfg.py"     10b5fb5f 10b5fcd8 --writes edi > "$D/edi_writes.txt"
python3 "$W/cfg.py"     10b5fb5f 10b5fcd8 --dom 10b5fc95 > "$D/dom_10b5fc95.txt"

# The two gate globals, the ceiling, and the ceiling's parameter k.
python3 "$W/globrefs.py" 10c2e2fc 10c46318 10c2e310 10c2ea98 > "$D/globrefs.out"
python3 "$W/readva.py"   10c2ea98 10c46318 10c2e2fc 10c2e310 \
                         10c3de20 10c6f1c8 10c2eaac         > "$D/readva.out"

# The ceiling's sole writer pair, and the option-word decode that feeds both
# gates.  FUN_10b5e4cc = 101 B; FUN_10b82338 = 374 B (FUNCS.tsv).
python3 "$W/listing.py" 10b5e4cc 10b5e531             > "$D/ceiling_writer.asm"
python3 "$W/listing.py" 10b82338 10b823f0             > "$D/optword_decode.asm"

# The three call sites of candidacy, with the 14 instructions before each —
# the evidence that no caller writes edi.
python3 "$W/callsites.py" 10b5fb5f                    > "$D/callsites.txt"

# The exhaustive ceiling-vs-ladder check.
python3 "$W/ceiling_range.py"                         > "$D/ceiling_range.out"

ls -l "$D"
