#!/usr/bin/env python3
"""nothingread.py — a SECOND instrument for board #1053's seeding production,
crate-free and reaching the same answer by a different route.

Board **#971** condition 3 and `w-inl0` §7's P7: a reader and the thing that
grades it are the same code, so "it fired N times" is one instrument reporting on
itself. This one is written against the `.ex` bytes with no `c2-il` in it, and it
locates the production the *opposite* way round from the shipped Rust:

* the Rust walks a CURSOR forward from `body_start` -> `ops_start` -> the scope ->
  the statement -> `eat_return_plumbing`'s terminal, and refuses at the first byte
  outside the vocabulary;
* this ANCHORS on the statement pattern anywhere in the segment and then checks
  that what surrounds it is only the body opener and the return plumbing —
  i.e. it starts from the middle and works outwards.

Two routes that agree on a count are two routes; two spellings of the same walk
are one. If they disagree, the difference is the finding.

    nothingread.py <file.ex>
"""
import sys

FN_MARK = bytes([0x4F, 0x1F])
BODY_OPEN = bytes([0x4C, 0x4F, 0x11, 0x53])
INT_TYPE = bytes([0x86, 0x41, 0x74])
VOID_TAGKIND = bytes([0x82, 0x07])


def segments(data):
    starts = [i for i in range(len(data) - 1) if data[i : i + 2] == FN_MARK]
    return [
        data[s : (starts[k + 1] if k + 1 < len(starts) else len(data))]
        for k, s in enumerate(starts)
    ]


def varint_end(b, i):
    """End index of the LEB-ish field at `i`. `read_varint`'s two forms: a
    0x80-marked multi-byte run, or a single signed byte."""
    if i >= len(b):
        return None
    if b[i] & 0x80:
        n = b[i] & 0x7F
        return i + 1 + n if i + 1 + n <= len(b) else None
    return i + 1


def type_end(b, i):
    """End index of a TYPE token at `i`: tag, optional wide mark, kind, then a
    LEB id. Aggregates are not reachable in this production and are not modelled;
    a body that carried one would fail the tag/kind check first."""
    if i >= len(b) or not b[i] & 0x80:
        return None
    j = i + 1
    if b[i] & 0x40:
        if j >= len(b) or not b[j] & 0x80:
            return None
        j += 1
    j += 1  # kind
    while j < len(b):
        c = b[j]
        j += 1
        if not c & 0x80:
            return j
    return None


def line_markers(b, i):
    """Skip any run of `4F 01 <varint>` source-line markers."""
    while i + 1 < len(b) and b[i] == 0x4F and b[i + 1] == 0x01:
        e = varint_end(b, i + 2)
        if e is None:
            return i
        i = e
    return i


def statement_at(b, i):
    """`33 <INT_TYPE> <v> 33 82 07 <id> <v> 44 4B` at `i`; end index or None."""
    if b[i : i + 1] != b"\x33" or b[i + 1 : i + 4] != INT_TYPE:
        return None
    j = type_end(b, i + 1)
    if j is None:
        return None
    j = varint_end(b, j)
    if j is None or b[j : j + 1] != b"\x33" or b[j + 1 : j + 3] != VOID_TAGKIND:
        return None
    k = type_end(b, j + 1)
    if k is None:
        return None
    k = varint_end(b, k)
    if k is None or b[k : k + 2] != b"\x44\x4B":
        return None
    return k + 2


def is_nothing_body(seg):
    """The body opener, one statement, then plumbing that reaches the segment end.

    The plumbing is checked for its SHAPE rather than walked field by field:
    `3A <tok> 54 02 29 <tok> 4F 12 47 54 01 54 00` (optionally `4D` and zero fill
    for the last segment of a bundle). Anything else after the statement — a call,
    a second statement, a stray byte — leaves a residue and refuses.
    """
    k = seg.find(BODY_OPEN)
    if k < 0:
        return False
    i = line_markers(seg, k + len(BODY_OPEN))
    end = statement_at(seg, i)
    if end is None:
        return False
    rest = seg[line_markers(seg, end) :]
    if rest[:1] != b"\x3A":
        return False
    # `3A <tok>` then a `54`-run, `29 <tok>`, and the function tail.
    if b"\x4F\x12\x47\x54\x01\x54\x00" not in rest:
        return False
    tail = rest[rest.index(b"\x4F\x12\x47\x54\x01\x54\x00") + 7 :]
    # Either the segment ends, or the module trailer plus zero fill.
    return tail == b"" or set(tail) <= {0x4D, 0x00, 0x4F, 0x02, 0x20, 0x01} or all(
        c == 0 for c in tail
    )


def main(argv):
    data = open(argv[1], "rb").read()
    segs = segments(data)
    hits = [i for i, s in enumerate(segs) if is_nothing_body(s)]
    print(f"segments: {len(segs)}")
    print(f"NOTHING-BODY segments: {len(hits)}")
    print(f"indices: {hits}")


if __name__ == "__main__":
    main(sys.argv)
