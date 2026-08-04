#!/usr/bin/env python3
"""analyse.py — read rank.json and answer the questions the ranking raises.

Lane w-frame. (1) Do the two ranking keys agree? (2) Has the port ever emitted a
function that is BOTH framed and branching? (3) Which frontier TUs need that
product?
"""
import json, os, sys, struct
HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "scripts"))
sys.path.insert(0, HERE)
import featmap

FRAME = {"frame:stwu", "frame:mflr", "frame:savegprlr"}
# Every branch form that makes a body multi-block. `insn:b` is EXCLUDED: a tail
# call is a `b` and the port emits those in straight-line leaves, so counting it
# would make the product look witnessed when it is not.
BRANCH = {"insn:bf", "insn:bt", "insn:bclr", "insn:bdnz", "insn:bnelr", "insn:beqlr",
          "insn:bltlr", "insn:bgelr", "insn:bgtlr", "insn:blelr", "insn:bne", "insn:beq"}


def spearman(a, b):
    def rank(v):
        s = sorted(range(len(v)), key=lambda i: v[i])
        r = [0.0] * len(v)
        i = 0
        while i < len(s):
            j = i
            while j + 1 < len(s) and v[s[j + 1]] == v[s[i]]:
                j += 1
            avg = (i + j) / 2.0 + 1
            for k in range(i, j + 1):
                r[s[k]] = avg
            i = j + 1
        return r
    ra, rb = rank(a), rank(b)
    n = len(a)
    ma, mb = sum(ra) / n, sum(rb) / n
    num = sum((ra[i] - ma) * (rb[i] - mb) for i in range(n))
    da = sum((x - ma) ** 2 for x in ra) ** 0.5
    db = sum((x - mb) ** 2 for x in rb) ** 0.5
    return num / (da * db) if da and db else 0.0


d = json.load(open(os.path.join(HERE, "rank.json")))
rows = d["rows"]

print("=== A1 — do the keys agree? (Spearman rho over the 17)")
blk = [r["blocked"] for r in rows]
print("  blocked-fn  vs  lexical gap : rho = %+.3f" % spearman(blk, [r["gap"] for r in rows]))
print("  blocked-fn  vs  witness sum : rho = %+.3f" % spearman(blk, [r["wit_sum"] for r in rows]))
print("  lexical gap vs  witness sum : rho = %+.3f" % spearman([r["gap"] for r in rows],
                                                              [r["wit_sum"] for r in rows]))

# --- rebuild the witness function sets (cheap: objs are cached on disk)
fixdir = os.path.join(REPO, "fixtures", "cpp")
wits = []
for fx in [l.strip() for l in open(os.path.join(HERE, "match_fixtures.txt")) if l.strip()]:
    p = os.path.join(HERE, "obj", "fix", fx + ".obj")
    if os.path.exists(p):
        for f in featmap.obj_features(p)[0]:
            wits.append((fx + ":" + f["name"], set(f["features"])))
for name in os.listdir(os.path.join(HERE, "obj", "wl")):
    p = os.path.join(HERE, "obj", "wl", name)
    for f in featmap.obj_features(p)[0]:
        wits.append((name + ":" + f["name"], set(f["features"])))

framed = [w for w in wits if w[1] & FRAME]
branchy = [w for w in wits if w[1] & BRANCH]
both = [w for w in wits if (w[1] & FRAME) and (w[1] & BRANCH)]
print("\n=== The product test, over %d functions the port emits BYTE-EXACT today" % len(wits))
print("  framed (stwu | mflr | savegprlr) : %d" % len(framed))
print("  branching (a real block boundary): %d" % len(branchy))
print("  BOTH                             : %d   %s" % (len(both), [w[0] for w in both]))

print("\n=== Which frontier functions need the product?")
need = 0
tus = set()
for r in rows:
    for p in r["funcs"]:
        fs = set()
        # per-function feature sets are not stored verbatim; recover them
    pass
for r in rows:
    obj = os.path.join(HERE, "obj", "fr", r["src"].replace("/", "_") + ".obj")
    fns = featmap.obj_features(obj)[0]
    hit = [f["name"] for f in fns if (set(f["features"]) & FRAME) and (set(f["features"]) & BRANCH)]
    if hit:
        tus.add(r["src"])
        need += len(hit)
    print("  %-44s %d/%d functions framed AND branching" % (r["src"].replace("src/", ""), len(hit), len(fns)))
print("\n  %d of 17 frontier TUs contain at least one framed-and-branching function (%d functions)"
      % (len(tus), need))

print("\n=== A4 — is any frontier TU leaf-only end to end?")
for r in rows:
    obj = os.path.join(HERE, "obj", "fr", r["src"].replace("/", "_") + ".obj")
    fns = featmap.obj_features(obj)[0]
    fr = [f["name"] for f in fns if set(f["features"]) & FRAME]
    _, tu = featmap.obj_features(obj)
    print("  %-44s framed %d/%d  pdata=%s" % (r["src"].replace("src/", ""), len(fr), len(fns),
                                              "yes" if "sect:.pdata" in tu else "no"))

print("\n=== A5 — how many missing constructs appear in exactly ONE frontier TU?")
from collections import Counter
c = Counter(t for r in rows for t in r["missing"])
uniq = sorted(t for t, n in c.items() if n == 1)
print("  %d unique-to-one-TU tokens: %s" % (len(uniq), " ".join(uniq)))
print("  token frequency across the 17:")
for t, n in c.most_common():
    print("     %2d  %s" % (n, t))

print("\n=== the two branching witnesses")
for w in branchy:
    print("   ", w[0])
