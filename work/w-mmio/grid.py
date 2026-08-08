#!/usr/bin/env python3
"""w-mmio — the ENTRY-BLOCK PARK grid (board #275), generated and frozen.

Lane `w-clear` characterised the park's rule off 54 cells and refused to
implement it because **exactly one** of them (`[2,0,1]`) discriminates it.
This generator's whole job is to produce a population that discriminates it at
more than one point — and, where two rival rules cannot be separated, to say so
rather than to claim a discrimination it does not have.

STRUCTURAL AXES, crossed (values vary inside a cell):

  A  cycle length          2, 3 (in class) | 4, 5 (out-of-class control)
  B  cycle placement       every k-subset of the argument slots, arity 2..5,
                           both rotations at k=3
  C  guard target          the cycle MINIMUM's formal | a moved-but-unparked
                           formal | a formal OUTSIDE the cycle
  D  guard count           1, 2, 3
  E  trailing call count   1, 2
  F  a literal in a slot   absent | present (the `?mmioGetInfo` shape)

THE TWO RIVAL RULES, both predicted per cell:

  R-INC   the chain is `m<-s(m), s(m)<-s^2(m), ..., t<-r11` where `m` is the
          LOWEST argument register in the cycle. Hoist the park plus every move
          c_j such that dest(c_i) < dest(c_i+1) for all i <= j. Everything from
          the first descent on, inclusive, stays at the call.
          (Board #1414's rule, with the cycle-closing move INCLUDED in the scan
          — as literally written, excluding it, #1414 mis-predicts its own
          discriminating cell.)

  R-SCAN  emit the parallel copy in ascending destination order, taking a move
          only when it is ready; stop at the move sourced from r11.

The generator ASSERTS that they are indistinguishable in class and
distinguishable in the out-of-class control, and refuses to write otherwise.

Usage:
    grid.py gen  <outdir>          write the cells + manifest, print the sha256
    grid.py run  <outdir> <root>   compile every cell with the real c2 and read
                                   the split back out of the obj
"""

import hashlib
import itertools
import json
import os
import struct
import subprocess
import sys

ARG_REG = [3, 4, 5, 6, 7, 8, 9, 10]

# ---------------------------------------------------------------------------
# The permutation, its cycle, and the two rival predictions.
# ---------------------------------------------------------------------------


def cycles_of(perm):
    """sigma(slot) = perm[slot]: the destination slot is filled from perm[slot]."""
    seen, out = set(), []
    for d in range(len(perm)):
        if d in seen:
            continue
        c, x = [], d
        while x not in seen:
            seen.add(x)
            c.append(x)
            x = perm[x]
        if len(c) > 1:
            out.append(c)
    return out


def guarded_chain(perm, cyc):
    """The guarded cycle break, anchored at the cycle's LOWEST register.

    Returns the move list in dependency order as (dest_reg, src_reg) with
    src_reg == 11 for the closing read-back.
    """
    m = min(cyc)
    moves = []
    cur = m
    while True:
        nxt = perm[cur]
        if nxt == m:
            moves.append((ARG_REG[cur], 11))
            break
        moves.append((ARG_REG[cur], ARG_REG[nxt]))
        cur = nxt
    return moves


def split_r_inc(moves):
    """R-INC: hoist while the destination sequence is strictly increasing."""
    dests = [d for d, _ in moves]
    run = 1
    while run < len(dests) and dests[run - 1] < dests[run]:
        run += 1
    hoist = run - 1
    return moves[:hoist], moves[hoist:]


def split_r_scan(moves):
    """R-SCAN: ascending destination order, ready-only, stop at the r11 move."""
    dests = [d for d, _ in moves]
    hoist = 0
    for i in range(len(moves)):
        if moves[i][1] == 11:            # the r11-sourced move is deferred
            break
        if dests[i] != min(dests[i:]):   # a lower destination is still pending
            break
        hoist += 1
    return moves[:hoist], moves[hoist:]


# ---------------------------------------------------------------------------
# Source generation.
# ---------------------------------------------------------------------------

GUARD_RET = [5, 11, 7]


def cell_source(name, n, perm, guard_slots, ncalls, lit_slot):
    """One probe .cpp. `perm[i]` is the formal passed in argument slot i."""
    params = ", ".join("void *a%d" % i for i in range(n))
    args = []
    for i in range(n):
        args.append("72" if i == lit_slot else "a%d" % perm[i])
    sig = ", ".join("unsigned" if i == lit_slot else "void *" for i in range(n))
    lines = ["// w-mmio park grid cell %s" % name,
             "// arity %d perm %s guards %s calls %d lit %s"
             % (n, perm, guard_slots, ncalls, lit_slot),
             "void g%d(%s);" % (n, sig)]
    if ncalls > 1:
        lines.append("void h(void *);")
    lines.append("int f(%s) {" % params)
    for j, gs in enumerate(guard_slots):
        lines.append("    if (a%d == 0) return %d;" % (gs, GUARD_RET[j]))
    lines.append("    g%d(%s);" % (n, ", ".join(args)))
    for j in range(ncalls - 1):
        lines.append("    h(a%d);" % (n - 1))
    lines.append("    return 0;")
    lines.append("}")
    return "\n".join(lines) + "\n"


def rotations(sub):
    """Every cyclic order of `sub` as a slot->slot map, as (perm_pairs, tag)."""
    k = len(sub)
    first = sub[0]
    out = []
    for rest in itertools.permutations(sub[1:]):
        order = (first,) + rest
        # order = c0 -> c1 -> ... means sigma(c0) = c1 (c0 is FILLED FROM c1)
        pairs = {order[i]: order[(i + 1) % k] for i in range(k)}
        out.append((pairs, "".join(str(s) for s in order)))
    return out


def build_cells():
    cells = []

    def add(kind, n, pairs, guard_slots, ncalls, lit_slot):
        perm = [pairs.get(i, i) for i in range(n)]
        cyc = cycles_of(perm)
        name = "%s_n%d_p%s_g%s_c%d_l%s" % (
            kind, n, "".join(str(x) for x in perm),
            "".join(str(x) for x in guard_slots) or "0", ncalls,
            "n" if lit_slot is None else str(lit_slot))
        cells.append(dict(
            name=name, kind=kind, n=n, perm=perm,
            guard_slots=list(guard_slots), ncalls=ncalls, lit_slot=lit_slot,
            cycles=cyc,
            src=cell_source(name, n, perm, guard_slots, ncalls, lit_slot)))

    # ---- A/B: the complete k=2,3 placement grid, arity 2..5, one guard on the
    #           cycle's MINIMUM formal --------------------------------------
    for n in range(2, 6):
        for k in (2, 3):
            if k > n:
                continue
            for sub in itertools.combinations(range(n), k):
                for pairs, _tag in rotations(list(sub)):
                    add("base", n, pairs, [min(sub)], 1, None)

    # ---- C: guard target -------------------------------------------------
    #   C2 a MOVED-but-unparked formal   C3 a formal OUTSIDE the cycle
    for n in (4, 5):
        for sub in itertools.combinations(range(n), 3):
            for pairs, _tag in rotations(list(sub)):
                mx = max(sub)
                if mx != min(sub):
                    add("gtgt", n, pairs, [mx], 1, None)          # C2
                outside = [s for s in range(n) if s not in sub]
                if outside:
                    add("gout", n, pairs, [outside[0]], 1, None)  # C3

    # ---- D: guard count --------------------------------------------------
    for n in (3, 4, 5):
        for sub in itertools.combinations(range(n), min(3, n)):
            for pairs, _tag in rotations(list(sub)):
                for gc in (2, 3):
                    gs = [min(sub)] + [s for s in range(n) if s != min(sub)][:gc - 1]
                    if len(gs) == gc:
                        add("gcnt", n, pairs, gs, 1, None)
            break  # one subset per arity is enough for a count axis

    # ---- E: trailing call count -----------------------------------------
    for n in (3, 4):
        for sub in itertools.combinations(range(n), 3):
            for pairs, _tag in rotations(list(sub)):
                add("calls", n, pairs, [min(sub)], 2, None)
            break

    # ---- F: a literal in a slot — the `?mmioGetInfo` shape ---------------
    for n in (3, 4):
        for sub in itertools.combinations(range(n), 2):
            for pairs, _tag in rotations(list(sub)):
                free = [s for s in range(n) if s not in sub]
                if free:
                    add("lit", n, pairs, [min(sub)], 1, free[-1])

    # ---- the UNGUARDED control (the shipped, byte-exact lowering) --------
    for n in (2, 3, 4):
        for k in (2, 3):
            if k > n:
                continue
            for sub in itertools.combinations(range(n), k):
                for pairs, _tag in rotations(list(sub)):
                    add("ung", n, pairs, [], 1, None)

    # ---- the OUT-OF-CLASS control: 4- and 5-cycles ----------------------
    for n in (4, 5):
        for sub in itertools.combinations(range(n), n):
            for pairs, _tag in rotations(list(sub)):
                add("long", n, pairs, [min(sub)], 1, None)

    return cells


def annotate(cells):
    """Attach both rivals' predictions and the in-class flag."""
    for c in cells:
        cyc = c["cycles"]
        c["in_class"] = (len(cyc) == 1 and len(cyc[0]) <= 3
                         and c["lit_slot"] is None and len(c["guard_slots"]) >= 1)
        if len(cyc) != 1:
            c["pred"] = None
            continue
        moves = guarded_chain(c["perm"], cyc[0])
        c["chain"] = moves
        hi, ci = split_r_inc(moves)
        hs, cs = split_r_scan(moves)
        c["pred"] = dict(
            park=[11, ARG_REG[min(cyc[0])]],
            r_inc=dict(entry=hi, call=ci),
            r_scan=dict(entry=hs, call=cs),
            agree=(hi == hs))
    return cells


def gen(outdir):
    cells = annotate(build_cells())
    os.makedirs(outdir, exist_ok=True)

    # ---- the generator asserts its own classes --------------------------
    kinds = {}
    for c in cells:
        kinds.setdefault(c["kind"], 0)
        kinds[c["kind"]] += 1
    for want in ("base", "gtgt", "gout", "gcnt", "calls", "lit", "ung", "long"):
        assert kinds.get(want, 0) > 0, "structural class %r is EMPTY" % want

    # F2's requirement: cells where the GUARD's formal and the cycle minimum
    # are different registers, or the grid repeats w-clear's confound.
    confound_free = [c for c in cells
                     if c["guard_slots"] and len(c["cycles"]) == 1
                     and c["guard_slots"][0] != min(c["cycles"][0])]
    assert len(confound_free) >= 20, \
        "F2 needs cells where the guard's formal is not the cycle minimum: %d" \
        % len(confound_free)

    # The rivals: indistinguishable IN CLASS, distinguishable OUT of it.
    inc = [c for c in cells if c["in_class"] and c["pred"]]
    assert all(c["pred"]["agree"] for c in inc), \
        "R-INC and R-SCAN were expected to AGREE on every in-class cell"
    out = [c for c in cells if not c["in_class"] and c["pred"]
           and len(c["cycles"][0]) >= 4]
    sep = [c for c in out if not c["pred"]["agree"]]
    assert sep, "the out-of-class control does not SEPARATE the two rivals — " \
                "the grid would be claiming a discrimination it does not have"

    # The descent clause needs more than one witness, over more than one triple.
    descent = [c for c in inc
               if len(c["cycles"][0]) == 3
               and [d for d, _ in c["chain"]] != sorted(d for d, _ in c["chain"])]
    triples = {tuple(sorted(ARG_REG[s] for s in c["cycles"][0])) for c in descent}
    assert len(descent) >= 8 and len(triples) >= 6, \
        "the descent clause has %d witnesses over %d triples — w-clear had 1" \
        % (len(descent), len(triples))

    for c in cells:
        open(os.path.join(outdir, c["name"] + ".cpp"), "w").write(c["src"])
    manifest = [{k: v for k, v in c.items() if k != "src"} for c in cells]
    open(os.path.join(outdir, "manifest.json"), "w").write(
        json.dumps(manifest, indent=1, sort_keys=True))

    h = hashlib.sha256()
    for c in sorted(cells, key=lambda x: x["name"]):
        h.update(c["name"].encode())
        h.update(c["src"].encode())
    print("cells            %d" % len(cells))
    print("by class         %s" % json.dumps(kinds, sort_keys=True))
    print("in class         %d" % len(inc))
    print("descent witness  %d over %d distinct register triples"
          % (len(descent), len(triples)))
    print("rivals separate  %d out-of-class cells" % len(sep))
    print("guard != cycmin  %d cells (F2)" % len(confound_free))
    print("sha256           %s" % h.hexdigest())


# ---------------------------------------------------------------------------
# Reading c2 back.
# ---------------------------------------------------------------------------


def decode(w):
    op = w >> 26
    rs, ra, rb = (w >> 21) & 31, (w >> 16) & 31, (w >> 11) & 31
    if op == 31 and ((w >> 1) & 0x3FF) == 444 and rs == rb:
        return ("mr", ra, rs)
    if op == 14 and ra == 0:
        return ("li", rs, w & 0xFFFF)
    if op == 14:
        return ("addi", rs, ra, w & 0xFFFF)
    if op in (10, 11):
        return ("cmp", rs >> 2, ra, w & 0xFFFF)
    if op == 31 and ((w >> 1) & 0x3FF) in (0, 32):
        return ("cmp", rs >> 2, ra, rb)
    if op == 16:
        return ("bc",)
    if op == 18:
        return ("bl" if (w & 1) else "b",)
    if op == 37:
        return ("stwu",)
    return ("?", op)


def read_split(words):
    """(entry_moves, call_moves) — moves before the first compare, and the
    moves between the last branch and the first `bl`."""
    start = 0
    for i, w in enumerate(words):
        if decode(w)[0] == "stwu":
            start = i + 1
            break
    entry, i = [], start
    while i < len(words):
        d = decode(words[i])
        if d[0] in ("cmp", "bc", "b", "bl"):
            break
        if d[0] in ("mr", "li"):
            entry.append(d)
        i += 1
    # the first `bl` is the call; walk back over its setup
    bl = None
    for j in range(i, len(words)):
        if decode(words[j])[0] == "bl":
            bl = j
            break
    call = []
    if bl is not None:
        j = bl - 1
        while j >= 0 and decode(words[j])[0] in ("mr", "li"):
            call.append(decode(words[j]))
            j -= 1
        call.reverse()
    return entry, call


def run(outdir, root):
    sys.path.insert(0, os.path.join(root, "scripts"))
    from gt_dump import Obj

    manifest = json.load(open(os.path.join(outdir, "manifest.json")))
    dc3 = os.environ["C2RS_DC3"]
    flags = os.path.join(root, "work/dc3-workload/flags.txt")
    c2rs = os.path.join(root, "target/release/c2rs")
    objdir = os.path.join(outdir, "obj")
    os.makedirs(objdir, exist_ok=True)
    rows = []
    for c in manifest:
        cpp = os.path.join(outdir, c["name"] + ".cpp")
        obj = os.path.join(objdir, c["name"] + ".obj")
        if not os.path.exists(obj):
            # The probe includes nothing, so its own directory is the cwd; the
            # source name must be relative to it or c1xx cannot open the file.
            r = subprocess.run([c2rs, "compile", c["name"] + ".cpp",
                                "--keep-obj", obj,
                                "--flags-file", flags, "--cwd", outdir],
                               capture_output=True, text=True, cwd=outdir)
            _ = (cpp, dc3)
            if not os.path.exists(obj):
                rows.append(dict(name=c["name"], error=r.stderr.strip()[:200]))
                continue
        o = Obj(open(obj, "rb").read())
        words = None
        for s in o.sections:
            if not s["name"].startswith(".text"):
                continue
            owner = None
            for sym in o.symbols:
                if sym["sec"] == s["idx"] and sym["type"] == 0x0020 and sym["sec"] > 0:
                    owner = sym["name"]
                    break
            if owner and owner.startswith("?f@@"):
                d = o.raw(s)
                words = list(struct.unpack(">%dI" % (len(d) // 4), d))
                break
        if words is None:
            rows.append(dict(name=c["name"], error="no ?f@@ .text"))
            continue
        entry, call = read_split(words)
        rows.append(dict(name=c["name"], nbytes=len(words) * 4,
                         entry=entry, call=call,
                         words=["%08x" % w for w in words]))
    open(os.path.join(outdir, "measured.json"), "w").write(
        json.dumps(rows, indent=1))
    print("measured %d cells -> %s/measured.json" % (len(rows), outdir))


if __name__ == "__main__":
    if sys.argv[1] == "gen":
        gen(sys.argv[2])
    elif sys.argv[1] == "run":
        run(sys.argv[2], sys.argv[3])
