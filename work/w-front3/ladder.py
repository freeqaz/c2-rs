#!/usr/bin/env python3
"""w-front3 — climb a TU's refusal ladder by LIFTING, one rung at a time.

    python3 work/w-front3/ladder.py <tus.txt> <flags> <dc3> <cache> <c2rs> <out> [--bound N]

`w-mrslot/ladder.sh` is the prototype and this is it generalised: lift the
clause the TU is stopped at, re-run **on the real dc3 TU at the workload's own
flags and cwd**, read what it reports next, repeat. A first-blocker key is a
NAME, not a DISTANCE — the port stops at the first refusal by design, so every
blocked body reports exactly one blocker however many it has, and the only way
to learn a ladder's length is to climb it.

Two lift mechanisms, and which one a rung used is part of its provenance:

  SINK   `C2RS_SINK_CHAIN` — COMMITTED, POISONED (`w-depth`, board #660). A body
         that walks to the end through one refuses at `expr-chain-sink-poison`,
         so the sink can never move an obj byte. Expression layer only.
  HATCH  `W_FRONT3_LIFT` — UNCOMMITTED, and deliberately NOT poisoned
         (`work/w-front3/hatch.py`). A poisoned lift cannot reach
         `select_function`, and the CODEGEN column is precisely what a poisoned
         lift cannot see. The price is that a hatched tree can emit, so this
         driver reads `fn_blockers` / `emit_blockers` / `fn_gate_refusals` and
         **never** the differential verdict.

# What this driver fixes in `work/w-depth/chain.py`, and why it matters

`chain.py`'s `sink_token` predates board **#816**'s typed DIVIDE/MODULO key. It
maps `expr-op-0x05-8641` to the token `op:05-8641`, which `ChainSink::parse`
rejects — and a rejected token sets `bad`, which disables **the whole sink** and
makes every function in the TU refuse at `expr-chain-badtoken`. The driver then
counts that round as a step and reports the exit as `badtoken-0xB9`, a string
with the shape of a parser refusal and none of the content.

`src/system/utl/Pool.cpp` is the live instance: `w-front2` published its READER
price as **6** with exit `badtoken-0xB9`. Round 5 measured NOTHING.

So this driver carries a POSITIVE CONTROL that `chain.py` does not have:
**`assert_sink_live`** re-reads the blockers after every round and fails loudly
if `expr-chain-badtoken` appears anywhere. A control that cannot go red is not a
control, and this one goes red on exactly the defect that produced the number it
replaces.
"""

import json
import os
import subprocess
import sys

TERMINAL = "expr-chain-sink-poison"
TAIL = "expr-chain-noform-0x4F"
BADTOKEN = "expr-chain-badtoken"

SCAFFOLD = ["op:41", "op:29", "op:3A", "op:4B", "op:4F", "op:53", "op:54"]
SEED = ["op:41"]

NAMED = {
    "cmp-eq": 0x1F, "cmp-ne": 0x20, "cmp-le": 0x21, "cmp-lt": 0x22,
    "cmp-ge": 0x23, "cmp-gt": 0x24,
    "not": 0x1A, "or-or": 0x1B, "and-and": 0x1C,
    "shl": 0x09, "shr": 0x0A, "bit-and": 0x0B, "bit-or": 0x0C, "bit-xor": 0x0D,
    "convert": 0x2C, "intrinsic-call": 0x40, "class-descriptor": 0x66,
    "ternary": 0x43, "call-in-expr": 0x26,
    "label": 0x29, "brfalse": 0x38, "brtrue": 0x39, "jump": 0x3A,
    "switch-dispatch": 0x3B, "switch-table": 0x3C, "switch-case": 0x3D,
}

# key prefix -> the `W_FRONT3_LIFT` clause name that opens it.
# Every one of these is a PRODUCTION refusal, outside `parse_expr`, which the
# committed sink structurally cannot reach.
HATCHES = [
    ("param-width-undetermined", "param-width"),
    ("assign-store-type", "assign-store-type"),
    ("call-arg-lit-permuted", "call-arg-lit-permuted"),
    ("call-arg-outer-formal", "call-arg-outer-formal"),
    ("expr-shr-mixed-sign", "expr-shr-mixed-sign"),
    ("store-run-bind-mixed-kind-alloc", "store-run-bind-mixed-kind"),
]


# A hatch whose lift is known to PANIC downstream is excluded by name rather
# than by catching the crash: `call-arg-outer-formal` on `src/keygen_xbox.cpp`
# panics at `calls.rs:71` (`index out of bounds: the len is 2 but the index is
# 2`), which is the exact failure that guard's own comment documents. The panic
# is a MEASUREMENT — the guard is reachable and load-bearing — and the row below
# it is then read with that one hatch withheld, so the rest of the ladder is
# still climbed. `W_FRONT3_SKIP_HATCH=call-arg-outer-formal`.
SKIP = [s for s in os.environ.get("W_FRONT3_SKIP_HATCH", "").split(",") if s]


def lift_for(key):
    """(kind, token) that opens `key`, or (None, reason) when nothing does.

    `kind` is 'sink' or 'hatch' — the provenance of the rung.
    """
    if key.startswith(TERMINAL) or key.startswith(TAIL):
        return ("terminal", None)
    for pre, name in HATCHES:
        if key.startswith(pre):
            if name in SKIP:
                return (None, "hatch-withheld:%s" % name)
            return ("hatch", name)
    if key.startswith("expr-chain-noform") or key.startswith("expr-chain-short"):
        # The INSTRUMENT's own width table, not the port's. Reported as such.
        return (None, "noform")
    if key.startswith(BADTOKEN):
        return (None, "badtoken")
    if key.startswith("expr-load-type-") or key.startswith("expr-lit-type-"):
        return ("sink", "type")
    if key.startswith("expr-convert-target"):
        return ("sink", "convert")
    if key.startswith("expr-intrinsic-"):
        return ("sink", "intrinsic")
    if key.startswith("expr-call-in-expr"):
        return ("sink", "op:26")
    if key.startswith("expr-op-0x"):
        # THE FIX. Board #816 refines `expr-op-0xNN` into `expr-op-0xNN-TTTT`
        # (the operand TYPE). The sink is keyed on the OPCODE, so only the two
        # hex digits after `0x` are the token — `chain.py` passed the whole
        # `05-8641` and silently disabled the sink.
        rest = key[len("expr-op-0x"):]
        return ("sink", "op:%s" % rest.split("-", 1)[0])
    if key.startswith("expr-"):
        r = key[len("expr-"):]
        if r in NAMED:
            return ("sink", "op:%02X" % NAMED[r])
    return (None, "no-lift")


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
    js = os.path.join(outdir, "L-%s.jsonl" % tag)
    cmd = [c2rs, "gap", "--list", one, "--flags-file", flags, "--cwd", cwd,
           "--jobs", "4", "--cache", cache, "--jsonl", js]
    r = subprocess.run(cmd, env=env, capture_output=True, text=True)
    if r.returncode != 0:
        return {"__error__": "gap rc=%d %s" % (r.returncode, r.stderr[-400:])}
    out = {}
    for ln in open(js):
        d = json.loads(ln)
        if d.get("record") == "provenance":
            continue
        out[d["src"]] = d
    return out


def climb(c2rs, tu, flags, cwd, cache, outdir, bound):
    one = os.path.join(outdir, "one-%s.txt" % tu.replace("/", "_"))
    open(one, "w").write(tu + "\n")
    sinks, hatches = list(SEED), []
    rungs = []          # (kind, token, the key it opened)
    rounds = []
    status = "BOUND"
    rec = None
    for rnd in range(bound + 1):
        tag = "%s-%d" % (tu.replace("/", "_"), rnd)
        got = scan(c2rs, one, flags, cwd, cache, sinks, hatches, outdir, tag)
        if "__error__" in got:
            status = "SCANFAIL:" + got["__error__"][:80]
            break
        rec = got.get(tu)
        if rec is None:
            status = "NORECORD"
            break
        blk = rec.get("fn_blockers") or {}
        emit = rec.get("emit_blockers") or {}
        gate = rec.get("fn_gate_refusals") or {}
        rounds.append({"round": rnd, "reader": dict(sorted(blk.items())),
                       "emit": dict(sorted(emit.items())),
                       "gate": dict(sorted(gate.items())),
                       "sinks": list(sinks), "hatches": sorted(hatches)})
        # --- POSITIVE CONTROL: the sink spec was ACCEPTED this round --------
        if any(k.startswith(BADTOKEN) for k in blk):
            status = "CONTROL-RED:badtoken — the sink was globally DISABLED this round"
            break
        live = {k: v for k, v in blk.items()
                if not (k.startswith(TERMINAL) or k.startswith(TAIL))}
        if not live:
            status = "READER-CLEAR"
            break
        added, stuck = [], []
        for k in sorted(live):
            kind, tok = lift_for(k)
            if kind is None:
                stuck.append("%s(%s)" % (k, tok))
            elif kind == "terminal":
                continue
            elif kind == "sink" and tok not in sinks:
                sinks.append(tok)
                added.append(("sink", tok, k))
            elif kind == "hatch" and tok not in hatches:
                hatches.append(tok)
                added.append(("hatch", tok, k))
        if stuck and not added:
            status = "EXIT:" + ";".join(stuck)
            break
        if not added:
            status = "STUCK:" + ",".join(sorted(live))
            break
        rungs.extend(added)
        if stuck:
            # Some keys advanced, others cannot. Keep climbing the ones that can
            # and carry the stuck set forward — the row is a LOWER BOUND and the
            # reason is named.
            rounds[-1]["stuck"] = stuck
    net = [r for r in rungs if not (r[0] == "sink" and r[1] in SCAFFOLD)]
    return {
        "tu": tu, "status": status,
        "rungs_raw": len(rungs), "rungs_net": len(net),
        "sink_rungs": sum(1 for r in net if r[0] == "sink"),
        "hatch_rungs": sum(1 for r in net if r[0] == "hatch"),
        "ladder": [{"kind": k, "token": t, "opened": key} for k, t, key in net],
        "rounds": rounds,
        "final_emit": rounds[-1]["emit"] if rounds else {},
        "final_gate": rounds[-1]["gate"] if rounds else {},
    }


def main():
    a = sys.argv[1:]
    bound = 30
    if "--bound" in a:
        i = a.index("--bound")
        bound = int(a[i + 1])
        del a[i:i + 2]
    tusf, flags, cwd, cache, c2rs, outdir = a[:6]
    os.makedirs(outdir, exist_ok=True)
    out = []
    for tu in [l.strip() for l in open(tusf) if l.strip()]:
        r = climb(c2rs, tu, flags, cwd, cache, outdir, bound)
        out.append(r)
        print("%-46s net=%-3d sink=%-3d hatch=%-2d %s"
              % (tu, r["rungs_net"], r["sink_rungs"], r["hatch_rungs"], r["status"]),
              flush=True)
    json.dump(out, open(os.path.join(outdir, "ladder.json"), "w"), indent=1)


if __name__ == "__main__":
    main()
