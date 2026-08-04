#!/usr/bin/env python3
"""cftargets.py — the MECHANISM behind cflabels.py's table.

`cflabels.py` measures what each control-flow construct costs the compiler-label
counter. This script asks *why*, from the same objs' own bytes: for every probe
it disassembles P's `.text`, collects every **intra-section** branch target
(`b`/`bc` with no relocation on the word — `docs/CFG_SHAPE.md` §3.3: a branch
carries its true displacement and never takes a relocation, a call carries a
section-start placeholder and always does), and classifies each distinct target:

    EPI       the epilogue -- the target is at or after the last block's start,
              i.e. the layout was going to end there anyway
    INTERIOR  anything else -- a block the layout does not already give the
              branch for free

The claim under test, registered in `work/w-label/PREREG.md` before this ran:

    control-flow surcharge == the number of DISTINCT INTERIOR branch targets

with the `docs/LABEL_COUNTER.md` §1.1 surcharges (visible in `minted`) removed
first. A row where the two disagree is printed with `<== MISS`, because a table
fitted by all its cells and tested by none is exactly what `CFG_SHAPE.md` §3.5's
declined fold model was.

    work/w-label/cftargets.py [--mode '/O1 /GS- /c'] [probe ...]
"""

import os
import struct
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
import gt_label_stride as G  # noqa: E402

sys.path.insert(0, HERE)
import cflabels  # noqa: E402


def branch_targets(o, sec_index):
    """Every intra-section branch target in one function's .text, plus the
    epilogue's start offset.

    A word is an intra-section branch iff its primary opcode is 16 (`bc`) or 18
    (`b`) with AA=0 and LK=0 **and there is no relocation on it**. The
    relocation is the discriminator, not the opcode: `48000008` and `4bffffec`
    are the same instruction and only the second is a call (§3.3).
    """
    sec = o.sections[sec_index - 1]
    d = o.raw(sec)
    reloc_at = {va for va, sym, ty in o.relocs(sec)}
    targets = []
    n = len(d) // 4
    for i in range(n):
        at = i * 4
        if at in reloc_at:
            continue
        w = struct.unpack_from(">I", d, at)[0]
        op = w >> 26
        if op == 18:                      # b / ba / bl / bla
            if w & 3:                     # AA or LK set -> not a plain intra b
                continue
            li = w & 0x03FFFFFC
            if li & 0x02000000:
                li -= 0x04000000
            targets.append((at, at + li))
        elif op == 16:                    # bc
            if w & 3:
                continue
            bd = w & 0x0000FFFC
            if bd & 0x8000:
                bd -= 0x10000
            targets.append((at, at + bd))
    return targets, len(d)


def epilogue_start(o, sec_index):
    """Offset of the epilogue's first word: the `addi r1,r1,F` that undoes the
    `stwu r1,-F(r1)`, searched from the end so a body with its own `addi` early
    is not mistaken for it. `None` if the function is not framed."""
    sec = o.sections[sec_index - 1]
    d = o.raw(sec)
    frame = None
    for i in range(0, len(d), 4):
        w = struct.unpack_from(">I", d, i)[0]
        if (w >> 26) == 37 and ((w >> 21) & 31) == 1 and ((w >> 16) & 31) == 1:
            frame = -((w & 0xFFFF) - 0x10000 if (w & 0x8000) else (w & 0xFFFF))
            break
    if frame is None:
        return None
    for i in range(len(d) - 4, -1, -4):
        w = struct.unpack_from(">I", d, i)[0]
        # addi r1,r1,frame
        if (w >> 26) == 14 and ((w >> 21) & 31) == 1 and ((w >> 16) & 31) == 1 \
                and (w & 0xFFFF) == (frame & 0xFFFF):
            return i
    return None


def main(argv):
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode")
        mode = argv[i + 1]
        del argv[i:i + 2]
    want = [a for a in argv[1:] if not a.startswith("--")]
    pool = cflabels.PROBES + cflabels.HELDOUT if "--heldout" in argv else cflabels.PROBES
    probes = [p for p in pool if not want or p[0] in want]

    print("mode: %s" % mode)
    print("`sur` = stride - base_stride - (minted - 5), i.e. the surcharge with")
    print("        LABEL_COUNTER.md §1.1's minted surcharges (helper pairs,")
    print("        pooled constants) removed.  `int` = distinct INTERIOR targets.")
    print()
    print("%-18s %5s %5s %4s %4s %4s  %s"
          % ("probe", "strid", "mint", "sur", "int", "join", "targets (from -> to; E=epilogue)"))
    wd = tempfile.mkdtemp(prefix="cftgt")
    base = None
    miss = 0
    for p in probes:
        row = G.run(p[0], p[1], p[2], p[3], p[4], mode, wd)
        if row is None or "error" in row:
            print("%-18s  FAILED" % p[0])
            continue
        # Re-capture so we have the obj here too (G.run does not return it).
        o = G.capture(G.build_src(p[1], p[2], p[3]), mode, wd,
                      p[0].replace("-", "_") + "_t")
        gs = G.groups(o)
        P = None
        for g in gs:
            if g["name"].startswith("?P@@") or g["name"] == "P":
                P = g
        if P is None:
            print("%-18s  no P group" % p[0])
            continue
        tg, n = branch_targets(o, P["sec"])
        epi = epilogue_start(o, P["sec"])
        sec = o.sections[P["sec"] - 1]
        d = o.raw(sec)
        if p[0] == "cf-none":
            base = row["stride"]
        sur = (row["stride"] - base - (row["minted"] - 5)) if base is not None else None
        interior = sorted({t for _, t in tg
                           if epi is None or t < epi})
        # A target's predecessors: the branches naming it, plus a fall-through
        # from the preceding word unless that word is an unconditional transfer
        # (`b` with LK=0, or `blr`). An interior target with >= 2 predecessors
        # is a JOIN; one with exactly 1 is a plain forward skip.
        def preds(t):
            n = sum(1 for _, u in tg if u == t)
            if t >= 4:
                prev = struct.unpack_from(">I", d, t - 4)[0]
                uncond = ((prev >> 26) == 18 and not (prev & 1)) or prev == 0x4E800020
                if not uncond:
                    n += 1
            return n
        joins = [t for t in interior if preds(t) >= 2]
        shown = " ".join("%x->%x%s" % (a, t, "E" if (epi is not None and t >= epi) else "")
                         for a, t in tg)
        mark = ""
        if sur is not None and sur != len(joins):
            mark = "   <== MISS"
            miss += 1
        print("%-18s %5d %5d %4s %4d %4d  %s%s"
              % (p[0], row["stride"], row["minted"],
                 "?" if sur is None else "+%d" % sur, len(interior), len(joins),
                 shown, mark))
    print()
    print("rows where surcharge != distinct interior JOINS: %d" % miss)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
