#!/usr/bin/env python3
"""grid3.py — lane w-parse. WHERE `mr rN,r3` GOES (prereg R4).

`docs/STORE_SCHEDULE.md` §5 records `xboxheap.cpp`'s `mr r31,r3` — the
live-range save of `this` across the trailing call — as **one fact, n = 1, a
hypothesis and not a rule**, sitting between the third and fourth store.  This
is its grid.

The probe is a member function `M* f(...) { <store run>; h(f0); return this; }`
rather than a constructor.  That substitution is not free and is CHECKED, not
assumed: `work/w-parse/probe1.cpp` compiles the two side by side and the
constructor's body and the member function's body are byte-identical, all
twenty instructions, `mr` slot included.  Using member functions is what lets
each cell carry its own name through the `.cod` listing.

Axes: the produced word, the fillers and their identity (`this` or a formal),
the base-symbol partition (a bound reference, as in `xboxheap`), and the number
of distinct producers.

Imports by explicit file path (docs/ORDER.md §6's trap).
"""
import hashlib
import importlib.util
import itertools
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(W, "..", ".."))


def load_by_path(name, path):
    s = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(s)
    sys.modules[name] = m
    s.loader.exec_module(m)
    return m


LIB = load_by_path("wparse_alloc_lib3",
                   os.path.join(REPO, "work", "w-alloc", "alloc_lib.py"))
G1 = load_by_path("wparse_grid1c", os.path.join(W, "grid.py"))

E_OFF, T_OFF = G1.E_OFF, G1.T_OFF

HDR_TOP = (
    "struct E { unsigned " + ",".join("e%d" % i for i in range(8)) + "; };\n"
    "struct M { unsigned " + ",".join("m%d" % i for i in range(10)) + "; "
    "E e; unsigned " + ",".join("t%d" % i for i in range(4)) + ";\n"
    "  void h(unsigned);\n")


def value_expr(spec, kinds):
    """As grid.py, but inside a member function `this` replaces `p`."""
    if spec == "T":
        return "(unsigned)this"
    if spec[0] == "F":
        return "f%s" % spec[1:]
    j = int(spec[1:])
    k = kinds[j]
    if k == "L":
        return "%d" % (1 + j)
    if k == "A":
        return "(unsigned)&t%d" % j
    if k == "R":
        return "(unsigned)&e"
    raise ValueError(k)


def build_cells():
    cells = {}

    def add(cid, nf, specs, kinds, syms):
        if cid in cells:
            return
        cells[cid] = (nf, specs, kinds, syms)

    for base in ("", "0", "01", "011", "001", "010", "0011", "0110"):
        for k in range(0, 4):
            n = len(base) + k
            if n == 0 or n > 6:
                continue
            for slots in itertools.combinations(range(n), k):
                for fkind in ("F", "T"):
                    for sym in (0, 1):
                        specs, wi, fi = [], 0, 0
                        for i in range(n):
                            if i in slots:
                                if fkind == "F":
                                    specs.append("F%d" % (fi + 1))
                                    fi += 1
                                else:
                                    specs.append("T")
                            else:
                                specs.append("V" + base[wi])
                                wi += 1
                        last = "V" + base[-1] if base else None
                        syms = [1 if (sym and s == last) else 0 for s in specs]
                        if sym and 1 not in syms:
                            continue
                        cid = "c_%s_%s%s_s%d" % (
                            base or "z",
                            "".join("f" if i in slots else "v"
                                    for i in range(n)), fkind, sym)
                        add(cid, fi + 1, specs, ["L", "L"], syms)
    # xboxheap's own word with the two producer kinds it actually has
    add("c_xbox", 2, ["F1", "T", "V0", "T", "V1", "V1"], ["L", "R"],
        [0, 0, 0, 0, 1, 1])
    add("c_xbox_1sym", 2, ["F1", "T", "V0", "T", "V1", "V1"], ["L", "R"],
        [0, 0, 0, 0, 0, 0])
    return cells


def emit(cid, nf, specs, kinds, syms):
    args = ", ".join("unsigned f%d" % i for i in range(nf))
    body = []
    if any(syms):
        body.append("E& l = e;")
    for k, s in enumerate(specs):
        dest = ("l.e%d" % k) if syms[k] else ("m%d" % k)
        body.append("%s = %s;" % (dest, value_expr(s, kinds)))
    body.append("h(f0);")
    body.append("return this;")
    return ("  M* %s(%s);\n" % (cid, args),
            "M* M::%s(%s) { %s }\n" % (cid, args, " ".join(body)))


def canon(ann):
    toks = []
    for d in ann:
        mn = d["mn"]
        if mn in ("blr", "mflr", "mtlr", "stwu", "std", "ld", "bl") or \
                (mn in ("stw", "lwz", "addi") and d.get("base") == "r1"):
            continue
        if d["role"] == "store" and d.get("base") == "r3":
            off = d["off"]
            if off < E_OFF:
                toks.append("S0.%d@%s" % (off // 4, d["src"]))
            elif off < T_OFF:
                toks.append("S1.%d@%s" % ((off - E_OFF) // 4, d["src"]))
            else:
                toks.append("?off%x" % off)
        elif d["role"] == "mr" and d.get("dst") == "r31":
            toks.append("MR")
        elif d["role"] == "mr" and d.get("dst") == "r3":
            toks.append("RET")
        elif d.get("dst"):
            toks.append("P%s" % d["dst"])
        else:
            toks.append("?" + mn)
    return toks


def main():
    cells = build_cells()
    decls, defs = [], []
    for cid, (nf, specs, kinds, syms) in cells.items():
        d, b = emit(cid, nf, specs, kinds, syms)
        decls.append(d)
        defs.append(b)
    src = os.path.join(W, "grid3.cpp")
    with open(src, "w") as f:
        f.write(HDR_TOP)
        f.writelines(decls)
        f.write("};\n")
        f.writelines(defs)
    txt = LIB.compile_cod(src, os.path.join(W, "grid3.cod"),
                          os.path.join(W, "grid3.obj"))
    fns = LIB.parse_cod(txt)
    got = {k.split("::")[-1]: v for k, v in fns.items()}
    missing = [c for c in cells if c not in got]
    if missing:
        raise SystemExit("FAIL: %d of %d cells produced no PROC: %s"
                         % (len(missing), len(cells), missing[:5]))
    rows, nbad = [], 0
    for cid, (nf, specs, kinds, syms) in sorted(cells.items()):
        toks = canon(LIB.classify(got[cid]))
        bad = [t for t in toks if t.startswith("?")]
        nbad += bool(bad)
        if "MR" not in toks:
            bad.append("noMR")
        rows.append("\t".join([cid, str(nf), ",".join(specs), "".join(kinds),
                               "".join(str(s) for s in syms), " ".join(toks),
                               ";".join(bad)]))
    hdr = "cid\tnf\tspecs\tkinds\tsyms\temitted\tunclaimed\n"
    open(os.path.join(W, "mr.tsv"), "w").write(hdr + "\n".join(rows) + "\n")
    print("cells generated : %d" % len(cells))
    print("PROCs parsed    : %d" % len(got))
    print("cells with an UNCLAIMED token or no MR: %d" % nbad)
    print("wrote work/w-parse/mr.tsv")


if __name__ == "__main__":
    main()
