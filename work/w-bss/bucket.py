#!/usr/bin/env python3
"""Exact bucket index (h(name) mod 1024) for arbitrary identifiers.
Pads the probe names with a filler set so all 1024 buckets are occupied, then
derives bucket boundaries from `within-bucket order == reverse declaration order`
using several random declaration orders."""
import sys, os, string, random, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from probe import order_extern

L = string.ascii_lowercase
KW = set("""and and_eq asm auto bitand bitor bool break case catch char class compl const continue
delete do double else enum explicit export extern false float for friend goto if inline int long
mutable new not not_eq operator or or_eq private public register return short signed sizeof
static struct switch template this throw true try typedef typeid union unsigned using virtual
void while xor xor_eq main""".split())
FILLER = [a + b + c for a in L for b in L for c in L if a + b + c not in KW]

def buckets(probe_names, total=11000, rounds=26, seed=4242, tag="bk"):
    probe = list(dict.fromkeys(probe_names))
    fill = [f for f in FILLER if f not in set(probe)][: total - len(probe)]
    names = probe + fill
    n = len(names)
    out = order_extern(names, tag + "_0")
    boundary = set()
    def addb(d, o2):
        dp = {x: i for i, x in enumerate(d)}
        for i in range(len(o2) - 1):
            if dp[o2[i]] < dp[o2[i + 1]]:
                boundary.add(i)
    addb(names, out)
    rnd = random.Random(seed)
    outs = [out]
    for t in range(rounds):
        d = rnd.sample(names, n)
        o2 = order_extern(d, "%s_%d" % (tag, t + 1))
        outs.append(o2); addb(d, o2)
    nb = len(boundary) + 1
    bid = {}; k = 0
    for i, x in enumerate(out):
        bid[x] = k
        if i in boundary: k += 1
    for o2 in outs:
        ids = [bid[x] for x in o2]
        assert ids == sorted(ids), "bucket order not stable"
    return nb, bid

if __name__ == "__main__":
    probe = list(L) + [a + b for a in L for b in L if a + b not in KW]
    nb, bid = buckets(probe, tag="short")
    print("occupied buckets:", nb)
    json.dump(bid, open("bid_short.json", "w"))
    print("single letters -> bucket:")
    for a in L:
        print("   %s %4d   (ascii %d)" % (a, bid[a], ord(a)))
