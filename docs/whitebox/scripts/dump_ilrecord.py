#!/usr/bin/env python3
"""Dump c2.dll's IL-record -> codegen dispatch `FUN_10bc2d7a` (read R5).

Whitebox tooling (outside the std-only `crates/` workspace, per CLAUDE.md).
Reads the pinned image directly.  Companion to `dump_opcode_tables.py`, whose
`Image` (PE VA->offset) class this reuses rather than re-deriving.

    0x10bc2d7a   the dispatch body, 5,080 B, ending exactly at the jump table
    0x10bc2e20   the `jmp dword ptr [eax*4 + 0x10bc4152]` dispatch site
    0x10bc424a   the BYTE index table, 189 entries, opcodes 0x01..0xBD
    0x10bc4152   the DWORD target table, 62 entries -> 62 distinct arms
    0x10b25e48   the operand-class table (read R2 era, board #1591) -- joined
    0x10b25f10   the per-opcode u16 attribute table (board #1591) -- joined

**The "189 arms" in READ_PLAN_2026-08-21.md §3, C2_MAP.md:1012 and
STEP5_PRICING_2026-08-21.md:139 is an OPCODE count, not an arm count.**  The
switch is MSVC's two-level form: a 189-entry byte table maps opcode-1 to an
arm index in 0..61, and a 62-entry DWORD table holds the arm addresses.  See
`ref/P_ILRECORD.md` §1 and `WB_ILRECORD_FINDINGS.md` P1.2.

Usage:
    python3 docs/whitebox/scripts/dump_ilrecord.py <c2.dll> --tables
    python3 docs/whitebox/scripts/dump_ilrecord.py <c2.dll> --arms
    python3 docs/whitebox/scripts/dump_ilrecord.py <c2.dll> --tsv
    python3 docs/whitebox/scripts/dump_ilrecord.py <c2.dll> --disasm <armidx|VA>
    python3 docs/whitebox/scripts/dump_ilrecord.py <c2.dll> --sample N [--seed S]

`--sample` draws stratum S-C of `WB_ILRECORD_PREREG.md` §P5: a uniform random
sample of the arms NOT reached by the ten residue constructs, seed 20260823.
It is a pure function of (seed, N) so the draw is reproducible from this file.

The image this record is written against is
sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
(C2_MAP_METHOD.md §0); the script verifies the digest and refuses otherwise.
"""

import random
import struct
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dump_opcode_tables import Image, PINNED_SHA256          # noqa: E402

try:
    import capstone
except ImportError:                                          # pragma: no cover
    capstone = None

BODY_VA = 0x10BC2D7A
BODY_LEN = 5080
JUMP_TABLE_VA = 0x10BC4152          # 62 DWORD arm targets
JUMP_TABLE_LEN = 62
BYTE_TABLE_VA = 0x10BC424A          # 189 byte arm-indices, opcode 0x01..0xBD
BYTE_TABLE_LEN = 189
FIRST_OPCODE = 0x01
LAST_OPCODE = 0xBD
DISPATCH_VA = 0x10BC2E20
OUT_OF_RANGE_ARM = 0x10BC4143       # the `ja` target: opcode outside 0x01..0xBD

CLASS_TABLE_VA = 0x10B25E48         # operand-format class, board #1591
ATTR_TABLE_VA = 0x10B25F10          # per-opcode u16 attributes, board #1591

# The ten residue constructs, ROADMAP_SLICING_2026-08-21.md:162-169,277-280.
# Only the opcodes the record states explicitly are listed; C4..C10 have no
# published opcode set, which is itself reported (findings §P4.3).
CONSTRUCT_OPCODES = {
    "C1 off-add": [0x27],
    "C2 intrinsic": [0x40],
    "C3 bind": [0x99, 0x9A, 0x9B],
}

# .data is where mutable globals live; read-only tables sit inside .text.
# Keeping the two apart is the whole point of the context-dependence question
# (`ref/P_ILRECORD.md` §4): a table indexed by the opcode is a constant map, a
# .data word is compiler state.
DATA_LO = 0x10C2E000
DATA_HI = 0x10C70750


def load(path):
    img = Image(path)
    if img.digest != PINNED_SHA256:
        sys.exit("image digest %s != pinned %s" % (img.digest, PINNED_SHA256))
    return img


def tables(img):
    """(targets[62], byte_index[189], opcodes_by_arm{target: [opcodes]})."""
    targets = [img.u32(JUMP_TABLE_VA + 4 * i) for i in range(JUMP_TABLE_LEN)]
    o = img.off(BYTE_TABLE_VA)
    idx = list(img.blob[o:o + BYTE_TABLE_LEN])
    by_arm = {}
    for i, ai in enumerate(idx):
        by_arm.setdefault(targets[ai], []).append(FIRST_OPCODE + i)
    return targets, idx, by_arm


def arm_spans(targets):
    """Linear span per arm: [target, next distinct target), clipped to body.

    Stated as a definition, not a discovery: MSVC lays switch arms out
    contiguously, so the span is the arm's own code plus any fallthrough tail
    it shares.  An arm that jumps away immediately has a span longer than its
    semantic body; `--disasm` shows the instructions so a reader can see which.
    """
    end = BODY_VA + BODY_LEN
    pts = sorted(set(targets))
    spans = {}
    for i, t in enumerate(pts):
        spans[t] = (t, pts[i + 1] if i + 1 < len(pts) else end)
    return spans


def _md():
    if capstone is None:
        sys.exit("capstone required for --arms/--disasm/--tsv")
    md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
    md.detail = True
    return md


def analyse_arm(img, md, lo, hi):
    """Static facts about one arm's linear span.

    Returns a dict.  Everything here is a COUNT or a SET -- no classification.
    The DECODE/SELECT judgement is made by a human against the rule fixed in
    `WB_ILRECORD_PREREG.md` §P2 and recorded in `ref/P_ILRECORD.md`, never by
    this script.  Keeping the instrument judgement-free is deliberate: the
    classification is the deliverable and must stay auditable.
    """
    o = img.off(lo)
    code = img.blob[o:o + (hi - lo)]
    ins_list = list(md.disasm(code, lo))
    calls, cbranch, data_refs, text_refs, writes = [], 0, set(), set(), 0
    for ins in ins_list:
        if ins.mnemonic == "call":
            op = ins.operands[0] if ins.operands else None
            if op is not None and op.type == capstone.x86.X86_OP_IMM:
                calls.append(op.imm)
            else:
                calls.append(None)                    # indirect
        elif ins.mnemonic.startswith("j") and ins.mnemonic != "jmp":
            cbranch += 1
        for op in ins.operands:
            if op.type == capstone.x86.X86_OP_MEM:
                d = op.mem.disp & 0xFFFFFFFF
                if op.mem.base == 0 and DATA_LO <= d < DATA_HI:
                    data_refs.add(d)
                elif op.mem.base == 0 and 0x10B01000 <= d < DATA_LO:
                    text_refs.add(d)
            elif op.type == capstone.x86.X86_OP_IMM:
                v = op.imm & 0xFFFFFFFF
                if DATA_LO <= v < DATA_HI:
                    data_refs.add(v)
        if ins.mnemonic == "mov" and ins.operands and \
                ins.operands[0].type == capstone.x86.X86_OP_MEM:
            writes += 1
    return {
        "lo": lo, "hi": hi, "bytes": hi - lo, "ins": len(ins_list),
        "calls": calls, "ncalls": len(calls), "cbranch": cbranch,
        "data_refs": sorted(data_refs), "text_refs": sorted(text_refs),
        "writes": writes,
    }


def cmd_tables(img):
    targets, idx, by_arm = tables(img)
    print("body            %08x .. %08x  (%d B)" % (BODY_VA, BODY_VA + BODY_LEN, BODY_LEN))
    print("dispatch site   %08x" % DISPATCH_VA)
    print("byte table      %08x .. %08x  (%d entries, opcode %#04x..%#04x)"
          % (BYTE_TABLE_VA, BYTE_TABLE_VA + BYTE_TABLE_LEN, BYTE_TABLE_LEN,
             FIRST_OPCODE, LAST_OPCODE))
    print("jump table      %08x .. %08x  (%d entries)"
          % (JUMP_TABLE_VA, JUMP_TABLE_VA + 4 * JUMP_TABLE_LEN, JUMP_TABLE_LEN))
    print("distinct arms   %d" % len(set(targets)))
    inside = [t for t in targets if BODY_VA <= t < BODY_VA + BODY_LEN]
    print("arms inside body %d / %d" % (len(inside), len(targets)))
    print()
    hist = {}
    for i, ai in enumerate(idx):
        hist[ai] = hist.get(ai, 0) + 1
    print("arm  target    opcodes  opcode list")
    for ai in range(JUMP_TABLE_LEN):
        ops = [FIRST_OPCODE + i for i, a in enumerate(idx) if a == ai]
        print("%3d  %08x  %5d    %s" % (
            ai, targets[ai], len(ops),
            " ".join("%02x" % v for v in ops[:24]) + (" ..." if len(ops) > 24 else "")))


def cmd_arms(img):
    md = _md()
    targets, idx, by_arm = tables(img)
    spans = arm_spans(targets)
    rows = []
    for ai in range(JUMP_TABLE_LEN):
        t = targets[ai]
        lo, hi = spans[t]
        a = analyse_arm(img, md, lo, hi)
        ops = [FIRST_OPCODE + i for i, x in enumerate(idx) if x == ai]
        a["arm"] = ai
        a["nops"] = len(ops)
        rows.append(a)
    b = sorted(r["bytes"] for r in rows)
    print("arm bodies (linear span): min %d  median %d  max %d  total %d"
          % (b[0], b[len(b) // 2], b[-1], sum(b)))
    print("arms with >=1 direct call : %d / %d"
          % (sum(1 for r in rows if r["ncalls"]), len(rows)))
    print("arms with >=1 .data global: %d / %d"
          % (sum(1 for r in rows if r["data_refs"]), len(rows)))
    print("arms with >=1 cond branch : %d / %d"
          % (sum(1 for r in rows if r["cbranch"]), len(rows)))
    allg = {}
    for r in rows:
        for g in r["data_refs"]:
            allg[g] = allg.get(g, 0) + 1
    print("distinct .data globals    : %d ; top:" % len(allg))
    for g, n in sorted(allg.items(), key=lambda kv: -kv[1])[:14]:
        print("    %08x  %d arms" % (g, n))


def cmd_tsv(img):
    md = _md()
    targets, idx, by_arm = tables(img)
    spans = arm_spans(targets)
    o = img.off(CLASS_TABLE_VA)
    cls = list(img.blob[o:o + 0xC0])
    print("\t".join(["arm", "target", "lo", "hi", "bytes", "ins", "ncalls",
                     "cbranch", "writes", "nops", "opclasses", "calls",
                     "data_refs", "text_refs", "opcodes"]))
    for ai in range(JUMP_TABLE_LEN):
        t = targets[ai]
        lo, hi = spans[t]
        a = analyse_arm(img, md, lo, hi)
        ops = [FIRST_OPCODE + i for i, x in enumerate(idx) if x == ai]
        opcls = sorted({cls[v] for v in ops if v < len(cls)})
        print("\t".join([
            str(ai), "%08x" % t, "%08x" % lo, "%08x" % hi, str(a["bytes"]),
            str(a["ins"]), str(a["ncalls"]), str(a["cbranch"]), str(a["writes"]),
            str(len(ops)),
            ",".join("%02x" % c for c in opcls),
            ",".join("%08x" % c if c else "indirect" for c in a["calls"]),
            ",".join("%08x" % g for g in a["data_refs"]),
            ",".join("%08x" % g for g in a["text_refs"]),
            ",".join("%02x" % v for v in ops)]))


def cmd_disasm(img, what):
    md = _md()
    targets, idx, by_arm = tables(img)
    spans = arm_spans(targets)
    if what.startswith("0x") or len(what) == 8:
        va = int(what, 16)
        lo, hi = spans.get(va, (va, va + 96))
    else:
        t = targets[int(what)]
        lo, hi = spans[t]
    ops = [FIRST_OPCODE + i for i, x in enumerate(idx) if targets[x] == lo]
    print("; arm %08x  span %08x..%08x (%d B)  opcodes: %s"
          % (lo, lo, hi, hi - lo, " ".join("%02x" % v for v in ops)))
    o = img.off(lo)
    for ins in md.disasm(img.blob[o:o + (hi - lo)], lo):
        print("%08x  %-7s %s" % (ins.address, ins.mnemonic, ins.op_str))


def cmd_sample(img, n, seed):
    targets, idx, by_arm = tables(img)
    keyed = set()
    for ops in CONSTRUCT_OPCODES.values():
        for op in ops:
            keyed.add(idx[op - FIRST_OPCODE])
    pool = sorted(set(range(JUMP_TABLE_LEN)) - keyed)
    rng = random.Random(seed)
    pick = sorted(rng.sample(pool, min(n, len(pool))))
    print("; stratum S-C, seed %d, %d of %d eligible arms "
          "(excluded %d construct-keyed)" % (seed, len(pick), len(pool), len(keyed)))
    for ai in pick:
        ops = [FIRST_OPCODE + i for i, x in enumerate(idx) if x == ai]
        print("%3d  %08x  %s" % (ai, targets[ai], " ".join("%02x" % v for v in ops)))


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    img = load(sys.argv[1])
    cmd = sys.argv[2]
    if cmd == "--tables":
        cmd_tables(img)
    elif cmd == "--arms":
        cmd_arms(img)
    elif cmd == "--tsv":
        cmd_tsv(img)
    elif cmd == "--disasm":
        cmd_disasm(img, sys.argv[3])
    elif cmd == "--sample":
        n = int(sys.argv[3]) if len(sys.argv) > 3 else 12
        seed = int(sys.argv[5]) if len(sys.argv) > 5 else 20260823
        cmd_sample(img, n, seed)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
