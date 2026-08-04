#!/usr/bin/env python3
"""dataref.py <va> [...]  -- every .text instruction whose bytes contain the
absolute address <va>, disassembled with 8 bytes of lead-in context.

Used to find the writers/readers of a c2.dll global. DISASSEMBLY-DERIVED,
navigation only; stdlib only.
"""
import os, struct, subprocess, sys

ROOT = os.environ.get("C2RS_ROOT") or os.path.abspath(
    os.path.join(os.path.dirname(__file__), "..", ".."))
DLL = os.environ.get("C2RS_C2DLL") or os.path.join(
    ROOT, "compilers/X360/16.00.11886.00/c2.dll")
TEXT_RAW, TEXT_SZ, TEXT_VA = 0x400, 0x12CE00, 0x10B01000

d = open(DLL, "rb").read()
for a in sys.argv[1:]:
    va = int(a, 16)
    pat = struct.pack("<I", va)
    print("==== refs to %08x ====" % va)
    i = TEXT_RAW
    while True:
        i = d.find(pat, i, TEXT_RAW + TEXT_SZ)
        if i < 0:
            break
        site = i - TEXT_RAW + TEXT_VA
        start = site - 8
        out = subprocess.run(
            ["objdump", "-D", "-b", "binary", "-m", "i386", "-M", "intel",
             "--adjust-vma=0x10b00c00", DLL,
             "--start-address=%#x" % start, "--stop-address=%#x" % (site + 10)],
            capture_output=True, text=True).stdout
        lines = [l for l in out.splitlines() if l.strip().startswith("10b") or
                 l.strip().startswith("10c")]
        print("-- operand at %08x" % site)
        for l in lines[-4:]:
            print("   " + l.strip())
        i += 1
