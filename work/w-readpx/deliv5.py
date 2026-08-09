#!/usr/bin/env python3
"""w-readpx — deliverable 5. The reader rungs, ranked, with the per-TU
counterfactual and the CFG screen applied to each of the frontier's 41.

Two questions, both answered by counting rather than by argument:

  Q1  Which of the 41 sits on a CFG class `Selected` can already express?
      A reader admission on a body whose CFG the emitter cannot hold buys a
      census row and no byte, because there is nowhere to put the body
      (`GapReport::cfg_reach`'s own doc).
  Q2  Which single key, granted alone, empties a frontier TU's reader column?
      That is the ONLY shape in which a reader rung converts a TU.

`PORT_CFG_CLASSES` is read out of the scan's own LEDGER block, never
hard-coded -- a lane that transcribes the list gets the answer its
transcription deserves.
"""
import collections
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
STEM = sys.argv[1] if len(sys.argv) > 1 else "hex"


def main():
    out = os.path.join(HERE, STEM + ".out")
    # the port's CFG classes, off the scan's own LEDGER
    port = []
    for line in open(out, encoding="utf-8", errors="replace"):
        s = line.strip()
        if "| WHOLE    |" in s:
            port.append(s.split("|")[0].strip())
    print("PORT CFG CLASSES, read off the scan's own LEDGER: %s\n" % port)

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

    rows = []
    for line in open(os.path.join(HERE, STEM + ".err"),
                     encoding="utf-8", errors="replace"):
        if not line.startswith("READPX\t"):
            continue
        f = line.rstrip("\n").split("\t")
        if len(f) < 10 or f[1] not in fr or f[3] != "fnbyte-refused":
            continue
        rows.append({"tu": f[1], "name": f[2], "key": f[4], "cflow": f[5],
                     "off": f[6], "bytes": int(f[8])})

    print("## Q1 — the frontier's %d, by whether the EMITTER can hold the CFG\n"
          % len(rows))
    ok = [r for r in rows if r["cflow"] in port]
    no = [r for r in rows if r["cflow"] not in port]
    print("**%d of %d sit on a port CFG class**; %d do not.\n"
          % (len(ok), len(rows), len(no)))
    print("| CFG class | n | in `PORT_CFG_CLASSES` |")
    print("|---|---:|---|")
    for c, n in collections.Counter(r["cflow"] for r in rows).most_common():
        print("| `%s` | %d | %s |" % (c, n, "**yes**" if c in port else "no"))
    print("| **total** | **%d** | |" % len(rows))
    assert len(ok) + len(no) == len(rows)
    print("\nCOLUMN-SUM ASSERT: %d + %d == %d  OK" % (len(ok), len(no), len(rows)))

    print("\n### the %d candidates a reader rung could actually reach\n" % len(ok))
    print("| TU | function | key | cflow | bytes |")
    print("|---|---|---|---|---:|")
    for r in sorted(ok, key=lambda r: (r["tu"], r["name"])):
        print("| `%s` | `%s` | `%s` | `%s` | %d |"
              % (r["tu"].split("/")[-1], r["name"], r["key"], r["cflow"],
                 r["bytes"]))

    print("\n## Q2 — the per-TU counterfactual: does ANY single key empty a TU?\n")
    print("| frontier TU | reader | distinct keys | single-key? | all bodies on a port CFG class? | converts on one reader rung? |")
    print("|---|---:|---:|---|---|---|")
    conv = []
    for tu in fr:
        rs = [r for r in rows if r["tu"] == tu]
        ks = {r["key"] for r in rs}
        allok = all(r["cflow"] in port for r in rs)
        single = len(ks) == 1
        yes = single and allok
        if yes:
            conv.append(tu)
        print("| `%s` | %d | %d | %s | %s | %s |"
              % (tu, len(rs), len(ks), "**yes**" if single else "no",
                 "**yes**" if allok else "no",
                 "**YES**" if yes else "no"))
    print("\n**TUs a single reader rung converts: %d** %s"
          % (len(conv), conv))


main()
