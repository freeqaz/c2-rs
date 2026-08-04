#!/usr/bin/env python3
import json
from collections import Counter
import os
# repo root = four levels up from this file (.claude/worktrees/<lane>/work/w-bss/census)
W = os.environ.get("C2RS_LANE_ROOT") or os.path.abspath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
W = os.path.join(W, "work", "w-bss", "census")
recs = [json.loads(l) for l in open(W + "/sections.jsonl")]
N = len(recs)
print("population:", N)

# ---- Q7 positional
print("\n=== Q7 positional ===")
c2n = Counter(o.count(".XBLD$W:C2") for o in (r["order"] for r in recs))
c1n = Counter(o.count(".XBLD$W:C1") for o in (r["order"] for r in recs))
oth = Counter(t for r in recs for t in r["order"] if t.startswith(".XBLD"))
print("  .XBLD$W tag counts per obj: C2:", dict(c2n), " C1:", dict(c1n))
print("  all .XBLD tags seen:", dict(oth))
adj = Counter(); pos = Counter()
bss_between = []; data_between = []
rel = Counter()
for r in recs:
    o = r["order"]
    try:
        i2 = o.index(".XBLD$W:C2"); i1 = o.index(".XBLD$W:C1")
    except ValueError:
        rel["no-two-XBLD"] += 1
        continue
    adj[i1 - i2] += 1
    pos[(i2, i1)] += 1
    between = o[i2 + 1:i1]
    if between:
        rel["something-between"] += 1
        if ".bss" in between:
            bss_between.append((r["src"], between))
        if ".data" in between:
            data_between.append((r["src"], between))
    else:
        rel["XBLD-adjacent"] += 1
    di = [k for k, t in enumerate(o) if t == ".data"]
    bi = [k for k, t in enumerate(o) if t == ".bss"]
    if di:
        rel["data:first-after-C1" if di[0] > i1 else "data:first-before-C1"] += 1
        rel["data:ALL-after-C1" if min(di) > i1 else "data:some-at-or-before-C1"] += 1
    if bi:
        rel["bss:first-after-C1" if bi[0] > i1 else "bss:first-before-C1"] += 1
        rel["bss:ALL-after-C1" if min(bi) > i1 else "bss:some-at-or-before-C1"] += 1
    if di and bi:
        rel["firstData<firstBss" if di[0] < bi[0] else "firstBss<firstData"] += 1
print("  (i1 - i2) gap histogram:", dict(sorted(adj.items())))
print("  (idx of C2, idx of C1) top:", pos.most_common(6))
for k, v in sorted(rel.items()):
    print("   %-32s %d" % (k, v))
print("  objs with .bss BETWEEN the two XBLD$W: %d" % len(bss_between))
for s, b in bss_between:
    print("     ", s, "->", b)
print("  objs with .data BETWEEN the two XBLD$W: %d" % len(data_between))
for s, b in data_between[:10]:
    print("     ", s, "->", b)
# what appears between when non-empty
btw = Counter()
for r in recs:
    o = r["order"]
    if ".XBLD$W:C2" in o and ".XBLD$W:C1" in o:
        for t in o[o.index(".XBLD$W:C2") + 1:o.index(".XBLD$W:C1")]:
            btw[t] += 1
print("  section names seen between C2 and C1:", dict(btw))
# prefix before C2
pre = Counter()
for r in recs:
    o = r["order"]
    if ".XBLD$W:C2" in o:
        pre[tuple(o[:o.index(".XBLD$W:C2")])] += 1
print("  prefixes before .XBLD$W:C2 (top 6):")
for k, v in pre.most_common(6):
    print("     %4d  %s" % (v, " ".join(k) if len(k) < 12 else " ".join(k[:6]) + " ...(%d)" % len(k)))

# ---- refined symbol classification
def cls2(n):
    if n.startswith("??_R0"): return "??_R0"
    for k in "1234":
        if n.startswith("??_R" + k): return "??_R" + k
    if n.startswith("??_7"): return "??_7 vftable"
    if n.startswith("??_8"): return "??_8 vbtable"
    if n.startswith("??_C"): return "??_C string"
    if n.startswith("$SG"): return "$SG"
    if n.startswith("?"):
        i = n.find("@@")
        if i >= 0 and i + 2 < len(n):
            return "?..@@%s.." % n[i + 2]
        return "?other"
    return "undecorated"

for key, q in (("data", 4), ("bss", 5)):
    print("\n=== Q%d .%s symbols (refined) ===" % (q, key))
    h = Counter(); sc = Counter(); samples = {}
    for r in recs:
        for s in r[key]:
            for sy in s["syms"]:
                c = cls2(sy["n"]); h[c] += 1; sc[(c, sy["sc"])] += 1
                samples.setdefault(c, []).append(sy["n"])
    for k, v in h.most_common():
        ex = samples[k][0][:90]
        print("   %-16s %6d   e.g. %s" % (k, v, ex))
    print("   class x storage-class:")
    for (c, s), v in sorted(sc.items()):
        print("     %-16s sc=%d  %6d" % (c, s, v))
    # COMDAT vs not, by content class
    cc = Counter()
    for r in recs:
        for s in r[key]:
            cs = frozenset(cls2(sy["n"]) for sy in s["syms"])
            cc[(tuple(sorted(cs)), s["comdat"], s["ch"])] += 1
    print("   (content-classes, COMDAT, ch) top 12:")
    for (k, cd, ch), v in cc.most_common(12):
        print("     %6d  ch=0x%08x comdat=%-5s %s" % (v, ch, cd, list(k)))

# CheckSum split by COMDAT
for key in ("data", "bss"):
    z = Counter()
    for r in recs:
        for s in r[key]:
            for ss in s["secsym"]:
                z[(s["comdat"], ss["sel"], ss["cks"] == 0)] += 1
    print("\n  .%s (COMDAT, Selection, CheckSum==0) ->" % key, dict(z))
