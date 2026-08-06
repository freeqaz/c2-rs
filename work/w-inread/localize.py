#!/usr/bin/env python3
"""localize.py — WHERE in the corpus the unmeasured element kinds live, so the
cells can be aimed instead of guessed.

**A DRIVER, NOT EVIDENCE.**  Nothing this prints is a grammar fact.  It reads
the workload's `.in` streams with `work/w-emitp2/strictin.py`'s sequential
parser, keys each record's owner token to its `.gl` name through
`il.gl_symbol_index` (#918: the per-record binding, never a positional name),
and reports which SYMBOL FAMILIES carry a scalar type `03`, a scalar type `04`
or an element tag `08`, together with the VALUE distribution of each.

The grammar is decided by frozen cells graded against real `c2` and by nothing
else.  This tells me which C++ to write.

    usage: localize.py <cacheidx.tsv> [jobs] [limit]

stdlib only.  Reads no c2 output.
"""
import collections
import os
import re
import struct
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
MAIN = os.environ.get("C2RS_LANEROOT", os.path.abspath(os.path.join(HERE, "..", "..")))
sys.path.insert(0, os.path.join(MAIN, "work", "w-emitp2"))
sys.path.insert(0, os.path.join(MAIN, "work", "emitpred", "pipeline"))
sys.path.insert(0, os.path.join(MAIN, "work", "w-roots"))
sys.path.insert(0, os.path.join(MAIN, "work", "w-mark"))
import il                        # noqa: E402
import instream                  # noqa: E402
from glflags import i16c, i32c   # noqa: E402
from chain import i64c           # noqa: E402

REC_TAGS = instream.REC_TAGS
SYM = 0x02


class Desync(Exception):
    pass


def node_v(b, p, elems):
    """One element, appended as (tag, a, b, value) — the VALUE is kept."""
    k = b[p]
    p += 1
    if k == 0x01:
        t, p = i32c(b, p)
        w, p = i32c(b, p)
        if t == 5:
            if w not in (4, 8) or p + w > len(b):
                raise Desync("fp-width")
            elems.append((0x01, t, w, None))
            return p + w
        if w == 2:
            v, p = i16c(b, p)
        elif w in (1, 4):
            v, p = i32c(b, p)
        elif w == 8:
            v, p = i64c(b, p)
        else:
            raise Desync("scalar-width")
        elems.append((0x01, t, w, v))
        return p
    if k == SYM:
        tok, p = instream.var_u_be(b, p)
        off, p = i32c(b, p)
        w, p = i32c(b, p)
        elems.append((SYM, tok, w, off))
        return p
    if k == 0x03:
        n, p = i16c(b, p)
        if n < 0 or p + n > len(b):
            raise Desync("blob-len")
        elems.append((0x03, 0, n, bytes(b[p:p + n])))
        return p + n
    if k == 0x08:
        v, p = i32c(b, p)
        elems.append((0x08, 0, 0, v))
        return p
    raise Desync("element-tag-%02x" % k)


def parse_v(data):
    recs = []
    p = 0
    n = len(data)
    try:
        while p < n:
            if p == n - 1 and data[p] == 0x07:
                break
            tag = data[p]
            if tag not in REC_TAGS:
                break
            q = p + 1
            if tag == 0x07:
                q += 1
            owner, q = instream.var_u_be(data, q)
            _, q = i32c(data, q)
            elems = []
            while q < n and data[q] not in REC_TAGS:
                q = node_v(data, q, elems)
            recs.append((owner, elems))
            p = q
    except (Desync, IndexError, ValueError, struct.error):
        pass
    return recs


def member(entry, sfx):
    for n in os.listdir(entry):
        if n.startswith("_CL_") and n.endswith(sfx):
            return os.path.join(entry, n)
    return None


def family(name):
    """A coarse bucket for a decorated name — the cell-authoring axis."""
    for pfx in ("??_R0", "??_R1", "??_R2", "??_R3", "??_R4", "??_7", "??_8",
                "??_C@", "__CT??", "__TI", "_CT??", "_TI", "??_S", "??_9"):
        if name.startswith(pfx):
            return pfx
    if name.startswith("??_"):
        return "??_other"
    if name.startswith("?"):
        return "?ordinary-data"
    return "undecorated"


def one(row):
    src, entry = row[0], row[1]
    inp, glp = member(entry, "in"), member(entry, "gl")
    if inp is None or glp is None:
        return None
    idx = il.gl_symbol_index(open(glp, "rb").read())
    recs = parse_v(open(inp, "rb").read())
    r = {"t03_fam": collections.Counter(), "t04_fam": collections.Counter(),
         "e08_fam": collections.Counter(),
         "t03_val": collections.Counter(), "t04_val": collections.Counter(),
         "e08_val": collections.Counter(),
         "t03_ex": [], "t04_ex": [], "e08_ex": [],
         "t03_w": collections.Counter(), "t04_w": collections.Counter(),
         "t04_pos": collections.Counter(), "t03_pos": collections.Counter(),
         "e08_pos": collections.Counter(), "e08_last": 0, "e08_n": 0}
    for owner, el in recs:
        nm = idx.get(owner)
        fam = family(nm) if nm else "UNRESOLVED"
        for i, (k, a, w, v) in enumerate(el):
            if k == 0x01 and a == 0x03:
                r["t03_fam"][fam] += 1
                r["t03_val"][v if abs(v or 0) < 1 << 20 else "big"] += 1
                r["t03_w"][w] += 1
                r["t03_pos"][(i, len(el))] += 1
                if nm and len(r["t03_ex"]) < 4:
                    r["t03_ex"].append((nm, [(e[0], e[1], e[2], e[3])
                                             for e in el][:12]))
            elif k == 0x01 and a == 0x04:
                r["t04_fam"][fam] += 1
                r["t04_val"][v if abs(v or 0) < 1 << 20 else "big"] += 1
                r["t04_w"][w] += 1
                r["t04_pos"][(i, len(el))] += 1
                if nm and len(r["t04_ex"]) < 6:
                    r["t04_ex"].append((nm, [(e[0], e[1], e[2], e[3])
                                             for e in el][:12]))
            elif k == 0x08:
                r["e08_fam"][fam] += 1
                r["e08_val"][v if abs(v or 0) < 1 << 16 else "big"] += 1
                r["e08_pos"][(i, len(el))] += 1
                r["e08_n"] += 1
                if i == len(el) - 1:
                    r["e08_last"] += 1
                if nm and len(r["e08_ex"]) < 4:
                    r["e08_ex"].append((nm, [(e[0], e[1], e[2], e[3])
                                             for e in el][:12]))
    return r


def fmt_el(el):
    out = []
    for k, a, w, v in el:
        if k == 0x01:
            out.append("01 t%02x w%d = %s" % (a, w, v))
        elif k == SYM:
            out.append("02 tok=%04x off=%s n=%s" % (a, v, w))
        elif k == 0x03:
            out.append("03 len=%d %r" % (w, v[:20]))
        else:
            out.append("08 n=%s" % v)
    return " | ".join(out)


def main():
    idxp = sys.argv[1]
    jobs = int(sys.argv[2]) if len(sys.argv) > 2 else 8
    limit = int(sys.argv[3]) if len(sys.argv) > 3 else 0
    rows = [l.rstrip("\n").split("\t") for l in open(idxp)]
    if limit:
        rows = rows[:limit]
    agg = {k: collections.Counter() for k in
           ("t03_fam", "t04_fam", "e08_fam", "t03_val", "t04_val", "e08_val",
            "t03_w", "t04_w", "t03_pos", "t04_pos", "e08_pos")}
    ex = {"t03_ex": [], "t04_ex": [], "e08_ex": []}
    e08_last = e08_n = 0
    with cf.ProcessPoolExecutor(max_workers=jobs) as pool:
        for r in pool.map(one, rows, chunksize=8):
            if r is None:
                continue
            for k in agg:
                agg[k].update(r[k])
            for k in ex:
                if len(ex[k]) < 12:
                    ex[k].extend(r[k][:2])
            e08_last += r["e08_last"]
            e08_n += r["e08_n"]

    for lbl, fk, vk, wk, pk in (
            ("SCALAR TYPE 03", "t03_fam", "t03_val", "t03_w", "t03_pos"),
            ("SCALAR TYPE 04", "t04_fam", "t04_val", "t04_w", "t04_pos"),
            ("ELEMENT TAG 08", "e08_fam", "e08_val", None, "e08_pos")):
        print("== %s ==" % lbl)
        print("  total %d" % sum(agg[fk].values()))
        print("  by owner family:")
        for k, v in agg[fk].most_common(12):
            print("    %-18s %8d" % (k, v))
        print("  by value (top 12):")
        for k, v in agg[vk].most_common(12):
            print("    %-18s %8d" % (k, v))
        if wk:
            print("  by width: %s" % dict(agg[wk]))
        print("  by (element index, record arity), top 8:")
        for k, v in agg[pk].most_common(8):
            print("    idx=%-3d arity=%-3d  %8d" % (k[0], k[1], v))
        print()
    print("tag-08 elements that are the LAST element of their record: %d of %d"
          % (e08_last, e08_n))
    print()
    for k, lbl in (("t03_ex", "SCALAR TYPE 03"), ("t04_ex", "SCALAR TYPE 04"),
                   ("e08_ex", "ELEMENT TAG 08")):
        print("== SAMPLE RECORDS CARRYING %s ==" % lbl)
        for nm, el in ex[k][:10]:
            print("  %s" % nm[:110])
            print("      %s" % fmt_el(el)[:400])
        print()


if __name__ == "__main__":
    main()
