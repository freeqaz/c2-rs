#!/usr/bin/env python3
"""symlib.py — lane w-sym. The probe model, the canon, and the loaders.

Everything here is MEASUREMENT. Nothing in `crates/` consults it; the oracle is
the real `c2.dll` under wibo, compiled at the WORKLOAD's own flags through
`cl /FAsc`.

The unit of observation is **model-free**: every producer's value is unique, so
the listing itself says which producer each instruction materialises and which
source statement each store belongs to. No allocator, no schedule and no rank
is needed to read a cell — which is what lets the producer order be scored
*conditional on the observed store order*, the quantity board #582 asks for.

    struct E { unsigned e0..e7; };
    struct M { unsigned m0..m9; E e; unsigned t0..t5; };

      symbol 0   `p->m<k>`          r3, offset 4k          (k < 10)
      symbol 1   `l.e<k>`           r3, offset 0x28 + 4k   (k < 8), `E& l = p->e`
      symbol 2   `q->m<k>`          r4, offset 4k          (k < 10)

    producers    L<j>  `1+j`                  -> li   rD, 1+j
                 A<j>  `(unsigned)&p->t<j>`   -> addi rD, r3, 0x48+4j
                 R     `(unsigned)&p->e`      -> addi rD, r3, 0x28
    non-producers  F<i> a formal, T `(unsigned)p`

Imports are by explicit file path (docs/ORDER.md §6's trap: `work/w-alloc/`,
`work/w-order2/`, `work/w-parse/` and now `work/w-sym/` all carry a `model.py`
and a `search.py`).
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


LIB = load_by_path("wsym_alloc_lib",
                   os.path.join(REPO, "work", "w-alloc", "alloc_lib.py"))

E_OFF = 0x28
T_OFF = 0x48
HDR = ("struct E { unsigned " + ",".join("e%d" % i for i in range(8)) + "; };\n"
       "struct M { unsigned " + ",".join("m%d" % i for i in range(10)) + "; "
       "E e; unsigned " + ",".join("t%d" % i for i in range(6)) + "; };\n")


# ------------------------------------------------------------------- source --
def value_expr(spec, kinds, symkind):
    """`spec` is V<j> / F<i> / T; `kinds[j]` is the producer's kind letter."""
    if spec == "T":
        return "(unsigned)p"
    if spec[0] == "F":
        return "f%s" % spec[1:]
    j = int(spec[1:])
    k = kinds[j]
    if k == "L":
        return "%d" % (1 + j)
    if k == "A":
        return "(unsigned)&p->t%d" % j
    if k == "R":
        return "(unsigned)&p->e"
    raise ValueError(k)


def store_dest(sym, k, symform):
    if sym == 0:
        return "p->m%d" % k
    if sym == 1:
        return ("p->e.e%d" % k) if symform == "direct" else ("l.e%d" % k)
    return "q->m%d" % k


def emit_cell(cid, nf, specs, kinds, syms, symform, need_q):
    params = ["M* p"]
    if need_q:
        params.append("M* q")
    params += ["unsigned f%d" % i for i in range(nf)]
    body = []
    if 1 in syms and symform != "direct":
        body.append("E& l = p->e;")
    for k, s in enumerate(specs):
        body.append("%s = %s;" % (store_dest(syms[k], k, symform),
                                  value_expr(s, kinds, need_q)))
    return "void %s(%s) { %s }\n" % (cid, ", ".join(params), " ".join(body))


def formal_reg(i, need_q):
    """r3 = p, then q, then f0, f1, … — the PPC integer argument registers."""
    return "r%d" % (4 + (1 if need_q else 0) + i)


# -------------------------------------------------------------------- canon --
def decode(ann, need_q, kinds):
    """Annotated listing -> (tokens, stores, producers).

    `stores`    the source statement indices, in EMITTED order.
    `producers` the producer indices, in EMITTED order.
    A token beginning with `?` is UNCLAIMED and disqualifies the cell.
    """
    toks, stores, prods = [], [], []
    for d in ann:
        if d["mn"] == "blr":
            continue
        if d["role"] == "store":
            off, base = d["off"], d["base"]
            if base == "r3":
                if off < E_OFF:
                    sym, k = 0, off // 4
                elif off < T_OFF:
                    sym, k = 1, (off - E_OFF) // 4
                else:
                    toks.append("?st%x" % off)
                    continue
            elif base == "r4" and need_q:
                sym, k = 2, off // 4
            else:
                toks.append("?stbase%s" % base)
                continue
            toks.append("S%d.%d@%s/%s" % (sym, k, d["src"], base))
            stores.append(k)
        elif d["role"] == "li":
            j = d["imm"] - 1
            if not (0 <= j < len(kinds)) or kinds[j] != "L":
                toks.append("?li%d" % d["imm"])
                continue
            toks.append("P%d@%s" % (j, d["dst"]))
            prods.append(j)
        elif d["role"] == "addi":
            if d["base"] == "r3" and d["imm"] == E_OFF:
                cand = [j for j, k in enumerate(kinds) if k == "R"]
            elif d["base"] == "r3" and d["imm"] >= T_OFF:
                j = (d["imm"] - T_OFF) // 4
                cand = [j] if 0 <= j < len(kinds) and kinds[j] == "A" else []
            else:
                cand = []
            if len(cand) != 1:
                toks.append("?addi%s,%x" % (d["base"], d["imm"]))
                continue
            toks.append("P%d@%s" % (cand[0], d["dst"]))
            prods.append(cand[0])
        else:
            toks.append("?" + d["mn"])
    return toks, stores, prods


# ---------------------------------------------------------------- partition --
def held_out(cid, specs, kinds, syms):
    """PREREGISTERED — docs/rungs/_2026-08-05-w-sym-prereg.md §6.

    Decided by the GENERATOR and by nothing else. The fitter never sees it.
    """
    nprod = len({s for s in specs if s[0] == "V"})
    if len(set(syms)) >= 3:
        return "arity"
    if len(set(kinds[:nprod])) > 1:
        return "kindmix"
    if nprod >= 4:
        return "prod4"
    if len(specs) > 6:
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

    Not a convention — a raise, demonstrated in `work/w-sym/raise_check.py`.
    """
    if "holdout" in os.path.basename(path).lower():
        raise RuntimeError(
            "REFUSED: %s is the preregistered HOLDOUT partition; the fitter "
            "may not open it (prereg §6)" % path)
    return read_rows_unchecked(path)


def read_rows_unchecked(path):
    if not os.path.exists(path):
        raise SystemExit("FAIL: %s absent — run work/w-sym/grid.py first" % path)
    lines = open(path).read().splitlines()
    head = lines[0].split("\t")
    return [parse_row(head, ln) for ln in lines[1:] if ln.strip()]


# ------------------------------------------------------- derived quantities --
def producers(specs):
    """-> {j: [source indices]} in first-use order."""
    out = {}
    for k, s in enumerate(specs):
        if s[0] == "V":
            out.setdefault(int(s[1:]), []).append(k)
    return out


def sched_syms(row):
    """The SCHEDULING partition.

    The `direct` control (`p->e.eK = v`) writes the same bytes at the same
    displacement through the same base register as `l.eK = v` and is ONE
    symbol — board #580. The canon labels its stores `S1.k` because it reads
    the offset, so the partition has to be collapsed here or the control would
    be scored as a two-symbol cell.
    """
    return [0] * len(row["syms"]) if row["symform"] == "direct" else row["syms"]


def global_rank(specs):
    """ORDER's rank (#561): use count DESC, first-use ASC."""
    pos = producers(specs)
    return sorted(pos, key=lambda j: (-len(pos[j]), pos[j][0]))


def grank_table(specs, syms):
    """{(sym, j): position of j among sym's producers, in the GLOBAL rank}."""
    order = global_rank(specs)
    out = {}
    for g in sorted(set(syms)):
        ps = [j for j in order
              if any(specs[k] == "V%d" % j and syms[k] == g
                     for k in range(len(specs)))]
        for i, j in enumerate(ps):
            out[(g, j)] = i
    return out
