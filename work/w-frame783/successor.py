#!/usr/bin/env python3
"""w-frame783 — THE SUCCESSOR FRONTIER.

The framing is shipped and the acceptance path's bound did not move: the gate
reads 34 where the walk-free instrument reads 414. So the question the next lane
needs answered is *what stops the walk on those 380*, by clause, with counts —
not "the walk" as a word.

Reads this lane's own tip scan's `--jsonl`, whose per-TU `bind_checks` already
carries the four subset memberships.
"""
import sys, json
from collections import Counter

path = sys.argv[1] if len(sys.argv) > 1 else "work/w-frame783/tip3.jsonl"
rows = [json.loads(l) for l in open(path) if '"record"' not in l[:14]]


def has(r, k):
    return (r.get("bind_checks") or {}).get(k, 0) > 0


gate = [r for r in rows if has(r, "selbind-emit-subset-gate-tus")]
prec = [r for r in rows if has(r, "selbind-emit-subset-scan-precise-tus")]
narrow = [r for r in rows if has(r, "selbind-emit-subset-scan-narrow-tus")]
wide = [r for r in rows if has(r, "selbind-emit-subset-wide-tus")]
print(f"populations: narrow {len(narrow)}  precise {len(prec)}  wide {len(wide)}  "
      f"gate {len(gate)}")

gs = {r["src"] for r in gate}
ps = {r["src"] for r in prec}
print(f"  gate ⊆ precise? {gs <= ps}   |precise ∖ gate| = {len(ps - gs)}"
      f"   |gate ∖ precise| = {len(gs - ps)}")

resid = [r for r in prec if r["src"] not in gs]
print(f"\nTHE SUCCESSOR: {len(resid)} TUs whose emit set IS entirely named by a "
      f"framed record\nand which the GATE'S WALK still binds nothing on. "
      f"First stop cause:")
for c, n in Counter(r.get("gate_cause") for r in resid).most_common():
    print(f"   {n:5d}  {c}")

print("\n…and the FULL cause SET on those TUs (a repair owes every one of them):")
for c, n in Counter(c for r in resid for c in (r.get("gate_causes") or [])).most_common():
    print(f"   {n:5d}  {c}")

print("\nHow many of the residue are ALSO complete under the incumbent framing "
      "(i.e. would\nhave been in the successor list before this lane shipped "
      "anything)?")
ns = {r["src"] for r in narrow}
print(f"   {len(ns - gs)} of {len(resid)}")

print("\nThe 23 matches and the residue do not overlap: "
      f"{len([r for r in resid if r['class'] == 'match'])} matches in the residue")

print("\nSample of the residue, smallest .ex first:")
for r in sorted(resid, key=lambda r: r.get("ex_len") or 0)[:12]:
    bc = r.get("bind_checks") or {}
    print(f"   {r['src']:64s} .ex {r.get('ex_len'):>8}  emitted "
          f"{bc.get('selbind-emitted')}  gl_body_starts {r.get('gl_body_starts')}  "
          f"cause {r.get('gate_cause')}")
