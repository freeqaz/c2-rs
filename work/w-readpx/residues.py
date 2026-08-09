#!/usr/bin/env python3
"""w-readpx — the named residues, sized at this tip, and the calibrated prior
for "if the reader admits a body of shape X, what is P(fnbyte-exact)?".

Three blocks:

  A. every key matching a residue pattern the commission names, on the EMITTED
     column, with the replication columns.
  B. the exact rate of the ALREADY-ADMITTED population, split by CFG class and
     by body size. This is the only calibrated predictor of an
     `fnbyte-exact` delta that exists at this tip, because every BLOCKED
     emitted row is `fnbyte-refused` by construction -- the census's blocked
     column and the byte judge's refused column are the SAME 130,117 rows.
  C. the frontier's single-key TUs crossed with CFG reachability -- the
     deliverable-5 counterfactual.
"""
import collections
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
STEM = sys.argv[1] if len(sys.argv) > 1 else "hex"


def rows(path):
    for line in open(path, encoding="utf-8", errors="replace"):
        if not line.startswith("READPX\t"):
            continue
        f = line.rstrip("\n").split("\t")
        if len(f) < 10:
            continue
        yield {
            "tu": f[1], "name": f[2], "fnb": f[3], "key": f[4],
            "cflow": f[5], "off": f[6], "cls": f[7], "bytes": int(f[8]),
            "byte": f[9], "win": f[10] if len(f) > 10 else "",
        }


def main():
    all_rows = list(rows(os.path.join(HERE, STEM + ".err")))
    blk = [r for r in all_rows if r["cls"] == "blk"]
    inc = [r for r in all_rows if r["cls"] == "in"]

    by = collections.defaultdict(list)
    for r in blk:
        by[r["key"]].append(r)

    print("## A. the named residues, sized on the EMITTED column at this tip\n")
    pats = [
        ("the walker's `9B` (w-value #1943)", r"0x9B|0X9B"),
        ("the walker's `64`", r"0x64|0X64"),
        ("the designator layer `0x27`", r"0x27"),
        ("the designator layer `0x28`", r"0x28"),
        ("wb-eh R1 -- the `.sy` binding / `param-*` seam", r"^param-"),
        ("`callee-defined-in-tu` (w-inlfence #2226)", r"callee-defined-in-tu"),
    ]
    for label, pat in pats:
        rx = re.compile(pat)
        hits = sorted(((k, v) for k, v in by.items() if rx.search(k)),
                      key=lambda kv: -len(kv[1]))
        tot = sum(len(v) for _, v in hits)
        print("### %s — %d keys, **%d emitted**" % (label, len(hits), tot))
        if not hits:
            print("  (no key on the emitted column)\n")
            continue
        print("| key | emitted | dTU | dname | e/TU | exact | differs |")
        print("|---|---:|---:|---:|---:|---:|---:|")
        for k, v in hits[:12]:
            c = collections.Counter(r["fnb"] for r in v)
            dtu = len({r["tu"] for r in v})
            print("| `%s` | %d | %d | %d | %.2f | %d | %d |"
                  % (k, len(v), dtu, len({r["name"] for r in v}),
                     len(v) / dtu, c.get("fnbyte-exact", 0),
                     c.get("fnbyte-differs", 0)))
        if len(hits) > 12:
            print("| … %d further keys | %d | | | | | |"
                  % (len(hits) - 12, sum(len(v) for _, v in hits[12:])))
        print()

    print("\n## B. P(fnbyte-exact) on the ALREADY-ADMITTED population\n")
    print("The calibrated prior. `differs` includes `reloc-differs`.\n")
    print("| CFG class | in-class emitted | exact | differs | reloc-differs | P(exact) |")
    print("|---|---:|---:|---:|---:|---:|")
    byc = collections.defaultdict(list)
    for r in inc:
        byc[r["cflow"] or "<none>"].append(r)
    tot = collections.Counter()
    for k, v in sorted(byc.items(), key=lambda kv: -len(kv[1])):
        c = collections.Counter(r["fnb"] for r in v)
        tot.update(c)
        print("| `%s` | %d | %d | %d | %d | %.3f |"
              % (k, len(v), c.get("fnbyte-exact", 0),
                 c.get("fnbyte-differs", 0), c.get("fnbyte-reloc-differs", 0),
                 c.get("fnbyte-exact", 0) / len(v)))
    print("| **all** | %d | %d | %d | %d | **%.3f** |"
          % (len(inc), tot["fnbyte-exact"], tot["fnbyte-differs"],
             tot["fnbyte-reloc-differs"], tot["fnbyte-exact"] / len(inc)))

    print("\n### by body size (bytes of c2's own COMDAT)\n")
    print("| size | in-class | exact | P(exact) |")
    print("|---|---:|---:|---:|")
    buckets = [(0, 8), (8, 16), (16, 32), (32, 64), (64, 128), (128, 256),
               (256, 1 << 30)]
    for lo, hi in buckets:
        v = [r for r in inc if lo <= r["bytes"] < hi]
        if not v:
            continue
        e = sum(1 for r in v if r["fnb"] == "fnbyte-exact")
        print("| %d–%s B | %d | %d | %.3f |"
              % (lo, "∞" if hi > 1 << 20 else hi, len(v), e, e / len(v)))

    print("\n## C. the frontier's single-key TUs\n")
    out = os.path.join(HERE, STEM + ".out")
    fr, on = [], False
    for line in open(out, encoding="utf-8", errors="replace"):
        if line.startswith("  FRONTIER — "):
            on = True
            continue
        if on:
            parts = [p.strip() for p in line.split("|")]
            if len(parts) != 3:
                break
            fr.append(parts[2])
    reach = {}
    on = False
    for line in open(out, encoding="utf-8", errors="replace"):
        if "FRONTIER BY CFG REACHABILITY" in line:
            on = True
            continue
        if on:
            if "LABEL CHANNEL" in line:
                break
            parts = [p.strip() for p in line.split("|")]
            if len(parts) == 4:
                reach[parts[1]] = parts[3]
    print("| frontier TU | reader | distinct keys | the key(s) | CFG reachability |")
    print("|---|---:|---:|---|---|")
    for tu in fr:
        rf = [r for r in all_rows if r["tu"] == tu and r["fnb"] == "fnbyte-refused"]
        ks = collections.Counter(r["key"] for r in rf)
        print("| `%s` | %d | %d | %s | %s |"
              % (tu, len(rf), len(ks),
                 " ".join("`%s`*%d" % (k, v) for k, v in sorted(ks.items()))
                 if len(ks) <= 3 else "%d keys" % len(ks),
                 reach.get(tu, "?")))


main()
