#!/usr/bin/env python3
"""dump_globregs.py -- regenerate the byte/decompilation listings behind read R4.

Read R4 (docs/whitebox/WB_GLOBREGS_PREREG.md, ref/P_GLOBREGS.md) reads the
globregs mint/merge chain.  This script re-extracts every listing that read
quotes, so a later reader can re-derive any address rather than trusting the
prose.  Output goes to docs/whitebox/labels/globregs/ by default.

It is FENCED on the pinned image digest: the flat Ghidra export is only
quotable while its input still matches the image the reference pins.  If the
digest does not match, the script refuses (exit 1) rather than emitting
listings that silently describe a different binary -- READ_PLAN section 5.4.

Two independent sources are used on purpose (C2_MAP_METHOD.md section 1: no
claim rests on Ghidra alone):
  * objdump_intel.asm  -- GNU binutils, raw instruction bytes
  * decomp_all.c       -- Ghidra decompiler output

Outside the std-only Rust workspace on purpose -- tooling, same status as
scripts/gt_dump.py and docs/whitebox/scripts/dump_opcode_tables.py.

Usage:
    docs/whitebox/scripts/dump_globregs.py             # write the listings
    docs/whitebox/scripts/dump_globregs.py --check     # verify digest only
"""

import hashlib
import os
import re
import sys

PINNED_SHA256 = "c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258"
IMAGE_REL = "compilers/X360/16.00.11886.00/c2.dll"

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(
    os.path.dirname(os.path.abspath(__file__)))))
EXPORT = os.environ.get(
    "C2RS_C2_EXPORT", os.path.expanduser("~/ghidra-projects/export/c2"))
OUTDIR = os.path.join(ROOT, "docs", "whitebox", "labels", "globregs")

# The chain R4 reads, in the order the phase executes it.  Each row is
# (address, name, one-line role).  Every one of these is cited in
# docs/whitebox/ref/P_GLOBREGS.md.
CHAIN = [
    (0x10B57633, "globregs phase driver -- resets the counter, runs 1/2/3"),
    (0x10BD2343, "symbol-chunk allocator -- stamps sym+0x1c, APPENDS the chunk"),
    (0x10BD3225, "symbol-table construction / one symbol allocation"),
    (0x10B550E5, "STEP 1 INDEX -- assigns aux[0x00], allocates sym+0x34"),
    (0x10B568AF, "fills DAT_10c400d0 (index -> symbol) and the per-block sets"),
    (0x10B55732, "STEP 2 RENAME -- the read plan's entry point (F1)"),
    (0x10B54BAD, "version stamp -- aux[0x14]=v, PREPENDS to aux[0x0c], ret v+1"),
    (0x10B54BF0, "candidate-list link (sym+0x30 next, aux[0x10] prev)"),
    (0x10B54C07, "STEP 2b MERGE at joins -- ascending bitset index"),
    (0x10B55DBE, "STEP 3 MINT -- one candidate per (symbol, version)"),
    (0x10B54D32, "the candidate constructor -- id = DAT_10c400d4++"),
    (0x10B27290, "stateful ascending bitset iterator"),
    (0x10C2022A, "the arena allocator -- memsets a fresh chunk"),
]


def die(msg):
    sys.stderr.write("dump_globregs: %s\n" % msg)
    sys.exit(1)


def check_digest():
    img = os.path.join(ROOT, IMAGE_REL)
    if not os.path.exists(img):
        sys.stderr.write("SKIP: image absent (%s)\n" % IMAGE_REL)
        sys.exit(2)
    h = hashlib.sha256()
    with open(img, "rb") as f:
        for blk in iter(lambda: f.read(1 << 20), b""):
            h.update(blk)
    got = h.hexdigest()
    if got != PINNED_SHA256:
        die("image digest %s != pinned %s -- refusing" % (got, PINNED_SHA256))
    return got


def load_funcs():
    p = os.path.join(EXPORT, "functions.tsv")
    if not os.path.exists(p):
        sys.stderr.write("SKIP: flat export absent (%s)\n" % p)
        sys.exit(2)
    out = {}
    with open(p) as f:
        next(f)
        for line in f:
            c = line.rstrip("\n").split("\t")
            if len(c) >= 3:
                out[int(c[0], 16)] = (int(c[1]), c[2])
    return out


def load_asm():
    p = os.path.join(EXPORT, "objdump_intel.asm")
    rows = []
    pat = re.compile(r"^\s*([0-9a-f]{8}):")
    with open(p, encoding="utf-8", errors="replace") as f:
        for line in f:
            m = pat.match(line)
            if m:
                rows.append((int(m.group(1), 16), line.rstrip()))
    rows.sort()
    return rows


def asm_range(rows, lo, hi):
    return [t for a, t in rows if lo <= a < hi]


def decomp_body(src, addr):
    key = "// ===== FUNC %08x " % addr
    i = src.find(key)
    if i < 0:
        return None
    j = src.find("// ===== FUNC ", i + 10)
    return src[i:j if j > 0 else len(src)]


def main():
    digest = check_digest()
    if "--check" in sys.argv:
        print("OK: image digest matches pin (%s)" % digest)
        return
    funcs = load_funcs()
    rows = load_asm()
    with open(os.path.join(EXPORT, "decomp_all.c"),
              encoding="utf-8", errors="replace") as f:
        src = f.read()

    os.makedirs(OUTDIR, exist_ok=True)
    index = []
    for addr, role in CHAIN:
        if addr not in funcs:
            sys.stderr.write("WARN: %08x has no functions.tsv entry\n" % addr)
            size, name = 0, "FUN_%08x" % addr
        else:
            size, name = funcs[addr]
        index.append((addr, size, name, role))
        out = os.path.join(OUTDIR, "%08x.txt" % addr)
        with open(out, "w") as g:
            g.write("# %08x  %s  size=%d\n" % (addr, name, size))
            g.write("# role: %s\n" % role)
            g.write("# image sha256 %s\n" % PINNED_SHA256)
            g.write("# regenerate: docs/whitebox/scripts/dump_globregs.py\n")
            g.write("\n## disassembly (objdump, GNU binutils)\n\n")
            for t in asm_range(rows, addr, addr + max(size, 1)):
                g.write(t + "\n")
            body = decomp_body(src, addr)
            g.write("\n## decompilation (Ghidra)\n\n")
            g.write(body if body else "(no decompiled body in the export)\n")
        print("wrote %s (%d B)" % (out, size))

    with open(os.path.join(OUTDIR, "INDEX.tsv"), "w") as g:
        g.write("addr\tsize\tname\trole\n")
        for addr, size, name, role in index:
            g.write("%08x\t%d\t%s\t%s\n" % (addr, size, name, role))
    print("wrote %s" % os.path.join(OUTDIR, "INDEX.tsv"))


if __name__ == "__main__":
    main()
