#!/usr/bin/env python3
"""w-memcpy GRID-L — WHERE DOES A LITERAL ARGUMENT'S `li` GO in a FRAMED
sequence call's marshalling?  (`callseq-multiarg-lit`, board #1444.)

`c2_il::…::shapes::calls::seq_call_arg_sources` refuses `SlotArg::Lit` in a
framed sequence call, and its own doc gives the reason positively:

    a framed call's marshalling interleaves with the callee-saved copies
    (`plan_saved_gprs`'s hoist/trail rule) and with the previous `bl`'s result
    save, and EVERY WITNESS of that interleaving is a `mr`.

So the gap is exactly one question — the literal's POSITION relative to the
`mr`s — and this grid answers it by enumeration.

WHY `?mmioGetInfo` ALONE CANNOT ANSWER IT (the w-clear confound, repeating)
--------------------------------------------------------------------------
`?mmioGetInfo`'s literal is in slot 2 and its only surviving move writes slot 1,
so `li r5,72` precedes `mr r4,r11` under R-DESC *and* under R-LITFIRST *and*
under any rule that puts a higher destination first.  One cell separates
nothing.  This generator therefore ASSERTS a minimum count of cells on which
each pair of rivals disagrees, and refuses to write the grid otherwise — the
same standard `work/w-mmio/grid.py` sets, for the same reason.

THE RIVALS, as RELATIVE ORDERINGS
---------------------------------
A rival is frozen per cell as a total prediction over every (literal, move)
pair, plus one bit per literal for "is it in the entry block".  Stating them
relatively rather than absolutely is deliberate: the `mr` ORDER is board
#1414/#1443's question and was settled over 886 cells by lane `w-mmio`; this
grid does not re-open it, it re-checks it as a control and asks only where the
`li` lands inside it.

  R-DESC      lit at dest d precedes a move to dest d'  iff  d > d'
              (`c2_core::codegen::permute_args_parts`' descending walk — the
               rule the TAIL-call form already ships, WLA)
  R-LITLAST   every lit follows every move
  R-LITFIRST  every lit precedes every move
  R-ASC       lit at dest d precedes a move to dest d'  iff  d < d'
  R-PARKLIT   (guarded cells only) every lit is HOISTED INTO THE ENTRY BLOCK,
              the way the park's own moves are

STRUCTURAL AXES, crossed; values vary inside a cell
---------------------------------------------------
  A  arity                 2, 3, 4, 5
  B  literal slot(s)       every single slot; every PAIR of slots (n <= 4)
  C  the other slots' map  every injective map of the non-literal slots onto
                           the caller's formals (n <= 4); a structural sample
                           at n = 5
  D  frame driver          g1  one guarded early return + one call  (Class A,
                                the park's own shape — `?mmioGetInfo`)
                           g2  two guarded early returns + one call
                           c2  two calls, no guard                  (Class A)
                           lv  a formal live ACROSS the call        (Class B,
                                the callee-saved interleave the refusal names)
  E  literal value         5 | 72 | -1 | 0x7fff (the `li` immediate boundary)
                           and 0x8000 as an OUT-OF-CLASS control (`lis`+`ori`)
  F  control               the UNGUARDED TAIL form of the same slot list —
                           WLA's own class, byte-exact today when no formal
                           moves, `call-arg-lit-permuted` when one does

Usage:
    gridl.py gen  <outdir>          write cells + manifest, print the sha256
    gridl.py run  <outdir> <root>   compile each cell with the real c2, read
                                    the setup order back out of the obj
"""

import hashlib
import itertools
import json
import os
import struct
import subprocess
import sys

ARG_REG = [3, 4, 5, 6, 7, 8, 9, 10]
GUARD_RET = [5, 11, 7]
LIT_VALUES = [72, 5, -1, 0x7FFF]
LIT_WIDE = 0x8000  # out of class: `li` cannot hold it


# ---------------------------------------------------------------------------
# Source generation
# ---------------------------------------------------------------------------


def cell_source(name, n, slots, guards, kind, meta):
    """One probe .cpp.

    `slots[i]` is either ('f', formal_index) or ('l', value).
    """
    params = ", ".join("void *a%d" % i for i in range(n))
    args, sig = [], []
    for i in range(n):
        k, v = slots[i]
        if k == "f":
            args.append("a%d" % v)
            sig.append("void *")
        else:
            args.append(str(v))
            sig.append("int")
    lines = [
        "// w-memcpy GRID-L cell %s" % name,
        "// %s" % json.dumps(meta, sort_keys=True),
        "void g%d(%s);" % (n, ", ".join(sig)),
    ]
    if kind in ("c2", "lv"):
        lines.append("void h(void *);")
    lines.append("int f(%s) {" % params)
    for j, gs in enumerate(guards):
        lines.append("    if (a%d == 0) return %d;" % (gs, GUARD_RET[j]))
    lines.append("    g%d(%s);" % (n, ", ".join(args)))
    if kind == "c2":
        lines.append("    h(0);")           # a second call, nothing live across
    elif kind == "lv":
        lines.append("    h(a0);")          # a0 LIVE ACROSS the first call
    lines.append("    return 0;")
    lines.append("}")
    return "\n".join(lines) + "\n"


def tail_control_source(name, n, slots, meta):
    """F — the UNGUARDED TAIL form of the same slot list (WLA's own class)."""
    params = ", ".join("void *a%d" % i for i in range(n))
    args, sig = [], []
    for i in range(n):
        k, v = slots[i]
        args.append("a%d" % v if k == "f" else str(v))
        sig.append("void *" if k == "f" else "int")
    return (
        "// w-memcpy GRID-L tail control %s\n"
        "// %s\n"
        "void g%d(%s);\n"
        "void f(%s) { g%d(%s); }\n"
        % (name, json.dumps(meta, sort_keys=True), n, ", ".join(sig),
           params, n, ", ".join(args))
    )


# ---------------------------------------------------------------------------
# The rivals — every one a TOTAL relative prediction, frozen per cell
# ---------------------------------------------------------------------------


def predictions(n, slots, guarded):
    """{rival: {'pairs': {"lit@d|mv@d'": 'lit-first'|'mv-first'},
                'entry': [lit dest regs hoisted into the entry block]}}

    A "move" here is any non-literal slot whose formal is not already in place;
    the grid does not claim to know the move ORDER (board #1443 settled that),
    only where the literals sit relative to each move.
    """
    lits = [ARG_REG[i] for i in range(n) if slots[i][0] == "l"]
    moves = [ARG_REG[i] for i in range(n)
             if slots[i][0] == "f" and slots[i][1] != i]
    out = {}
    for rival in ("R-DESC", "R-LITLAST", "R-LITFIRST", "R-ASC", "R-PARKLIT"):
        pairs = {}
        for ld in lits:
            for md in moves:
                if rival == "R-DESC":
                    first = "lit" if ld > md else "mv"
                elif rival == "R-ASC":
                    first = "lit" if ld < md else "mv"
                elif rival == "R-LITLAST":
                    first = "mv"
                else:  # R-LITFIRST, R-PARKLIT (which is lit-first and higher)
                    first = "lit"
                pairs["lit@%d|mv@%d" % (ld, md)] = first
        out[rival] = dict(
            pairs=pairs,
            entry=(lits if (rival == "R-PARKLIT" and guarded) else []),
        )
    return out


def disagreements(pred):
    """{('R-A','R-B'): True} for every rival pair this cell separates."""
    names = sorted(pred)
    sep = {}
    for a, b in itertools.combinations(names, 2):
        differs = pred[a]["pairs"] != pred[b]["pairs"] or \
            pred[a]["entry"] != pred[b]["entry"]
        sep["%s/%s" % (a, b)] = differs
    return sep


# ---------------------------------------------------------------------------
# Cell construction
# ---------------------------------------------------------------------------


def build_cells():
    cells = []
    seen = set()

    def add(kind, n, slots, guards, litval_tag):
        meta = dict(n=n, kind=kind, guards=list(guards),
                    slots=[list(s) for s in slots], litval=litval_tag)
        name = "%s_n%d_%s_g%s_%s" % (
            kind, n,
            "".join(("L" if s[0] == "l" else str(s[1])) for s in slots),
            "".join(str(g) for g in guards) or "n", litval_tag)
        if name in seen:
            return
        seen.add(name)
        guarded = bool(guards)
        pred = predictions(n, slots, guarded)
        cells.append(dict(
            name=name, kind=kind, n=n,
            slots=[list(s) for s in slots], guards=list(guards),
            guarded=guarded, litval=litval_tag,
            nlit=sum(1 for s in slots if s[0] == "l"),
            nmove=sum(1 for i, s in enumerate(slots)
                      if s[0] == "f" and s[1] != i),
            in_class=(litval_tag != "wide"),
            pred=pred, sep=disagreements(pred),
            src=cell_source(name, n, slots, guards, kind, meta)))

    def add_tail(n, slots, litval_tag):
        meta = dict(n=n, kind="tail", slots=[list(s) for s in slots],
                    litval=litval_tag)
        name = "tail_n%d_%s_%s" % (
            n, "".join(("L" if s[0] == "l" else str(s[1])) for s in slots),
            litval_tag)
        if name in seen:
            return
        seen.add(name)
        pred = predictions(n, slots, False)
        cells.append(dict(
            name=name, kind="tail", n=n,
            slots=[list(s) for s in slots], guards=[], guarded=False,
            litval=litval_tag,
            nlit=sum(1 for s in slots if s[0] == "l"),
            nmove=sum(1 for i, s in enumerate(slots)
                      if s[0] == "f" and s[1] != i),
            in_class=(litval_tag != "wide"),
            pred=pred, sep=disagreements(pred),
            src=tail_control_source(name, n, slots, meta)))

    def slot_lists(n, lit_slots, val):
        """Every injective map of the NON-literal slots onto the formals."""
        free = [i for i in range(n) if i not in lit_slots]
        for assign in itertools.permutations(range(n), len(free)):
            slots = [None] * n
            for i in lit_slots:
                slots[i] = ("l", val)
            for i, formal in zip(free, assign):
                slots[i] = ("f", formal)
            yield slots

    # ---- A/B/C: one literal, every slot, every map, at n = 2..4 ----------
    for n in (2, 3, 4):
        for ls in range(n):
            for slots in slot_lists(n, {ls}, 72):
                add("g1", n, slots, [0], "72")

    # ---- D: the four frame drivers, over the n = 3 population ------------
    for ls in range(3):
        for slots in slot_lists(3, {ls}, 72):
            add("g2", 3, slots, [0, 1], "72")
            add("c2", 3, slots, [], "72")
            add("lv", 3, slots, [], "72")

    # ---- B': TWO literals, n = 3 and 4 -----------------------------------
    for n in (3, 4):
        for ls in itertools.combinations(range(n), 2):
            for slots in slot_lists(n, set(ls), 72):
                add("g1", n, slots, [0], "72")

    # ---- C': a structural sample at n = 5 (full enumeration is 5*120) ----
    for ls in range(5):
        free = [i for i in range(5) if i != ls]
        for assign in (tuple(free),                       # identity
                       tuple(reversed(free)),             # full reversal
                       tuple(free[1:] + free[:1]),        # rotate
                       tuple([free[1], free[0]] + free[2:])):  # one swap
            slots = [None] * 5
            slots[ls] = ("l", 72)
            for i, formal in zip(free, assign):
                slots[i] = ("f", formal)
            add("g1", 5, slots, [0], "72")

    # ---- E: the literal VALUE axis, over a fixed structural sample -------
    for val, tag in ((5, "5"), (-1, "neg"), (0x7FFF, "imax"),
                     (LIT_WIDE, "wide")):
        for n in (3, 4):
            for ls in range(n):
                for slots in slot_lists(n, {ls}, val):
                    add("g1", n, slots, [0], tag)
                    break   # one map per (value, slot) — value is not structural
                for slots in slot_lists(n, {ls}, val):
                    add("g1", n, slots, [0], tag)

    # ---- F: the UNGUARDED TAIL control for the whole n = 3 population ----
    for ls in range(3):
        for slots in slot_lists(3, {ls}, 72):
            add_tail(3, slots, "72")

    return cells


# ---------------------------------------------------------------------------
# gen
# ---------------------------------------------------------------------------


def gen(outdir):
    cells = build_cells()
    os.makedirs(outdir, exist_ok=True)

    kinds = {}
    for c in cells:
        kinds[c["kind"]] = kinds.get(c["kind"], 0) + 1
    for want in ("g1", "g2", "c2", "lv", "tail"):
        assert kinds.get(want, 0) > 0, "structural class %r is EMPTY" % want

    # ---- the generator asserts its own DISCRIMINATION --------------------
    #
    # The whole point of the grid.  `?mmioGetInfo` separates R-DESC from
    # R-LITLAST and from nothing else; a grid that could not do better would be
    # `w-clear`'s five cells one production over.
    pair_counts = {}
    for c in cells:
        if not c["in_class"]:
            continue
        for pair, differs in c["sep"].items():
            if differs:
                pair_counts[pair] = pair_counts.get(pair, 0) + 1
    for pair in ("R-ASC/R-DESC", "R-DESC/R-LITFIRST", "R-DESC/R-LITLAST",
                 "R-LITFIRST/R-LITLAST", "R-ASC/R-LITFIRST",
                 "R-ASC/R-LITLAST"):
        assert pair_counts.get(pair, 0) >= 20, \
            "rival pair %s is separated by only %d cells (need >= 20)" \
            % (pair, pair_counts.get(pair, 0))
    # R-PARKLIT is separated from R-LITFIRST only by GUARDED cells.
    guarded_sep = sum(1 for c in cells
                      if c["in_class"] and c["sep"].get("R-LITFIRST/R-PARKLIT"))
    assert guarded_sep >= 20, \
        "R-PARKLIT is separated by only %d guarded cells" % guarded_sep

    # The `?mmioGetInfo` shape itself must BE in the grid, and the grid must
    # contain cells it does not cover — stated, not assumed.
    mmio_like = [c for c in cells
                 if c["n"] == 3 and c["guarded"] and c["kind"] == "g1"
                 and c["slots"] == [["f", 1], ["f", 0], ["l", 72]]]
    assert mmio_like, "the ?mmioGetInfo slot list is NOT in the grid"
    # ...and it must NOT separate R-DESC from R-LITFIRST.  That is the
    # confound, stated as an assertion rather than discovered later: the target
    # function's literal sits ABOVE both of its moves, so "descending
    # destination" and "literals first" predict the same three words, and a lane
    # that graded itself on `?mmioGetInfo` alone would ship whichever of them it
    # happened to write.  This is `w-clear`'s five-cells-all-guarding-`a0` one
    # production over, and #1443 is what it cost there.
    mmio_sep = sum(1 for p, d in mmio_like[0]["sep"].items() if d)
    assert not mmio_like[0]["sep"]["R-DESC/R-LITFIRST"], \
        "the ?mmioGetInfo cell was expected NOT to separate R-DESC from " \
        "R-LITFIRST — the confound this grid exists to break"
    assert not mmio_like[0]["sep"]["R-ASC/R-LITLAST"], \
        "the ?mmioGetInfo cell was expected NOT to separate R-ASC from R-LITLAST"

    # A literal BELOW a move's destination is what separates R-DESC from
    # R-LITFIRST; count it explicitly so its absence cannot pass silently.
    below = 0
    for c in cells:
        if not c["in_class"]:
            continue
        lits = [ARG_REG[i] for i, s in enumerate(c["slots"]) if s[0] == "l"]
        moves = [ARG_REG[i] for i, s in enumerate(c["slots"])
                 if s[0] == "f" and s[1] != i]
        if any(ld < md for ld in lits for md in moves):
            below += 1
    assert below >= 20, "only %d cells put a literal BELOW a move" % below

    for c in cells:
        open(os.path.join(outdir, c["name"] + ".cpp"), "w").write(c["src"])
    manifest = [{k: v for k, v in c.items() if k != "src"} for c in cells]
    open(os.path.join(outdir, "manifest.json"), "w").write(
        json.dumps(manifest, indent=1, sort_keys=True))

    h = hashlib.sha256()
    for c in sorted(cells, key=lambda x: x["name"]):
        h.update(c["name"].encode())
        h.update(c["src"].encode())
    print("cells                %d" % len(cells))
    print("by class             %s" % json.dumps(kinds, sort_keys=True))
    print("in class             %d" % sum(1 for c in cells if c["in_class"]))
    print("cells with lit+move  %d" % sum(1 for c in cells
                                          if c["nlit"] and c["nmove"]))
    print("lit BELOW a move     %d" % below)
    print("R-PARKLIT separated  %d guarded cells" % guarded_sep)
    print("rival-pair sep       %s" % json.dumps(pair_counts, sort_keys=True))
    print("?mmioGetInfo cell    %s (separates %d of 10 pairs)"
          % (mmio_like[0]["name"], mmio_sep))
    print("sha256               %s" % h.hexdigest())


# ---------------------------------------------------------------------------
# Reading c2 back
# ---------------------------------------------------------------------------


def decode(w):
    op = w >> 26
    rs, ra, rb = (w >> 21) & 31, (w >> 16) & 31, (w >> 11) & 31
    if op == 31 and ((w >> 1) & 0x3FF) == 444 and rs == rb:
        return ("mr", ra, rs)
    if op == 14 and ra == 0:
        return ("li", rs, w & 0xFFFF)
    if op == 15 and ra == 0:
        return ("lis", rs, w & 0xFFFF)
    if op == 24:
        return ("ori", ra, rs, w & 0xFFFF)
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
    if op == 36:
        return ("stw",)
    if op == 62:
        return ("std",)
    if op == 31 and ((w >> 1) & 0x3FF) == 339:
        return ("mfspr",)
    return ("?", "%08x" % w)


def read_setup(words):
    """(entry, call) — the setup words in the ENTRY block (before the first
    compare/branch) and the run immediately preceding the first `bl`/`b`."""
    start = 0
    for i, w in enumerate(words):
        if decode(w)[0] in ("stwu",):
            start = i + 1
            break
    entry, i = [], start
    while i < len(words):
        d = decode(words[i])
        if d[0] in ("cmp", "bc", "b", "bl"):
            break
        if d[0] in ("mr", "li", "lis", "ori", "addi"):
            entry.append(d)
        i += 1
    # **The call run is the words before the first `bl`, and `bl` means LK=1.**
    #
    # This used to take the first `b` OR `bl` at or after the entry block, and
    # in a guarded cell that is the early-return arm's own `b` to the epilogue —
    # so the "call run" came back as the guard's `li r3,5`, the return VALUE,
    # and 445 of 633 in-class cells graded no (literal, move) pair at all while
    # R-DESC and R-LITLAST tied at 143 on the rest.  Recorded rather than
    # quietly fixed: a reader that mistakes a return value for a marshalling
    # word produces a tie between two rules that the grid separates at 390
    # cells, which is a green-looking result from a broken instrument.
    branch = None
    for j in range(len(words)):
        if decode(words[j])[0] == "bl":
            branch = j
            break
    call = []
    if branch is not None:
        j = branch - 1
        while j >= 0 and decode(words[j])[0] in ("mr", "li", "lis", "ori"):
            call.append(decode(words[j]))
            j -= 1
        call.reverse()
    return entry, call


def run(outdir, root):
    sys.path.insert(0, os.path.join(root, "scripts"))
    from gt_dump import Obj

    manifest = json.load(open(os.path.join(outdir, "manifest.json")))
    flags = os.path.join(root, "work/dc3-workload/flags.txt")
    c2rs = os.path.join(root, "target/release/c2rs")
    objdir = os.path.join(outdir, "obj")
    os.makedirs(objdir, exist_ok=True)
    rows = []
    for c in manifest:
        obj = os.path.join(objdir, c["name"] + ".obj")
        if not os.path.exists(obj):
            r = subprocess.run([c2rs, "compile", c["name"] + ".cpp",
                                "--keep-obj", obj,
                                "--flags-file", flags, "--cwd", outdir],
                               capture_output=True, text=True, cwd=outdir)
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
        entry, call = read_setup(words)
        rows.append(dict(name=c["name"], nbytes=len(words) * 4,
                         entry=entry, call=call,
                         words=["%08x" % w for w in words]))
    open(os.path.join(outdir, "measured.json"), "w").write(json.dumps(rows, indent=1))
    print("measured %d cells -> %s/measured.json" % (len(rows), outdir))


if __name__ == "__main__":
    if sys.argv[1] == "gen":
        gen(sys.argv[2])
    elif sys.argv[1] == "run":
        run(sys.argv[2], sys.argv[3])
