#!/usr/bin/env python3
"""instream.py — the `_CL_*in` DATA-INITIALIZER stream, decoded to its
symbol-reference nodes.

Why this stream.  c2's emit set is the least fixpoint of `flags4c |= 0x20`
(`Mark`, 0x10b276e4).  Six external call sites reach `Mark`; two of them
(0x10b98be8/0x10b98c08 in 0x10b98b00, and 0x10b98c7f in 0x10b98c0f) are driven
by 0x10b98e26, which walks the global initializer-record list at
`ds:0x10c67db4` and, for every node whose kind DWORD is 2 or 0x14, resolves
`node[+8]` as a symbol token and Marks it when the target is a function
(`[+0x30]==4`, `[+0x37] & 0x200000`, `!(+0x37 & 0x400)`).  That list is filled
from the `in` sub-stream (`mov edx,0x10b13380 ; call 0x10b7e276` at 0x10b7f311).

So the `02 <token>` node in this stream is the ADDRESS-TAKE-IN-A-DATA-INITIALIZER
edge — the one `docs/rungs/_2026-08-04-w-refs-findings.md` §3 proves the `.gl`
per-symbol reference list does not carry (it is a tag-0x0E field; a vftable is a
target of the list and a dead end in it).

GRAMMAR.  Transcribed shape, gated by exact consumption (see `parse`):

    record  := <tag=0x07> <byte> <varU owner> <i32c 0> node*
             | <tag=0x00> <varU owner> <i32c 0> node*        (the leading
                                                              __C1_<build> record)
    node    := 0x01 <i32c type> <i32c width> <value>              scalar
             | 0x02 <varU token> <i32c addend> <i32c width>       SYMBOL REFERENCE
             | 0x03 <i16c len> <len bytes>                        byte blob
             | 0x08 <i32c len>                                    zero fill

    value   := i16c                 when type != 5 and width == 2
             | i32c                 when type != 5 and width in (1, 4)
             | i64c                 when type != 5 and width == 8
             | <width raw bytes>    when type == 5   (float / double)

`type` 5 is floating point and is the one flavour stored raw — an FP constant
has no small-value bias for `i32c`'s one-byte form to exploit.  This clause was
added AFTER the first terminus-gate run (823/876 = 0.93950 clean) and BEFORE any
truth was read; the gate consults no c2 output, so it is not tuning.  With it the
gate closes (see the rung doc §1f).

The record grammar is confirmed independently by the C++ ABI structures it has
to reproduce: `_TypeDescriptor` decodes as `ptr(??_7type_info@@6B@)`, one int 0,
one blob — three fields; `_CatchableType` decodes as int 0, `ptr(??_R0…)`,
int 0, int -1, zero-fill 4, int 268, `ptr(copy ctor)` — the seven fields of
`_s__CatchableType`, with the `268` in the symbol's own decorated name landing in
`sizeOrOffset`.

`varU`, `i16c`, `i32c` are c2's own primitives at 0x10c1f91b / 0x10c1f9a6 /
0x10c1f9e9, re-used from `work/w-roots/glflags.py` unchanged.  Token spelling is
`il.read_token_var`'s, so tokens join `il.gl_symbol_index` directly.

THE KNOWN-ANSWER GATE.  `parse` is fail-closed: a wrong width or a missed escape
desyncs and the walk lands somewhere that is not a record boundary.  A file is
`clean` only when the walk consumes it to the last byte.  The gate is reported as
a count of clean files and a count of decoded reference nodes, never as a status.

stdlib only.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "emitpred", "pipeline"))
sys.path.insert(0, os.path.join(HERE, "..", "w-roots"))
import il  # noqa: E402
from glflags import i16c, i32c  # noqa: E402
from chain import i64c  # noqa: E402

SYM_NODE = 0x02
REC_TAGS = (0x00, 0x07)


def var_u_be(b, p):
    """il.read_token_var, as (value, next) — the `.gl` index's spelling."""
    t = il.read_token_var(b, p)
    if t is None:
        raise IndexError("varU past end")
    return t[0], p + t[1]


def node(b, p, out):
    k = b[p]
    p += 1
    if k == 0x01:
        t, p = i32c(b, p)
        w, p = i32c(b, p)
        if t == 5:
            if w not in (4, 8) or p + w > len(b):
                raise ValueError("fp width %d" % w)
            return p + w
        if w == 2:
            _, p = i16c(b, p)
        elif w in (1, 4):
            _, p = i32c(b, p)
        elif w == 8:
            _, p = i64c(b, p)
        else:
            raise ValueError("scalar width %d" % w)
        return p
    if k == SYM_NODE:
        tok, p = var_u_be(b, p)
        _, p = i32c(b, p)          # addend
        _, p = i32c(b, p)          # width
        out.append(tok)
        return p
    if k == 0x03:
        n, p = i16c(b, p)
        if n < 0 or p + n > len(b):
            raise ValueError("blob len")
        return p + n
    if k == 0x08:
        _, p = i32c(b, p)
        return p
    raise ValueError("node kind 0x%02x" % k)


def parse(data):
    """-> (clean, [(owner_token, [referenced_token, ...]), ...])."""
    recs = []
    p = 0
    n = len(data)
    try:
        while p < n:
            if p == n - 1 and data[p] == 0x07:
                return (True, recs)          # lone trailing tag = end of stream
            tag = data[p]
            if tag not in REC_TAGS:
                return (False, recs)
            q = p + 1
            if tag == 0x07:
                q += 1                      # the tag-0x07 flags byte
            owner, q = var_u_be(data, q)
            _, q = i32c(data, q)            # offset
            refs = []
            while q < n and data[q] not in REC_TAGS:
                q = node(data, q, refs)
            recs.append((owner, refs))
            p = q
    except (IndexError, ValueError):
        return (False, recs)
    return (True, recs)


def refs_by_name(indir):
    """(clean, {owner_name: [target_name...]}, nnodes) for one captured TU dir."""
    gl = open(os.path.join(indir, "gl"), "rb").read()
    inb = open(os.path.join(indir, "in"), "rb").read()
    idx = il.gl_symbol_index(gl)
    clean, recs = parse(inb)
    out = {}
    n = 0
    for owner, refs in recs:
        on = idx.get(owner)
        for t in refs:
            n += 1
            tn = idx.get(t)
            if tn is not None:
                out.setdefault(on, []).append(tn)
    return clean, out, n


if __name__ == "__main__":
    for d in sys.argv[1:]:
        clean, m, n = refs_by_name(d)
        tot = sum(len(v) for v in m.values())
        print("%-60s clean=%s owners=%d nodes=%d resolved=%d"
              % (os.path.basename(d), clean, len(m), n, tot))
