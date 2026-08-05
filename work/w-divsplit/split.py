#!/usr/bin/env python3
"""INSTRUMENT 1 — the BYTE-SCAN split of `expr-op-0x05` / `expr-op-0x06`.

Board #783 says the witness costs one reader: split the 4,670 on the operand
TYPE preceding the division opcode. This is that reader, written against the
`C2RS_ROW_DUMP` TSV (`crates/c2-harness/src/gap/witness.rs`), whose columns are

    src index key EMITTED|not-emitted name frame cflow eh dispatch
        production completeness hex_mark hex...

#644 says a producer is not one contiguous instruction, so this does **not**
read a byte at a fixed offset before the mark. It decodes the operand TOKEN that
ends exactly at the mark, by trying every token form the operand grammar has and
requiring the decode to land on the mark. Sites where no form lands, or where
more than one does, are **counted and printed** — never defaulted into a bucket.

The fixed-offset reader the board suggested is implemented too, as
`fixed_offset_class`, precisely so the two can be compared: PREREG R2 says it
misclassifies >= 100 sites.

Usage:  split.py rows.tsv [--mutate SHIFT]

`--mutate N` is the must-fail calibration: it shifts the decode target off the
mark by N bytes. A control that does not catch that is worthless.
"""
import sys
import collections

# Mirrors crates/c2-il/src/func/readers.rs:222,258,285.
AGGREGATE_CLASS = 0x6
TYPE_TAG_WIDE_BIT = 0x40
TYPE_WIDE_MARK_BIT = 0x80

# The TYPE kind's low nibble is the type CLASS
# (crates/c2-il/src/func/body/mod.rs:1040-1048, capture-verified vocabulary).
CLASS = {
    0x1: "int-signed",
    0x2: "int-unsigned",
    0x3: "data-pointer",
    0x4: "code-pointer",
    0x5: "REAL",
    0x6: "aggregate",
    0x7: "void",
}


def read_lit_payload(b, i):
    """The LITERAL payload, readers.rs:423 `read_varint`.

    **Not** LEB128 — a signed byte, or `0x80` + a 4-byte LE i32. Written as its
    own function because the TYPE id below IS LEB128: the container uses two
    different variable-length encodings and conflating them is what made a first
    draft of this probe report 41 sites as UNDECODABLE that are ordinary
    divisions by a struct size >= 128.
    """
    if i >= len(b):
        return None
    if b[i] == 0x80:
        if i + 4 >= len(b):
            return None
        v = int.from_bytes(bytes(b[i + 1:i + 5]), "little", signed=True)
        return (v, i + 5)
    v = b[i] - 256 if b[i] > 127 else b[i]
    return (v, i + 1)


def read_varint(b, i):
    """LEB128 — the TYPE id only (readers.rs read_type). -> (val, end)."""
    val = 0
    shift = 0
    while True:
        if i >= len(b):
            return None
        x = b[i]
        val |= (x & 0x7F) << shift
        i += 1
        if not (x & 0x80):
            return (val, i)
        shift += 7
        if shift > 28:
            return None


def read_type(b, p):
    """Mirrors readers.rs read_type. -> (tag, kind, id, width) or None."""
    if p >= len(b):
        return None
    tag = b[p]
    if not (tag & 0x80):
        return None
    i = p + 1
    if tag & TYPE_TAG_WIDE_BIT:
        if i >= len(b) or not (b[i] & TYPE_WIDE_MARK_BIT):
            return None
        i += 1
    if i >= len(b):
        return None
    kind = b[i]
    i += 1
    if (kind & 0x0F) == AGGREGATE_CLASS:
        size5 = ((tag & 0x01) << 4) | (kind >> 4)
        if size5 == 0:
            r = read_varint(b, i)
            if r is None or r[0] < 32:
                return None
            i = r[1]
    r = read_varint(b, i)
    if r is None:
        return None
    return (tag, kind, r[0], r[1] - p)


# Single-byte operators parse_expr_classed consumes with no operand of their own
# (expr.rs: 02 03 04 add/sub/mul, 09 0A shifts, 0B 0C 0D bitwise) plus the
# relational band 1F..24. An operand ENDING at the mark that is one of these is
# a nested sub-expression whose type the bytes do not carry -- reported as
# `op-result`, never guessed.
ONE_BYTE_OPS = set(range(0x02, 0x0E)) | set(range(0x1F, 0x25))


def decode_operand_ending_at(b, m):
    """Every operand token form that ends EXACTLY at index m.

    Returns a list of (start, form, tag, kind) -- length 0 (undecodable), 1
    (clean) or >1 (ambiguous). Nothing here assumes a stride.
    """
    hits = []
    lo = max(0, m - 24)
    for j in range(m - 1, lo - 1, -1):
        c = b[j]
        if c == 0xB9:  # LOAD <varint token> <TYPE>
            r = read_varint(b, j + 1)
            if r is not None:
                t = read_type(b, r[1])
                if t is not None and r[1] + t[3] == m:
                    hits.append((j, "load", t[0], t[1]))
        if c == 0x33:  # LITERAL <TYPE> <payload>
            t = read_type(b, j + 1)
            if t is not None:
                r = read_lit_payload(b, j + 1 + t[3])
                if r is not None and r[1] == m:
                    hits.append((j, "lit", t[0], t[1]))
        if c == 0x2C:  # CONVERT <TYPE> 00
            t = read_type(b, j + 1)
            if t is not None and j + 1 + t[3] + 1 == m and b[j + 1 + t[3]] == 0x00:
                hits.append((j, "convert", t[0], t[1]))
        if c == 0x27:  # byte-offset add <TYPE>
            t = read_type(b, j + 1)
            if t is not None and j + 1 + t[3] == m:
                hits.append((j, "off-add", t[0], t[1]))
        if j == m - 1 and c in ONE_BYTE_OPS:
            hits.append((j, "op-result", None, None))
    return hits


def fixed_offset_class(b, m):
    """The NAIVE reader board #783 proposed: the triple at mark-3.

    Kept so R2 can be graded. Returns (tag, kind) or None.
    """
    if m < 3:
        return None
    t = read_type(b, m - 3)
    if t is None or t[3] != 3:
        return None
    return (t[0], t[1])


def main():
    path = sys.argv[1]
    mutate = 0
    if "--mutate" in sys.argv:
        mutate = int(sys.argv[sys.argv.index("--mutate") + 1])

    n = 0
    by_key = collections.Counter()
    forms = collections.Counter()
    classes = collections.Counter()
    tagkind = collections.Counter()
    emitted_classes = collections.Counter()
    undecodable = []
    ambiguous = []
    fixed_agree = 0
    fixed_none = 0
    fixed_differs = 0
    per_key_class = collections.Counter()
    emitted_n = 0
    resolved = 0

    for line in open(path):
        f = line.rstrip("\n").split("\t")
        if len(f) < 13:
            continue
        key, emitted, mark = f[2], f[3] == "EMITTED", int(f[11])
        b = bytes(int(x, 16) for x in f[12].split())
        n += 1
        by_key[key] += 1
        if emitted:
            emitted_n += 1
        # Control: the mark must actually be the opcode the key names.
        want = int(key[-2:], 16)
        if mark >= len(b) or b[mark] != want:
            undecodable.append((f[0], f[1], "MARK-DOES-NOT-HOLD-THE-OPCODE"))
            continue
        hits = decode_operand_ending_at(b, mark + mutate)
        if not hits:
            undecodable.append((f[0], f[1], "no-token-ends-at-mark"))
            forms["UNDECODABLE"] += 1
            classes["UNDECODABLE"] += 1
            per_key_class[(key, "UNDECODABLE")] += 1
            if emitted:
                emitted_classes["UNDECODABLE"] += 1
            continue
        if len(set(hits)) > 1:
            # RESOLUTION RULE, stated rather than silent: prefer the LONGEST
            # decode (the smallest start index). A one-byte `op-result`
            # candidate at mark-1 is a false positive whenever a literal's
            # payload byte happens to land in the operator band -- the literal
            # decode explains the same bytes and more. Counted, and the count is
            # printed; if the two disagreed on the TYPE this would print as a
            # conflict instead.
            typed = set((t, k) for (_, _, t, k) in hits if k is not None)
            if len(typed) > 1:
                ambiguous.append((f[0], f[1], hits))
                forms["CONFLICT"] += 1
                classes["CONFLICT"] += 1
                per_key_class[(key, "CONFLICT")] += 1
                if emitted:
                    emitted_classes["CONFLICT"] += 1
                continue
            resolved += 1
            hits = [min(hits, key=lambda h: h[0])]
        _, form, tag, kind = hits[0]
        forms[form] += 1
        if kind is None:
            cls = "op-result"
        else:
            cls = CLASS.get(kind & 0x0F, "unknown-class-%x" % (kind & 0x0F))
            tagkind["%02X%02X" % (tag, kind)] += 1
        classes[cls] += 1
        per_key_class[(key, cls)] += 1
        if emitted:
            emitted_classes[cls] += 1
        # The naive fixed-offset reader, graded against the honest one.
        fo = fixed_offset_class(b, mark)
        if fo is None:
            fixed_none += 1
        elif kind is not None and fo == (tag, kind):
            fixed_agree += 1
        else:
            fixed_differs += 1

    print("rows: %d   (mutation shift: %+d)" % (n, mutate))
    for k, v in sorted(by_key.items()):
        print("  key %-16s %d" % (k, v))
    print("emitted rows: %d of %d" % (emitted_n, n))
    print()
    print("OPERAND TOKEN FORM ending at the opcode (denominator %d):" % n)
    for k, v in forms.most_common():
        print("  %-14s %6d  %5.1f%%" % (k, v, 100.0 * v / n))
    print()
    print("OPERAND TYPE CLASS (denominator %d):" % n)
    for k, v in classes.most_common():
        print("  %-14s %6d  %5.1f%%" % (k, v, 100.0 * v / n))
    print()
    print("per key x class:")
    for (k, c), v in sorted(per_key_class.items()):
        print("  %-16s %-14s %6d" % (k, c, v))
    print()
    print("EMITTED subset by class (denominator %d):" % emitted_n)
    for k, v in emitted_classes.most_common():
        print("  %-14s %6d" % (k, v))
    print()
    print("distinct (tag,kind) pairs: %d" % len(tagkind))
    for k, v in tagkind.most_common():
        print("  %-6s %6d" % (k, v))
    print()
    print("CONTROLS")
    print("  undecodable sites : %d" % len(undecodable))
    print("  multi-candidate sites resolved by longest-match : %d" % resolved)
    print("  sites where two candidates disagreed on the TYPE : %d"
          % classes.get("CONFLICT", 0))

    print("  fixed-offset(mark-3) agrees with the decoded token : %d" % fixed_agree)
    print("  fixed-offset(mark-3) finds no 3-byte TYPE          : %d" % fixed_none)
    print("  fixed-offset(mark-3) reads a DIFFERENT type        : %d" % fixed_differs)
    print("  => a fixed-offset reader is wrong or blind on %d of %d sites (%.1f%%)"
          % (fixed_none + fixed_differs, n, 100.0 * (fixed_none + fixed_differs) / n))
    for s in undecodable[:10]:
        print("  UNDECODABLE %s #%s %s" % s)
    for s in ambiguous[:10]:
        print("  CONFLICT    %s #%s %s" % s)


if __name__ == "__main__":
    main()
