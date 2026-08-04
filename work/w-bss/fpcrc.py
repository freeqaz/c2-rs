#!/usr/bin/env python3
"""Lane w-bss section 4.2.1 -- the registered .data CheckSum grid, plus the three
exploratory cells.  Predictions were committed in
docs/rungs/_2026-08-04-w-bss-fpcrc-prereg.md BEFORE any of these were compiled."""
import sys, os, struct
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
os.environ.setdefault("WBSS_FLAGS", "flags-w.txt")
from probe import compile_src
from coffdump import Obj

GRID = [
    ('f0',  'int a=1;\nint b=2;\n',                              0xD36E489C),
    ('f1',  'float f=1.0f;\n',                                   0x00000000),
    ('f2',  'double d=1.0;\n',                                   0x00000000),
    ('f3',  'int a=1;\nfloat f=1.0f;\n',                         0x77073096),
    ('f4',  'float f=1.0f;\nint a=1;\n',                         0x77073096),
    ('f5',  'int a=1;\nfloat f=1.0f;\nint b=2;\n',               0xD36E489C),
    ('f6',  'int a=1;\ndouble d=1.0;\nint b=2;\n',               0xD36E489C),
    ('f7',  'float f=1.0f;\nfloat g=2.0f;\n',                    0x00000000),
    ('f8',  'char c=1;\nfloat f=1.0f;\n',                        0xB8BC6765),
    ('f9',  'char c=1;\nchar e=2;\nfloat f=1.0f;\nchar g=3;\n',  0x9015E0C8),
    ('f10', 'float p[2]={1.0f,2.0f};\nint a=1;\n',               0x77073096),
]
EXPLORATORY = [
    ('x1', '#pragma pack(1)\nstruct P{char c; float f;};\nP p={1,1.0f};\n'),
    ('x2', 'struct Q{int i; float f;};\nQ q={7,1.0f};\n'),
    ('x3', 'struct R{int i; int j;};\nR r={7,9};\n'),
]

def crc0(b):
    c = 0
    for ch in b:
        c ^= ch
        for _ in range(8):
            c = (c >> 1) ^ (0xEDB88320 if c & 1 else 0)
    return c

def measure(tag, src):
    o = Obj(open(compile_src(src, 'fp_' + tag), 'rb').read())
    s = [x for x in o.secs if x['name'] == '.data']
    if not s:
        return None
    s = s[0]
    cks = 0
    for sy in o.syms:
        if sy['sec'] == s['idx'] and sy['naux'] and sy['name'] == '.data':
            cks = struct.unpack_from('<IHHIHB', sy['aux'][0], 0)[3]
    raw = o.secdata(s)
    syms = sorted((sy['val'], sy['name']) for sy in o.syms
                  if sy['sec'] == s['idx'] and sy['naux'] == 0)
    return s, cks, raw, syms

hits = 0
print("=== registered grid (predictions committed before compiling) ===")
for tag, src, pred in GRID:
    s, cks, raw, syms = measure(tag, src)
    ok = (cks == pred)
    hits += ok
    print("%-4s size=0x%-3x cks=0x%08x pred=0x%08x %s  crcALL=0x%08x  %s"
          % (tag, s['size'], cks, pred, "HIT " if ok else "MISS", crc0(raw),
             " ".join("%s@%x" % (n.split('@')[0].lstrip('?'), v) for v, n in syms)))
    print("       raw = %s" % raw.hex(' '))
print("\n%d/%d registered predictions hit\n" % (hits, len(GRID)))

print("=== exploratory cells (NOT pre-registered) ===")
for tag, src in EXPLORATORY:
    s, cks, raw, syms = measure(tag, src)
    print("%-3s size=0x%-3x cks=0x%08x crcALL=0x%08x  raw=%s"
          % (tag, s['size'], cks, crc0(raw), raw.hex(' ')))
    for lbl, sub in [('drop bytes 1..5 (float at byte 1)', raw[:1] + raw[5:]),
                     ('drop the last 4 (float member)',    raw[:-4]),
                     ('drop the first 4',                  raw[4:]),
                     ('keep all bytes',                    raw)]:
        if crc0(sub) == cks:
            print("       reproduced by: %s" % lbl)
