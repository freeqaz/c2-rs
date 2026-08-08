#!/usr/bin/env python3
"""extok.py -- tokenize a c2 `.ex` byte range with c2's OWN operand-class table.

WHITEBOX INSTRUMENT (lane wb-eh).  The class table is read out of the pinned
c2.dll image at DAT_10b25e48 (0x10b25e48, 192 entries over opcodes 0x00-0xBF);
the per-class operand grammar is transcribed from the 29 class arms at
0x10b3d641-0x10b3d94d and the scalar primitives at 0x10c1f8fc / 0x10c1f90a /
0x10c1f91b / 0x10c1f9a6 / 0x10c1f9e9 / 0x10c1fe40.  Navigation only: nothing
here is adopted into crates/.

usage: extok.py <c2dll> <ex-file> <start-hex> [end-hex]
"""
import sys, struct

def load_class_table(dll):
    d = open(dll, 'rb').read()
    base = 0x10b25e48 - 0x10b00c00           # .text VA -> file offset
    return list(d[base:base + 192]), d

class R:
    def __init__(self, b, i): self.b, self.i = b, i
    def byte(self):
        v = self.b[self.i]; self.i += 1; return v
    def skip(self):                          # 0x10c1f90a
        while True:
            if self.byte() < 0x80: return
    def varu(self):                          # 0x10c1f91b: 2 or 4 bytes, never 1
        b0, b1 = self.byte(), self.byte()
        if not (b1 & 0x80): return b0 | (b1 << 8)
        b2, b3 = self.byte(), self.byte()
        return b0 | ((b1 & 0x7f) << 8) | (b2 << 15) | (b3 << 23)
    def i16c(self):                          # 0x10c1f9a6
        b = self.byte()
        if b != 0x80: return b - 256 if b > 127 else b
        lo, hi = self.byte(), self.byte()
        v = lo | (hi << 8); return v - 65536 if v > 32767 else v
    def i32c(self):                          # 0x10c1f9e9
        b = self.byte()
        if b != 0x80: return b - 256 if b > 127 else b
        v = struct.unpack_from('<i', bytes(self.b[self.i:self.i+4]))[0]; self.i += 4
        return v
    def typeword(self):                      # 0x10c1fe40
        b1 = self.byte()
        if not (b1 & 0x80): return b1
        b2 = self.byte()
        if b1 & 0x40:
            b3 = self.byte()
            return ((b2 & 0x7f) << 16) | ((b1 & 0x7f) << 8) | b3
        return ((b1 & 0x7f) << 8) | b2
    def TYPE(self, op):                       # 0x10b3d546
        v = self.typeword()
        cls = v & 0xf
        ext = (v >> 4) & 0x1f
        if cls == 6 and ext == 0: self.i32c()          # aggregate out-of-line size
        self.skip()                                    # the globally gated "id"
        return v

# 0x4F sub-record format strings, table at 0x10b26268 (stride 8, ptr at +0)
def fmt_of(d, sub):
    o = 0x10b26268 - 0x10b00c00 + sub * 8
    p = struct.unpack_from('<I', d, o)[0]
    if p == 0: return None
    fo = p - 0x10b00c00
    e = d.index(b'\0', fo)
    return d[fo:e]

def tok(tbl, d, r, op):
    c = tbl[op] if op < 0xc0 else None
    if c is None: raise ValueError(f"opcode {op:02x} >= 0xC0")
    if c == 0x00: pass
    elif c == 0x01: r.TYPE(op)
    elif c == 0x02:
        if op != 0x42: r.varu()
        else: r.varu()
    elif c == 0x03: r.varu(); r.TYPE(op); r.byte(); r.byte()
    elif c == 0x04: r.varu(); r.byte()
    elif c == 0x05: r.TYPE(op); r.byte()
    elif c == 0x06:
        v = r.TYPE(op)
        if (v & 0xf) == 5: raise ValueError("class06 real path not modelled")
        elif (v & 0xfff) == 8: r.i32c(); r.i32c()
        else: r.i32c()
    elif c == 0x07: r.TYPE(op); r.varu()
    elif c == 0x08: r.varu()
    elif c == 0x09: r.TYPE(op); r.byte()
    elif c == 0x0a: r.byte()
    elif c == 0x0c:
        sub = r.i16c()
        f = fmt_of(d, sub)
        if f:
            for ch in f:
                if ch == 0x6c: r.i32c()          # 'l'
                elif ch == 0x73: r.varu()        # 's'
                elif ch == 0x14: r.i32c()
                elif ch == 0x15: r.i16c()
                elif ch == 0x0e: r.i16c()
                elif ch == 0x0b: r.byte()
                elif ch == 0x16: r.varu()
                else: raise ValueError(f"4F sub {sub:02x} field {ch:02x} not modelled")
    elif c == 0x0d: r.i32c()
    elif c == 0x0e: r.TYPE(op); r.varu(); r.varu()
    elif c == 0x0f: r.i16c()
    elif c == 0x12: r.TYPE(op); r.varu()
    elif c == 0x13: r.TYPE(op); return ('state', r.i32c())
    elif c == 0x14: r.i32c(); r.i32c()
    elif c == 0x15: r.varu(); r.varu(); r.i16c()
    elif c == 0x17:
        n = r.i32c(); r.i += n
    elif c == 0x18: r.varu(); r.TYPE(op)
    elif c == 0x19: r.TYPE(op); r.byte(); r.i32c()
    elif c == 0x1a:
        n = r.i32c()
        for _ in range(n): r.skip()
    elif c == 0x1b: r.i32c(); r.varu()
    elif c == 0x1c: r.TYPE(op); r.i32c()
    else: raise ValueError(f"class {c:02x} (op {op:02x}) not modelled")
    return None

def main():
    dll, exf = sys.argv[1], sys.argv[2]
    start = int(sys.argv[3], 16)
    tbl, d = load_class_table(dll)
    b = open(exf, 'rb').read()
    end = int(sys.argv[4], 16) if len(sys.argv) > 4 else len(b)
    r = R(b, start)
    while r.i < end:
        s = r.i
        op = r.byte()
        try:
            extra = tok(tbl, d, r, op)
        except Exception as e:
            print(f"{s:04x}  {op:02x}  STOP: {e}"); return 1
        raw = " ".join(f"{x:02x}" for x in b[s:r.i])
        note = f"   <== {extra[0]}={extra[1]}" if extra else ""
        print(f"{s:04x}  op={op:02X} cls={tbl[op]:02X} [{raw}]{note}")
        if op == 0x4d: print("-- 4D end of stream"); return 0
    return 0

sys.exit(main())
