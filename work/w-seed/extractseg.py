#!/usr/bin/env python3
"""extractseg.py — pull ONE `.ex` function segment out of a kept IL bundle, as a
Rust byte-array literal.

The unit tests in `no_effect.rs` mutate a **pinned live capture** rather than a
hand-assembled buffer, because the source-level grid cannot express a mismatched
literal type or a truncated statement: every `.cpp` perturbation changes the
statement sequence first and the walk refuses there. So the pinned array has to be
transcribed from a real `.ex`, and transcribing it by hand is how a second copy of
the capture starts rotting against the first.

Segments start at `4F 1F` (the function-start marker `eat_fn_head` opens on) and
run to the next one; the last runs to the end of the file. Selection is by INDEX,
printed with a hexdump so the caller can check it against `c2rs census --fn`'s
own window rather than trusting the count.

    extractseg.py <file.ex> [index]      # no index: list every segment
"""
import sys

MARK = bytes([0x4F, 0x1F])


def segments(data):
    starts = [i for i in range(len(data) - 1) if data[i : i + 2] == MARK]
    out = []
    for k, s in enumerate(starts):
        e = starts[k + 1] if k + 1 < len(starts) else len(data)
        out.append((s, data[s:e]))
    return out


def main(argv):
    data = open(argv[1], "rb").read()
    segs = segments(data)
    if len(argv) < 3:
        for i, (off, s) in enumerate(segs):
            print(f"[{i:3d}] off={off:5d} len={len(s):4d}  {s[:24].hex(' ')} ...")
        return
    i = int(argv[2])
    off, s = segs[i]
    print(f"// segment {i}, offset {off}, {len(s)} bytes")
    for j in range(0, len(s), 15):
        row = ", ".join(f"0x{b:02X}" for b in s[j : j + 15])
        print(f"        {row},")


if __name__ == "__main__":
    main(sys.argv)
