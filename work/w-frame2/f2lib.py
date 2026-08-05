#!/usr/bin/env python3
"""f2lib.py — lane w-frame2. The LAYOUT lane's model, canon and loaders.

Everything here is MEASUREMENT. Nothing in `crates/` consults it; the oracle is
the real `c2.dll` under wibo, compiled at the WORKLOAD's own flags through
`cl /FAsc`.

The observation unit is `w-sym`'s and is deliberately unchanged: every
producer's value is unique, so the listing itself says which producer each
instruction materialises and which source statement each store belongs to. The
LAYOUT is then readable directly — the number of stores already emitted when a
producer instruction appears.

**Imports are by explicit file path.** `work/w-alloc/`, `work/w-order2/`,
`work/w-parse/`, `work/w-sym/` and now `work/w-frame2/` all carry a `model.py`
and/or a `search.py`; a bare `import model` resolves by `sys.path` order and
silently picks up another lane's module, surfacing as a MISSING ATTRIBUTE rather
than as a wrong module (docs/ORDER.md §6).
"""
import hashlib
import importlib.util
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(W, "..", ".."))


def load_by_path(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec)
    sys.modules[name] = m
    spec.loader.exec_module(m)
    return m


# `w-sym`'s probe model, by explicit path. Reused rather than re-spelled so the
# two lanes' cells are the same cells: same struct, same offsets, same canon.
SYM = load_by_path("wframe2_symlib", os.path.join(REPO, "work", "w-sym", "symlib.py"))
LIB = SYM.LIB                       # work/w-alloc/alloc_lib.py — the /FAsc seam
HDR = SYM.HDR
emit_cell = SYM.emit_cell
decode = SYM.decode
producers = SYM.producers
global_rank = SYM.global_rank


# ---------------------------------------------------------------- partition --
def held_out(cid, specs, kinds, syms, symform):
    """PREREGISTERED — docs/rungs/_2026-08-05-w-frame2-prereg.md §4.

    Decided by the GENERATOR from the cell's own description, before any
    listing is read. The fitter never sees it.
    """
    nprod = len({s for s in specs if s[0] == "V"})
    if len(set(syms if symform != "direct" else [0])) >= 3:
        return "arity"
    if len(set(kinds[:nprod])) > 1:
        return "kindmix"
    if nprod >= 3:
        return "prod3"
    if len(specs) >= 7:
        return "long"
    if hashlib.md5(cid.encode()).hexdigest()[0] in "012345":
        return "hash"
    return None


# ------------------------------------------------------------------- rows ----
FIELDS = ("cid tier nf specs kinds syms symform needq part "
          "emitted stores prods unclaimed").split()


class Row(dict):
    @property
    def n(self):
        return len(self["specs"])


def parse_row(head, line):
    f = dict(zip(head, line.split("\t")))
    r = Row(f)
    r["specs"] = f["specs"].split(",")
    r["kinds"] = list(f["kinds"])
    r["syms"] = [int(c) for c in f["syms"]]
    r["nf"] = int(f["nf"])
    r["needq"] = f["needq"] == "1"
    r["stores"] = [int(x) for x in f["stores"].split(",")] if f["stores"] else []
    r["prods"] = [int(x) for x in f["prods"].split(",")] if f["prods"] else []
    return r


def read_rows(path):
    """Load a table. **RAISES on any path containing `holdout`.**

    Not a convention — a raise, demonstrated on four spellings in
    `work/w-frame2/raise_check.py`.
    """
    if "holdout" in os.path.basename(path).lower():
        raise RuntimeError(
            "REFUSED: %s is the preregistered HOLDOUT partition; the fitter "
            "may not open it (prereg §4)" % path)
    return read_rows_unchecked(path)


def read_rows_unchecked(path):
    if not os.path.exists(path):
        raise SystemExit("FAIL: %s absent — run work/w-frame2/grid.py first" % path)
    lines = open(path).read().splitlines()
    head = lines[0].split("\t")
    return [parse_row(head, ln) for ln in lines[1:] if ln.strip()]


# ------------------------------------------------------- derived quantities --
def sched_syms(row):
    """The SCHEDULING partition — board #580's `direct` control collapses."""
    return [0] * len(row["syms"]) if row["symform"] == "direct" else row["syms"]


def observed_layout(row):
    """-> [(number of stores already emitted, producer id)], in emitted order."""
    out, q = [], 0
    for t in row["emitted"].split():
        if t[0] == "S":
            q += 1
        elif t[0] == "P":
            out.append((q, int(t[1:].split("@")[0])))
    return out


def observed_slots(row):
    """-> [slot], indexed by PRODUCER EMISSION ORDER. The quantity #602 is."""
    return [q for q, _ in observed_layout(row)]


# ------------------------------------------------------------------ u terms --
BLOCK = 2


def u_count(row):
    """`min(2, #unproduced)` — the reading `order::schedule` ships."""
    return min(BLOCK, sum(1 for s in row["specs"] if s[0] != "V"))


def u_lead(row):
    """The leading run of unproduced stores in the FINAL store order, capped at
    2 — `w-parse`'s #584 correction, and the reading `w-sym`'s layout.py
    scored best."""
    u = 0
    for k in row["stores"]:
        if u >= BLOCK or row["specs"][k][0] == "V":
            break
        u += 1
    return u


def u_walk(row):
    """The `u` `order::store_order` actually selects: the LARGEST value whose
    walk succeeds. Recomputed here from the row so the term is available to the
    search without consulting the port."""
    syms = sched_syms(row)
    specs = row["specs"]
    rank = global_rank(specs)
    pos = producers(specs)

    def grank(k):
        if specs[k][0] != "V":
            return 0
        j = int(specs[k][1:])
        ps = [x for x in rank
              if any(specs[m] == "V%d" % x and syms[m] == syms[k]
                     for m in range(len(specs)))]
        return ps.index(j)

    ranks = [grank(k) for k in range(len(specs))]
    for u in range(min(BLOCK, sum(1 for s in specs if s[0] != "V")), -1, -1):
        left = list(range(len(specs)))
        out = []
        ok = True
        while left:
            q = len(out)
            pick = None
            for i, k in enumerate(left):
                if specs[k][0] == "V" and q < u + ranks[k]:
                    continue
                if any(syms[j] != syms[k] for j in left[:i]):
                    continue
                pick = i
                break
            if pick is None:
                ok = False
                break
            out.append(left.pop(pick))
        if ok:
            return u
    return 0
    _ = pos


# ------------------------------------------------------- per-producer terms --
def producer_terms(row):
    """One dict of integer features per producer, keyed by producer id.

    Every term is computed from the SOURCE description plus the OBSERVED store
    order — never from the observed layout, which is the quantity being
    predicted.
    """
    specs, syms, order = row["specs"], sched_syms(row), row["stores"]
    # position of each source statement in the final store order
    place = {k: q for q, k in enumerate(order)}
    groups = {}
    for q, k in enumerate(order):
        groups.setdefault(syms[k], []).append(q)
    grp_first = {}
    for q, k in enumerate(order):
        grp_first.setdefault(syms[k], q)
    rank = global_rank(specs)

    out = {}
    for j, uses in producers(specs).items():
        fc = min(place[k] for k in uses)                   # first consumption
        g = syms[[k for k in uses if place[k] == fc][0]]   # its symbol group
        fcg = groups[g].index(fc)                          # within its group
        ps = [x for x in rank
              if any(specs[m] == "V%d" % x and syms[m] == g
                     for m in range(len(specs)))]
        gr = ps.index(j)
        nsw = sum(1 for q in range(1, fc + 1)
                  if syms[order[q]] != syms[order[q - 1]])
        out[j] = dict(fc=fc, fcg=fcg, grank=gr, nsw=nsw,
                      gfirst=grp_first[g],
                      gidx=sorted(grp_first, key=lambda s: grp_first[s]).index(g))
    return out
