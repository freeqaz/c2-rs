#!/usr/bin/env python3
"""peerkeys.py — do the PEER LANES' scan keys still print after the resolution?

Lane w-splice merge evidence. **Read-only.**

    peerkeys.py <master-base.jsonl> <rebased.jsonl>

The rebase resolved two files both peer lanes had also edited
(`crates/c2-obj/src/lib.rs`, `crates/c2-harness/src/gap/fnbytes.rs`). A
resolution that compiles and passes *this* lane's checks can still have dropped
a peer's instrument on the floor — and a missing key is silent, which is
`docs/GAPS.md`'s most-recorded failure shape.

So every key family the peers publish is counted at BOTH ends and compared. A
family that appears at master's base and not at the tip is a regression this
lane caused; the check is per family and per total, and a family whose totals
differ is printed with both numbers rather than judged.
"""

import collections
import json
import sys

# The key PREFIXES each peer lane's instrument publishes.
PEERS = {
    "w-bytes  (diff signature, #976-#983)": [
        "fndiff-",
    ],
    "w-bytes/w-drop3 (reloc sites + call targets, #982/#984)": [
        "fnbyte-reloc-sites",
        "fnbyte-call-targets",
        "fnbyte-callsite",
        "fnbyte-calltarget",
    ],
    "w-seq    (splice forensics, #966-#975)": [
        "fnbyte-splice0", "fnbyte-spliceN", "fnbyte-splice|",
        "fnbyte-contains", "fnbyte-differs-why", "fnbyte-why|",
        "fnbyte-callee-",
    ],
    "w-empty/w-fix (mechanism E)": [
        "fnbyte-elided", "fnbyte-tu-empty-callees",
    ],
}


def families(path, prefixes):
    tot = collections.Counter()
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        for k, v in (r.get("emit") or {}).items():
            for p in prefixes:
                if k.startswith(p):
                    tot[p] += v
                    break
    return tot


def main():
    base, tip = sys.argv[1], sys.argv[2]
    bad = 0
    for lane, prefixes in PEERS.items():
        print("== %s" % lane)
        b = families(base, prefixes)
        t = families(tip, prefixes)
        for p in prefixes:
            nb, nt = b.get(p, 0), t.get(p, 0)
            if nb == 0 and nt == 0:
                print("   %-28s absent at BOTH ends (no such key in this build)" % p)
                continue
            flag = ""
            if nb > 0 and nt == 0:
                flag = "   <<<< VANISHED"
                bad += 1
            elif nb != nt:
                flag = "   (moved — expected where this lane converts a differ)"
            print("   %-28s master %8d   tip %8d%s" % (p, nb, nt, flag))
    print("\nFAMILIES THAT VANISHED: %d" % bad)


if __name__ == "__main__":
    main()
