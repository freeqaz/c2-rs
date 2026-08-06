#!/usr/bin/env python3
"""exdump.py — split a captured `.ex` into function segments and print the ones
matching a byte pattern.

Lane w-inl0 measurement tooling. The segment split is the port's own: `4F 1F`
(`crates/c2-il/src/func/bundle.rs`, `FN_START`) is what `IlBundle::functions()`
consumes, and `4C 4F 11` (`LO_MARKER`) opens the body inside a segment.

    exdump.py <file.ex> [--pat HEXBYTES] [--index N] [--max N] [--names <file.gl>]

With no `--pat` every segment is listed with its length and its first bytes.
"""
import sys

FN_START = bytes([0x4F, 0x1F])
LO_MARKER = bytes([0x4C, 0x4F, 0x11])


def segments(ex: bytes):
    """Split on FN_START, yielding (ordinal, offset-in-file, bytes)."""
    out = []
    i = ex.find(FN_START)
    while i >= 0:
        j = ex.find(FN_START, i + 2)
        end = j if j >= 0 else len(ex)
        out.append((len(out), i, ex[i:end]))
        i = j
    return out


def hexs(b: bytes) -> str:
    return " ".join(f"{x:02x}" for x in b)


def main(argv):
    path = argv[1]
    pat = None
    index = None
    limit = 8
    a = 2
    while a < len(argv):
        if argv[a] == "--pat":
            pat = bytes.fromhex(argv[a + 1].replace(" ", ""))
            a += 2
        elif argv[a] == "--index":
            index = int(argv[a + 1])
            a += 2
        elif argv[a] == "--max":
            limit = int(argv[a + 1])
            a += 2
        else:
            raise SystemExit(f"unknown arg {argv[a]}")
    ex = open(path, "rb").read()
    segs = segments(ex)
    print(f"{len(segs)} segments in {path} ({len(ex)} bytes)")
    shown = 0
    for ord_, off, seg in segs:
        if index is not None and ord_ != index:
            continue
        if pat is not None and pat not in seg:
            continue
        lo = seg.find(LO_MARKER)
        print(f"\n-- segment #{ord_} @0x{off:x} len {len(seg)} body@{lo}")
        print(hexs(seg))
        shown += 1
        if shown >= limit:
            print(f"\n… stopping after {limit} (use --max)")
            break
    if shown == 0:
        print("no segment matched")


if __name__ == "__main__":
    main(sys.argv)
