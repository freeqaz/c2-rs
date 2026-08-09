#!/usr/bin/env python3
"""Split a captured `.ex` into per-function segments and hexdump them.

A transcription of `c2_il::func::bundle`'s two-signal splitter, at the
resolution a conversion lane needs to read a body by hand:

  * `4F 1F` is the function-start marker, two bytes, and it collides inside
    payloads (the crate measures ~2 %);
  * `4C 4F 11` is the `LO` body marker;
  * a segment starts at the greatest `4F 1F` at or below its `LO`.

This is a READING aid, never a gate — `IlBundle::functions` is the only
acceptance path (`func/diag.rs`'s own warning, one crate over).

    work/w-xtea2/exdump.py <bundle.ex> [index]
"""
import sys

FN_START = bytes([0x4F, 0x1F])
LO_MARKER = bytes([0x4C, 0x4F, 0x11])


def segments(ex: bytes):
    los, i = [], 0
    while True:
        k = ex.find(LO_MARKER, i)
        if k < 0:
            break
        los.append(k)
        i = k + 1
    starts = []
    for lo in los:
        j = ex.rfind(FN_START, 0, lo + 1)
        starts.append(j if j >= 0 else lo)
    # dedupe, keep order
    out, seen = [], set()
    for s in starts:
        if s not in seen:
            seen.add(s)
            out.append(s)
    out.sort()
    ends = out[1:] + [len(ex)]
    return list(zip(out, ends))


def dump(ex: bytes, a: int, b: int):
    for off in range(a, b, 16):
        row = ex[off:min(off + 16, b)]
        hexs = " ".join(f"{c:02x}" for c in row)
        txt = "".join(chr(c) if 32 <= c < 127 else "." for c in row)
        print(f"  {off:05x}  {hexs:<47}  {txt}")


def main() -> int:
    ex = open(sys.argv[1], "rb").read()
    segs = segments(ex)
    want = int(sys.argv[2]) if len(sys.argv) > 2 else None
    print(f"{len(ex)} B, {len(segs)} segments")
    for n, (a, b) in enumerate(segs):
        print(f"== segment {n}: [{a:#x}, {b:#x})  {b - a} B")
        if want is None or want == n:
            dump(ex, a, b)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
