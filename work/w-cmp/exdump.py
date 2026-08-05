#!/usr/bin/env python3
"""exdump.py — split a captured `.ex` on the `4C 4F 11` ('LO') anchor and print
each function segment as annotated hex, so a lane can read the whole body a
census window truncates to 40 bytes.

Read-only measurement tooling, outside the std-only Rust workspace on purpose,
same status as `scripts/gt_dump.py`. Nothing under `crates/` is touched.

Names come from the operand-stream tables in
`crates/c2-il/src/func/body/mod.rs` (`expr_opcode_name`, `cflow_opcode_name`) —
copied, not re-derived, so this dump cannot disagree with the census about what
a byte is called. Anything not in those tables prints as bare hex; a GUESSED
name would be exactly the lie those tables exist to prevent.

USAGE  exdump.py <file.ex> [seg-index ...]
"""
import sys

ANCHOR = bytes([0x4C, 0x4F, 0x11])

EXPR = {
    0x1F: "cmp-eq", 0x20: "cmp-ne", 0x21: "cmp-le", 0x22: "cmp-lt",
    0x23: "cmp-ge", 0x24: "cmp-gt", 0x1A: "not", 0x1B: "or-or",
    0x1C: "and-and", 0x09: "shl", 0x0A: "shr", 0x0B: "bit-and",
    0x0C: "bit-or", 0x0D: "bit-xor", 0x2C: "convert", 0x40: "intrinsic-call",
    0x66: "class-descriptor", 0x43: "ternary", 0x26: "call-in-expr",
}
CFLOW = {
    0x29: "label", 0x38: "brfalse", 0x39: "brtrue", 0x3A: "jump",
    0x3B: "switch-dispatch", 0x3C: "switch-table", 0x3D: "switch-case",
}
# Names taken from the doc comments in body/mod.rs's grammar block and from
# `readers.rs`. Only tokens whose meaning that source states are named here.
OTHER = {
    0xB9: "LOAD", 0x33: "LIT", 0x53: "SS", 0x54: "SE", 0x41: "result",
    0x4B: "stmt-end", 0x4C: "end", 0x32: "STORE", 0x30: "deref-load",
    0x4F: "stmt", 0xBD: "CALL", 0x55: "arg-sep", 0x02: "add",
    0x03: "sub", 0x04: "mul", 0x27: "off-add", 0x86: "TYPE",
}


def segments(buf):
    """Segment starts, on the census's own `4C 4F 11` anchor."""
    out, i = [], 0
    while True:
        j = buf.find(ANCHOR, i)
        if j < 0:
            break
        out.append(j)
        i = j + 1
    return out


def name(b):
    return EXPR.get(b) or CFLOW.get(b) or OTHER.get(b) or ""


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    buf = open(sys.argv[1], "rb").read()
    starts = segments(buf)
    want = [int(a) for a in sys.argv[2:]] or list(range(len(starts)))
    print(f"segments found: {len(starts)}  (a positive count, not a status)")
    for idx in want:
        if idx >= len(starts):
            print(f"seg {idx}: OUT OF RANGE ({len(starts)} segments)")
            continue
        s = starts[idx]
        e = starts[idx + 1] if idx + 1 < len(starts) else len(buf)
        seg = buf[s:e]
        print(f"\n===== seg {idx}  off 0x{s:x}  len {len(seg)}")
        for off in range(0, len(seg), 16):
            row = seg[off:off + 16]
            hx = " ".join(f"{b:02x}" for b in row)
            ann = " ".join(f"{name(b)}" for b in row if name(b))
            print(f"  {off:04x}  {hx:<48}  {ann}")


if __name__ == "__main__":
    main()
