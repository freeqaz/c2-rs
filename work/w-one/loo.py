#!/usr/bin/env python3
"""w-one — the LEAVE-ONE-OUT control on a climbed ladder.

    python3 work/w-one/loo.py <lad/ladder.json> <flags> <dc3> <cache> <c2rs> <out>

# Why this control and not `w-front3`'s

`w-front3`'s null-lift grid asks *"would a hatch that is not the binding one
have moved the round-0 key?"* — 85 cells, 5 discriminating. On **this** lane's
population that grid is nearly vacuous: six of the seven `1 | 1` frontier TUs
open on a **sink token**, not on a hatch, so at most one cell can discriminate
and the other forty-something are structurally SAME. A control whose
discriminating count is 1 of 42 is not a control, and saying so is the point of
running it (§P5 of `work/w-one/PREREG.md`).

So the control that carries the weight here is the other one: take the ladder
the climb produced, hold it fixed, and **remove one token at a time**. Two arms,
because a grid that can only move in one direction is the same defect wearing a
different hat:

  DROP  remove a token the climb ADDED (or the SEED).  A token that was
        load-bearing re-exposes the key it opened, so the final key MOVES.
        A token that does NOT move the final key was counted and bought
        nothing — the published integer is an over-count by that many.

  ADD   grant a token the climb never needed.  The final key must NOT move.
        A cell that moves here means the sink's tokens are not independent and
        the whole ladder is an artifact of the order they were granted in.

Both arms are printed as counts. **"No cells" must be loud**, so the script
prints the discriminating count for each arm even when it is zero, and exits
non-zero if the DROP arm discriminates on nothing at all.

# What it is NOT

The sink is POISONED. Every verdict read here is `fn_blockers`, never the
differential. A row reading `expr-chain-noform-0x4F` is the chain reaching the
end-of-body opcode, not the port accepting anything.
"""

import json
import os
import subprocess
import sys

# Tokens the climb never needed on ANY of the seven, used for the ADD arm.
# Chosen from `ladder.py`'s own NAMED table so they are well-formed sink tokens
# and the arm tests independence rather than the token parser.
ADD_POOL = ["op:1A", "op:1B", "op:3B", "op:3C", "op:3D", "op:43", "op:66"]


def scan(c2rs, one, flags, cwd, cache, sinks, hatches, outdir, tag):
    env = dict(os.environ)
    if sinks:
        env["C2RS_SINK_CHAIN"] = ",".join(sinks)
    else:
        env.pop("C2RS_SINK_CHAIN", None)
    if hatches:
        env["W_FRONT3_LIFT"] = ",".join(sorted(hatches))
    else:
        env.pop("W_FRONT3_LIFT", None)
    js = os.path.join(outdir, "C-%s.jsonl" % tag)
    r = subprocess.run([c2rs, "gap", "--list", one, "--flags-file", flags,
                        "--cwd", cwd, "--jobs", "2", "--cache", cache,
                        "--jsonl", js], env=env, capture_output=True, text=True)
    if r.returncode != 0:
        return "SCANFAIL rc=%d" % r.returncode
    for ln in open(js):
        d = json.loads(ln)
        if d.get("record") == "provenance":
            continue
        return json.dumps(dict(sorted((d.get("fn_blockers") or {}).items())))
    return "NORECORD"


def main():
    ladj, flags, cwd, cache, c2rs, outdir = sys.argv[1:7]
    os.makedirs(outdir, exist_ok=True)
    rows = json.load(open(ladj))
    tot_drop = tot_drop_moved = tot_add = tot_add_moved = 0
    report = []
    for r in rows:
        tu = r["tu"]
        one = os.path.join(outdir, "one-%s.txt" % tu.replace("/", "_"))
        open(one, "w").write(tu + "\n")
        last = r["rounds"][-1]
        sinks, hatches = list(last["sinks"]), list(last["hatches"])
        base = scan(c2rs, one, flags, cwd, cache, sinks, hatches, outdir,
                    "%s-base" % tu.replace("/", "_"))
        moved_d, same_d, moved_a, same_a = [], [], [], []
        for t in sinks:
            k = scan(c2rs, one, flags, cwd, cache,
                     [s for s in sinks if s != t], hatches, outdir,
                     "%s-drop-%s" % (tu.replace("/", "_"), t.replace(":", "")))
            (moved_d if k != base else same_d).append("sink/" + t)
        for h in hatches:
            k = scan(c2rs, one, flags, cwd, cache, sinks,
                     [x for x in hatches if x != h], outdir,
                     "%s-droph-%s" % (tu.replace("/", "_"), h))
            (moved_d if k != base else same_d).append("hatch/" + h)
        for t in ADD_POOL:
            if t in sinks:
                continue
            k = scan(c2rs, one, flags, cwd, cache, sinks + [t], hatches, outdir,
                     "%s-add-%s" % (tu.replace("/", "_"), t.replace(":", "")))
            (moved_a if k != base else same_a).append(t)
        tot_drop += len(moved_d) + len(same_d)
        tot_drop_moved += len(moved_d)
        tot_add += len(moved_a) + len(same_a)
        tot_add_moved += len(moved_a)
        report.append({"tu": tu, "base": base, "net": r["rungs_net"],
                       "raw": r["rungs_raw"], "tokens": len(sinks) + len(hatches),
                       "drop_moved": moved_d, "drop_same": same_d,
                       "add_moved": moved_a, "add_same": same_a})
        print("%-40s tokens=%-3d DROP %d/%d moved   ADD %d/%d moved   "
              "not-load-bearing: %s"
              % (tu, len(sinks) + len(hatches), len(moved_d),
                 len(moved_d) + len(same_d), len(moved_a),
                 len(moved_a) + len(same_a),
                 ",".join(same_d) if same_d else "NONE"), flush=True)
    json.dump(report, open(os.path.join(outdir, "loo.json"), "w"), indent=1)
    print("\nDISCRIMINATING CELLS")
    print("  DROP arm: %d of %d moved  (must be > 0 or the grid tests nothing)"
          % (tot_drop_moved, tot_drop))
    print("  ADD  arm: %d of %d moved  (must be 0 — a move here means the "
          "tokens are not independent)" % (tot_add_moved, tot_add))
    if tot_drop_moved == 0:
        raise SystemExit("CONTROL VACUOUS: the DROP arm discriminated on NO cell")


if __name__ == "__main__":
    main()
