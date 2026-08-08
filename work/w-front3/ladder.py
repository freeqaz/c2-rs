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
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
EXPR_RS = os.path.join(ROOT, "crates/c2-il/src/func/body/expr.rs")

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
    # ADDED BY LANE w-one. `w-front3` filed this key as "a real refusal with no
    # lift" on `src/Main.cpp`; it is the model's own class stack running out, not
    # the stream, and the unsunk workload witnesses it on 829 of 878 TUs.
    ("expr-convert-no-value", "expr-convert-no-value"),
]


# A hatch whose lift is known to PANIC downstream is excluded by name rather
# than by catching the crash: `call-arg-outer-formal` on `src/keygen_xbox.cpp`
# panics at `calls.rs:71` (`index out of bounds: the len is 2 but the index is
# 2`), which is the exact failure that guard's own comment documents. The panic
# is a MEASUREMENT — the guard is reachable and load-bearing — and the row below
# it is then read with that one hatch withheld, so the rest of the ladder is
# still climbed. `W_FRONT3_SKIP_HATCH=call-arg-outer-formal`.
SKIP = [s for s in os.environ.get("W_FRONT3_SKIP_HATCH", "").split(",") if s]


def pinned_opcodes():
    """The bytes `chain_skip_form` has a WIDTH for, DERIVED from the tree. (w-ladders)

    Read out of `expr.rs`'s own match arms rather than copied into a list here.
    A second copy of a width table is a second thing to go stale, and this
    driver already has one instrument-vs-port confusion in it (`noform` is
    reported as `(noform)`, which is honest, and then the rung that produced it
    is counted, which is not).

    **It FAILS LOUDLY when it cannot parse.** An empty set would make every
    opcode look unpinned and every ladder look like renames — absence reading as
    a result, which is STATUS.md trap 5 and the thing #1322 cost a day over.
    """
    try:
        src = open(EXPR_RS).read()
        body = src[src.index("fn chain_skip_form"):]
        body = body[:body.index("\n}\n")]
    except (OSError, ValueError) as e:
        raise SystemExit("LADDER-NOWIDTHTABLE — cannot read chain_skip_form from "
                         "%s (%s). Refusing to classify rungs against an empty "
                         "width table." % (EXPR_RS, e))
    out = set()
    pat = r'^\s+(0x[0-9A-Fa-f]{2}(?:\s*\|\s*0x[0-9A-Fa-f]{2})*(?:\s*\.\.=\s*0x[0-9A-Fa-f]{2})?)\s*=>'
    for m in re.finditer(pat, body, re.M):
        s = m.group(1)
        if "..=" in s:
            a, b = s.split("..=")
            out.update(range(int(a.strip(), 16), int(b.strip(), 16) + 1))
        else:
            for t in s.split("|"):
                out.add(int(t.strip(), 16))
    if len(out) < 20:
        raise SystemExit("LADDER-NOWIDTHTABLE — parsed only %d pinned opcode(s) "
                         "from chain_skip_form; the table has never been that "
                         "small. Refusing rather than reporting every rung as a "
                         "rename." % len(out))
    return out


PINNED = None       # filled in `main`, so `import ladder` never touches the tree


def is_rename(kind, token):
    """A grant that CANNOT ADVANCE THE STREAM BY ONE BYTE. (lane w-ladders)

    `chain_step_with` reads the byte, looks it up in `chain_skip_form`, and on a
    miss returns `Err("expr-chain-noform")` **before moving the cursor**. So
    granting an opcode this tree has not pinned changes the reported key from
    `expr-op-0xNN` to `expr-chain-noform-0xNN` — the same byte, at the same
    position, under a second name — and buys **zero** decode distance.

    The round test (`blk` moved) passes on that, because the key text moved. The
    leave-one-out check in `verify` passes on it too, for the same reason. Both
    therefore book a RENAME as a rung, and six of the sixteen FRONTIER ladders
    end on exactly one: `Biquad` and `EncryptXTEA` (`op:00`), `wordwrap`
    (`op:1C`, via the named key `expr-and-and`), `Pool` (`op:10`),
    `IPP_basicmath` (`op:11`), `keygen_xbox` (`op:13`).

    This is board **#1285**'s shape a third time — a round that measured nothing
    counted as a step — and it is the reason the `EXIT` line and the rung count
    disagreed without either being wrong on its own terms: the row correctly says
    `(noform)`, *the instrument ran out*, and then charges a rung for running out.

    `net` is NOT changed, so every published table stays comparable. `stepped`
    is published beside it and is the number that is a price.
    """
    if kind != "sink" or not token.startswith("op:"):
        return False
    try:
        return int(token[3:], 16) not in PINNED
    except ValueError:
        return False


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


def tail_opcode(key):
    """The OPCODE a composite key names at its tail, or None. (lane w-one)

    `expr-call-in-expr-recv-object-then-op-0x5C` is one refusal naming TWO
    things: the production it was inside (`call-in-expr`) and the byte it
    actually stopped on (`5C`). `lift_for`'s prefix table matches the FIRST and
    returns `op:26`, which is already granted by the time this key can appear —
    so the driver reports STUCK on a key whose blocker is right there in its own
    name. `src/Main.cpp` is the live instance: it read `net=2 STUCK` before this
    and `net=3` after, and the rung it was missing is a plain opcode.

    This cannot manufacture a fictitious successor. The token goes to the
    committed sink, and `chain_skip_form` refuses any opcode whose payload width
    this tree has not pinned — so an unknown byte comes back as `noform-0xNN`,
    the honest terminal, exactly as it does when the ladder reaches one directly.
    """
    for marker in ("-then-op-0x", "-op-0x", "-0x"):
        i = key.rfind(marker)
        if i < 0:
            continue
        rest = key[i + len(marker):]
        hexpart = rest.split("-", 1)[0]
        if len(hexpart) == 2 and all(c in "0123456789ABCDEFabcdef" for c in hexpart):
            return "op:%s" % hexpart.upper()
    return None


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
        if not added:
            # SECOND PASS — the opcode a composite key names at its own tail,
            # and it is SPECULATIVE AND SELF-CONTROLLING (lane w-one).
            #
            # `expr-call-in-expr-recv-object-then-op-0x5C` names two things: the
            # production it was inside and the byte it stopped on. The prefix
            # table matches the first and returns a token already granted, so the
            # driver reports STUCK on a byte its own key named.
            #
            # But granting that byte to the chain sink is NOT the same as lifting
            # the refusal, because the refusal need not be in `parse_expr` at
            # all. Measured, on this lane's own first attempt: granting `op:5C`
            # left `src/Main.cpp` on the identical key and granting `op:26` left
            # `xlrcimpl.cpp` on the identical key — **two rungs counted for
            # nothing**, which is `Pool.cpp`'s defect (§4.1) in a new place.
            #
            # So each candidate is TRIED and kept only if the blocker set MOVES.
            # A candidate that changes nothing is discarded, named in
            # `tail_inert`, and does not enter the count.
            inert = []
            for k in sorted(live):
                tok = tail_opcode(k)
                if not tok or tok in sinks:
                    continue
                trial = scan(c2rs, one, flags, cwd, cache, sinks + [tok],
                             hatches, outdir, "%s-try-%s" % (tag, tok.replace(":", "")))
                trec = trial.get(tu) if "__error__" not in trial else None
                tblk = dict(sorted(((trec or {}).get("fn_blockers") or {}).items()))
                if trec is not None and tblk != dict(sorted(blk.items())):
                    sinks.append(tok)
                    added.append(("sink", tok, k))
                    stuck = [x for x in stuck if not x.startswith(k + "(")]
                else:
                    inert.append("%s:%s" % (k, tok))
            if inert:
                rounds[-1]["tail_inert"] = inert
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
    renames = [r for r in net if is_rename(r[0], r[1])]
    return {
        "tu": tu, "status": status,
        "rungs_raw": len(rungs), "rungs_net": len(net),
        # A RENAME is a grant that cannot advance the stream one byte; see
        # `is_rename`. `net` keeps its old meaning so published tables still
        # compare; `rungs_stepped` is the one that is a price.
        "renames": [{"token": t, "opened": k} for _, t, k in renames],
        "rungs_stepped": len(net) - len(renames),
        "sink_rungs": sum(1 for r in net if r[0] == "sink"),
        "hatch_rungs": sum(1 for r in net if r[0] == "hatch"),
        "ladder": [{"kind": k, "token": t, "opened": key} for k, t, key in net],
        "rounds": rounds,
        "final_emit": rounds[-1]["emit"] if rounds else {},
        "final_gate": rounds[-1]["gate"] if rounds else {},
        # **`final_gate` READS ONE ROUND AND THE SIGNAL IS NOT IN IT.** (w-ladders)
        #
        # `fn_gate_refusals` is the census/gate cross-check, and `TuResult`'s own
        # doc says it must be EMPTY — anything in it is the census over-claiming.
        # It is the ONLY field this driver reads that comes from past
        # `IlBundle::functions()`, and on the hatched instrument it is NOT empty:
        # `vsnprnc.cpp` carries `{"not implemented": 1}` for THIRTEEN consecutive
        # rounds and then clears, so the last round — the only one `final_gate`
        # publishes — sees nothing and the row reads as though the invariant held
        # all the way up. Board #275's shape, on a real workload TU, recorded
        # thirteen times and published zero times.
        #
        # The union over every round, so a transient cannot be dropped by
        # finishing quietly. Absence reading as success is STATUS.md trap 5.
        "gate_seen": {k: max(rd["gate"].get(k, 0) for rd in rounds)
                      for rd0 in rounds for k in rd0["gate"]},
        "gate_seen_rounds": sum(1 for rd in rounds if rd["gate"]),
    }


def verify(c2rs, tu, flags, cwd, cache, outdir, res):
    """LEAVE-ONE-OUT: is each NET rung LOAD-BEARING for the final state? (w-ladders)

    # The hole this closes, and why it is the same hole twice

    The SECOND pass (`tail_opcode`, lane `w-one`) already refuses to bank a
    candidate that does not move the blocker set — it trials it and files an inert
    one under `tail_inert`. **The FIRST pass does not.** `lift_for` maps a key to a
    token and the token is granted unconditionally; the round then advances as
    long as *some* key moved, so a grant that bought nothing is still counted as a
    rung whenever a sibling key in the same round advanced.

    That is board **#1285**'s defect — `Pool.cpp`'s published READER price came out
    of a round that measured nothing — with the disabling moved from the sink spec
    to one token inside an accepted spec. `w-one` wrote the rule down for the
    second pass and it was never applied to the first: *granting a byte is not the
    same as lifting the refusal, because the refusal need not be in `parse_expr`
    at all.*

    # What this measures, stated so it is not over-read

    For each net rung, re-scan with **that one token removed from the FINAL grant
    set** and compare `fn_blockers`. Identical ⇒ the rung is not load-bearing for
    the row's final state, and the row's `rungs_net` counts it anyway.

    This is a statement about the FINAL set, not about the order. A rung that was
    load-bearing when it was granted and was later subsumed by a token granted
    above it reads INERT here — correctly, because the price it is quoted for is
    the length of the ladder that ends where this row ends. It is also a LOWER
    bound on inertness: two tokens that are jointly redundant both read
    load-bearing under leave-ONE-out, and no amount of single-drop testing sees a
    pair. Both directions are stated because a count that is honest in one
    direction only is how #1404 credited a sink with TUs that were already clear.

    Never SCAFFOLD/SEED — `rungs_net` already subtracts those, so dropping one
    would measure the scaffold rather than the ladder.
    """
    rounds = res.get("rounds") or []
    if not rounds:
        return {"checked": 0, "inert": [], "note": "no rounds"}
    final = rounds[-1]
    sinks, hatches = list(final["sinks"]), list(final["hatches"])
    one = os.path.join(outdir, "one-%s.txt" % tu.replace("/", "_"))
    open(one, "w").write(tu + "\n")
    got = scan(c2rs, one, flags, cwd, cache, sinks, hatches, outdir,
               "%s-vbase" % tu.replace("/", "_"))
    rec = got.get(tu) if "__error__" not in got else None
    if rec is None:
        return {"checked": 0, "inert": [], "note": "verify base scan failed"}
    base = dict(sorted((rec.get("fn_blockers") or {}).items()))
    inert, checked = [], 0
    for rung in res["ladder"]:
        kind, tok = rung["kind"], rung["token"]
        if kind == "sink":
            trial_s = [s for s in sinks if s != tok]
            trial_h = hatches
        else:
            trial_s = sinks
            trial_h = [h for h in hatches if h != tok]
        checked += 1
        if is_rename(kind, tok):
            # Decided STATICALLY and reported without a scan, because a scan
            # cannot tell this apart from a real rung: the key text moves either
            # way. See `is_rename`.
            inert.append({"kind": kind, "token": tok, "opened": rung["opened"],
                          "verdict": "RENAME"})
            continue
        t = scan(c2rs, one, flags, cwd, cache, trial_s, trial_h, outdir,
                 "%s-vdrop-%s" % (tu.replace("/", "_"), tok.replace(":", "")))
        trec = t.get(tu) if "__error__" not in t else None
        if trec is None:
            # A drop that makes the scan FAIL is not an inert rung; name it.
            inert.append({"kind": kind, "token": tok, "opened": rung["opened"],
                          "verdict": "SCANFAIL-ON-DROP"})
            continue
        tb = dict(sorted((trec.get("fn_blockers") or {}).items()))
        if tb == base:
            inert.append({"kind": kind, "token": tok, "opened": rung["opened"],
                          "verdict": "INERT"})
    return {"checked": checked, "inert": inert,
            "load_bearing": checked - len(inert)}


def main():
    global PINNED
    PINNED = pinned_opcodes()
    a = sys.argv[1:]
    bound = 30
    do_verify = "--verify" in a
    if do_verify:
        a.remove("--verify")
    if "--bound" in a:
        i = a.index("--bound")
        bound = int(a[i + 1])
        del a[i:i + 2]
    tusf, flags, cwd, cache, c2rs, outdir = a[:6]
    os.makedirs(outdir, exist_ok=True)
    out = []
    for tu in [l.strip() for l in open(tusf) if l.strip()]:
        r = climb(c2rs, tu, flags, cwd, cache, outdir, bound)
        if do_verify:
            r["verify"] = verify(c2rs, tu, flags, cwd, cache, outdir, r)
        out.append(r)
        extra = ""
        if do_verify:
            v = r["verify"]
            extra = "  verify=%d/%d load-bearing%s" % (
                v.get("load_bearing", 0), v["checked"],
                ("  " + ",".join("%s:%s" % (i["verdict"], i["token"])
                                 for i in v["inert"])) if v["inert"] else "")
        print("%-46s net=%-3d step=%-3d sink=%-3d hatch=%-2d %s%s"
              % (tu, r["rungs_net"], r["rungs_stepped"], r["sink_rungs"],
                 r["hatch_rungs"], r["status"], extra),
              flush=True)
    json.dump(out, open(os.path.join(outdir, "ladder.json"), "w"), indent=1)


if __name__ == "__main__":
    main()
