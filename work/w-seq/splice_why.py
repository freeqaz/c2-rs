#!/usr/bin/env python3
"""splice_why.py — WHAT the splice perturbs, when it is not byte-exact.

Lane w-seq measurement tooling. **Read-only with respect to `crates/`.**

    splice_why.py <scan.jsonl>

`taxonomy.py` says how often SPLICE-P holds. A ratio is not a diagnosis: if the
splice fails, the next brief needs the *first disagreeing word* and the two
lengths, which is what `fnbyte-splice-why|<shape>|first@<i>:spl=…,ref=…` carries.

Printed with its denominator and with the success rows beside it, so a failure
census cannot be read as the whole population (`docs/STATUS.md` trap 5).
"""

import collections
import json
import sys


def main():
    why = collections.Counter()
    fn = collections.Counter()
    per_shape_pw = collections.Counter()
    for line in open(sys.argv[1]):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        for k, v in (r.get("emit") or {}).items():
            if k.startswith("fnbyte-splice-why|"):
                why[k.split("|", 1)[1]] += v
            elif k.startswith("fnbyte-splice-fn|"):
                _, shape, verdict, words, _sym = k.split("|", 4)
                fn[(shape, verdict, words)] += v
                per_shape_pw[(shape, verdict, words.split("/")[0])] += v

    print("=== SPLICE VERDICT x port-word count === (this is the P1 table)")
    den = collections.Counter()
    for (shape, verdict, pw), n in per_shape_pw.items():
        den[(shape, pw)] += n
    for (shape, verdict, pw), n in sorted(per_shape_pw.items()):
        d = den[(shape, pw)]
        print("  %-7s %-8s %-5s %5d of %5d  (%5.1f%%)"
              % (shape, verdict, pw, n, d, 100.0 * n / d))

    print("\n=== SPLICE VERDICT x (port/ref/callee words), top 40 ===")
    for (shape, verdict, w), n in sorted(fn.items(), key=lambda x: -x[1])[:40]:
        print("  %5d  %-7s %-8s %s" % (n, shape, verdict, w))

    tot = sum(why.values())
    print("\n=== SPLICE FAILURE WITNESSES === (%d failures, %d distinct)"
          % (tot, len(why)))
    if not why:
        print("  (none)")
    for k, n in why.most_common(30):
        print("  %5d  %5.1f%%  %s" % (n, 100.0 * n / tot, k))


if __name__ == "__main__":
    main()
