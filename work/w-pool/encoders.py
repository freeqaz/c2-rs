#!/usr/bin/env python3
"""encoders.py — does the port already have an encoder for every instruction a
reference obj contains?

    python3 work/w-pool/encoders.py work/w-pool/ref/Pool.obj [more.obj ...]

The question a price needs and no existing instrument answers.  `fnbyte-*` says
whether a body came out right; the census says why the READER refused; neither
says whether the *writer* owns the vocabulary, and a lane that priced a TU at
"three lowerings" without checking would be pricing a fourth thing it never
named.

Method, and its two error directions, both stated because the number is quoted:

  * the mnemonics come from `scripts/gt_dump.py`'s own decoder, so this measures
    what that decoder prints, not what the ISA contains.  A mnemonic it renders
    as an extended form (`rotlwi` for `rlwinm`, `li` for `addi`, `clrldi` for
    `rldicl`) is mapped back through ALIASES below, by hand;
  * the encoder side is a grep of `pub fn encode_<name>` in
    `crates/c2-core/src/codegen/encode.rs`.  An encoder that exists but is
    wrong-arity for this use still counts as PRESENT here — this is a
    *vocabulary* check and deliberately the optimistic direction, so a MISSING
    row is a hard fact and a PRESENT row is a floor.
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

# gt_dump's extended mnemonic -> the base form encode.rs names it by.
ALIASES = {
    "rotlwi": "rlwinm", "srwi": "rlwinm", "slwi": "rlwinm", "clrlwi": "rlwinm",
    "clrldi": "rldicl", "li": "addi", "lis": "addis", "mr": "or", "sub": "subf",
    "nop": "ori", "blt": "bc", "bge": "bc", "beq": "bc", "bne": "bc",
    "bf": "bc", "bt": "bc", "b": "b_intra", "bdnz": "bdnz", "bclr": "bclr",
    "blr": "blr", "mtctr": "mtctr", "cmpwi": "cmpwi", "cmplwi": "cmplwi",
}
# `mr` is its own encoder too; prefer the direct name when it exists.
DIRECT = {"mr", "li", "sub", "b_intra"}

# **Emitted, but NOT from `encode.rs`.**  Counting these as missing would
# overstate the price by two on any framed TU, and the first run of this script
# did exactly that -- recorded here rather than silently corrected, because the
# correction is the interesting half: the port's instruction vocabulary is not
# all in one file.
#   `mflr`  -- `codegen::frame`'s Class A / Class C prologues (frame.rs:346,430)
#   `bl`    -- `codegen::calls::encode_call_branch` (a REL24 site, so its word is
#              a placeholder the writer relocates, which is why it is not a
#              plain encoder)
#   `b`     -- likewise `codegen::calls::encode_tail_branch` for the INTER-section
#              form; `encode.rs`'s `encode_b_intra` is the intra-section one.
ELSEWHERE = {
    "mflr": "codegen::frame (Class A/C prologue)",
    "bl": "codegen::calls::encode_call_branch (REL24)",
}
# gt_dump renders `rldicl ra,rs,0,n` as the extended `clrldi`; they are one
# instruction and must not be counted twice.
ALIASES["clrldi"] = "rldicl"


def encoders():
    src = (ROOT / "crates/c2-core/src/codegen/encode.rs").read_text()
    return {m.group(1) for m in re.finditer(r"pub fn encode_([a-z0-9_]+)", src)}


def mnemonics(obj):
    out = subprocess.run(
        [sys.executable, str(ROOT / "scripts/gt_dump.py"), obj],
        capture_output=True, text=True, check=True).stdout
    ms = []
    for line in out.splitlines():
        m = re.match(r"^\s{2,}[0-9a-f]{4}\s+[0-9a-f]{8}\s+([a-z][a-z0-9_.]*)", line)
        if m:
            ms.append(m.group(1))
    return ms


def main(objs):
    have = encoders()
    rc = 0
    for obj in objs:
        ms = mnemonics(obj)
        seen, missing = {}, []
        for m in ms:
            seen[m] = seen.get(m, 0) + 1
        print(f"== {obj}  {len(ms)} instructions, {len(seen)} distinct")
        for m in sorted(seen):
            base = m.rstrip(".")
            rec = m.endswith(".")
            cands = [base]
            if base in ALIASES:
                cands.append(ALIASES[base])
            if base in DIRECT:
                cands.insert(0, base)
            if rec:
                cands = [c + "_record" for c in cands] + cands
            ok = next((c for c in cands if c in have), None)
            if ok:
                mark = f"encode_{ok}"
            elif base in ELSEWHERE:
                mark = f"[elsewhere] {ELSEWHERE[base]}"
            else:
                mark = "*** MISSING ***"
                missing.append(m)
                rc = 1
            print(f"   {seen[m]:3d} x {m:<10s} {mark}")
        # Distinct INSTRUCTIONS, not distinct spellings: two extended mnemonics
        # of one instruction are one thing to build.
        base_missing = sorted({ALIASES.get(m.rstrip("."), m.rstrip(".")) for m in missing})
        print(f"   -> {len(base_missing)} MISSING (distinct instructions): "
              f"{base_missing or 'none'}   [spellings: {missing or 'none'}]")
    return rc


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
