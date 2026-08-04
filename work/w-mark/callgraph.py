#!/usr/bin/env python3
"""callgraph.py callers|callees <va> [depth]  -- direct-call graph of c2.dll.

Built by scanning .text for E8 rel32 and attributing each site to the enclosing
function from docs/whitebox/c2_functions.tsv. DISASSEMBLY-DERIVED, navigation
only; stdlib only.
"""
import os, struct, sys, bisect

ROOT = os.environ.get("C2RS_ROOT") or os.path.abspath(
    os.path.join(os.path.dirname(__file__), "..", ".."))
DLL = os.environ.get("C2RS_C2DLL") or os.path.join(
    ROOT, "compilers/X360/16.00.11886.00/c2.dll")
FNS = os.path.join(ROOT, "docs/whitebox/c2_functions.tsv")
TEXT_RAW, TEXT_SZ, TEXT_VA = 0x400, 0x12CE00, 0x10B01000


def funcs():
    out = []
    for line in open(FNS):
        if line.startswith("#") or line.startswith("addr"):
            continue
        p = line.rstrip("\n").split("\t")
        out.append((int(p[0], 16), int(p[1]), p[3]))
    out.sort()
    return out


FN = funcs()
STARTS = [f[0] for f in FN]


def owner(va):
    i = bisect.bisect_right(STARTS, va) - 1
    if i < 0:
        return None
    a, sz, cl = FN[i]
    return (a, sz, cl) if va < a + sz else (a, sz, cl + "?")


def edges():
    d = open(DLL, "rb").read()
    e = []
    for o in range(TEXT_RAW, TEXT_RAW + TEXT_SZ - 5):
        if d[o] != 0xE8:
            continue
        rel = struct.unpack_from("<i", d, o + 1)[0]
        src = o - TEXT_RAW + TEXT_VA
        e.append((src, (src + 5 + rel) & 0xFFFFFFFF))
    return e


E = None


def build():
    global E
    if E is None:
        E = edges()
    return E


def callers(va):
    out = []
    for src, tgt in build():
        if tgt == va:
            out.append((src, owner(src)))
    return out


def tus():
    t = []
    for line in open(os.path.join(ROOT, "docs/whitebox/c2_tus.tsv")):
        if line.startswith("#") or line.startswith("file"):
            continue
        p = line.split("\t")
        t.append((int(p[1], 16), p[0]))
    t.sort()
    return t


TU = tus()
TUS = [x[0] for x in TU]


def tu_of(va):
    i = bisect.bisect_right(TUS, va) - 1
    lo = TU[i][1] if i >= 0 else "?"
    hi = TU[i + 1][1] if i + 1 < len(TU) else "?"
    return lo if i >= 0 and TU[i][0] <= va else "<%s" % hi


if __name__ == "__main__":
    mode = sys.argv[1]
    va = int(sys.argv[2], 16)
    if mode == "callers":
        seen = {}
        for src, ow in callers(va):
            seen.setdefault(ow[0] if ow else 0, []).append(src)
        for f, sites in sorted(seen.items()):
            o = owner(f)
            print("  fn %08x [%s] tu~%s   sites: %s" % (
                f, o[2] if o else "?", tu_of(f),
                " ".join("%08x" % s for s in sites)))
    elif mode == "tree":
        depth = int(sys.argv[3]) if len(sys.argv) > 3 else 3
        seen = set()

        def rec(f, d, pre):
            if d > depth or f in seen:
                return
            seen.add(f)
            cs = sorted(set((owner(s)[0] if owner(s) else 0) for s, _ in callers(f)))
            for c in cs:
                o = owner(c)
                print("%s%08x [%s] tu~%s" % (pre, c, o[2] if o else "?", tu_of(c)))
                rec(c, d + 1, pre + "  ")

        o = owner(va)
        print("%08x [%s] tu~%s" % (va, o[2] if o else "?", tu_of(va)))
        rec(va, 1, "  ")
