#!/usr/bin/env python3
"""xrefs.py <target-va> [more-vas...]  -- find every reference to a VA in c2.dll.

Scans .text for E8/E9 rel32 whose computed target equals the VA, and scans the
whole image for the little-endian absolute DWORD (address-taken / vtable slot).
DISASSEMBLY-DERIVED, navigation only; stdlib only.
"""
import os, struct, sys

ROOT = os.environ.get("C2RS_ROOT") or os.path.abspath(
    os.path.join(os.path.dirname(__file__), "..", ".."))
DLL = os.environ.get("C2RS_C2DLL") or os.path.join(
    ROOT, "compilers/X360/16.00.11886.00/c2.dll")

TEXT_RAW, TEXT_SZ, TEXT_VA = 0x400, 0x12CE00, 0x10B01000


def load():
    return open(DLL, "rb").read()


def off2va(o):
    return o - TEXT_RAW + TEXT_VA


def scan(d, targets):
    hits = []
    for o in range(TEXT_RAW, TEXT_RAW + TEXT_SZ - 5):
        b = d[o]
        if b != 0xE8 and b != 0xE9:
            continue
        rel = struct.unpack_from("<i", d, o + 1)[0]
        tgt = (off2va(o) + 5 + rel) & 0xFFFFFFFF
        if tgt in targets:
            hits.append((off2va(o), "call" if b == 0xE8 else "jmp", tgt))
    return hits


def scan_abs(d, targets):
    hits = []
    for t in targets:
        pat = struct.pack("<I", t)
        i = 0
        while True:
            i = d.find(pat, i)
            if i < 0:
                break
            hits.append((i, off2va(i) if TEXT_RAW <= i < TEXT_RAW + TEXT_SZ else None, t))
            i += 1
    return hits


if __name__ == "__main__":
    tg = set(int(a, 16) for a in sys.argv[1:])
    d = load()
    for va, kind, t in scan(d, tg):
        print("%s %08x -> %08x" % (kind, va, t))
    for off, va, t in scan_abs(d, tg):
        print("abs  file=%06x va=%s -> %08x" % (off, ("%08x" % va) if va else "-", t))
