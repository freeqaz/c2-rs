#!/usr/bin/env python3
"""rootmodel.py — `JFP_ALIAS` plus a ROOT SET, and the owner features a root
rule may read.

The incumbent is reproduced BY VALUE: `state()` recomputes exactly what
`work/w-quar/predict.py::one()` computes, from the same landed modules, and
`model(st, roots=frozenset())` is `JFP_ALIAS` -- so any lane can assert the
incumbent's digits from this file (`ka.py` does, on the fit side).

    JFP_ALIAS       fixpoint(Seed,          merged(res(ce), res(de)), U, W, skip) & U
    JFP_ALIAS_R(pi) fixpoint(Seed | R(pi),  merged(res(ce), res(de)), U, W, skip) & U

`R(pi)` is a subset of `W` (the `.in` initializer owners, i.e. the DEFINED
file-scope data objects of the TU) chosen by a truth-free predicate `pi` over the
owner's `.gl` record.  Nothing here reads the reference obj, `D`, `E`, or any
quantity derived from them.

FEATURES a predicate may read, all from the `.gl`/`.in` streams:

    cls   boundary2.kind(name).  `other (3)` is a mangled file-scope VARIABLE
          (`?x@@3<type><cv>`); `undecorated` is an `extern "C"` object.
    tag   the `.gl` record tag byte           (glowner.read_symbols)
    kind  the header kind, 1 or 4             (glowner)
    sc    the storage-class byte              (glowner)
    f4d   the kind-1 `+0x4d` byte             (glowner)
    f20b* every individual bit of the `+0x20` flag word
    cv    the mangled cv-modifier: the LAST character of a `?...@@3<type><cv>`
          name.  `A` = non-const, `B` = const.  This is the axis
          `PHASE7_PLAN.md` section 2 root clause (5) and
          `OBJ_DATA_BSS_SHAPE.md` P6 both turn on.
    nptr  how many initializer pointees the owner has, bucketed
    inU   the owner also has a tag-0x0E body record (should be rare for data)

stdlib only.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MAIN = os.environ.get("C2RS_LANEROOT")
if not MAIN:
    raise SystemExit("set C2RS_LANEROOT to the main repo root")
# the same search order w-emitp/scan.py and w-quar/predict.py set up, so every
# ambiguous module name (`scan`, `marks`, `glowner`) resolves to the same file
for _p in (os.path.join(MAIN, "work", "w-emitp"),
           os.path.join(MAIN, "work", "emitpred", "pipeline"),
           os.path.join(MAIN, "work", "w-roots"),
           os.path.join(MAIN, "work", "w-refs"),
           os.path.join(MAIN, "work", "w-mark"),
           os.path.join(MAIN, "work", "w-skip"),
           os.path.join(MAIN, "work", "w-db")):
    sys.path.insert(0, os.path.abspath(_p))
import il             # noqa: E402
import refs           # noqa: E402
import glowner        # noqa: E402
import marks as mk    # noqa: E402
import alias as al    # noqa: E402
import boundary2      # noqa: E402

WIDE_COUNT = True
cls = boundary2.kind


def base_of(entry):
    try:
        for n in os.listdir(entry):
            if n.startswith("_CL_") and n.endswith("gl"):
                return n[:-2]
    except OSError:
        pass
    return None


def fixpoint(seed, edges, U, enterable, skip):
    """w-db's JFP operator, unchanged (w-emitp/scan.py::fixpoint by value)."""
    live = set(x for x in seed if x in U or x in enterable)
    stack = list(live)
    while stack:
        a = stack.pop()
        for b in edges.get(a, ()):
            if b in live or b in skip:
                continue
            if b not in U and b not in enterable:
                continue
            live.add(b)
            stack.append(b)
    return live


def _res(edges, A):
    return dict((k, set(A.get(t, t) for t in v)) for k, v in edges.items())


def state(entry):
    """Everything the model and the features need, from the IL quintet."""
    base = base_of(entry)
    if base is None:
        return None
    glb = open(os.path.join(entry, base + "gl"), "rb").read()
    exb = open(os.path.join(entry, base + "ex"), "rb").read()
    inb = open(os.path.join(entry, base + "in"), "rb").read()

    recs, _st = refs.scan(glb, exb, wide_count=WIDE_COUNT)
    U = set(recs)
    seed = set(k for k, v in recs.items() if v["seed"])
    xskip = set(k for k, v in recs.items() if v["skip"])
    gidx = il.gl_symbol_index(glb)
    syms, _ = glowner.read_symbols(glb)
    by_name = {}
    for r in syms.values():
        by_name.setdefault(r["name"], r)

    AL, _at, _ast = al.scan(glb, shift=0)

    ce = {}
    for nm, r in recs.items():
        if not r["refs"]:
            continue
        a = set()
        for tok, cnt, _p in r["refs"]:
            f = gidx.get(tok)
            if f is None or f == nm or not cnt:
                continue
            a.add(f)
        if a:
            ce[nm] = a

    _clean, inrecs = mk.parse_records(inb)
    de = {}
    W = set()
    for _tag, _fl, ownt, toks in inrecs:
        on = gidx.get(ownt) if ownt is not None else None
        if on is None:
            continue
        W.add(on)
        acc = de.setdefault(on, set())
        for t in toks:
            n = gidx.get(t)
            if n is not None and n != on:
                acc.add(n)

    m = {}
    for k, v in _res(ce, AL).items():
        m.setdefault(k, set()).update(v)
    for k, v in _res(de, AL).items():
        m.setdefault(k, set()).update(v)

    return {"U": U, "seed": seed, "skip": xskip, "W": W, "de": de,
            "edges": m, "syms": by_name}


def live_nodes(st, roots=frozenset()):
    return fixpoint(st["seed"] | set(roots), st["edges"], st["U"], st["W"],
                    st["skip"])


def model(st, roots=frozenset()):
    return live_nodes(st, roots) & st["U"]


# ---------------------------------------------------------------- features ---
def _bucket(n):
    if n == 0:
        return "0"
    if n == 1:
        return "1"
    if n <= 4:
        return "2-4"
    if n <= 16:
        return "5-16"
    if n <= 64:
        return "17-64"
    return "65+"


def cvmod(name):
    """The cv-modifier of a `?x@@3<type><cv>` file-scope variable, or None."""
    i = name.find("@@")
    if i < 0 or name[i + 2:i + 3] != "3":
        return None
    return name[-1:] or None


def feat(st, d):
    r = st["syms"].get(d)
    f = {"cls": cls(d),
         "cv": cvmod(d) or "-",
         "nptr": _bucket(len(st["de"].get(d, ()))),
         "inU": "yes" if d in st["U"] else "no"}
    if r is None:
        f["tag"] = "none"
        f["kind"] = "none"
        f["sc"] = "none"
        f["f4d"] = "none"
        return f
    f["tag"] = "0x%02x" % r["tag"]
    f["kind"] = str(r["kind"])
    f["sc"] = "0x%02x" % (r["sc"] & 0xFF)
    f["f4d"] = "none" if r["f4d"] is None else "0x%02x" % r["f4d"]
    v = r["f20"]
    for b in range(20):
        f["f20b%02d(0x%x)" % (b, 1 << b)] = "1" if (v >> b) & 1 else "0"
    return f
