#!/usr/bin/env python3
import json, statistics
from collections import Counter, defaultdict
import os
# repo root = four levels up from this file (.claude/worktrees/<lane>/work/w-bss/census)
W = os.environ.get("C2RS_LANE_ROOT") or os.path.abspath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
W = os.path.join(W, "work", "w-bss", "census")
recs = [json.loads(l) for l in open(W + "/sections.jsonl")]
N = len(recs)
print("population:", N)

# Q1
hasd = sum(1 for r in recs if r["data"])
hasb = sum(1 for r in recs if r["bss"])
both = sum(1 for r in recs if r["data"] and r["bss"])
neither = sum(1 for r in recs if not r["data"] and not r["bss"])
print("\nQ1 has .data=%d  has .bss=%d  both=%d  neither=%d  data-only=%d  bss-only=%d"
      % (hasd, hasb, both, neither, hasd - both, hasb - both))
nd = Counter(len(r["data"]) for r in recs)
nb = Counter(len(r["bss"]) for r in recs)
print("  count of .data sections per obj:", dict(sorted(nd.items())))
print("  count of .bss  sections per obj:", dict(sorted(nb.items())))
for r in recs:
    if len(r["data"]) > 1:
        print("   >1 .data:", r["src"], len(r["data"]))
    if len(r["bss"]) > 1:
        print("   >1 .bss:", r["src"], len(r["bss"]))

# Q2/Q3
for key in ("data", "bss"):
    print("\nQ%s .%s characteristics distribution (per SECTION, n=%d)"
          % (2 if key == "data" else 3, key, sum(len(r[key]) for r in recs)))
    c = Counter()
    for r in recs:
        for s in r[key]:
            c[(s["ch"], s["chdec"])] += 1
    for (ch, dec), n in c.most_common():
        print("   0x%08x  %5d   %s   COMDAT=%s" % (ch, n, dec, bool(ch & 0x1000)))
    sel = Counter(); cks = Counter(); nsec_sym = Counter()
    for r in recs:
        for s in r[key]:
            nsec_sym[len(s["secsym"])] += 1
            for ss in s["secsym"]:
                sel[ss["sel"]] += 1
                cks[ss["cks"]] += 1
    print("   section-symbol count per section:", dict(sorted(nsec_sym.items())))
    print("   aux Selection histogram:", dict(sel))
    print("   aux CheckSum distinct values: %d ; top:" % len(cks), cks.most_common(5))

# Q4/Q5
for key in ("data", "bss"):
    print("\nQ%s .%s symbol classification (naux==0, SectionNumber==sec idx)"
          % (4 if key == "data" else 5, key))
    h = Counter(); sc = Counter(); tot = 0
    for r in recs:
        for s in r[key]:
            for sy in s["syms"]:
                h[sy["cls"]] += 1; sc[sy["sc"]] += 1; tot += 1
    print("   total symbols:", tot)
    for k, v in h.most_common():
        print("     %-18s %6d" % (k, v))
    print("   storage class:", {("%d" % k): v for k, v in sorted(sc.items())})
    # per-section composition
    comp = Counter(); mixed = []
    for r in recs:
        for s in r[key]:
            cs = frozenset(sy["cls"] for sy in s["syms"])
            comp[tuple(sorted(cs))] += 1
            if "??_R0" in cs and len(cs) > 1:
                mixed.append((r["src"], sorted(cs)))
    print("   per-section class-set composition (top 15):")
    for k, v in comp.most_common(15):
        print("     %6d  %s" % (v, list(k)))
    print("   sections mixing ??_R0 with anything else: %d" % len(mixed))
    for m in mixed[:10]:
        print("      ", m)
    # symbols-per-section
    spc = Counter(len(s["syms"]) for r in recs for s in r[key])
    print("   symbols per section:", dict(sorted(spc.items())))

# Q6
for key in ("data", "bss"):
    sizes = [s["size"] for r in recs for s in r[key]]
    if not sizes:
        print("\nQ6 .%s: none" % key); continue
    print("\nQ6 .%s SizeOfRawData: n=%d min=%d median=%d mean=%.1f max=%d"
          % (key, len(sizes), min(sizes), statistics.median(sizes),
             statistics.mean(sizes), max(sizes)))
    print("   size histogram (top):", Counter(sizes).most_common(12))
    bad_ptr = [(r["src"], s["ptr"]) for r in recs for s in r[key] if s["ptr"] != 0]
    bad_vsz = [(r["src"], s["vsz"]) for r in recs for s in r[key] if s["vsz"] != 0]
    bad_rel = [(r["src"], s["nrel"]) for r in recs for s in r[key] if s["nrel"] != 0]
    print("   PointerToRawData != 0: %d  %s" % (len(bad_ptr), bad_ptr[:5]))
    print("   VirtualSize     != 0: %d  %s" % (len(bad_vsz), bad_vsz[:5]))
    print("   NumberOfRelocations != 0: %d  %s" % (len(bad_rel), bad_rel[:5]))

# Q7
print("\nQ7 section order")
print("  distinct full orders:", len(Counter(tuple(r["order"]) for r in recs)))
oc = Counter(tuple(r["order"]) for r in recs)
for k, v in oc.most_common(12):
    print("   %4d  %s" % (v, " ".join(k)))
