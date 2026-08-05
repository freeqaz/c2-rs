#!/usr/bin/env python3
"""layout.py — lane w-sym. The THIRD component, isolated.

A store run's emission is three separable facts:

    1. the STORE order          (`model.py`, SO_RULES)
    2. the PRODUCER order       (`model.py`, PO_RULES — board #582)
    3. the LAYOUT: at which store slots the producers are interleaved

`docs/ORDER.md` §1's layout clause, with `w-parse`'s #584 correction:

  > let `u` = the length of the LEADING RUN of unproduced stores in the final
  > store order, capped at 2. The first `u` producers go one apiece
  > immediately before store slots `0 … u-1`; every remaining producer is
  > emitted CONTIGUOUSLY immediately before store slot `u`.

Scored here **given the observed store order AND the observed producer order**,
so it is the layout alone. Also scored: `u = min(2, #unproduced)`, the reading
#584 replaced.

RAISES on any path containing `holdout`.
"""
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, W)
import symlib as S  # noqa: E402

BLOCK = 2


def observed_layout(row):
    """-> [(store slot before which this producer sits, producer id)]."""
    out, q = [], 0
    for t in row["emitted"].split():
        if t[0] == "S":
            q += 1
        elif t[0] == "P":
            out.append((q, int(t[1:].split("@")[0])))
    return out


def u_leading(row):
    u = 0
    for k in row["stores"]:
        if u >= BLOCK or row["specs"][k][0] == "V":
            break
        u += 1
    return u


def u_count(row):
    return min(BLOCK, sum(1 for s in row["specs"] if s[0] != "V"))


def predicted_layout(row, prods, u):
    out = []
    for i, j in enumerate(prods):
        out.append((i if i < u else u, j))
    return out


def main():
    argv = sys.argv[1:]
    if "--holdout" in argv:
        rows = S.read_rows_unchecked(os.path.join(W, "holdout.tsv"))
        label = "HOLDOUT"
    elif "--external" in argv:
        rows = S.read_rows_unchecked(os.path.join(W, "external.tsv"))
        label = "EXTERNAL"
    else:
        rows = S.read_rows(os.path.join(W, "fit.tsv"))
        label = "FIT"
    n = nm = 0
    hit = {"leading": [0, 0], "count": [0, 0]}
    miss = []
    for r in rows:
        if not S.producers(r["specs"]):
            continue
        n += 1
        multi = len(set(S.sched_syms(r))) > 1
        nm += multi
        obs = observed_layout(r)
        for name, u in (("leading", u_leading(r)), ("count", u_count(r))):
            ok = predicted_layout(r, r["prods"], u) == obs
            hit[name][0] += ok
            hit[name][1] += ok and multi
            if name == "leading" and not ok:
                miss.append((r, obs, predicted_layout(r, r["prods"], u)))
    print("== LAYOUT, %s ==  %d cells with a producer (%d multi-symbol)"
          % (label, n, nm))
    for name in ("leading", "count"):
        print("   u = %-8s %5d / %5d (%5.1f%%)   multi %5d / %5d (%5.1f%%)"
              % (name, hit[name][0], n, 100.0 * hit[name][0] / max(n, 1),
                 hit[name][1], nm, 100.0 * hit[name][1] / max(nm, 1)))
    print("   leading-run misses: %d" % len(miss))
    by = {}
    for r, obs, pred in miss:
        by.setdefault((len(set(S.sched_syms(r))),
                       len(S.producers(r["specs"]))), []).append(r["cid"])
    for k in sorted(by):
        print("     nsym=%d nprod=%d : %d   e.g. %s"
              % (k[0], k[1], len(by[k]), by[k][0]))
    if "--show" in argv:
        for r, obs, pred in miss[:12]:
            print("  %-22s syms=%-7s stores=%s" %
                  (r["cid"], "".join(map(str, S.sched_syms(r))),
                   ",".join(map(str, r["stores"]))))
            print("      obs %s   pred %s" % (obs, pred))
            print("      %s" % r["emitted"])
    return 0


if __name__ == "__main__":
    sys.exit(main())
