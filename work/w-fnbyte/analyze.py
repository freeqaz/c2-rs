"""w-fnbyte — the differ taxonomy, from a `c2rs gap --jsonl` scan's own keys.

Reads the `fnbyte-differs-fn|<shape>|w<pw>/<rw>/eq<eq>|<first-word>|<symbol>`
witness keys and prints the four families, the per-shape split and the top
signatures. Tooling, outside the std-only workspace.

Usage:  python3 work/w-fnbyte/analyze.py work/w-fnbyte/final.jsonl
"""

import collections
import json
import sys


def main(path):
    fam = collections.Counter()
    famshape = collections.Counter()
    shape = collections.Counter()
    sig = collections.Counter()
    sig_ex = {}
    tus = collections.Counter()
    first0 = 0
    total = 0
    for line in open(path, encoding="utf-8"):
        d = json.loads(line)
        for k, v in (d.get("emit") or {}).items():
            if not k.startswith("fnbyte-differs-fn|"):
                continue
            sh, words, firstw, name = k[len("fnbyte-differs-fn|"):].split("|", 3)
            pw, rw, eq = words[1:].split("/")
            pw, rw, eq = int(pw), int(rw), int(eq[2:])
            total += v
            shape[sh] += v
            tus[d.get("src", "?")] += v
            if firstw.startswith("first@0"):
                first0 += v
            if firstw.endswith("ref=4e800020") and rw == 1:
                f = "A  c2's whole body is a bare `blr`"
            elif eq > 0:
                f = "D  shared prefix, then a divergence"
            elif rw < pw:
                f = "B  c2 emitted FEWER words, no shared prefix"
            elif rw > pw:
                f = "C  c2 emitted MORE words, no shared prefix"
            else:
                f = "E  same length, word 0 differs"
            fam[f] += v
            famshape[(f, sh)] += v
            sig[(sh, words, firstw)] += v
            sig_ex.setdefault((sh, words, firstw), name)

    print(f"differs total (from witness keys): {total}")
    print(f"diverge at word 0: {first0}   later: {total - first0}")
    print(f"distinct signatures: {len(sig)}   TUs carrying a differ: {len(tus)}")
    print()
    print("by shape:")
    for s, n in shape.most_common():
        print(f"  {n:6}  {s}")
    print()
    print("families:")
    for f, n in sorted(fam.items()):
        print(f"  {n:6}  {f}")
    print()
    print("families x shape:")
    for (f, s), n in sorted(famshape.items()):
        print(f"  {n:6}  {f}  [{s}]")
    print()
    print("top signatures:")
    for (s, w, fw), n in sig.most_common(20):
        print(f"  {n:6}  {s}|{w}|{fw}")
        print(f"          e.g. {sig_ex[(s, w, fw)]}")
    print()
    print("top TUs:")
    for t, n in tus.most_common(15):
        print(f"  {n:6}  {t}")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "work/w-fnbyte/final.jsonl")
