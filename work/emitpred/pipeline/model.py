#!/usr/bin/env python3
"""model.py — the emit-set model: candidate universe, reference graph, roots,
least-fixpoint closure.

Everything here reads ONLY c1xx-side artifacts: the `_CL_*` IL quintet from a
`/Bd /d2nop` front-end-only capture (c2 never runs) and `/Wall` stderr.
"""
import bisect
import struct

import il

SEPARATORS = (0x00, 0x26)
MAX_NAME_TO_OFFSET = 64


def indexable_runs(glb):
    """Separator-aware `.gl` name runs, restricted to the runs that can BE a
    symbol name (`gl.rs::is_indexable_name`: >= 3 chars, symbol alphabet, opens
    with `?`/alpha/`_`).

    The length+alphabet filter is the fix for the `records_nameless` population:
    without it a 1-byte junk run (e.g. the `70` attribute byte inside an
    out-of-line *virtual*'s record) sits between the record's real name and its
    offset field and steals the binding, which is why out-of-line virtuals were
    missing from the framed-record name set.
    """
    out = []
    n = len(glb)
    i = 0
    while i < n:
        if glb[i] not in SEPARATORS:
            i += 1
            continue
        s = i + 1
        e = s
        while e < n and glb[e] not in SEPARATORS:
            e += 1
        if e >= n or e == s:
            i += 1
            continue
        b = glb[s:e]
        if (
            len(b) >= 3
            and (b[0] == 0x3F or (0x41 <= b[0] <= 0x5A) or (0x61 <= b[0] <= 0x7A) or b[0] == 0x5F)
            and all(c in il.SYMBOL_CHARS for c in b)
        ):
            out.append((s, e, b.decode("latin1"), glb[i]))
            i = e
        else:
            i += 1
    return out


def named_bodies(glb, exb):
    """{`.ex` body offset: name} for every `.gl` record whose `80 <LE32>` field
    holds a real `4F 1F` body-start offset.

    The `.ex` segment set is the known-answer gate: a `80 <LE32>` that does not
    land exactly on a body start is not an offset field, so no name is bound to
    it. That replaces the fixed `80 XX 10 00 00 00 00` framing, which only
    matched one of the record shapes c1xx writes (the plain one) and rejected
    the out-of-line-virtual shape `80 XX 24 00 00 70 00`.
    """
    starts = set(il.split_ex(exb))
    runs = indexable_runs(glb)
    ends = [r[1] for r in runs]
    named = {}
    n = len(glb)
    o = 0
    while o < n - 5:
        if glb[o] != 0x80:
            o += 1
            continue
        v = struct.unpack_from("<I", glb, o + 1)[0]
        if v in starts:
            k = bisect.bisect_right(ends, o)
            if k >= 1 and o - ends[k - 1] <= MAX_NAME_TO_OFFSET:
                named.setdefault(v, runs[k - 1][2])
        o += 1
    return named


def ref_graph(glb, exb, named):
    """{name: {referenced names}} — the ODR-use edges, over-approximated.

    Every byte position inside a body segment is read as a variable-width
    operand token (`readers.rs::read_token_var`); a token the `.gl` symbol index
    resolves is taken as a reference. This is an over-approximation (a token
    value can occur by accident inside an unrelated operand) and deliberately
    so: the fixpoint must not lose an edge, per PHASE7_PLAN §2's direction that
    references from *removed* definitions never count but references from kept
    ones always do.

    Unnamed segments are folded into the *nearest preceding named* segment: a
    body c1xx did not give a `.gl` record is still code that a named body owns
    (statement-expression / EH funclet bodies), and dropping it would break
    edges out of the named body.
    """
    idx = il.gl_symbol_index(glb)
    segs = il.segments(exb)
    out = {}
    owner = None
    for (s, e) in segs:
        nm = named.get(s)
        if nm is not None:
            owner = nm
        if owner is None:
            continue
        acc = out.setdefault(owner, set())
        for p in range(s, e):
            t = il.read_token_var(exb, p)
            if t is not None:
                nm2 = idx.get(t[0])
                if nm2 is not None:
                    acc.add(nm2)
    return out


def closure(roots, graph, universe):
    """Least fixpoint: everything reachable from `roots` through `graph`,
    intersected with `universe` (the names that have a body in this TU)."""
    seen = set(r for r in roots if r in universe)
    work = list(seen)
    while work:
        x = work.pop()
        for y in graph.get(x, ()):
            if y in universe and y not in seen:
                seen.add(y)
                work.append(y)
    return seen
