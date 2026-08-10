#!/usr/bin/env python3
"""w-frame783 — re-derive board #2783 from this lane's OWN capture.

Nothing is inherited: the framing test, the PREV field's location and its value
range are all read off the bytes here. Usage:

    frame783.py <dir-with-_CL_*.gl-and-.ex>   [--dump N]

What it prints, per capture:
  * the `.ex` `4F 1F` split offsets (the segment set a record could name)
  * every `80 <LE32>` field, under the GATE framing and under the RELAXED one
  * the PREV field's value for each, so "what does the byte the gate pins
    actually hold" is a histogram and not a claim
  * false positives: framed offsets that are NOT `.ex` split points, at either
    width, and duplicate offsets

The two framings, spelled out so this file is readable without the crate:

  GATE  (crates/c2-il/src/codec.rs :: gl_offset_framed)
      gl[o]==0x80 && gl[o-7]==0x80 && gl[o-5]==0x10
      && gl[o-4]==0 && gl[o-3]==0 && gl[o-2]==0 && gl[o-1]==0
  WIDE  (crates/c2-il/src/func/bind.rs :: emit_offset_framed) — the same with
      the `gl[o-5]==0x10` clause dropped.

The record shape both read is

      80 <LE32 PREV> 00 00 | 80 <LE32 BODY-START>
      ^o-7  ^o-6..o-3  ^o-2 ^o
so `gl[o-5]` is PREV's byte 1 and the gate's clause is PREV in [0x1000,0x10FF]
given that gl[o-4]==gl[o-3]==0.  The relaxed test is PREV < 0x10000.
"""
import sys, os, glob, struct
from collections import Counter


def le32(b, o):
    return struct.unpack_from("<I", b, o)[0]


def gate_framed(gl, o):
    return (o >= 7 and gl[o] == 0x80 and gl[o - 7] == 0x80 and gl[o - 5] == 0x10
            and gl[o - 4] == 0 and gl[o - 3] == 0 and gl[o - 2] == 0 and gl[o - 1] == 0)


def wide_framed(gl, o):
    return (o >= 7 and gl[o] == 0x80 and gl[o - 7] == 0x80
            and gl[o - 4] == 0 and gl[o - 3] == 0 and gl[o - 2] == 0 and gl[o - 1] == 0)


def scan(gl, pred):
    """The crate's own loop: step 5 on a hit, 1 on a miss."""
    out, p = [], 0
    while p + 5 <= len(gl):
        if pred(gl, p):
            out.append((p, le32(gl, p + 1), le32(gl, p - 6) & 0xFFFFFFFF))
            p += 5
        else:
            p += 1
    return out


def ex_splits(ex):
    starts, i = [], 0
    while i + 1 < len(ex):
        j = ex.find(b"\x4f", i, len(ex) - 1)
        if j < 0:
            break
        if ex[j + 1] == 0x1F:
            starts.append(j)
            i = j + 2
        else:
            i = j + 1
    return starts


def prev_of(gl, o):
    # PREV is the LE32 that begins at o-6 (its tag byte is at o-7).
    return le32(gl, o - 6)


def report(d, dump=0):
    gl = open(glob.glob(os.path.join(d, "*.gl"))[0], "rb").read()
    ex = open(glob.glob(os.path.join(d, "*.ex"))[0], "rb").read()
    segs = ex_splits(ex)
    segset = set(segs)
    print(f"== {d}   .gl {len(gl)} B   .ex {len(ex)} B   segments {len(segs)}")
    for label, pred in (("GATE", gate_framed), ("WIDE", wide_framed)):
        hits = [(p, le32(gl, p + 1), prev_of(gl, p)) for p in range(7, len(gl) - 4)
                if pred(gl, p)]
        # …and the crate's stepping loop, which can differ from the naive one
        step = scan(gl, pred)
        offs = [v for _, v, _ in step]
        fp = [v for v in offs if v not in segset]
        dup = len(offs) - len(set(offs))
        prevs = sorted({pv for _, _, pv in step})
        print(f"  {label:4s} records {len(step):6d} (naive {len(hits)})   "
              f"offsets NOT an .ex split: {len(fp)}   duplicate offsets: {dup}")
        if prevs:
            print(f"       PREV range 0x{prevs[0]:x}..0x{prevs[-1]:x}  distinct {len(prevs)}"
                  f"  >=0x10000: {sum(1 for x in prevs if x >= 0x10000)}")
            hi = Counter((pv >> 8) & 0xFF for _, _, pv in step)
            print(f"       PREV byte1 histogram (the byte the gate pins to 0x10): "
                  + ", ".join(f"0x{k:02x}:{v}" for k, v in sorted(hi.items())[:12])
                  + (" …" if len(hi) > 12 else ""))
        if dump:
            for p, v, pv in step[:dump]:
                print(f"       @{p:6d}  body-start {v:8d}  PREV 0x{pv:08x}  "
                      f"{gl[p-7:p+5].hex(' ')}")
    return gl, ex, segs


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    dump = 0
    for a in sys.argv[1:]:
        if a.startswith("--dump"):
            dump = int(a.split("=")[1]) if "=" in a else 6
    for d in args:
        report(d, dump)
