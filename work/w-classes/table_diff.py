#!/usr/bin/env python3
"""Row-by-row diff of two `mode_classes.txt` — THE INCUMBENT IS THE CONTROL.

Regenerating the table re-derives every fragment's lane set from this lane's
sample, so a regeneration is 60-odd opportunities to narrow somebody else's
coverage with no textual conflict and no failing test. The direction of the
risk is not symmetric and this prints it that way:

    WIDENED  the new row grades MORE lanes than the old one. Free — the cross
             pays for it and no coverage is lost. Not a claim.
    SAME     identical lane set.
    NEW      a fragment that had no row and now has one. Its old cost was ALL
             lanes (the fail-safe), so every row it gains is a narrowing and
             every one needs the measurement behind it.
    NARROWED the new row grades FEWER lanes. **This is a claim**, and the
             lanes it drops are lanes nothing else grades.
    GONE     a fragment that had a row and now has none. Fail-safe in cost
             terms (it falls back to every lane) but it means the measurement
             did not reach it, which is worth knowing.

    work/w-classes/table_diff.py <old> <new>
"""

import sys


def read(path):
    out = {}
    for line in open(path):
        if line.startswith("# measured-over-lanes:"):
            continue
        code = line.partition("#")[0].strip()
        if not code:
            continue
        p = code.split()
        if len(p) != 3:
            raise SystemExit("malformed row in %s: %r" % (path, line))
        out[p[0]] = (set(p[1].split(",")), p[2])
    return out


def main():
    old, new = read(sys.argv[1]), read(sys.argv[2])
    cats = {"WIDENED": [], "SAME": [], "NEW": [], "NARROWED": [], "GONE": []}
    for f in sorted(set(old) | set(new)):
        if f not in new:
            cats["GONE"].append((f, sorted(old[f][0]), []))
            continue
        if f not in old:
            cats["NEW"].append((f, [], sorted(new[f][0])))
            continue
        o, n = old[f][0], new[f][0]
        if o == n:
            cats["SAME"].append((f, sorted(o), sorted(n)))
        elif o < n:
            cats["WIDENED"].append((f, sorted(o), sorted(n)))
        elif n < o:
            cats["NARROWED"].append((f, sorted(o), sorted(n)))
        else:
            cats["NARROWED"].append((f, sorted(o), sorted(n)))   # incomparable
        if old[f][1] != new[f][1]:
            print("NOTE %s: case digest changed %s -> %s (the generator moved)"
                  % (f, old[f][1], new[f][1]))
    for k in ("NARROWED", "NEW", "GONE", "WIDENED", "SAME"):
        print()
        print("== %s: %d" % (k, len(cats[k])))
        if k == "SAME":
            print("   " + " ".join(f for f, _, _ in cats[k]))
            continue
        for f, o, n in cats[k]:
            print("   %-27s %d -> %d lanes" % (f, len(o), len(n)))
            if k in ("NARROWED",):
                print("        dropped: %s" % ",".join(sorted(set(o) - set(n))))
                print("        added:   %s" % (",".join(sorted(set(n) - set(o))) or "-"))
            elif k == "NEW":
                print("        grades:  %s" % ",".join(n))
    print()
    print("rows that NARROW (a claim): %d" % (len(cats["NARROWED"]) + len(cats["NEW"])))
    print("rows that WIDEN or hold (free): %d" % (len(cats["WIDENED"]) + len(cats["SAME"])))


main()
