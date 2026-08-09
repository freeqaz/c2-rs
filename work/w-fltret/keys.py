#!/usr/bin/env python3
"""w-fltret — per-key body/emitted totals out of a `c2rs gap --jsonl` scan.

Usage:
    keys.py SCAN.jsonl [--prefix P] [--top N]
    keys.py SCAN.jsonl --diff OTHER.jsonl      # key-by-key delta, both columns

`fn_blockers` counts BODIES, `emit_blockers` counts EMITTED symbols; both are
per-TU maps of census key -> count. `fn_prod` is the production tag axis.
Nothing here assumes a total: every column is summed from the rows and the
totals are printed so a caller can assert against the scan's own summary.
"""
import json
import sys


def load(path):
    fn, em, prod, tus, tot, inclass, emit_tot, emit_in = {}, {}, {}, {}, 0, 0, 0, 0
    verdict = {}
    with open(path) as f:
        for line in f:
            r = json.loads(line)
            if r.get("record") == "provenance":
                continue
            verdict[r["src"]] = r.get("class")
            tot += r.get("fn_total") or 0
            inclass += r.get("fn_in_class") or 0
            e = r.get("emit") or {}
            emit_tot += e.get("emit-emitted") or 0
            emit_in += e.get("emit-in-class") or 0
            for k, v in (r.get("fn_blockers") or {}).items():
                fn[k] = fn.get(k, 0) + v
                tus.setdefault(k, set()).add(r["src"])
            for k, v in (r.get("emit_blockers") or {}).items():
                em[k] = em.get(k, 0) + v
            for k, v in (r.get("fn_prod") or {}).items():
                prod[k] = prod.get(k, 0) + v
    return dict(fn=fn, em=em, prod=prod, tus=tus, tot=tot, inclass=inclass,
                emit_tot=emit_tot, emit_in=emit_in, verdict=verdict)


def main():
    a = load(sys.argv[1])
    if "--diff" in sys.argv:
        b = load(sys.argv[sys.argv.index("--diff") + 1])
        print("census  %d -> %d  (%+d)" % (a["inclass"], b["inclass"],
                                           b["inclass"] - a["inclass"]))
        print("emitted %d -> %d  (%+d)" % (a["emit_in"], b["emit_in"],
                                           b["emit_in"] - a["emit_in"]))
        print("denominators: fn %d -> %d ; emit %d -> %d" %
              (a["tot"], b["tot"], a["emit_tot"], b["emit_tot"]))
        for col in ("fn", "em", "prod"):
            ka, kb = a[col], b[col]
            moved = [(k, ka.get(k, 0), kb.get(k, 0)) for k in set(ka) | set(kb)
                     if ka.get(k, 0) != kb.get(k, 0)]
            moved.sort(key=lambda t: -abs(t[2] - t[1]))
            print("\n== %s: %d keys base, %d keys tip, %d moved, %d appeared, %d vanished"
                  % (col, len(ka), len(kb), len(moved),
                     len(set(kb) - set(ka)), len(set(ka) - set(kb))))
            for k, x, y in moved:
                print("   %+8d  %8d -> %-8d %s" % (y - x, x, y, k))
        av, bv = a["verdict"], b["verdict"]
        ch = [s for s in av if av[s] != bv.get(s)]
        print("\n== per-TU verdict set: %d changed, %d only-in-base, %d only-in-tip"
              % (len(ch), len(set(av) - set(bv)), len(set(bv) - set(av))))
        for s in ch:
            print("   %s: %s -> %s" % (s, av[s], bv.get(s)))
        return

    prefix = ""
    if "--prefix" in sys.argv:
        prefix = sys.argv[sys.argv.index("--prefix") + 1]
    top = 40
    if "--top" in sys.argv:
        top = int(sys.argv[sys.argv.index("--top") + 1])
    print("census in-class %d / %d ; emitted in-class %d / %d"
          % (a["inclass"], a["tot"], a["emit_in"], a["emit_tot"]))
    rows = [(k, v, a["em"].get(k, 0), len(a["tus"][k]))
            for k, v in a["fn"].items() if k.startswith(prefix)]
    rows.sort(key=lambda t: -t[2])
    print("%10s %10s %6s  %s" % ("emitted", "bodies", "TUs", "key"))
    for k, b, e, t in rows[:top]:
        print("%10d %10d %6d  %s" % (e, b, t, k))
    print("%10d %10d %6s  TOTAL over prefix %r (%d keys)"
          % (sum(r[2] for r in rows), sum(r[1] for r in rows), "-", prefix, len(rows)))


main()
