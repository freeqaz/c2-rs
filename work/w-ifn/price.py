#!/usr/bin/env python3
"""price.py — SCRIPT-COUNT the port's distance from one reference body.

The commission requires the decline to be *script-counted* and not
hand-partitioned, because eight lanes have now hand-counted this TU and no two
used the same unit. This counts two things that a script can count without
judgement:

  1. **encoders** — every distinct instruction MNEMONIC in the body, and
     whether `crates/c2-core/src/codegen/encode.rs` defines an `encode_<m>` for
     it. An absent encoder is a word the port cannot write at all.
  2. **statement forms** — every distinct top-level statement head in the
     body's `.ex` segment, and whether the shipped reader
     (`guard_ret_chain.rs`) has a clause that consumes it.

Neither number is "the price". They are the two components of it a script can
produce; the mechanisms that are neither an encoder nor a statement form (an
interprocedural fact, an acceptance seam) are named in `MMIO_PRICE.md` and
counted there by hand, which is stated rather than hidden.

Usage:  price.py <dis.txt> <function-name>
"""
import re
import os
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
# `encode.rs` is where an instruction encoder lives, with one documented
# exception: the call branch encodes its own `.text` offset and therefore sits
# beside the call lowering in `calls.rs`. Both are read, so "MISSING" means the
# port cannot write the word ANYWHERE and not merely "not in this file".
ENC = [os.path.join(ROOT, "crates", "c2-core", "src", "codegen", "encode.rs"),
       os.path.join(ROOT, "crates", "c2-core", "src", "codegen", "calls.rs")]

dis, want = sys.argv[1], sys.argv[2]
enc_src = "".join(open(p, encoding="utf-8").read() for p in ENC)
have = set(re.findall(r"fn encode_([a-z0-9_]+)\(", enc_src))

body, seen = [], False
for line in open(dis, encoding="utf-8"):
    m = re.match(r"-- \.text #\d+ \(\d+ B\) (\S+)", line)
    if m:
        seen = m.group(1) == want
        continue
    if seen:
        m = re.match(r"\s+[0-9a-f]{4}\s+[0-9a-f]{8}\s+(\S+)", line)
        if m:
            body.append(m.group(1).rstrip(","))

mnemonics = []
for m in body:
    if m not in mnemonics:
        mnemonics.append(m)

print(f"{want}: {len(body)} words, {len(mnemonics)} distinct mnemonics")
missing = []
for m in mnemonics:
    # `bf`/`bt` are the extended forms of `bc`; `b` of `b_intra`; `li` of `addi`.
    alias = {"bf": "bc", "bt": "bc", "b": "b_intra", "li": "addi",
             "bl": "call_branch",
             "mflr": "PROLOGUE", "mtlr": "PROLOGUE"}.get(m, m)
    ok = alias == "PROLOGUE" or alias in have
    print(f"  {m:8s} -> encode_{alias:10s} {'HAVE' if ok else 'MISSING'}")
    if not ok:
        missing.append(m)
print(f"MISSING ENCODERS: {len(missing)} {missing}")
