#!/usr/bin/env python3
"""The NULL-LIFT GRID — can this instrument tell a real lift from a fake one?

    python3 work/w-ladders/nullgrid.py <tus> <flags> <dc3> <cache> <plain> <hatch> <out>

**A control that cannot go red is not a control.** `w-one`'s grid came out
near-vacuous at <= 1 discriminating cell on its population and it PRINTED THAT
rather than banking the pass; this file exists to be able to do the same.

# The cell, and what "discriminating" means here

For each TU, at the ladder's own starting point (`SEED = ["op:41"]`, no hatch),
four scans:

    A0   seed only                                     the baseline blocker set
    A1   seed + the TU's OWN round-0 lift token        the REAL lift
    A2   seed + ANOTHER TU's round-0 lift token        the NULL sink (permuted)
    A3   seed, and `W_FRONT3_LIFT=<a clause no call site tests>`   the NULL hatch

A cell **DISCRIMINATES** iff `A1 != A0` **and** `A2 == A0` **and** `A3 == A0`:
the instrument moved on the real lift and did not move on either fake.

Three properties of this design that are the reason for it:

* **The null sink is a PERMUTATION, not an invention.** Every token used as a
  null is some *other* frontier TU's genuine round-0 token, so it is well-formed
  by construction and cannot trip `ChainSink::parse`'s `bad` flag — which would
  disable the whole sink and make the null look maximally "discriminating" for
  the worst possible reason (board **#1285**). If a permuted token happens to be
  live on its host TU that is a MEASURED fact and the cell is reported as
  NON-discriminating, not repaired away.
* **The null hatch runs against the HATCHED binary.** Setting `W_FRONT3_LIFT` on
  the plain binary is trivially inert — there is no `front3_lift` call site in
  it at all — so a green there would be a control over a population of zero.
* **A0/A1/A2 use the plain binary** so the sink axis is not confounded by the
  hatch being compiled in.
"""

import json
import os
import subprocess
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                "..", "w-front3"))
import ladder as L                                            # noqa: E402

NULL_CLAUSE = "w-ladders-null-clause-that-no-call-site-tests"


def blockers(c2rs, one, flags, cwd, cache, sinks, hatches, out, tag):
    got = L.scan(c2rs, one, flags, cwd, cache, sinks, hatches, out, tag)
    if "__error__" in got:
        return None
    rec = got.get(open(one).read().strip())
    if rec is None:
        return None
    return dict(sorted((rec.get("fn_blockers") or {}).items()))


def main():
    L.PINNED = L.pinned_opcodes()
    tusf, flags, cwd, cache, plain, hatch, out = sys.argv[1:8]
    os.makedirs(out, exist_ok=True)
    tus = [l.strip() for l in open(tusf) if l.strip()]

    # --- each TU's OWN round-0 lift token, read from a seed-only scan --------
    real = {}
    base = {}
    for tu in tus:
        one = os.path.join(out, "one-%s.txt" % tu.replace("/", "_"))
        open(one, "w").write(tu + "\n")
        b = blockers(plain, one, flags, cwd, cache, L.SEED, [], out,
                     "A0-%s" % tu.replace("/", "_"))
        base[tu] = b
        tok = None
        if b:
            live = [k for k in sorted(b)
                    if not (k.startswith(L.TERMINAL) or k.startswith(L.TAIL))]
            for k in live:
                kind, t = L.lift_for(k)
                if kind == "sink" and t not in L.SEED:
                    tok = t
                    break
        real[tu] = tok

    pool = [t for t in (real[u] for u in tus) if t]
    rows, disc = [], 0
    for i, tu in enumerate(tus):
        one = os.path.join(out, "one-%s.txt" % tu.replace("/", "_"))
        tag = tu.replace("/", "_")
        a0 = base[tu]
        tok = real[tu]
        # the permuted null: the next DIFFERENT token in the pool
        null_tok = None
        for j in range(1, len(pool) + 1):
            cand = pool[(i + j) % len(pool)]
            if cand != tok:
                null_tok = cand
                break
        a1 = blockers(plain, one, flags, cwd, cache, L.SEED + [tok], [], out,
                      "A1-%s" % tag) if tok else None
        a2 = blockers(plain, one, flags, cwd, cache, L.SEED + [null_tok], [], out,
                      "A2-%s" % tag) if null_tok else None
        a3 = blockers(hatch, one, flags, cwd, cache, L.SEED, [NULL_CLAUSE], out,
                      "A3-%s" % tag)
        # A3 is taken on the HATCHED binary, so its baseline is that binary's
        # own seed-only scan, not A0's.
        a3b = blockers(hatch, one, flags, cwd, cache, L.SEED, [], out,
                       "A3b-%s" % tag)
        moved = (a1 is not None and a1 != a0)
        null_s = (a2 is not None and a2 == a0)
        null_h = (a3 is not None and a3b is not None and a3 == a3b)
        d = bool(moved and null_s and null_h)
        disc += d
        rows.append({"tu": tu, "real_token": tok, "null_token": null_tok,
                     "real_moved": moved, "null_sink_inert": null_s,
                     "null_hatch_inert": null_h, "discriminates": d})
        print("%-46s real=%-8s moved=%-5s | null=%-8s inert=%-5s | nullhatch "
              "inert=%-5s | %s"
              % (tu, tok, moved, null_tok, null_s, null_h,
                 "DISCRIMINATES" if d else "no"), flush=True)

    print("\nDISCRIMINATING CELLS: %d of %d" % (disc, len(tus)))
    if disc <= 4:
        print("*** THIS CONTROL IS NEAR-VACUOUS AT %d CELLS AND IS NOT BANKED "
              "AS A PASS. ***" % disc)
    json.dump(rows, open(os.path.join(out, "nullgrid.json"), "w"), indent=1)


if __name__ == "__main__":
    main()
