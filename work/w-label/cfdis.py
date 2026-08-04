#!/usr/bin/env python3
"""cfdis.py — dump P's .text for one cflabels.py probe, with branch targets
resolved and each target's predecessor count printed.

The point is the predecessor count: `cflabels.py` finds `cf-if2` (+0) and
`cf-ifelse` (+1) both emitting a single forward `bc` over a two-word block, so
"forward vs backward" cannot be the whole discriminator and the disassembly is
where the difference has to be visible.
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
from cftargets import branch_targets, epilogue_start  # noqa: E402


def main(argv):
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode")
        mode = argv[i + 1]
        del argv[i:i + 2]
    want = [a for a in argv[1:] if not a.startswith("--")]
    wd = tempfile.mkdtemp(prefix="cfdis")
    for p in cflabels.PROBES:
        if want and p[0] not in want:
            continue
        o = G.capture(G.build_src(p[1], p[2], p[3]), mode, wd,
                      p[0].replace("-", "_"))
        if o is None:
            print("%s: capture failed" % p[0]); continue
        P = None
        for g in G.groups(o):
            if g["name"].startswith("?P@@") or g["name"] == "P":
                P = g
        if P is None:
            print("%s: no P" % p[0]); continue
        sec = o.sections[P["sec"] - 1]
        d = o.raw(sec)
        rel = {va: o.sym_by_index(s)["name"] for va, s, t in o.relocs(sec)}
        tg, n = branch_targets(o, P["sec"])
        epi = epilogue_start(o, P["sec"])
        tmap = {}
        for a, t in tg:
            tmap.setdefault(t, []).append(a)
        # A block's predecessors: every branch naming it, plus a fall-through
        # from the preceding word if that word is not an unconditional transfer.
        print("=== %s   (%s)" % (p[0], p[4]))
        print("    epilogue starts at 0x%x" % (epi if epi is not None else -1))
        for i in range(0, len(d), 4):
            w = struct.unpack_from(">I", d, i)[0]
            note = ""
            if i in rel:
                note = "  -> %s (REL)" % rel[i]
            else:
                for a, t in tg:
                    if a == i:
                        note = "  -> 0x%x %s" % (
                            t, "EPI" if (epi is not None and t >= epi) else "INTERIOR")
            mark = ""
            if i in tmap:
                prev = struct.unpack_from(">I", d, i - 4)[0] if i >= 4 else 0
                # unconditional transfer: b (op 18, LK=0), blr, bctr
                uncond = ((prev >> 26) == 18 and not (prev & 1)) or prev == 0x4E800020
                fall = 0 if (i == 0 or uncond) else 1
                mark = "   <== TARGET of %d branch(es) + %d fall-through = %d preds" % (
                    len(tmap[i]), fall, len(tmap[i]) + fall)
            print("  %04x  %08x%s%s" % (i, w, note, mark))
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
