#!/usr/bin/env python3
"""w-nc — the EMITTED-SIDE refusal-key histogram.

`c2rs gap`'s published `fn_blockers` is a histogram over **every IL body** (2.4M),
which `w-inlfence` (#2220) showed is fail-open on 845 of 871 TUs. This one is
over the **emitted** functions only — the population the byte judge grades and
the only discriminating one (`w-readpx`, #2280). It sums to `fnbyte-refused`.

Needs a scan produced with `C2RS_NC_KEYS=1` (the reverted scratch in
`crates/c2-harness/src/gap/fnbytes.rs`; the diff is quoted in the rung).

usage: keys.py INST.jsonl
"""
import json
import re
import sys
from collections import Counter


def main():
    rows = [json.loads(l) for l in open(sys.argv[1])]
    rows = [r for r in rows if "src" in r]
    c = Counter()
    for r in rows:
        for k, v in r["emit"].items():
            if k.startswith("fnbyte-parsekey|"):
                c[k.split("|", 1)[1]] += v
    tot = sum(c.values())
    print("EMITTED-SIDE refusal keys (fnbyte-refused only)")
    print(f"distinct {len(c)}  total {tot}   (must equal gap-metric fnbyte-refused)")
    print("\ntop 60:")
    for k, v in c.most_common(60):
        print(f"  {v:>7}  {k}")

    # **NC-3, the #764 family** — a refusal whose key ends in a hex TYPE TAG is a
    # positive-list membership question, not a construct the reader cannot
    # represent. `.sy`'s `plain_int` was exactly this shape.
    pat = re.compile(r"(type|target)-[0-9A-Fa-f]{4}")
    tt = {k: v for k, v in c.items() if pat.search(k)}
    print(f"\nTYPE-TAG family (NC-3, #764's shape): keys {len(tt)}  "
          f"functions {sum(tt.values())}  share {100*sum(tt.values())/tot:.2f}%")
    for k, v in sorted(tt.items(), key=lambda x: -x[1]):
        print(f"  {v:>7}  {k}")

    # **NC-4, the mislocation family** — board #1416 demonstrated that
    # `expr-cmp-eq`/`-ne` is a FALL-THROUGH key that "names none of their
    # refusals", and #420 measured the whole relational family absorbing into
    # branch keys. Any ranking off this histogram inherits that defect, so the
    # size of the family is published beside the histogram, not hidden in it.
    rel = [k for k in c if k.startswith("expr-cmp-") or k in ("expr-brfalse", "expr-brtrue")]
    s = sum(c[k] for k in rel)
    print(f"\nFALL-THROUGH family (NC-4, #420/#440/#1416): keys {len(rel)}  "
          f"functions {s}  share {100*s/tot:.2f}%")
    for k in sorted(rel, key=lambda k: -c[k]):
        print(f"  {c[k]:>7}  {k}")


if __name__ == "__main__":
    main()
