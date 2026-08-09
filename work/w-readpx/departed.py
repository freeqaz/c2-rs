#!/usr/bin/env python3
"""w-readpx — where did wb-reader's eight departed functions GO?

`WB_READER_FINDINGS.md` §1 lists 48 reader-refused frontier functions at base
`c34c388c`. This tip reads 41. This script names the difference by FUNCTION
and reports each departed function's TU verdict at this tip — the difference
between "the reader recovered it" and "its TU left the frontier".
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
STEM = sys.argv[1] if len(sys.argv) > 1 else "base"

# `WB_READER_FINDINGS.md` §1's table, transcribed: the key, its count, and the
# example function it names. The table does not print all 48 names, so the
# comparison below is by (key, count) with the named examples resolved.
WB48 = {
    "expr-cmp-eq": 11, "expr-jump": 10, "assign-store-type-8643": 4,
    "expr-op-0x27": 4, "expr-brfalse": 3, "assign-rhs-call-0x26": 1,
    "call-arg-lit-permuted:mid": 1, "call-arg-outer-formal:eof": 1,
    "expr-brtrue": 1,
    "expr-call-in-expr-data-addr-then-plain-call-and-op-more": 1,
    "expr-call-in-expr-op-0x1F": 1,
    "expr-call-in-expr-recv-load-then-plumbing-0x3A": 1,
    "expr-cmp-ge": 1, "expr-cmp-ne": 1, "expr-intrinsic-memcpy": 1,
    "expr-lit-type-9641": 1, "expr-load-type-8211": 1,
    "expr-load-type-8882": 1, "expr-op-0x0F": 1, "expr-op-0x30": 1,
    "param-width-undetermined:mid": 1,
}
# The functions §1 names by example for keys that shrank, plus the two the
# per-key TU note identifies (`negate_test` x2 for `assign-store-type-8643`,
# `jsonwriter` for `expr-brfalse`).
NAMED = [
    ("?NextHashPrime@@YAHH@Z", "expr-jump"),
    ("CXLrcImpl_CreateClientWithTransport", "assign-rhs-call-0x26"),
    ("_free_osfhnd", "expr-cmp-ge"),
    ("?append@DName@@QAAXPAVDNameNode@@@Z", "expr-cmp-ne"),
    ("?GetBuffer@JsonWriter@@QAAJPAGPAK@Z", "expr-brfalse"),
    ("?FindNodeA@@YAPBUCharGraphNode@@W4PlayBlend@@PAXM@Z",
     "assign-store-type-8643"),
]


def tip_keys(path):
    out = {}
    for line in open(path, encoding="utf-8", errors="replace"):
        if not line.startswith("READPX\t"):
            continue
        f = line.rstrip("\n").split("\t")
        if len(f) != 9 or f[3] != "fnbyte-refused":
            continue
        out.setdefault(f[4], []).append((f[1], f[2]))
    return out


def main():
    err = os.path.join(HERE, STEM + ".err")
    # restrict to the frontier
    out = os.path.join(HERE, STEM + ".out")
    fr, on = set(), False
    for line in open(out, encoding="utf-8", errors="replace"):
        if line.startswith("  FRONTIER — "):
            on = True
            continue
        if on:
            parts = [p.strip() for p in line.split("|")]
            if len(parts) != 3:
                break
            fr.add(parts[2])
    tip = {}
    for line in open(err, encoding="utf-8", errors="replace"):
        if not line.startswith("READPX\t"):
            continue
        f = line.rstrip("\n").split("\t")
        if len(f) != 9 or f[1] not in fr or f[3] != "fnbyte-refused":
            continue
        tip.setdefault(f[4], []).append((f[1], f[2]))

    print("| key | wb-reader (`c34c388c`) | this tip | delta |")
    print("|---|---:|---:|---:|")
    keys = sorted(set(WB48) | set(tip))
    tb = tn = 0
    for k in keys:
        b = WB48.get(k, 0)
        n = len(tip.get(k, []))
        tb += b
        tn += n
        mark = "" if b == n else " **" + ("%+d" % (n - b)) + "**"
        print("| `%s` | %d | %d |%s |" % (k, b, n, mark or " 0"))
    print("| **total** | **%d** | **%d** | **%+d** |" % (tb, tn, tn - tb))

    # Where are the named departures now?
    verd = {}
    for line in open(os.path.join(HERE, STEM + ".jsonl"),
                     encoding="utf-8", errors="replace"):
        d = json.loads(line)
        verd[d["src"]] = (d["class"], d.get("reason", ""),
                          set(d.get("fn_names") or []))
    print("\n--- the departed, by name: which TU, and that TU's verdict now ---")
    print("| departed function | key it held | TU | TU verdict at this tip |")
    print("|---|---|---|---|")
    for nm, k in NAMED:
        hit = [s for s, (_, _, names) in verd.items() if nm in names]
        for s in hit or ["<no TU carries this name>"]:
            cl = verd[s][0] if s in verd else "?"
            print("| `%s` | `%s` | `%s` | **%s** |" % (nm, k, s, cl))


main()
