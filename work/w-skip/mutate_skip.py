#!/usr/bin/env python3
"""mutate_skip.py — the OWNER SKIPS of `0x10b98e26`, tested against the SOLE
JUDGE, in the direction that can go RED.

PREREG §5 M13/M14 plus the W2 arm.  w-roots flipped the seed bit; w-refs zeroed a
use count; w-mark retargeted an initializer node.  All three moved a *target*.
This moves the **owner**: it writes the owner's `.gl` flag word `+0x20` — the
`varU` read at `0x10b9ba0d` — so that a gate this lane claims to have decoded
fires, and asks the real `c2.dll` whether the functions that owner's initializer
names stop being emitted.

The arms, and why the second one is the whole point:

    A  SKIP 1 POSITIVE (M13)      f20 |= 0x20   -> (f20 & 0x60) == 0x20
                                  0x10b98e9f skips the record ENTIRELY
                                  PREDICTION: the record's functions are LOST
    B  SKIP 1 CONTROL  (M14)      f20 |= 0x60   -> (f20 & 0x60) == 0x60
                                  the compare at 0x10b98ea2 does NOT match
                                  PREDICTION: NOTHING is lost
    C  W2 SET                     f20 |= 0x400  on an owner that lacks 0x480
                                  0x10b98b14 / 0x10b98c89 stop refusing it
                                  PREDICTION: its functions are GAINED
    D  W2 CLEAR                   f20 &= ~0x400 on an owner that has it (and not
                                  0x80), so the walk starts refusing it
                                  PREDICTION: its functions are LOST

**Arm B is the arm that decides whether arm A means anything.**  A and B write
the *same byte* at the *same offset* and differ in one bit.  If any write to that
byte broke emission, B would go red too and A would be worthless.  A red B
invalidates A, and that is registered before the run.

Byte-length preservation: `enc_var_u` is 2 bytes iff the value is < 0x8000, so
every mutation asserts the re-encoded width equals the original.  Bit `0x200` —
which gates an *extra* `varU` in the header (`0x10b9ba5f`) — is never touched,
and that is asserted too: changing it would resync the whole record and the
control would be meaningless rather than red.

    usage: mutate_skip.py <src.cpp> [n_per_arm]
"""
import os
import shutil
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(REPO, "work", "w-roots"))
sys.path.insert(0, os.path.join(REPO, "work", "w-refs"))
sys.path.insert(0, os.path.join(REPO, "work", "w-mark"))
sys.path.insert(0, os.path.join(REPO, "work", "emitpred", "pipeline"))
import il          # noqa: E402
import refs        # noqa: E402
import glowner     # noqa: E402
import marks as mk  # noqa: E402
import mutate      # noqa: E402  (w-roots' capture / replay / leaders, verbatim)
from glflags import enc_var_u   # noqa: E402

mutate.WORK = os.path.join(HERE, "mut")


def closure(seed, edges, U, skip=()):
    seen = set(x for x in seed if x in U)
    stack = list(seen)
    while stack:
        a = stack.pop()
        for f in edges.get(a, ()):
            if f not in seen and f in U and f not in skip:
                seen.add(f)
                stack.append(f)
    return seen


def main():
    src = sys.argv[1]
    want = int(sys.argv[2]) if len(sys.argv) > 2 else 3

    bdir, base, argv, base_obj = mutate.capture(src)
    glp = os.path.join(bdir, base + "gl")
    glb0 = open(glp, "rb").read()
    exb = open(os.path.join(bdir, base + "ex"), "rb").read()
    inb = open(os.path.join(bdir, base + "in"), "rb").read()
    print("bundle", base, " gl", len(glb0), " in", len(inb))

    recs, _ = refs.scan(glb0, exb, wide_count=True)
    U = set(recs)
    seed = set(k for k, v in recs.items() if v["seed"])
    xskip = set(k for k, v in recs.items() if v["skip"])
    P = closure(seed, refs.edges(glb0, recs, U), U, xskip)
    idx = il.gl_symbol_index(glb0)
    syms, sst = glowner.read_symbols(glb0)
    ok, inrecs = mk.parse_records(inb)
    print("gl owner records bound=%d (k1=%d k4=%d), in clean=%s recs=%d"
          % (sst["bound"], sst["k1"], sst["k4"], ok, len(inrecs)))

    rb, err = mutate.replay(bdir, base, argv, os.path.join(bdir, "b", "out.obj"))
    if rb is None:
        raise SystemExit("baseline replay produced no obj: " + err[-2000:])
    L0 = mutate.leaders(rb)
    b0 = bytearray(base_obj)
    b0[4:8] = b"\0\0\0\0"
    r0 = bytearray(rb)
    r0[4:8] = b"\0\0\0\0"
    print("BASELINE replay: bytes==pipeline obj %s ; LEADER SET==pipeline obj "
          "%s ; leaders %d" % (bytes(b0) == bytes(r0),
                               mutate.leaders(base_obj) == L0, len(L0)))

    # per owner token: the emitted, not-Seed-reachable functions it names
    owned = {}
    for _t, _f, o, toks in inrecs:
        acc = owned.setdefault(o, set())
        for t in toks:
            nm = idx.get(t)
            if nm in U:
                acc.add(nm)
    interesting = {}
    for o, names in owned.items():
        r = syms.get(o)
        if r is None or r["kind"] != 1:
            continue
        # PAYLOAD: the emitted functions this owner's initializer names.
        # Preferring `not in P` would make arm A test "the ONLY reason", which
        # no owner on some TUs satisfies; the wider rule is reported with the
        # narrow count beside it so a hit can be read for what it is.
        payload = set(n for n in names if n in L0)
        solo = set(n for n in payload if n not in P)
        gainable = set(n for n in names if n not in L0)
        interesting[o] = (r, payload, gainable, solo)

    def run(tag, tok, newf20, expect):
        r = syms[tok]
        old = r["f20"]
        assert (old ^ newf20) & 0x200 == 0, "must not touch the 0x200 gate"
        enc = enc_var_u(newf20)
        if len(enc) != r["f20_width"]:
            return None
        mb = bytearray(glb0)
        mb[r["f20_pos"]:r["f20_pos"] + r["f20_width"]] = enc
        assert len(mb) == len(glb0)
        open(glp, "wb").write(bytes(mb))
        od = os.path.join(bdir, "m_%s_%d" % (tag, tok))
        shutil.rmtree(od, ignore_errors=True)
        mo, merr = mutate.replay(bdir, base, argv, os.path.join(od, "out.obj"))
        open(glp, "wb").write(glb0)
        if mo is None:
            print("  [%s] REPLAY FAIL %s: %s" % (tag, r["name"][:50], merr[-160:]))
            return None
        L1 = mutate.leaders(mo)
        return L0 - L1, L1 - L0, old

    score = {}
    # ---------------- arms A and B: SKIP 1, on the same owners ----------
    cands = sorted([(o, v) for o, v in interesting.items()
                    if v[1] and (v[0]["f20"] & 0x480) and (v[0]["f20"] & 0x60) == 0],
                   key=lambda kv: (-len(kv[1][3]), -len(kv[1][1]),
                                   kv[1][0]["name"]))
    print("\nSKIP-1 candidates (walk-enabled owners naming emitted non-Seed "
          "functions): %d" % len(cands))
    a_ok = b_ok = a_n = b_n = 0
    for o, (r, payload, _g, solo) in cands[:want]:
        res = run("A", o, r["f20"] | 0x20, "lost")
        if res is None:
            continue
        lost, gained, old = res
        a_n += 1
        hit = bool(payload & lost)
        a_ok += 1 if hit else 0
        print("  [A] %-50s f20 %#06x->%#06x  payload=%d(solo %d) lost=%d "
              "gained=%d  SKIP1-fires=%s" % (r["name"][:50], old, old | 0x20,
                                             len(payload), len(solo), len(lost),
                                             len(gained), hit))
        if not hit:
            print("      payload: %s" % sorted(payload)[:3])
            print("      lost:    %s" % sorted(lost)[:3])
        res = run("B", o, r["f20"] | 0x60, "nothing")
        if res is None:
            continue
        lost, gained, old = res
        b_n += 1
        clean = (len(lost) == 0 and len(gained) == 0)
        b_ok += 1 if clean else 0
        print("  [B] %-50s f20 %#06x->%#06x  lost=%d gained=%d  "
              "CONTROL-clean=%s" % (r["name"][:50], old, old | 0x60,
                                    len(lost), len(gained), clean))
        if not clean:
            print("      lost:   %s" % sorted(lost)[:3])
            print("      gained: %s" % sorted(gained)[:3])
    score["M13 SKIP1 positive (payload lost)"] = (a_ok, a_n)
    score["M14 SKIP1 control (nothing moves)"] = (b_ok, b_n)

    # ---------------- arm C: W2 SET on a refused owner ------------------
    cw = sorted([(o, v) for o, v in interesting.items()
                 if not (v[0]["f20"] & 0x480) and v[2]],
                key=lambda kv: (-len(kv[1][2]), kv[1][0]["name"]))
    print("\nW2-SET candidates (owners the walk refuses, naming unemitted "
          "functions): %d" % len(cw))
    c_ok = c_n = 0
    for o, (r, _p, gainable, _s) in cw[:want]:
        res = run("C", o, r["f20"] | 0x400, "gained")
        if res is None:
            continue
        lost, gained, old = res
        c_n += 1
        hit = bool(gainable & gained)
        c_ok += 1 if hit else 0
        print("  [C] %-50s f20 %#06x->%#06x  gainable=%d lost=%d gained=%d  "
              "W2-opens=%s" % (r["name"][:50], old, old | 0x400,
                               len(gainable), len(lost), len(gained), hit))
        if not hit:
            print("      gainable: %s" % sorted(gainable)[:3])
            print("      gained:   %s" % sorted(gained)[:3])
    score["W2 set (refused owner starts marking)"] = (c_ok, c_n)

    # ---------------- arm D: W2 CLEAR on a walked owner -----------------
    cd = sorted([(o, v) for o, v in interesting.items()
                 if (v[0]["f20"] & 0x480) == 0x400 and v[1]],
                key=lambda kv: (-len(kv[1][3]), -len(kv[1][1]),
                                kv[1][0]["name"]))
    print("\nW2-CLEAR candidates (owners the walk accepts via 0x400 only): %d"
          % len(cd))
    d_ok = d_n = 0
    for o, (r, payload, _g, _s) in cd[:want]:
        res = run("D", o, r["f20"] & ~0x400, "lost")
        if res is None:
            continue
        lost, gained, old = res
        d_n += 1
        hit = bool(payload & lost)
        d_ok += 1 if hit else 0
        print("  [D] %-50s f20 %#06x->%#06x  payload=%d lost=%d gained=%d  "
              "W2-closes=%s" % (r["name"][:50], old, old & ~0x400,
                                len(payload), len(lost), len(gained), hit))
        if not hit:
            print("      payload: %s" % sorted(payload)[:3])
            print("      lost:    %s" % sorted(lost)[:3])
    score["W2 clear (walked owner stops marking)"] = (d_ok, d_n)

    ce = sorted([(o, v) for o, v in interesting.items()
                 if (v[0]["f20"] & 0x480) and not (v[0]["f20"] & 0x4000)
                 and v[1]],
                key=lambda kv: (-len(kv[1][3]), -len(kv[1][1]),
                                kv[1][0]["name"]))
    print("\nSKIP-3 candidates (walked owners without 0x4000): %d" % len(ce))
    e_ok = e_n = 0
    for o, (r, payload, _g, solo) in ce[:want]:
        res = run("E", o, r["f20"] | 0x4000, "lost")
        if res is None:
            continue
        lost, gained, old = res
        e_n += 1
        hit = bool(payload & lost)
        e_ok += 1 if hit else 0
        print("  [E] %-50s f20 %#06x->%#06x  payload=%d(solo %d) lost=%d "
              "gained=%d  SKIP3-fires=%s" % (r["name"][:50], old, old | 0x4000,
                                             len(payload), len(solo), len(lost),
                                             len(gained), hit))
    score["SKIP3 set (S5 pass skipped)"] = (e_ok, e_n)

    print("\n---- %s" % src)
    for k, (a, b) in score.items():
        print("  %-46s %d/%d" % (k, a, b))


if __name__ == "__main__":
    main()
