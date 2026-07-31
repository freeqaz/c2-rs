#!/usr/bin/env python3
"""gt_inline_decline.py — WHICH inlines does the front end refuse, and on what axis?

`docs/LABEL_COUNTER.md` §6 measures what an inlined site *costs* the label
counter (law L'), and its own closing paragraph names the gap this script
exists for:

    Law L' predicts the declined tree exactly, so the modelling job is real
    and tractable; what does not exist yet is any rule for PREDICTING THE
    DECLINE ITSELF.

That gap is load-bearing rather than academic. `crates/c2-il/src/func/bundle.rs`
refuses any TU where a callee is also defined, because c2 may inline it; the
first rung that relaxes that gate has to know WHICH expansion tree it is
counting labels for, and today nothing can tell it.

WHY A SECOND INSTRUMENT
-----------------------
`gt_label_inline.py` detects a decline with `INLINE-DECLINED?`: P's `.text`
grew by much less than the hand-written control's did. That is a *proxy*, it
is one-sided, and §6.14 records it being widened from `<` to `<=` after it
missed the cheapest possible decline. It also cannot say **which level** of a
two-level tree was refused, or **how many** of the N sites were.

This script reads the answer directly out of the obj instead. An inlined call
leaves no trace in P's relocation table; a call the front end declined leaves
exactly one `bl` against the callee's symbol. So

    * the reloc COUNT for a callee = the number of sites it was NOT inlined at,
      with per-site resolution, from ONE capture;
    * the reloc NAMES = which instance of a two-level tree survived, which is
      the depth evidence §6.4 says only `.text` growth carries — carried here
      by a symbol name rather than by an inequality on byte counts;
    * `bl gs` counts cross-check the whole tree: a fully inlined N-site sweep
      of a body containing c calls holds N*c of them.

Every `--family` row prints BOTH detectors' verdicts side by side and tags
`<== DETECTORS DISAGREE`, so a disagreement is a printed row and not a memory.

WHAT IT MEASURES
----------------
Ladders. Each ladder holds a base tree fixed and varies exactly ONE feature
(k arithmetic statements / k calls / k dead locals / k live locals / …), and
for each rung reports `Nfull` — the largest N at which every site was inlined
— separately at `/O1` and `/Ox`. A budget that is a threshold on some monotone
size shows up as `Nfull` falling monotonically along the ladder; where it does
not, that is the finding and the row prints it.

IS `s` THE AXIS, OR A PROXY FOR ONE?
------------------------------------
Every ladder here grows `s` by ADDING IL, so `s` and any count the FRONT END
could hold move together in every cell — and the front end chooses before
register allocation, so `s` is a c2-side number standing in for a c1xx-side
decision. `--pressure` separates them: a matched PERMUTATION pair, the same
statements/declarations/calls/operators in a different ORDER, so every
source-side count is equal and only the live-value count differs. The
measured `ds` is then 100% prologue+epilogue (the `body` column proves it),
i.e. pure allocator idiom, and the question is whether the decision follows
it. `--ends` does the same for the two round boundaries and locates the
SPILL FLOOR — the smallest callee that actually spills.

WHICH CLASS IS THE TABLE ABOUT?
------------------------------
Every ladder here declares its callee `static int in1(int)`, and that was
invisible for two rounds. `--linkage` removes the `static` and nothing else:
the same body, the same emitted bytes, is refused at N=1 from 68 bytes up with
no graduated middle, while `inline` is worth exactly 8 bytes in BOTH classes
and `__forceinline` ignores the whole schedule. SCHEDULE D is a claim about an
INTERNAL-LINKAGE, non-`inline` callee — see §6.17.

WHICH HALF OF THE PAIR IS THE TABLE ABOUT?
------------------------------------------
Every probe above varies the CALLEE. The decision is a property of a (caller,
callee) pair and until §6.19 the caller was, without one exception,
`int P(int a){ int s=gs(a)+a; ... return s; }` — external, one parameter, `int`,
and non-leaf. `--caller` varies it: leafness, linkage, `inline`, parameter
count, return type, member-ness, varargs, and the call SITE's own form. 192
discriminating cells, zero disagreements — no property of the caller measured
here enters the decision. `--helper` closes the other half of §6.18.10's open
list: `bl __savegprlr_N` is NOT "a call" for the 48-byte leaf term (21 cells
against a matched control that adds one real call, 19 the other way), and an
indirect call IS one — including in tail position, where it is `bctr` with LK
clear and the shipped predicate missed it.

Usage:
    scripts/gt_inline_decline.py [--mode '/O1 /GS- /c'] [--max N] [ladder ...]
    scripts/gt_inline_decline.py --list
    scripts/gt_inline_decline.py --family NAME ...   # gt_label_inline families
    scripts/gt_inline_decline.py --cases             # the categorical refusals
    scripts/gt_inline_decline.py --linkage [--kmin K] [--kmax K]
                                                     # static vs extern vs
                                                     # inline. The range was
                                                     # hardcoded k=0..8 and
                                                     # §6.17.8's /Ox negative
                                                     # was read off it; the
                                                     # SPLIT SUMMARY now says
                                                     # how many cells could
                                                     # have disagreed.
    scripts/gt_inline_decline.py --axes [--scout] [kind ...]  # the held-fixed
                                                     # variables: return type,
                                                     # extern "C", storage
                                                     # duration, virtual,
                                                     # template
    scripts/gt_inline_decline.py --lawd              # the clamped form, and
                                                     # the ceiling clamp's
                                                     # held-out cells
    scripts/gt_inline_decline.py --caller [--callee c-framed|c-leaf]
                                         [--kmin K] [--kmax K]
                                                     # vary the CALLER: the
                                                     # other half of the pair.
                                                     # SWEEP TO THE CEILING —
                                                     # §6.19's site term is
                                                     # entirely above the
                                                     # default range.
    scripts/gt_inline_decline.py --helper            # what counts as "a call"
                                                     # for the 48-byte term
    scripts/gt_inline_decline.py --thisctl           # is `this` an ordinary p0?
    scripts/gt_inline_decline.py --pressure [--pair SUBSTR]  # same IL, diff `s`
    scripts/gt_inline_decline.py --ends              # the two round boundaries
    scripts/gt_inline_decline.py --sibling           # per-PAIR, not per-caller
    scripts/gt_inline_decline.py --padp              # …nor per-caller-size
    scripts/gt_inline_decline.py --max N --kmax K

Env: C2RS_WIBO / C2RS_COMPILERS as for scripts/gt_capture.sh.
Exit status is 0 if every capture succeeded; it says nothing about whether a
prediction held — read the table.
"""

import os
import struct
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from gt_label_stride import capture, groups  # noqa: E402
from gt_label_inline import (  # noqa: E402
    FAMILIES, GS, INT_HEAD, INT_TAIL, src_of,
)

# --------------------------------------------------------------------------
# the readout
# --------------------------------------------------------------------------
def extent(o, g):
    """One function's OWN byte range inside its `.text` section.

    THIS IS NOT COSMETIC. `/O1` implies `/Gy`, so every function gets its own
    `.text` COMDAT and "the section" and "the function" are the same bytes.
    **`/Ox` does not imply `/Gy`**: at `/Ox` this compiler packs every function
    of the TU into ONE `.text`, so anything that reads `len(section)` at `/Ox`
    is reading P *plus the callees plus all three anchors*. A detector built on
    that number is polluted at N=0->1 by the callee's own first emission — the
    largest single term in the sweep — which is exactly where a decline has to
    be caught. See the header: this is why the older `dtext` detector reports
    `noinline`, the family whose entire purpose is to be declined, as clean at
    `/Ox` N=1.
    """
    sec = o.sections[g["sec"] - 1]
    starts = sorted({s["value"] for s in o.symbols
                     if s["sec"] == g["sec"] and s["type"] == 0x0020
                     and s["sc"] in (2, 3)})
    me = None
    for s in o.symbols:
        if s["name"] == g["name"] and s["sec"] == g["sec"]:
            me = s["value"]
            break
    if me is None:
        return sec, 0, sec["rawsize"]
    nxt = [v for v in starts if v > me]
    return sec, me, (nxt[0] if nxt else sec["rawsize"])


# A surviving call site is a `bl`, i.e. a REL24. Nothing else is.
REL24 = 0x0006


def read(o, pfx="P"):
    """P's OWN surviving CALL SITES and every emitted function's OWN size.

    `pfx` names the CALLER, and defaults to the `P` every probe in §6.15-§6.18
    uses, so every existing row is byte-for-byte unaffected. It exists because
    §6.19 varies the caller for the first time and a MEMBER caller mangles as
    `?P@CP@@...`, which `?P@@` does not match — a silent `no P` on the one row
    the probe is about would be the same fault as §6.15.0 and §6.18.0(B).

    TWO THINGS THIS HAS TO GET RIGHT, both of which it got wrong first (§6.18):

    * **REL24 only.** The reloc table is not a call list. A callee with a
      `static` local mangles that local as `?t@?1??c1@@YAHH@Z@4HA` — the
      CALLEE'S OWN NAME IS A SUBSTRING OF IT — so a fully inlined `c1` whose
      body loads and stores `t` leaves three REFHI/REFLO relocs in P that
      `declined()`'s substring match counted as three surviving calls. The
      row read `Ndir = 0` on a callee that was inlined at every site. This is
      the same fault as §6.17.13's `size_of` collision, on the detector that
      one was not applied to.
    * **`bctrl` is a call the reloc table cannot see.** A virtual call through
      a pointer is `mtctr`/`bctrl` with NO relocation against the callee at
      all, so the reloc detector reads "no `bl` survived" — i.e. INLINED — on
      the one call kind that cannot be inlined. `nind` counts them, and any
      grader that is looking at an indirect-call shape must read it.
    """
    gs_ = groups(o)

    def find(sfx):
        for g in gs_:
            if g["name"].startswith("?" + sfx + "@@") or g["name"] == sfx:
                return g
        return None

    P = find(pfx)
    if P is None:
        return {"error": "no P among %s" % [g["name"] for g in gs_]}
    sec, lo, hi = extent(o, P)
    rel = {}
    for va, symidx, ty in o.relocs(sec):
        if not (lo <= va < hi):
            continue                      # another function's call, at /Ox
        if ty != REL24:
            continue                      # a data reference, not a call site
        s = o.sym_by_index(symidx)
        if s is None:
            continue
        rel[s["name"]] = rel.get(s["name"], 0) + 1
    d = o.raw(sec)[lo:hi]
    nind = 0
    for i in range(0, len(d) - 3, 4):
        w = struct.unpack_from(">I", d, i)[0]
        # bcctrl (`bctrl` is bcctrl 20,0): op 19, XO 528, LK set
        if (w >> 26) == 19 and ((w >> 1) & 0x3FF) == 528 and (w & 1):
            nind += 1
    emitted = {}
    for g in gs_:
        if g is P:
            continue
        _s, glo, ghi = extent(o, g)
        emitted[g["name"]] = ghi - glo
    return {"rel": rel, "nind": nind, "tsize": hi - lo, "emit": emitted}


def name_matches(nm, want):
    d = demangle_ish(nm)
    return d == want or d.endswith(want) or want in nm


def pressure_of(o, want):
    """(saved nonvolatile GPRs, frame bytes, post-prologue r1 stores/loads).

    A pressure probe that cannot PROVE it created pressure is worthless: "no
    divergence" and "no pressure" look identical in the `s` column. This reads
    the allocator's own answer out of the callee's bytes —

      * `nsave`  = 32 - N from `bl __savegprlr_N`, plus any explicit
                   `stw rX,d(r1)` ahead of the frame push. 18 means every
                   nonvolatile GPR (r14..r31) is in use and the next live
                   value MUST go to the stack.
      * `frame`  = the `stwu r1,-F(r1)` displacement.
      * `spill`  = `stw rX,d(r1)` / `lwz rX,d(r1)` AFTER the frame push. For
                   these probes every callee is `int f(int)` and every call
                   takes one register argument, so nothing is an outgoing
                   argument store: these are spills and reloads.
    """
    for g in groups(o):
        if name_matches(g["name"], want):
            break
    else:
        return None
    sec, lo, hi = extent(o, g)
    d = o.raw(sec)[lo:hi]
    rels = {va: sy for va, sy, _t in o.relocs(sec) if lo <= va < hi}
    nsave, nfsave, frame, st, ld, pushed = 0, 0, 0, 0, 0, False
    pro, epi = 0, len(d)
    for i in range(0, len(d) - 3, 4):
        w = struct.unpack_from(">I", d, i)[0]
        op, ra = w >> 26, (w >> 16) & 31
        # stwu r1,-F(r1)
        if op == 37 and ((w >> 21) & 31) == 1 and ra == 1:
            frame = 0x10000 - (w & 0xFFFF) if (w & 0x8000) else -(w & 0xFFFF)
            pushed, pro = True, i + 4
            continue
        # `addi r1,r1,F` pops the frame and opens the epilogue
        if pushed and op == 14 and ((w >> 21) & 31) == 1 and ra == 1 \
                and (w & 0xFFFF) == frame:
            epi = min(epi, i)
        if not pushed:
            if op == 36 and ra == 1:                            # stw rX,d(r1)
                nsave += 1
            elif op == 54 and ra == 1:                          # stfd fX,d(r1)
                nfsave += 1
            s = rels.get(lo + i)
            if s is not None:
                nm = o.sym_by_index(s)
                nm = nm["name"] if nm else ""
                if nm.startswith("__savegprlr_"):
                    nsave += 32 - int(nm.rsplit("_", 1)[1])
                elif nm.startswith("__savefpr_"):
                    nfsave += 32 - int(nm.rsplit("_", 1)[1])
        else:
            if op == 36 and ra == 1:
                st += 1
            elif op == 32 and ra == 1:
                ld += 1
    # The callee's own COFF storage class: 2 = EXTERNAL, 3 = STATIC. This is
    # NOT bookkeeping — §6.17 measures that the /O1 schedule is a claim about
    # INTERNAL-linkage callees only, and every grader here has to know which
    # class it is looking at before it grades anything.
    sc = None
    for sy in o.symbols:
        if sy["name"] == g["name"] and sy["sec"] == g["sec"]:
            sc = sy["sc"]
            break
    return {"nsave": nsave, "nfsave": nfsave, "frame": frame, "st": st,
            "ld": ld, "sc": sc,
            "pro": pro, "epi": len(d) - epi, "body": max(0, epi - pro)}


# --------------------------------------------------------------------------
# SCHEDULE D — the /O1 decline, MEASURED. Not a formula, and that is the point
# --------------------------------------------------------------------------
# At `/O1` the front end's decision is ALL-OR-NOTHING per (caller, callee)
# pair — never a mixture of inlined and declined sites, over 14 ladders, ~200
# rungs, and up to 24 sites — and the number of sites it will take is a
# function of ONE variable: `s`, the callee's OWN emitted `.text` size. §6.5
# guarantees that number is in every obj for free, because c2 emits the
# callee's COMDAT whether or not it was inlined.
#
# The boundaries below are pinned to a single 4-byte step by a ladder whose
# rungs are ONE INSTRUCTION each, and reproduced with ZERO disagreements by
# thirteen other ladders that move `s` by five independent mechanisms
# (1-instruction rungs, 1-statement integer arithmetic, calls, `if`s, and
# `double` arithmetic with an FPR frame and `_fltused`), at depth 1 and depth
# 2, with and without a loop.
#
# SCOPE, added in §6.17 and NOT a detail: this table is a claim about an
# INTERNAL-LINKAGE (`static`), non-`inline` callee. Remove the `static` and the
# same body — the same emitted bytes — is refused at N=1 from 68 bytes up, with
# no graduated middle at all. Mark it `inline` and the whole table shifts 8
# bytes (index = s-8). Mark it `__forceinline` and there is no table.
#
# THERE IS NO CLOSED FORM, and every rival that looked like one is in
# SUPERSEDED_D, re-refuted from each run's own numbers.
#
# What is refuted, stated at the width the arithmetic actually supports
# (§6.17.10 corrects an earlier overstatement here): for a per-site cost model
# `N_max = floor(B / f(s))`, each measured row pins `f(s)` to a half-open
# interval, `f(s) in (B/(n+1), B/n]`. The rows n(68)=n(72)=9 and n(76)=7 then
# force
#       f(72)/f(68) < 10/9 = 1.111   AND   f(76)/f(72) > 9/8 = 1.125
# — the relative growth per 4-byte step must INCREASE — so every `f` whose
# relative growth is non-increasing is refuted: all affine costs, all powers
# c*(s-h)^p, and every exponential. That subsumes LAW D and its kin.
# It does NOT refute "some product model", which is what this comment used to
# claim: `f(s) = B/N(s)` reproduces the table exactly and is non-decreasing, so
# a product model EXISTS and the honest negative is the log-concavity one.
#
# `/O1` ONLY. `/Ox` is a DIFFERENT MECHANISM — N-independent, all-or-nothing
# at every N up to 12 — not a different constant. Nothing here is inferred
# from one mode to the other.
#
#   (max s, Nfull)     s/4 = instructions
LAW_D_TABLE = [
    (64, None),        # <= 16 instr — inlined at ANY number of sites (>=24)
    (72, 9),           # 17-18
    (76, 7),           # 19
    (80, 5),           # 20
    (88, 4),           # 21-22
    (100, 3),          # 23-25
    (140, 2),          # 26-35
    (256, 1),          # 36-64
]                      # >= 260 bytes / 65 instr -> NEVER inlined, even once


def law_d(s):
    """Measured `Nfull` for a callee of `s` bytes; None means unbounded."""
    if s is None:
        return None
    for hi, n in LAW_D_TABLE:
        if s <= hi:
            return n
    return 0


# The readings this round retired, re-refuted from each run's own measurement
# rather than remembered — same discipline as gt_label_inline's SUPERSEDED.
SUPERSEDED_D = [
    # Exact on every cell at N <= 6, where the sweep's own cap hid the
    # counter-examples; dead at s=68 (says >=12, measures 9) and s=72 (says
    # 10, measures 9). Pre-registered in work/gt-inline-decline/ESTIMATE-
    # round28d.txt and killed by the first out-of-sample capture.
    # STILL REFUTED, and §6.18.9 says exactly where: those two cells and the
    # s>=260 ceiling are its ENTIRE failure, and clamping all three (`law_dc`)
    # reproduces the measured table cell for cell. It stays in this list
    # because a form that needs three hand-placed clamps, two of them fitted
    # on the very cells that killed it, is not the law — it is a description
    # with the failures moved into the constants.
    ("(N-1)*(s-64) < 80",
     lambda s: None if s is None or s <= 64 else 1 + 79 // (s - 64)),
    # The reading I would have written down first, killed at N <= 6.
    ("N*(s-64) < 80  ('the first copy is not free')",
     lambda s: None if s is None or s <= 64 else 79 // (s - 64)),
]


# --------------------------------------------------------------------------
# LAW Dc — SCHEDULE D's interior IS the retired LAW D; its two ends are clamps
#
# §6.15.7 asked for "a mechanism that produces exactly that" sequence —
# 9, 7, 5, 4, 3, 2, 1, skipping 8 and 6 — and §6.17.10 bounded the search:
# for a per-site cost model every affine, every power and every exponential is
# refuted at once, because n(68)=n(72)=9 and n(76)=7 force the relative growth
# per 4-byte step to INCREASE.
#
# Both statements survive. What is new (§6.18.9) is where the refutation is
# LOCATED. Retired LAW D is a net-duplication budget, and in INSTRUCTIONS
# (i = s/4) rather than bytes its constant comes out exact:
#
#       (N - 1) * (i - 16)  <=  19
#
# "sixteen instructions are free, and you may duplicate at most nineteen more
# beyond the first copy". That is EXACT on every band from i = 19 to i = 64 —
# six of the schedule's eight — and it GENERATES the two values the sequence
# skips: `1 + 19//d` can only ever take 1, 2, 3, 4, 5, 7, 10, 20, so **8 and 6
# are arithmetically unreachable**. §6.15.7 asked for "a mechanism that
# produces exactly that" sequence and this is it.
#
# It also generates the schedule's FIRST row rather than clamping it: at
# i <= 16 the left side is <= 0, so every N satisfies it — `<=64 B ->
# unbounded` falls out. The whole of its failure is two cells at the bottom
# (i = 17, 18, where it allows 20 and 10 against a measured 9) and the top
# (i >= 65, where it allows 1 against a measured 0). Clamp those two and the
# form reproduces EVERY 4-byte cell of LAW_D_TABLE:
#
#       N_max(i) = 0                              i >= 65
#                  min(9, 1 + floor(19/(i-16)))   otherwise (unbounded at i<=16)
#
# And the two constants are FORCED, not chosen: a search over budget 1..59 x
# cap 1..39 finds exactly one pair, (19, 9), that reproduces the table.
# `law_dc_selfcheck()` returns that search, so it is a computation in the run's
# own output and not a claim in a comment.
#
# HOW MUCH OF THIS IS FITTED, stated plainly because the answer is "most of
# the new part": the cap 9 is fitted on EXACTLY THE TWO CELLS THAT KILLED
# LAW D, which is the move this lane forbids. Its only hold-out is thin and
# is named as such: fit the cap on s=68 alone and s=72 is then held out, where
# the uncapped form says 10 and the capped one says 9 — measured 9, by six
# ladders. The `inline` class (§6.17.5) shifts the index by 8 and so lands a
# SECOND, differently-spelled source on the same two indices; that tests that
# the cap is a function of the INDEX rather than of `s`, not the value 9.
# The >= 260 clamp does have a real hold-out and `--lawd` takes it: a
# `static inline` callee of 268 bytes has index 260 and no version of this
# table has ever measured that cell.
#
# This is NOT a resurrection of LAW D. LAW D as written is refuted and stays
# refuted — §6.17.10's log-convexity negative kills its affine cost, and it
# kills the `1 + floor(B/f)` family too (the rows pin f(68), f(72) to
# (B/9, B/8] and f(76) to (B/7, B/6], so the growth must again increase). The
# clamp is not a cost, which is exactly why it can absorb a refutation that no
# cost function can.
LAW_DC_BUDGET, LAW_DC_CAP, LAW_DC_CEIL = 19, 9, 65     # instructions / sites


def law_dc(idx):
    """The clamped form, in INSTRUCTIONS, where its constant is exact.

        (N - 1) * (i - 16)  <=  19        i = idx/4, the callee's instructions

    `None` means unbounded, as `law_d` does. Note the `i <= 16 -> unbounded`
    row is NOT a clamp: `i - 16 <= 0` satisfies the inequality for every N, so
    the inequality GENERATES the schedule's first row. Only two clamps are
    hand-placed, and both are named in the header.
    """
    if idx is None:
        return None
    i = idx // 4
    if i >= LAW_DC_CEIL:
        return 0
    if i <= 16:
        return None
    return min(LAW_DC_CAP, 1 + LAW_DC_BUDGET // (i - 16))


def law_dc_selfcheck(lo=4, hi=400):
    """Cells the clamped form misses, and whether its constants are FORCED.

    Printed by `--lawd` rather than asserted, so a future edit to LAW_D_TABLE
    shows up as a printed disagreement instead of silently making the form
    wrong — and the second return value is the honest one: the search over
    (budget, cap) says whether the measured table pins them or merely admits
    them.
    """
    miss = [(s, law_d(s), law_dc(s)) for s in range(lo, hi, 4)
            if law_d(s) != law_dc(s)]
    fits = []
    for c in range(1, 60):
        for cap in range(1, 40):
            for s in range(lo, hi, 4):
                i = s // 4
                p = 0 if i >= LAW_DC_CEIL else \
                    (None if i <= 16 else min(cap, 1 + c // (i - 16)))
                if p != law_d(s):
                    break
            else:
                fits.append((c, cap))
    return miss, fits


def demangle_ish(name):
    """`?lsa@@YAHH@Z` -> `lsa`. Good enough to match a probe's own callees."""
    if name.startswith("?") and "@@" in name:
        return name[1:name.index("@@")]
    return name


def declined(rel, watch):
    """{callee -> how many sites kept a `bl`}, over the names we planted."""
    out = {}
    for nm, n in rel.items():
        d = demangle_ish(nm)
        # a ctor mangles as ?0CL@@... — its class name is what we planted
        for w in watch:
            if d == w or d.endswith(w) or w in nm:
                out[w] = out.get(w, 0) + n
    return out


# --------------------------------------------------------------------------
# ladders — one base tree, one varied feature
# --------------------------------------------------------------------------
def stmts_arith(k, var="v"):
    """k statements, 0 new locals, 0 calls.

    `v = v*3+K;` was the first spelling and it is USELESS: a chain of affine
    ops constant-folds to one multiply-add, so the callee's `.text` did not
    move by a single byte across k = 0..8 and neither did anything else. Each
    rung is a distinct shift-xor instead, which is not affine and cannot fold
    into its neighbour. Verified by `in.text` growing monotonically in k.
    """
    return " ".join("%s^=%s>>%d;" % (var, var, i + 2) for i in range(k))


def stmts_call(k, var="v"):
    """k statements, 0 new locals, k calls."""
    return " ".join("%s+=gs(%s+%d);" % (var, var, i + 1) for i in range(k))


def stmts_deadloc(k):
    """k DECLARED LOCALS THAT GENERATE NO CODE — `loc1-dead`'s spelling.

    Law L' charges one of these a full `E` unit (`loc1-dead` = 4 against
    `loc0` = 3). If the decline axis is the same axis as the charge, this
    ladder moves the threshold as fast as any other; if it is code, it does
    not move it at all. That is the sharpest single question here.
    """
    return " ".join("int dd%d=%d;" % (i, i + 5) for i in range(k))


def stmts_liveloc(k, var="v"):
    """k statements AND k locals, each used — `stmts_arith` plus a name."""
    parts = []
    src = var
    for i in range(k):
        parts.append("int xx%d=%s^(%s>>%d);" % (i, src, src, i + 2))
        src = "xx%d" % i
    if k:
        parts.append("%s=%s;" % (var, src))
    return " ".join(parts)


def stmts_fine(k, var="v"):
    """k statements of ONE instruction each — 4-byte resolution on `s`.

    `^= a` and `+= a` alternate so no two adjacent rungs can be folded or
    reassociated into one another, which is what killed the first spelling
    of `stmts_arith`.
    """
    return " ".join("%s%s=a;" % (var, "^" if i % 2 == 0 else "+")
                    for i in range(k))


def stmts_dbl(k, var="v"):
    """k statements of DOUBLE arithmetic — same instruction count, totally
    different opcodes, an FPR prologue and `_fltused`. `x*x` is not affine
    so the chain cannot fold."""
    if not k:
        return ""
    parts = ["double xd=(double)a;"]
    parts += ["xd=xd*xd+(double)a;" for _ in range(k)]
    parts.append("%s+=(int)xd;" % var)
    return " ".join(parts)


def stmts_if(k, var="v"):
    """k `if` statements — branches and basic blocks, which law L' charges an
    `E` unit each."""
    return " ".join("if(a>%d) %s+=%d;" % (i + 1, var, i + 3) for i in range(k))


# --- THE PRESSURE PAIR: same IL, different allocation ----------------------
# Every other ladder in this file grows `s` by ADDING IL, so `s` and any
# c1xx-side count of the source move together and the whole 449-rung dataset
# cannot say which of the two the front end is actually reading. These two
# generators emit the SAME 2k statements, the SAME k declarations, the SAME k
# calls and the SAME operators at every k, PERMUTED — so every statement-,
# node- or operator-count metric is equal — and differ only in how many values
# are simultaneously live, which is a register-allocator question and
# therefore a c2-side one. §6.15.7 names this as the one probe that was
# missing and the one failure mode that would be invisible.
def stmts_perm_lo(k, var="v"):
    """FIRST SPELLING, kept because it is INERT and the inertness is a finding.

    k call-defined temps each combined into `v` immediately with `(v<<1)^t`.
    That chain is LINEAR over xor — `((v<<1)^t0)<<1)^t1` is exactly
    `v<<2 ^ t0<<1 ^ t1` — so the compiler is free to re-associate it, and it
    does: this and `stmts_perm_hi` compile to BYTE-IDENTICAL code at every k,
    and to the HIGH-pressure schedule (nsave = k+1) in both. A permutation
    probe built on a re-associable operator measures nothing.
    """
    parts = []
    for i in range(k):
        parts.append("int t%d=gs(a+%d);" % (i, i + 1))
        parts.append("%s=(%s<<1)^t%d;" % (var, var, i))
    return " ".join(parts)


def stmts_perm_hi(k, var="v"):
    """…the same 2k statements permuted. Byte-identical to `perm_lo`: INERT."""
    defs = ["int t%d=gs(a+%d);" % (i, i + 1) for i in range(k)]
    uses = ["%s=(%s<<1)^t%d;" % (var, var, i) for i in range(k - 1, -1, -1)]
    return " ".join(defs + uses)


def stmts_live_lo(k, var="v"):
    """LOW pressure: k call-defined temps, each CONSUMED IMMEDIATELY.

    The use is `v=gs(v^t)` — an OPAQUE EXTERN CALL. Nothing algebraic connects
    a use to its def, so no re-association can move it (which is what made
    `stmts_perm_*` inert), and calls to an extern cannot be reordered among
    themselves, so the def sequence is pinned too. Each `t` dies at the very
    next statement: measured nsave = 1 at every k.
    """
    parts = []
    for i in range(k):
        parts.append("int t%d=gs(a+%d);" % (i, i + 1))
        parts.append("%s=gs(%s^t%d);" % (var, var, i))
    return " ".join(parts)


def stmts_live_hi(k, var="v"):
    """HIGH pressure: THE SAME 2k statements, PERMUTED.

    All k defs, then all k uses in REVERSE def order. Identical declaration,
    statement, call and operator counts to `stmts_live_lo` at every k — the
    two source texts are permutations of one multiset — but all k temps are
    simultaneously live across k calls: measured nsave = k+1.
    """
    defs = ["int t%d=gs(a+%d);" % (i, i + 1) for i in range(k)]
    uses = ["%s=gs(%s^t%d);" % (var, var, i) for i in range(k - 1, -1, -1)]
    return " ".join(defs + uses)


def stmts_fp_lo(k, var="v"):
    """LOW pressure, DOUBLE temps — the pressure pair in a different frame class.

    §6.16.11 named "every callee in this section is `int f(int)`" as the top
    remaining risk after round 29's GPR result. These two generators move the
    pair into an FPR frame: `double` temps defined by `gd` calls, `_fltused`,
    a `__savefpr_` set alongside the GPR one, and an entirely different opcode
    mix — with the statement, declaration, call and operator counts still
    equal between the two spellings at every k.
    """
    parts = []
    for i in range(k):
        parts.append("double d%d=gd((double)a+%d);" % (i, i + 1))
        parts.append("%s=(int)gd((double)%s+d%d);" % (var, var, i))
    return " ".join(parts)


def stmts_fp_hi(k, var="v"):
    """…the same 2k statements permuted: all k `double`s live at once."""
    defs = ["double d%d=gd((double)a+%d);" % (i, i + 1) for i in range(k)]
    uses = ["%s=(int)gd((double)%s+d%d);" % (var, var, i)
            for i in range(k - 1, -1, -1)]
    return " ".join(defs + uses)


def stmts_clive_lo(k, var="v"):
    """LOW pressure with CONSTANT-argument defs — the member pair's control.

    Identical in shape to `stmts_live_lo` except that each temp is defined from
    a CONSTANT (`gs(0x11*i)`) instead of from the parameter. That kills `a` at
    the callee's first statement, which is the whole point: `stmts_live_lo`
    holds {a, v} live in its LOW spelling, and a MEMBER function of that body
    holds {this, a, v} = THREE, which already takes the `__savegprlr_` helper —
    so the high-pressure spelling has no threshold left to cross and the pair
    measures ds = 0 (measured at k=1,2,3 before this generator was written).
    With `a` dead, member LOW holds {this, v} = 2 and the 24-byte idiom delta
    is back.
    """
    parts = []
    for i in range(k):
        parts.append("int t%d=gs(%d);" % (i, 0x11 * (i + 1)))
        parts.append("%s=gs(%s^t%d);" % (var, var, i))
    return " ".join(parts)


def stmts_clive_hi(k, var="v"):
    """…the same 2k statements permuted: all k temps live at once."""
    defs = ["int t%d=gs(%d);" % (i, 0x11 * (i + 1)) for i in range(k)]
    uses = ["%s=gs(%s^t%d);" % (var, var, i) for i in range(k - 1, -1, -1)]
    return " ".join(defs + uses)


def stmts_live_hi_cheap(k, var="v"):
    """Maximum live values per INSTRUCTION — built to look for a cheap spill.

    Each temp is one instruction (`a^K`) instead of a call, so k live values
    cost ~2k instructions rather than ~4k, and the rungs are 8 bytes each,
    which makes this a SEVENTH mechanism reproducing the schedule at fine
    resolution (68->9, 76->7, 84->4, 92->3, 100->3, 108->2, 144->1, 256->1,
    264->0).

    It did NOT find a cheap spill, and could not have: `a^K` is trivially
    REMATERIALISABLE, so c2 recomputes rather than spilling. `--ends` measures
    the real spill floor on call-defined values instead, and finds it at 19
    live values / 324 bytes — past the ceiling either way.
    """
    defs = ["int t%d=a^%d;" % (i, 0x11 * (i + 1)) for i in range(k)]
    uses = ["%s=(%s<<1)^t%d;" % (var, var, i) for i in range(k - 1, -1, -1)]
    return " ".join(defs + uses)


LOOP = "for(int i=0;i<a;i++) v+=gs(i);"
LOOP_LEAF = "for(int i=0;i<a;i++) v+=i*3;"


def tree_d2(inner_extra, loop=LOOP):
    """The `d2-loop-asctor` tree — §6.14's row that SURVIVES at /Ox to N=6.

    static int in2(int a){ int v=0; <loop> <extra> return v; }
    static int ou2(int a){ int c=in2(a); return c; }
    """
    leads = ("static int in2(int a){ int v=0; %s %s return v; }\n"
             "static int ou2(int a){ int c=in2(a); return c; }"
             % (loop, inner_extra))
    return leads, "s=ou2(s);", ["ou2", "in2"]


def tree_d1(inner_extra, loop=LOOP):
    """The same body at depth 1 — no wrapper."""
    leads = ("static int in1(int a){ int v=0; %s %s return v; }"
             % (loop, inner_extra))
    return leads, "s=in1(s);", ["in1"]


def tree_d1_noloop(inner_extra):
    leads = ("static int in1(int a){ int v=gs(a)+a; %s return v; }"
             % inner_extra)
    return leads, "s=in1(s);", ["in1"]


def tree_d1_fp(inner_extra):
    """depth 1, no loop, and the TU also declares the `double` barrier `gd`.

    `gd` is declared in the LEAD rather than in `GS` so that nothing about the
    integer ladders moves; `_fltused` is introduced by the callee itself,
    which is what an FP frame class means here.
    """
    leads = ("double gd(double);\n"
             "static int in1(int a){ int v=gs(a)+a; %s return v; }"
             % inner_extra)
    return leads, "s=in1(s);", ["in1"]


def tree_d1_member(inner_extra):
    """depth 1, no loop, and the callee is a NON-STATIC MEMBER FUNCTION.

    §6.16.11 named this as the cheapest untried candidate for the one thing the
    axis result still rested on: a LARGE allocator delta at a SMALL callee, in a
    class the schedule was not fitted to. `this` is live from ENTRY — the
    allocator sees a value the source never names — and `return v^m` reads
    `this->m` at the very END, after every call, so the pointer cannot die
    early. `gs` is opaque, so the load cannot be hoisted above the calls either.

    The object comes from an `extern MB*`, not a global instance: a global's
    address is a rematerialisable constant and would let the allocator drop the
    pointer instead of keeping it live.

    NOTE the body READS `this->m` and never STORES to it. §6.15.5 has member
    functions with member STORES categorically refused at /O1
    (`member-noloop-store`, `method-2store-call`), and a categorical refusal
    would make this probe uninformative rather than wrong — so the store is
    avoided deliberately and the N=1 column is what tells the two apart.
    """
    leads = ("struct MB { int m; int mf1(int a){ int v=gs(a)+a; %s"
             " return v^m; } };\n"
             "extern MB* mbp;" % inner_extra)
    return leads, "s=mbp->mf1(s);", ["mf1"]


def tree_d1_thisparam(inner_extra):
    """…the identical body as a FREE function taking `MB*` as parameter 1.

    The control for `tree_d1_member`: if `this` is an ordinary first parameter
    to the back end, these two compile to the same bytes. The port numbers
    `this` LAST among the parameters (measured; reverting it costs 272
    mismatches), so "does the allocator agree" is a real question and this is
    the cheapest place it can be asked.
    """
    leads = ("struct MB { int m; };\n"
             "static int pf1(MB *p,int a){ int v=gs(a)+a; %s"
             " return v^p->m; }\n"
             "extern MB* mbp;" % inner_extra)
    return leads, "s=pf1(mbp,s);", ["pf1"]


def tree_d2_noloop(inner_extra):
    leads = ("static int in2(int a){ int v=gs(a)+a; %s return v; }\n"
             "static int ou2(int a){ int c=in2(a); return c; }"
             % inner_extra)
    return leads, "s=ou2(s);", ["ou2", "in2"]


def tree_ctor(inner_extra, loop=LOOP):
    """§6.14's DC3 shape: the loop lives in a CONSTRUCTOR."""
    leads = ("struct CL { int v; CL(int a){ v=0; %s %s } };\n"
             "static int lcl(int a){ CL c(a); return c.v; }"
             % (loop, inner_extra))
    return leads, "s=lcl(s);", ["lcl", "CL"]


LADDERS = {}


def ladder(name, tree, gen, kmax, note):
    LADDERS[name] = (tree, gen, kmax, note)


# --- the four feature ladders, on the tree that SURVIVES at /Ox -----------
ladder("d2-arith", tree_d2, stmts_arith, 8,
       "d2 loop tree + k arithmetic statements (0 locals, 0 calls)")
ladder("d2-call", tree_d2, stmts_call, 6,
       "d2 loop tree + k CALLS")
ladder("d2-deadloc", tree_d2, stmts_deadloc, 8,
       "d2 loop tree + k DEAD locals (a full E unit each, ~no code)")
ladder("d2-liveloc", tree_d2, stmts_liveloc, 8,
       "d2 loop tree + k live locals (= d2-arith + k names)")
# --- the same four one level up, to separate depth from size --------------
ladder("d1-arith", tree_d1, stmts_arith, 10,
       "depth-1 loop callee + k arithmetic statements")
ladder("d1-call", tree_d1, stmts_call, 8,
       "depth-1 loop callee + k CALLS")
ladder("d1-deadloc", tree_d1, stmts_deadloc, 10,
       "depth-1 loop callee + k DEAD locals")
# --- and with no loop at all, to ask whether the loop is special ----------
ladder("d2-noloop-arith", tree_d2_noloop, stmts_arith, 14,
       "d2 tree, NO loop, + k arithmetic statements")
ladder("d2-noloop-call", tree_d2_noloop, stmts_call, 10,
       "d2 tree, NO loop, + k CALLS")
ladder("d1-noloop-arith", tree_d1_noloop, stmts_arith, 16,
       "depth-1, NO loop, + k arithmetic statements")
ladder("d1-noloop-call", tree_d1_noloop, stmts_call, 12,
       "depth-1, NO loop, + k CALLS")
# --- the DC3 shape itself -------------------------------------------------
# --- HELD OUT: move `s` by a DIFFERENT mechanism ---------------------------
# The eleven ladders above all grow `s` by appending statements to one body,
# so "Nfull is a function of s" could be "Nfull is a function of whatever my
# rungs move". These three move it another way.
ladder("d1-fine", tree_d1_noloop, stmts_fine, 64,
       "HELD OUT: 1-INSTRUCTION rungs — 4-byte resolution on the boundaries")
ladder("d1-dbl", tree_d1_noloop, stmts_dbl, 14,
       "HELD OUT: DOUBLE arithmetic — different opcodes, FPR frame, _fltused")
ladder("d1-if", tree_d1_noloop, stmts_if, 14,
       "HELD OUT: k `if` statements — branches and basic blocks")
ladder("ctor-arith", tree_ctor, stmts_arith, 4,
       "§6.14's ctor+loop+call shape + k arithmetic statements")
ladder("ctor-leaf-arith", lambda e: tree_ctor(e, LOOP_LEAF), stmts_arith, 6,
       "the ctor whose loop makes NO call + k arithmetic statements")
ladder("d2-leaf-arith", lambda e: tree_d2(e, LOOP_LEAF), stmts_arith, 8,
       "d2 tree whose loop makes NO call + k arithmetic statements")
# --- THE PRESSURE PAIR: the one probe §6.15.7 says was missing --------------
ladder("d1-live-lo", tree_d1_noloop, stmts_live_lo, 10,
       "PRESSURE control: k call-defined temps, each used IMMEDIATELY")
ladder("d1-live-hi", tree_d1_noloop, stmts_live_hi, 10,
       "PRESSURE probe: the SAME 2k statements permuted — all k live at once")
ladder("d2-live-lo", tree_d2_noloop, stmts_live_lo, 8,
       "…the low-pressure spelling one level down")
ladder("d2-live-hi", tree_d2_noloop, stmts_live_hi, 8,
       "…the high-pressure spelling one level down")
ladder("d1-perm-lo", tree_d1_noloop, stmts_perm_lo, 8,
       "INERT control: a RE-ASSOCIABLE use — c2 collapses the permutation")
ladder("d1-perm-hi", tree_d1_noloop, stmts_perm_hi, 8,
       "INERT control: byte-identical to d1-perm-lo at every k")
ladder("d1-fp-lo", tree_d1_fp, stmts_fp_lo, 8,
       "PRESSURE control in an FPR frame class: k double temps, used at once")
ladder("d1-fp-hi", tree_d1_fp, stmts_fp_hi, 8,
       "PRESSURE probe in an FPR frame class: the same 2k statements permuted")
ladder("d1-cheap-hi", tree_d1_noloop, stmts_live_hi_cheap, 30,
       "7th mechanism: 1-instruction temps, 8-byte rungs, all live at once")
# --- THE MEMBER-FUNCTION CLASS: `this` live from entry ---------------------
ladder("d1-mem-lo", tree_d1_member, stmts_clive_lo, 10,
       "MEMBER fn, `this` live from entry: LOW spelling, {this,v} live")
ladder("d1-mem-hi", tree_d1_member, stmts_clive_hi, 10,
       "MEMBER fn: the SAME 2k statements permuted — {this,v,t0..} live")
ladder("d1-clive-lo", tree_d1_noloop, stmts_clive_lo, 10,
       "FREE control for the member pair: same body, no `this`")
ladder("d1-clive-hi", tree_d1_noloop, stmts_clive_hi, 10,
       "…and its high-pressure spelling — crosses the idiom one k LATER")
ladder("d1-this-lo", tree_d1_thisparam, stmts_clive_lo, 10,
       "the member body as a FREE fn taking MB* — is `this` an ordinary p0?")
ladder("d1-this-hi", tree_d1_thisparam, stmts_clive_hi, 10,
       "…its high-pressure spelling")


# --------------------------------------------------------------------------
# CASES — one-off trees for the CATEGORICAL refusal, which is not a budget.
#
# §6.14 measured `ctor-loop` (a `for` with a call, inside a constructor)
# declined FROM THE FIRST SITE in both modes, while `ctor-loop-leaf` (same
# ctor, loop makes no call) and `d2-loop-asctor` (same tree, plain function
# where the ctor was) both inline to N=6, and concluded "it is the
# conjunction" of ctor and call-in-loop.
#
# That conclusion has a confound it never tested. The constructor stores to
# `this->v` INSIDE the loop; `d2-loop-asctor` accumulates into a LOCAL. So
# "constructor" and "a store through a pointer inside a loop" are perfectly
# aliased in §6.14's design, and the second is the one an optimiser would
# actually care about — it is the aliasing question, not a C++ one. These
# cases break the alias in both directions: a ctor that accumulates into a
# local, and a plain free function that stores through a pointer in a loop.
# --------------------------------------------------------------------------
CASES = []


def case(name, leads, site, watch, note):
    CASES.append((name, leads, site, watch, note))


case("ctor-loop-call",
     "struct C1 { int v; C1(int a){ v=0;"
     " for(int i=0;i<a;i++) v+=gs(i); } };\n"
     "static int f1(int a){ C1 c(a); return c.v; }",
     "s=f1(s);", ["f1", "C1"],
     "BASELINE = §6.14's ctor-loop: ctor, loop, call, store to this->v")
case("ctor-loop-leaf",
     "struct C2 { int v; C2(int a){ v=0;"
     " for(int i=0;i<a;i++) v+=i*3; } };\n"
     "static int f2(int a){ C2 c(a); return c.v; }",
     "s=f2(s);", ["f2", "C2"],
     "…the same ctor, loop makes NO call (§6.14 control: inlines)")
case("ctor-loop-local",
     "struct C3 { int v; C3(int a){ int t=0;"
     " for(int i=0;i<a;i++) t+=gs(i); v=t; } };\n"
     "static int f3(int a){ C3 c(a); return c.v; }",
     "s=f3(s);", ["f3", "C3"],
     "THE PROBE: ctor + loop + call, accumulating into a LOCAL and storing"
     " to the member ONCE. Breaks the ctor / store-in-loop alias one way")
case("method-loop-call",
     "struct C4 { int v; void fill(int a){ v=0;"
     " for(int i=0;i<a;i++) v+=gs(i); } };\n"
     "static int f4(int a){ C4 c; c.fill(a); return c.v; }",
     "s=f4(s);", ["f4", "fill"],
     "THE PROBE: the identical body as a MEMBER FUNCTION, not a ctor."
     " Breaks the alias the other way")
case("method-loop-local",
     "struct C5 { int v; void fill(int a){ int t=0;"
     " for(int i=0;i<a;i++) t+=gs(i); v=t; } };\n"
     "static int f5(int a){ C5 c; c.fill(a); return c.v; }",
     "s=f5(s);", ["f5", "fill"],
     "…member function accumulating into a local — the 2x2's fourth cell")
case("ptr-loop-call",
     "static void f6a(int *p,int a){ *p=0;"
     " for(int i=0;i<a;i++) *p+=gs(i); }\n"
     "static int f6(int a){ int r; f6a(&r,a); return r; }",
     "s=f6(s);", ["f6", "f6a"],
     "NO CLASS AT ALL: a free function storing THROUGH A POINTER inside a"
     " loop whose body calls. If this is refused, C++ is irrelevant")
case("ptr-loop-local",
     "static void f7a(int *p,int a){ int t=0;"
     " for(int i=0;i<a;i++) t+=gs(i); *p=t; }\n"
     "static int f7(int a){ int r; f7a(&r,a); return r; }",
     "s=f7(s);", ["f7", "f7a"],
     "…the same free function accumulating into a local — the control")
case("ctor-call-noloop",
     "struct C8 { int v; C8(int a){ v=gs(a)+a; } };\n"
     "static int f8(int a){ C8 c(a); return c.v; }",
     "s=f8(s);", ["f8", "C8"],
     "a ctor with a call and NO loop (§6.9 measured this inlining)")
case("ctor-loop-nocall-store",
     "struct C9 { int v; C9(int a){ v=0;"
     " for(int i=0;i<a;i++) v+=i*3; } };\n"
     "static int f9(int a){ C9 c(a); return c.v; }",
     "s=f9(s);", ["f9", "C9"],
     "store to this->v in a loop, loop makes NO call (= ctor-loop-leaf)")
case("ctor-loop-while",
     "struct CA { int v; CA(int a){ v=0; int i=a;"
     " while(i>0){ v+=gs(i); i--; } } };\n"
     "static int fa(int a){ CA c(a); return c.v; }",
     "s=fa(s);", ["fa", "CA"],
     "the refused conjunction with a `while` instead of a `for`")
case("glob-loop-call",
     "static int gv;\n"
     "static void fba(int a){ gv=0; for(int i=0;i<a;i++) gv+=gs(i); }\n"
     "static int fb(int a){ fba(a); return gv; }",
     "s=fb(s);", ["fb", "fba"],
     "store to a STATIC GLOBAL inside the calling loop — same aliasing"
     " shape, no pointer parameter")
case("member-noloop-store",
     "struct CC { int v; CC(int a){ v=gs(a); v+=gs(a+1); v+=gs(a+2); } };\n"
     "static int fc(int a){ CC c(a); return c.v; }",
     "s=fc(s);", ["fc", "CC"],
     "three stores to this->v with calls between them, NO loop")

# --- BATCH 2: separate "a ctor with a loop" from "a store to memory inside
#     a loop", and pin the /O1 constructor rule that batch 1 turned up.
case("ctor-2store-call",
     "struct D1 { int v; D1(int a){ v=gs(a); v+=gs(a+1); } };\n"
     "static int g1(int a){ D1 c(a); return c.v; }",
     "s=g1(s);", ["g1", "D1"],
     "/O1 PRED DECLINED: exactly TWO stores to a member with a CALL between")
case("ctor-2store-nocall",
     "struct D2 { int v; D2(int a){ v=a*3; v+=a^7; } };\n"
     "static int g2(int a){ D2 c(a); return c.v; }",
     "s=g2(s);", ["g2", "D2"],
     "/O1 PRED INLINED: two stores, NO call between them")
case("ctor-1store-2call",
     "struct D3 { int v; D3(int a){ v=gs(a)+gs(a+1); } };\n"
     "static int g3(int a){ D3 c(a); return c.v; }",
     "s=g3(s);", ["g3", "D3"],
     "/O1 PRED INLINED: ONE store, two calls — calls alone are not it")
case("ctor-2mem-call",
     "struct D4 { int v,w; D4(int a){ v=gs(a); w=gs(a+1); } };\n"
     "static int g4(int a){ D4 c(a); return c.v+c.w; }",
     "s=g4(s);", ["g4", "D4"],
     "two stores to DIFFERENT members with a call between — same member or"
     " any member?")
case("method-2store-call",
     "struct D5 { int v; void set(int a){ v=gs(a); v+=gs(a+1); } };\n"
     "static int g5(int a){ D5 c; c.set(a); return c.v; }",
     "s=g5(s);", ["g5", "set"],
     "the SAME body as a member function — /O1 PRED INLINED if the rule is"
     " constructor-specific")
case("ptr-2store-call",
     "static void g6a(int *p,int a){ *p=gs(a); *p+=gs(a+1); }\n"
     "static int g6(int a){ int r; g6a(&r,a); return r; }",
     "s=g6(s);", ["g6", "g6a"],
     "…and as a free function through a pointer — /O1 PRED INLINED")
case("ctor-loop-nostore",
     "struct D7 { int v; D7(int a){ int t=0;"
     " for(int i=0;i<a;i++) t+=i*3; v=t; } };\n"
     "static int g7(int a){ D7 c(a); return c.v; }",
     "s=g7(s);", ["g7", "D7"],
     "/Ox PRED DECLINED: a ctor with a loop that neither calls nor stores to"
     " memory — tests 'any loop in a ctor' at /Ox")
case("method-loop-nostore",
     "struct D8 { int v; void set(int a){ int t=0;"
     " for(int i=0;i<a;i++) t+=i*3; v=t; } };\n"
     "static int g8(int a){ D8 c; c.set(a); return c.v; }",
     "s=g8(s);", ["g8", "set"],
     "…the identical body as a member function — /Ox PRED INLINED")
case("ptr-store-noloop",
     "static void g9a(int *p,int a){ *p=gs(a); *p+=gs(a+1); *p+=gs(a+2); }\n"
     "static int g9(int a){ int r; g9a(&r,a); return r; }",
     "s=g9(s);", ["g9", "g9a"],
     "/Ox PRED INLINED: stores through a pointer with calls between, but NO"
     " loop — tests whether the /Ox trigger needs the loop")
case("ptr-loop-store-nocall",
     "static void gaa(int *p,int a){ *p=0;"
     " for(int i=0;i<a;i++) *p+=i*3; }\n"
     "static int ga2(int a){ int r; gaa(&r,a); return r; }",
     "s=ga2(s);", ["ga2", "gaa"],
     "/Ox PRED DECLINED: store through a pointer inside a loop, loop makes NO"
     " call")


def run_cases(mode, wd, nmax, want):
    print("    `s` is what the /O1 schedule is a function of, so a row with")
    print("    Nfull=0 at a small `s` is a CATEGORICAL refusal, not the budget.")
    print("    Ndir counts the DIRECT callee only; INNER-DECLINED means a")
    print("    deeper instance was refused, which is a different pair.")
    print("    %-22s %-5s %-5s %-6s %-22s %s"
          % ("case", "Nfull", "Ndir", "P@N=1", "callee .text (s)",
             "declined per N (1..%d)" % nmax))
    bad = 0
    for name, leads, site, watch, note in CASES:
        if want and name not in want:
            continue
        per_n, nfull, ndir, ptext, sizes = [], 0, 0, None, ""
        dsz = None
        for n in range(1, nmax + 1):
            body = " ".join([site] * n)
            src = src_of(GS, [leads],
                         "%s %s %s" % (INT_HEAD, body, INT_TAIL))
            o = capture(src, mode, wd, "c_%s_%d" % (name.replace("-", "_"), n))
            if o is None:
                per_n.append("!")
                bad += 1
                continue
            r = read(o)
            if "error" in r:
                per_n.append("!")
                bad += 1
                continue
            d = declined(r["rel"], watch)
            if n == 1:
                ptext = r["tsize"]
                dsz = size_of(r["emit"], watch[0])
                sizes = ",".join("%s=%d" % (demangle_ish(nm)[:6], sz)
                                 for nm, sz in sorted(r["emit"].items())
                                 if not demangle_ish(nm).startswith("a"))
            # the DIRECT callee alone — LAW D is a claim about ONE
            # (caller, callee) pair, so a deeper refusal must not be folded in
            ddir = {w: c for w, c in d.items() if w == watch[0]}
            nd = sum(ddir.values())
            if nd == 0 and ndir == n - 1:
                ndir = n
            tot = sum(d.values())
            per_n.append("." if tot == 0
                         else "".join("%s%d" % (w[:3], c)
                                      for w, c in sorted(d.items())))
            if tot == 0 and nfull == n - 1:
                nfull = n
        v = grade_d(ndir, dsz, nmax, "/O1" in mode)
        inner = "" if nfull == ndir else "   INNER-DECLINED (a different pair)"
        print("    %-22s %-5d %-5d %-6s %-22s %s"
              % (name, nfull, ndir, ptext, sizes, " ".join(per_n)))
        print("    %-22s        %s%s" % ("", v, inner))
        print("    %-22s        %s" % ("", note))
    return bad


def sib_source(na, nb, ka, kb):
    """Two DIFFERENT callees in one P — the subject `sba` and a sibling `sbb`.

    Everything else in this file puts exactly one callee in P, so nothing
    else here can say whether the front end's limit for one (caller, callee)
    pair moves when the caller has already absorbed an unrelated expansion.
    §6.12's `ptr-sibling` is the standing warning that a property of P's
    WHOLE expansion can reach across call sites.
    """
    leads = ("static int sba(int a){ int v=gs(a)+a; %s return v; }\n"
             "static int sbb(int a){ int v=gs(a)+a; %s return v; }"
             % (stmts_fine(ka), stmts_fine(kb)))
    body = " ".join(["s=sba(s);"] * na + ["s=sbb(s);"] * nb)
    return (src_of(GS, [leads], "%s %s %s" % (INT_HEAD, body, INT_TAIL)),
            ["sba", "sbb"])


def run_padp(mode, wd):
    """Does the CALLER's own source size move the callee's limit?

    §6.15.3a grew P by *inlining* and the limit did not move. That does not
    rule out the front end pricing the caller from its ORIGINAL body, which
    is what this pads.
    """
    print("=== padded-P   does the CALLER's OWN size move the limit?")
    print("    callee held at s=80 (schedule: 5 sites). P padded with K of")
    print("    its own statements ahead of the sites.")
    print("    %-3s %-3s %-7s %-7s  %s" % ("K", "N", "s", "P.text", "declined"))
    bad = 0
    for k in (0, 10, 20, 40):
        for n in (5, 6):
            leads = ("static int sba(int a){ int v=gs(a)+a; %s return v; }"
                     % stmts_fine(8))
            pad = stmts_fine(k, "s")
            probe = ("int P(int a){ int s=gs(a)+a; %s %s %s"
                     % (pad, " ".join(["s=sba(s);"] * n), INT_TAIL))
            o = capture(src_of(GS, [leads], probe), mode, wd,
                        "pad_%d_%d" % (k, n))
            if o is None:
                print("    capture failed")
                bad += 1
                continue
            r = read(o)
            if "error" in r:
                bad += 1
                continue
            d = declined(r["rel"], ["sba"])
            print("    %-3d %-3d %-7s %-7s  %s"
                  % (k, n, size_of(r["emit"], "sba"), r["tsize"],
                     ", ".join("%s*%d" % (w, c) for w, c in sorted(d.items()))
                     or "- (everything inlined)"))
        print()
    return bad


def run_sibling(mode, wd):
    o1 = "/O1" in mode
    print("=== sibling   does P's EXISTING expansion move the limit?")
    print("    subject `sba` is sized to sit exactly at its schedule limit;")
    print("    `sbb` is an unrelated callee at nB sites in the same P.")
    print("    %-3s %-3s %-7s %-7s %-7s  %s"
          % ("nA", "nB", "s(sba)", "s(sbb)", "P.text", "declined"))
    bad = 0
    for ka, kb, grid in ((8, 8, [(5, 0), (5, 1), (5, 2), (5, 3), (5, 5),
                                 (6, 0), (6, 1)]),
                         (8, 40, [(5, 0), (5, 1), (5, 2), (4, 2)])):
        for na, nb in grid:
            src, watch = sib_source(na, nb, ka, kb)
            o = capture(src, mode, wd, "sib_%d_%d_%d_%d" % (ka, kb, na, nb))
            if o is None:
                print("    capture failed")
                bad += 1
                continue
            r = read(o)
            if "error" in r:
                bad += 1
                continue
            d = declined(r["rel"], watch)
            sa, sb = size_of(r["emit"], "sba"), size_of(r["emit"], "sbb")
            print("    %-3d %-3d %-7s %-7s %-7s  %s"
                  % (na, nb, sa, sb, r["tsize"],
                     ", ".join("%s*%d" % (w, c) for w, c in sorted(d.items()))
                     or "- (everything inlined)"))
        print()
    if o1:
        print("    SCHEDULE D is a per-PAIR claim: sba at s=80 takes 5 sites,")
        print("    sbb at s=80 takes 5 and at s=208 takes 1, INDEPENDENTLY.")
        print("    A row where sba is declined at nA<=5 refutes that.")
    return bad


def ladder_source(tree, gen, k, n):
    leads, site, watch = tree(gen(k))
    body = " ".join([site] * n)
    probe = "%s %s %s" % (INT_HEAD, body, INT_TAIL)
    return src_of(GS, [leads], probe), watch


# --------------------------------------------------------------------------
def sweep_cell(src, watch, mode, wd, tag):
    o = capture(src, mode, wd, tag)
    if o is None:
        return None
    r = read(o)
    if "error" in r:
        return None
    r["declined"] = declined(r["rel"], watch)
    # the direct callee's storage class — every grader needs the linkage class
    # before it grades (§6.17)
    r["sc"] = (pressure_of(o, watch[0]) or {}).get("sc")
    return r


def size_of(emit, want):
    """The emitted `.text` size of one planted callee, by (rough) name.

    EXACT MATCHES FIRST. The loose `want in nm` fallback is needed for member
    functions (`?set@D5@@...` demangles to `set@D5`), but on its own it makes
    `g6` match `?g6a@@YAXPAHH@Z` — a DIFFERENT function, one band away in the
    schedule — and `--cases` printed two `*** REFUTES SCHEDULE D ***` lines
    that were nothing but that collision. Sixth cry-wolf; same fix as the rest,
    which is to make the instrument say what it means.
    """
    for nm, sz in emit.items():
        if demangle_ish(nm) == want:
            return sz
    for nm, sz in emit.items():
        d = demangle_ish(nm)
        if d.endswith("@" + want) or d.endswith(want) or want in nm:
            return sz
    return None


def grade_d(nfull, s, nmax, o1, sc=None):
    """SCHEDULE D's verdict for one rung, plus the retired readings.

    `sc` is the callee's COFF storage class. SCHEDULE D was measured entirely
    on `static` callees, and §6.17 measures that an EXTERNAL one obeys a
    different and far shorter schedule (<=64 B inlined at any N, >=68 B never).
    Grading an external callee against this table would print a refutation on
    every row of a class the table never covered — the same cry-wolf as
    §6.15.8 and §6.16.9, arriving from a third direction — so it abstains.
    """
    if sc == 2:
        return ("sched D: EXTERNAL LINKAGE — not a cell of this table "
                "(§6.17: <=64 B any N, >=68 B never)")
    if not o1:
        return "sched D: /Ox is a DIFFERENT MECHANISM — not graded here"
    p = law_d(s)
    if p is None:
        return ("sched D: unbounded (s<=%d)%s"
                % (LAW_D_TABLE[0][0],
                   "" if nfull >= nmax else "   <== *** REFUTES SCHEDULE D ***"))
    got = min(p, nmax)
    if got == nfull:
        v = "sched D %d OK" % p
    else:
        v = "sched D %s vs %d   <== *** REFUTES SCHEDULE D ***" % (p, nfull)
    for label, fn in SUPERSEDED_D:
        r = fn(s)
        r = nmax if r is None else min(r, nmax)
        if got == nfull and r != nfull:
            v += "   [retired '%s' said %d]" % (label, r)
    return v


def sweep_rung(tree, gen, k, mode, wd, nmax, tag):
    """One ladder rung swept over N — (Nfull, Ndirect, s, pressure, trace)."""
    nfull = ndir = bad = 0
    dsz, press, per = None, None, []
    for n in range(1, nmax + 1):
        src, watch = ladder_source(tree, gen, k, n)
        o = capture(src, mode, wd, "%s_k%d_n%d" % (tag, k, n))
        r = None if o is None else read(o)
        if r is None or "error" in r:
            per.append("!")
            bad += 1
            continue
        if n == 1:
            dsz = size_of(r["emit"], watch[0])
            press = pressure_of(o, watch[0])
        d = declined(r["rel"], watch)
        nd = sum(c for w, c in d.items() if w == watch[0])
        if nd == 0 and ndir == n - 1:
            ndir = n
        tot = sum(d.values())
        per.append("." if tot == 0 else "x")
        if tot == 0 and nfull == n - 1:
            nfull = n
    return nfull, ndir, dsz, press, "".join(per), bad


def pcol(p):
    """savedGPRs[+savedFPRs]/frame/postpush-stores+loads."""
    if p is None:
        return "-"
    n = "%d" % p["nsave"] if not p["nfsave"] else "%d+%df" % (p["nsave"],
                                                              p["nfsave"])
    return "%s/%d/%d+%d" % (n, p["frame"], p["st"], p["ld"])


# §6.15.4's /Ox rule for LOOP-FREE callees, stated on the callee's size AS
# EMITTED AT /O1 (the /Ox-emitted size demonstrably does not decide it: /Ox is
# /Ot and unrolls the standalone callee). Every callee in the pressure pair is
# loop-free, so this applies, and it was fitted entirely on ladders that grow
# the body by adding statements — the pressure pair is held out from it.
OX_LOOPFREE = (108, 112)
# …and §6.18.7a measures a LEAF term at /Ox too — but FORTY-FOUR bytes, not the
# /O1 class's forty-eight. Measured on the same ladder, same 4-byte rungs: a
# leaf callee is inlined at 152 bytes of /O1-emitted size and declined at 156.
# The two modes agree that a call in the callee costs a flat term and DISAGREE
# on its size by 4 bytes, which is one more instance of §6.15.6's "two
# mechanisms that are not a rescaling of each other".
OX_LEAF_BONUS = 44


def ref_size(tree, gen, k, wd, tag):
    """The callee's `s` AS EMITTED AT /O1, whatever mode the run is in."""
    src, watch = ladder_source(tree, gen, k, 1)
    o = capture(src, "/O1 /GS- /c", wd, tag)
    if o is None:
        return None
    r = read(o)
    return None if "error" in r else size_of(r["emit"], watch[0])


def grade_ox(s_o1, ndir, nmax, nparams=1, leaf=False):
    """/Ox is all-or-nothing (§6.15.4), so `ndir` is 0 or `nmax`.

    §6.18.7a: the threshold carries §6.17.6's parameter correction — 4 bytes
    per parameter beyond the first, `this` included. Fitted on a 2-parameter
    callee and HELD OUT on a 3-parameter one, which moves it twice. §6.15.4 was
    fitted entirely on 1-parameter callees, so the correction is invisible
    there and the constant 108/112 is unchanged for them.
    """
    if s_o1 is None:
        return "?", "?"
    s_o1 -= PARAM_BONUS * (nparams - 1)
    if leaf:
        s_o1 -= OX_LEAF_BONUS
    got = "inlined" if ndir >= nmax else ("declined" if ndir == 0 else "mixed")
    if s_o1 <= OX_LOOPFREE[0]:
        return "inlined", got
    if s_o1 >= OX_LOOPFREE[1]:
        return "declined", got
    return "?", got


PRESSURE_PAIRS = [
    ("d1 opaque use", tree_d1_noloop, stmts_live_lo, stmts_live_hi, 12),
    ("d2 opaque use", tree_d2_noloop, stmts_live_lo, stmts_live_hi, 8),
    ("d1 re-associable use (expected INERT)",
     tree_d1_noloop, stmts_perm_lo, stmts_perm_hi, 8),
    ("d1 FPR frame class, opaque use", tree_d1_fp, stmts_fp_lo, stmts_fp_hi, 8),
    # §6.16.11's named candidate. The FP pair separates `s` and still cannot
    # discriminate, because the cheapest FP callee (116 B) is already past the
    # narrow 68-100 B region where an 8-16 byte idiom delta could straddle a
    # band. The member class keeps the GPR helper idiom — the ONE delta large
    # enough (24 B) — and puts a new callee shape underneath it.
    ("d1 MEMBER fn, `this` live from entry",
     tree_d1_member, stmts_clive_lo, stmts_clive_hi, 11),
    # The free control for it: identical body, no `this`. Its idiom crossing
    # must come one k LATER if `this` is one more live value.
    ("d1 free control for the member pair",
     tree_d1_noloop, stmts_clive_lo, stmts_clive_hi, 11),
]


def run_pressure(mode, wd, nmax, kmax, only=None):
    """SAME IL, DIFFERENT ALLOCATION — is `s` the axis, or a proxy for one?

    Every ladder in §6.15 moves `s` by adding IL, so `s` and any c1xx-side
    count of the source move TOGETHER and 449 rungs of that design cannot say
    which one the front end reads. This pairs two bodies that are the same
    2k statements in a different ORDER: identical to every source-side count,
    different in how many values are live, and therefore different in `s`.

    The falsifier is one printed row where the pair straddles a schedule band
    (law_d(s_lo) != law_d(s_hi)) and Nfull comes out the SAME anyway. That is
    `s` moving a whole band with the IL held fixed and the front end not
    noticing, and it means the table is indexed on the wrong number.
    """
    bad = 0
    for label, tree, glo, ghi, kk in PRESSURE_PAIRS:
        if only and not any(o.lower() in label.lower() for o in only):
            continue
        bad += run_pressure_pair(label, tree, glo, ghi, mode, wd, nmax,
                                 min(kmax, kk))
    return bad


def run_pressure_pair(label, tree, glo, ghi, mode, wd, nmax, kmax):
    o1 = "/O1" in mode
    print("=== pressure [%s]" % label)
    print("    the SAME 2k statements, permuted: LOW = each temp used")
    print("    immediately (<=1 live), HIGH = all k defs then all k uses in")
    print("    REVERSE order (k live across k calls). Identical statement,")
    print("    declaration, call and operator counts at every k.")
    print("    pressure column = savedGPRs/frame/postpush-stores+loads;")
    print("    nsave=18 means every nonvolatile is gone and the next live")
    print("    value MUST hit the stack.")
    print()
    print("    body = the bytes BETWEEN the frame push and the frame pop,")
    print("    everything that is not prologue or epilogue. If body_lo ==")
    print("    body_hi at every k, the whole of `ds` is FRAME IDIOM — a")
    print("    register-allocator choice the front end cannot have seen.")
    print()
    print("    %-3s %-6s %-6s %-5s %-11s %-6s %-6s %-14s %-14s %s"
          % ("k", "s_lo", "s_hi", "ds", "body lo/hi", "Nlo", "Nhi",
             "press_lo", "press_hi", "verdict"))
    bad = disc = broke = inert = bodydiff = outside = 0
    pfx = "".join(c for c in label if c.isalnum())[:12]
    for k in range(1, kmax + 1):
        nlo, dlo, slo, plo_, _t1, b1 = sweep_rung(
            tree, glo, k, mode, wd, nmax, pfx + "lo")
        nhi, dhi, shi, phi_, _t2, b2 = sweep_rung(
            tree, ghi, k, mode, wd, nmax, pfx + "hi")
        bad += b1 + b2
        if slo is None or shi is None:
            continue
        cap = lambda p: nmax if p is None else min(p, nmax)
        v = ""
        # LINKAGE FIRST. §6.17: an EXTERNAL callee obeys a different schedule
        # (<=64 B at any N, >=68 B never), so a pair of external spellings that
        # straddles a SCHEDULE D band and reads Nfull=0 twice is not evidence
        # that `s` is the wrong axis — it is a pair of cells the table never
        # covered. The member-function pair printed exactly that as three
        # `*** s IS NOT THE AXIS ***` lines the first time it ran, which is the
        # fifth time an instrument in this document has cried wolf. Abstain.
        extl = [nm for nm, p in (("lo", plo_), ("hi", phi_))
                if p and p.get("sc") == 2]
        if extl:
            print("    %-3d %-6s %-6s %-5d %-11s %-6d %-6d %-14s %-14s %s"
                  % (k, slo, shi, shi - slo,
                     "%d/%d" % (plo_["body"] if plo_ else -1,
                                phi_["body"] if phi_ else -1),
                     dlo, dhi, pcol(plo_), pcol(phi_),
                     "not graded: EXTERNAL LINKAGE (%s) — outside SCHEDULE D's"
                     " class (§6.17)" % ",".join(extl)))
            outside += 1
            continue
        if not o1:
            # §6.15.4's threshold is stated on the /O1-emitted size, so read
            # that for the SAME source rather than leaving /Ox ungraded.
            #
            # Graded PER SPELLING, and SKIPPED where a deeper instance was
            # refused. Charging an inner decline to the direct pair is the
            # cry-wolf §6.15.8 fixed for SCHEDULE D, and it reappeared here
            # the first time this grader ran: at /Ox the d2 wrapper collapses
            # to an 8-byte tail-call thunk and is inlined everywhere while
            # `in2` — a different pair — is the one refused. An /O1 reference
            # size is not a measurement of that thunk either.
            parts, wants = [], []
            spellings = (("lo", glo, dlo, nlo), ("hi", ghi, dhi, nhi))
            for nm, gg, dd, nf in spellings:
                if nf != dd:
                    parts.append("%s not graded: INNER-DECLINED "
                                 "(a different pair)" % nm)
                    continue
                rs = ref_size(tree, gg, k, wd, pfx + "ref%s_%d" % (nm, k))
                want, got = grade_ox(rs, dd, nmax)
                wants.append(want)
                bad_ = want not in ("?", got)
                if bad_:
                    broke += 1
                parts.append("%s s@O1 %s->%s got %s%s"
                             % (nm, rs, want, got,
                                "  <== *** REFUTES the /Ox LOOP-FREE "
                                "THRESHOLD ***" if bad_ else ""))
            v = "/Ox loop-free: " + " | ".join(parts)
            if len(wants) == 2 and wants[0] != wants[1] and "?" not in wants:
                disc += 1
                v += "   <== s@O1 TRACKS at /Ox too"
        elif slo == shi:
            inert += 1
            v = "INERT (c2 collapsed the permutation)"
        else:
            plo, phi = cap(law_d(slo)), cap(law_d(shi))
            if plo == phi:
                v = "same band (not discriminating)"
            else:
                disc += 1
                if dlo == dhi:
                    broke += 1
                    v = ("sched D %d vs %d   <== *** s IS NOT THE AXIS: "
                         "same IL, %+d bytes, same Nfull ***" % (plo, phi,
                                                                 shi - slo))
                elif dlo == plo and dhi == phi:
                    v = ("sched D %d/%d OK   <== s TRACKS: %+d bytes of pure "
                         "allocation moved the decision" % (plo, phi,
                                                            shi - slo))
                else:
                    broke += 1
                    v = ("sched D %d/%d vs %d/%d   <== *** REFUTES SCHEDULE D "
                         "***" % (plo, phi, dlo, dhi))
        # §6.15.8: a deeper instance being refused is a DIFFERENT pair, and
        # folding it in here is exactly the cry-wolf the falsifier was fixed
        # for. Mark it instead.
        if (nlo != dlo or nhi != dhi) and "INNER-DECLINED" not in v:
            v += "   INNER-DECLINED (a different pair)"
        blo = plo_["body"] if plo_ else -1
        bhi = phi_["body"] if phi_ else -1
        if blo != bhi:
            bodydiff += 1
        print("    %-3d %-6s %-6s %-5d %-11s %-6d %-6d %-14s %-14s %s"
              % (k, slo, shi, shi - slo, "%d/%d" % (blo, bhi), dlo, dhi,
                 pcol(plo_), pcol(phi_), v))
    print()
    print("    discriminating cells: %d   refuting rows: %d   inert rows: %d"
          "   ungraded (external linkage): %d" % (disc, broke, inert, outside))
    print("    rows where the BODY sizes differ: %d — every other row's `ds`"
          % bodydiff)
    print("    is prologue+epilogue only, i.e. 100% allocator idiom.")
    if o1 and disc == 0:
        print("    NO DISCRIMINATING CELL — the probe did not separate")
        print("    axes and this run says NOTHING about which is real.")
        if outside:
            print("    (%d rows were ungraded: the callee has EXTERNAL"
                  " linkage and is outside the table's class.)" % outside)
    return bad


# --------------------------------------------------------------------------
# ENDS — the two ROUND boundaries a fixture author actually CONSTRUCTS with
#
# `<=16 instructions -> unbounded` and `>=65 -> never` are the two ends that
# let a fixture author build a KNOWN expansion tree instead of guessing at
# one. Both were pinned on bodies whose size is all statements and whose
# register demand is trivial. These bodies keep the statement count where the
# schedule wants it and push the LIVE-VALUE count as high as it will go.
# --------------------------------------------------------------------------
def live_body(k):
    """k values live AT ONCE, defined by opaque call, combined once at the end.

    The cheapest shape per live value this instrument can build: each value
    costs one `addi` + one `bl` + one `mr` to park it, and one `xor` to
    consume it. That matters because it decides whether a SPILLING callee can
    exist inside the schedule's non-trivial band at all.
    """
    return ("static int in1(int a){ %s return %s; }"
            % (" ".join("int t%d=gs(a+%d);" % (i, i) for i in range(k)),
               "^".join("t%d" % i for i in range(k))))


# `prereg` is what I wrote down BEFORE the capture. It is printed as a
# pre-registration score, NOT as an alarm: the only alarm on these rows is
# SCHEDULE D's own, because a falsifier that fires on my estimate being wrong
# is a falsifier that cries wolf (§6.15.8).
END_BODIES = [
    ("bot-3live", 24, "unbounded", live_body(3),
     "THREE call-defined values all live at once — enough to take the"
     " __savegprlr_ helper — and still inside the <=64 B / <=16 instr floor"),
    ("bot-4live", 24, "unbounded", live_body(4),
     "one more live value. PRE-REGISTERED as still <=64 B, and it is NOT:"
     " liveness alone carries a body over the floor"),
    ("spill-20", 3, "0", live_body(20),
     "TWENTY values live at once — past the 18 nonvolatile GPRs, so the"
     " allocator MUST spill. Does the >=65-instruction ceiling still hold"
     " when the bytes are spill code?"),
    ("spill-24", 3, "0", live_body(24),
     "…and deeper into the spill region"),
]


# --------------------------------------------------------------------------
# LINKAGE — SCHEDULE D is a claim about `static` callees, and nothing else
#
# Every ladder in §6.15 and §6.16 declares its callee `static int in1(int)`.
# Round 30 removed the `static` and nothing else: the SAME body, the SAME
# emitted bytes, is refused at N=1 where the static one takes its tabulated
# number of sites. Measured at 4-byte resolution with four independent rung
# kinds (1 instruction, 2 instructions, a call, and a DEAD LOCAL that emits no
# code at all), the /O1 decision splits into three classes:
#
#   internal (`static`)          SCHEDULE D — graduated, N-dependent, on `s`
#   external, not `inline`       est <= 64 -> inlined at ANY N; >= 68 -> NEVER
#   external, `inline`           est <= 72 -> inlined at ANY N; >= 76 -> NEVER
#   `__forceinline`              inlined at every size measured
#
# where `est` is the callee's own emitted size LESS one instruction per
# parameter beyond the first:
#
#       est = s - 4 * (nparams - 1)
#
# An in-class member function is implicitly `inline` and lands on the middle
# row EXACTLY; the same member defined OUT of class without the keyword lands
# on the top row EXACTLY. So class membership, `this`, `const`, the object's
# storage and the depth of the call site are all irrelevant — measured, each
# against its own control.
#
# `est` was fitted on 1- and 2-parameter callees and HELD OUT on 3- and
# 4-parameter ones: 24/24 cells. The matching prediction for the STATIC class
# was REFUTED — a two- and three-parameter static callee follows `s`, not
# `est`, on 10 discriminating cells — so the two classes do not share a size
# measure, and SCHEDULE D's index is confirmed as `s` against a shape it had
# never been tested on.
# and `inline` is worth exactly 8 bytes in BOTH classes: an external callee's
# threshold moves 64 -> 72, and an INTERNAL one moves a whole schedule, which
# is how it was found — `static inline` was written into this instrument as a
# CONTROL, with the note "inline does not move an internal callee", and it
# refuted SCHEDULE D on its own row at 68, 72, 76 and 80 bytes. Held out
# afterwards across the entire rest of the table (84, 88, 92, 104, 144, 260,
# 264): SCHEDULE D on `s-8`, 7 discriminating cells, 0 misses, including the
# ceiling — a 264-byte `static inline` callee is inlined once where a plain one
# is refused.
#
# `__forceinline` overrides all of it: 12 of 12 sites at every size measured,
# up to 264 bytes, in both linkage classes.
EXT_MAX = 64                  # == the top of SCHEDULE D's `unbounded` band
INLINE_BONUS, PARAM_BONUS, LEAF_BONUS = 8, 4, 48


# The allocator's own save/restore pair. It is a REL24 and it is NOT a call for
# the 48-byte term — §6.19.6, measured on a matched pair that holds the volatile
# shape, the liveness, the frame and the helper pair itself fixed and varies
# only whether the SOURCE contains a call: 21 discriminating cells on the leaf
# schedule for the helper-only callee, 19 on the non-leaf schedule for the one
# with a call added. §6.18.10 shipped this as an open risk and named it exactly.
ALLOC_HELPERS = ("__savegprlr_", "__restgprlr_", "__savefpr_", "__restfpr_",
                 "__savevmx_", "__restvmx_")


def callee_is_leaf(o, want):
    """Does the planted callee contain a CALL? Read out of the obj.

    §6.18.6/§6.18.7: a callee with no call is indexed 48 bytes lower, in BOTH
    linkage classes, with the schedule's whole shape preserved (61 cells, 0
    misses). The trigger is the CALL and not the frame — a TAIL-call callee has
    no frame, no `stwu` and no LR save and is on the NON-leaf schedule (46
    cells, 0 misses), and this compiler never gives a leaf a frame at all, so
    "has a frame" cannot be the reading.

    A REL24 anywhere in the callee is a call — EXCEPT the allocator's own
    save/restore pair, which §6.18.10 shipped as an unmeasured risk and §6.19.6
    measures. `bl __savegprlr_N` does NOT count: a callee whose only REL24 is
    the helper sits on the LEAF schedule (21 discriminating cells, 0 misses),
    while the matched control that adds ONE real call to the identical shape
    sits on the non-leaf one (19 cells, 0 misses). The trigger is a call the
    SOURCE contains, and the helper does not exist until after allocation.
    """
    for g in groups(o):
        if name_matches(g["name"], want):
            break
    else:
        return None
    sec, lo, hi = extent(o, g)
    for va, sy, ty in o.relocs(sec):
        if not (lo <= va < hi) or ty != REL24:
            continue
        s = o.sym_by_index(sy)
        if s is not None and s["name"].startswith(ALLOC_HELPERS):
            continue                  # the allocator's, not the source's
        return False
    return bcctr_count(o, want) == 0  # an indirect call is still a call


def sched_index(s, nparams=1, inline_kw=False, external=False, leaf=False):
    """The size the /O1 decision is indexed on, per §6.17 and §6.18.

    Internal: `s`, less 8 if the callee is `inline`. The parameter count does
    NOT enter — pre-registered as the opposite and refuted on 10 cells.
    External: additionally less 4 per parameter beyond the first — fitted on 1
    and 2 parameters and held out on 3 and 4, 24/24.
    BOTH classes: less 48 if the callee contains NO CALL (§6.18.6). That term
    is six times the `inline` one and was invisible for three rounds because
    every ladder callee in this document calls `gs`.
    """
    if s is None:
        return None
    idx = s - (INLINE_BONUS if inline_kw else 0)
    if external:
        idx -= PARAM_BONUS * (nparams - 1)
    if leaf:
        idx -= LEAF_BONUS
    return idx


def linkage_verdict(idx, sc, force=False):
    """Predicted /O1 verdict for a callee outside SCHEDULE D's class.

    An EXTERNAL callee gets the schedule's `unbounded` row and NOTHING ELSE:
    at or below 64 it is inlined at every N measured, above it never, with no
    graduated middle. That is the whole of the difference — one table, and
    linkage decides whether its 9/7/5/4/3/2/1 part exists for you.
    """
    if force:
        return "inlined"
    if sc != 2:
        return None                        # internal: SCHEDULE D applies
    return "inlined" if idx <= EXT_MAX else "declined"


def bcctr_count(o, want, linked=False):
    """Indirect control transfers inside one planted function.

    `linked=True` counts only `bcctrl` (LK set), i.e. indirect CALLS. The
    default counts `bcctr` as well, because a tail call through a function
    pointer is `bctr` with LK clear and is still a call for every purpose here.
    """
    for g in groups(o):
        if name_matches(g["name"], want):
            break
    else:
        return 0
    sec, lo, hi = extent(o, g)
    d = o.raw(sec)[lo:hi]
    n = 0
    for i in range(0, len(d) - 3, 4):
        w = struct.unpack_from(">I", d, i)[0]
        if (w >> 26) == 19 and ((w >> 1) & 0x3FF) == 528:
            if not linked or (w & 1):
                n += 1
    return n


def surviving(o, r, n, want="c1"):
    """How many of P's N sites the front end DECLINED — the ninth cry-wolf.

    `read()` counts every `bcctrl` in P, and §6.18.0(B) added that term for a
    real reason: a VIRTUAL site is an indirect branch with no relocation, so a
    reloc-only detector reports the one call kind that cannot be inlined as
    INLINED. But the term is one-sided, and the other side bites the moment the
    CALLEE itself contains an indirect call: inlining such a callee at n sites
    puts n `bcctrl`s into P, and the detector reads its own success as n
    declines. Measured: `indirect` and `tailind` read `Ndir = 0` at EVERY size,
    including s=20 — far below any band — which looks exactly like a new
    categorical class and is nothing but the detector counting the expansion.

    So the indirect term is kept, and made ACCOUNTED FOR rather than absolute:
    only indirect calls P has that its own expansion cannot explain are
    declines. On every callee in §6.15-§6.18 the callee's own count is 0, so
    this is identically the old expression there — the virtual rows still read
    `Ndir = 0` and still print their five disagreeing cells.
    """
    nd = sum(declined(r["rel"], [want]).values())
    cb = bcctr_count(o, want)
    return nd + max(0, r["nind"] - n * cb)


def callee_bytes(o, want):
    """One planted callee's OWN code bytes, for a byte-exact comparison."""
    for g in groups(o):
        if name_matches(g["name"], want):
            break
    else:
        return None
    sec, lo, hi = extent(o, g)
    return o.raw(sec)[lo:hi]


LINK_KINDS = [
    # (name, nparams, `inline`?, forceinline?, source template, site, note)
    ("sta-plain", 1, False, False, "static int c1(int a){ %s }", "s=c1(s);",
     "the class every ladder in §6.15/§6.16 is built from"),
    ("sta-inline", 1, True, False, "static inline int c1(int a){ %s }",
     "s=c1(s);",
     "`inline` moves an INTERNAL callee too — this row refuted SCHEDULE D"),
    ("sta-force", 1, False, True, "static __forceinline int c1(int a){ %s }",
     "s=c1(s);", "…and __forceinline ignores the schedule entirely"),
    ("ext-plain", 1, False, False, "int c1(int a){ %s }", "s=c1(s);",
     "THE PROBE: the same body with `static` removed"),
    ("ext-inline", 1, True, False, "inline int c1(int a){ %s }", "s=c1(s);",
     "…marked `inline`: 8 bytes more room, and that is the whole difference"),
    ("ext-force", 1, False, True, "__forceinline int c1(int a){ %s }",
     "s=c1(s);", "…and the keyword that overrides the lot"),
    ("mem-inclass", 1, True, False,
     "struct CB { static int c1(int a){ %s } };", "s=CB::c1(s);",
     "a STATIC member — no `this` — implicitly inline: == ext-inline"),
    ("mem-outclass", 1, False, False,
     "struct CB { static int c1(int a); };\nint CB::c1(int a){ %s }",
     "s=CB::c1(s);",
     "the same member defined OUT of class, no keyword: == ext-plain"),
    ("mem-this", 2, True, False,
     "struct CB { int m; int c1(int a){ %s } };\nextern CB* bp;",
     "s=bp->c1(s);", "a non-static member: `this` is one more parameter"),
    ("ext-2arg", 2, False, False, "int c1(int q,int a){ %s }", "s=c1(1,s);",
     "an unused SECOND parameter: +4 bytes of `s`, +0 of the index"),
    ("ext-3arg", 3, False, False, "int c1(int q,int r,int a){ %s }",
     "s=c1(1,2,s);", "HELD OUT from the parameter fit"),
]


def stmts_fine_dead(k, ndead):
    """`stmts_fine`'s k code rungs PLUS a fixed number of DEAD locals.

    §6.15.2's dead-local negative — "dead locals move the decline by ZERO" —
    and §6.17.4's restatement of it in both linkage classes are both measured
    on a ladder whose `s` does not move: every rung is the base body's size.
    That base sits at index 48, sixteen bytes below the nearest band boundary,
    and both classes read 12-of-12 at every rung. A charge of anything under
    16 bytes per local could not have shown up, so "zero" is a bound of
    `< 16/k`, not a measurement of zero (task #92).

    This generator walks the same question ACROSS the boundaries instead: the
    dead locals are held fixed and `stmts_fine`'s 4-byte rungs move the body
    over every band edge SCHEDULE D has. If a dead local carries any charge at
    all, this row's profile is the plain `fine` row's shifted by `ndead` times
    it — at 4-byte resolution, at eight boundaries, in both classes.
    """
    return (stmts_fine(k) + " "
            + " ".join("int dd%d=%d;" % (i, i + 5) for i in range(ndead)))


def run_linkage(mode, wd, nmax, krange=None, deadpad=False, kinds=None):
    """The /O1 decision is a function of LINKAGE first, `s` second.

    Four rung kinds so the axis cannot be confused with a statement count:
    `fine` (1 instruction), `2instr`, `call` (3 instructions) and `deadloc`
    (a declared local that emits NOTHING — §6.15.2's sharpest probe, which
    moves `s` by zero).

    THE RUNG RANGE IS A PARAMETER, and was not (task #92). `k = 0..8` was
    hardcoded, and §6.17.8's `/Ox` negative — "the `static` and `external`
    columns are IDENTICAL in every cell" — was read off it. At `/Ox` the only
    decision boundary is the 108/112 threshold (§6.15.4), and over `k = 0..8`
    three of the four rung kinds never take EITHER class past it: every cell
    is 12-of-12 against 12-of-12. A negative measured entirely on one side of
    the boundary it is about is not a measured negative, so `--kmin`/`--kmax`
    reach this mode now and the SPLIT SUMMARY below counts, per rung kind, how
    many cells could have disagreed.
    """
    o1 = "/O1" in mode
    kmin, kmax = krange or (0, 8)
    seen = {}
    rungs = [("fine", stmts_fine), ("2instr", lambda k: " ".join(
        "v=(v*3)^%d;" % (0x11 * (i + 1)) for i in range(k))),
        ("call", stmts_call), ("deadloc", stmts_deadloc)]
    if deadpad:
        rungs = [("fine", stmts_fine),
                 ("fine+3dead", lambda k: stmts_fine_dead(k, 3)),
                 ("fine+8dead", lambda k: stmts_fine_dead(k, 8))]
    bad = wrong = 0
    for rname, gen in rungs:
        print("=== linkage [rung: %s]   `s`=own size; index = s - 8*inline"
              % rname)
        print("    (external: also -4 per parameter beyond the first).")
        print("    Ndir = largest N at which P kept no `bl`; N is swept to %d,"
              % nmax)
        print("    so `unbounded` is measured, not assumed.")
        print("    %-16s %-3s %-6s %-6s %-6s %-10s %s"
              % ("kind", "k", "s", "index", "Ndir", "predicted", "note"))
        for kind, np_, inl, force, tmpl, site, note in LINK_KINDS:
            # NOT `want`: that name is already a per-rung LOCAL below
            # (the predicted verdict). Shadowing it made the filter
            # read `kind not in 'inlined'` from the second kind on and
            # silently drop every external row — caught by the SPLIT
            # SUMMARY printing `no cells`, which is what that counter
            # is for, before a single verdict was read off the run.
            if kinds and kind not in kinds:
                continue
            for k in range(kmin, kmax + 1):
                body = "int v=gs(a)+a; %s return v;" % gen(k)
                leads = tmpl % body
                s, ndir = None, 0
                for n in range(1, nmax + 1):
                    src = src_of(GS, [leads], "%s %s %s"
                                 % (INT_HEAD, " ".join([site] * n), INT_TAIL))
                    o = capture(src, mode, wd,
                                "lnk_%s_%s_%d_%d" % (rname, kind, k, n))
                    r = None if o is None else read(o)
                    if r is None or "error" in r:
                        bad += 1
                        break
                    if n == 1:
                        s = size_of(r["emit"], "c1")
                        sc = (pressure_of(o, "c1") or {}).get("sc")
                    nd = sum(declined(r["rel"], ["c1"]).values())
                    if nd == 0 and ndir == n - 1:
                        ndir = n
                    if n == 1 and nd:
                        break        # declined at the first site: all-or-none
                if s is None:
                    continue
                idx = sched_index(s, np_, inl, sc == 2)
                want = linkage_verdict(idx, sc, force) if o1 else None
                got = "inlined" if ndir else "declined"
                mark = ""
                if want:
                    mark = ("OK" if want == got
                            else "<== *** REFUTES THE LINKAGE MODEL ***")
                    if want != got:
                        wrong += 1
                elif o1:
                    # internal: SCHEDULE D, graded on the INDEX (= `s` unless
                    # the callee is `inline`, which is worth 8 bytes)
                    mark = grade_d(ndir, idx, nmax, o1, sc)
                    if "REFUTES" in mark:
                        wrong += 1
                else:
                    mark = ("not graded: /Ox is a different mechanism — and"
                            " §6.17.8 measures NO linkage split there")
                seen[(rname, kind, k)] = (s, ndir)
                print("    %-16s %-3d %-6s %-6d %-6d %-10s %s"
                      % (kind, k, s, idx, ndir, want or "sched D",
                         mark if k else (mark + "   " + note)))
            print()
        linkage_split_summary(rname, seen, nmax, kmin, kmax)
    if deadpad:
        deadpad_summary(seen, nmax, kmin, kmax)
    print("    rows refuting the linkage model (external) or SCHEDULE D on"
          " the index (internal): %d" % wrong)
    return bad


def deadpad_summary(seen, nmax, kmin, kmax):
    """Do DEAD LOCALS carry any charge, measured at every band boundary?

    Model-free and index-matched: `fine+Ndead` against `fine`, per linkage
    kind, at every rung. Two counters, per §6.16.2 — a cell where both rows
    are fully inlined cannot express a charge and is not counted as agreement.
    """
    print("    --- DEAD LOCALS ACROSS THE BOUNDARIES [k=%d..%d]" % (kmin, kmax))
    print("        §6.15.2/§6.17.4 measured this at ONE index (48), sixteen")
    print("        bytes from the nearest band edge and 12/12 in both classes:")
    print("        a charge under 16 bytes could not have shown up there. Here")
    print("        the SAME dead locals ride a 4-byte ladder over every edge.")
    for probe in ("fine+3dead", "fine+8dead"):
        for kind, _np, _inl, _f, _t, _s, _n in LINK_KINDS:
            cells = disc = dis = dsz = 0
            first = None
            for k in range(kmin, kmax + 1):
                ra, rb = seen.get(("fine", kind, k)), seen.get((probe, kind, k))
                if ra is None or rb is None:
                    continue
                cells += 1
                if ra[0] != rb[0]:
                    dsz += 1
                if ra[1] < nmax or rb[1] < nmax:
                    disc += 1
                if ra[1] != rb[1]:
                    dis += 1
                    if first is None:
                        first = (k, ra, rb)
            if not cells:
                continue
            if not disc:
                tail = ("<== NO DISCRIMINATING CELL — %d/%d everywhere"
                        % (nmax, nmax))
            elif dis:
                tail = ("<== *** DEAD LOCALS MOVE IT: %d of %d ***  first at"
                        " k=%d (s %s vs %s, Ndir %d vs %d)"
                        % (dis, cells, first[0], first[1][0], first[2][0],
                           first[1][1], first[2][1]))
            else:
                tail = "IDENTICAL on all %d cells, %d discriminating" \
                       % (cells, disc)
            print("        %-11s %-13s cells %-3d disc %-3d dNdir %-3d ds %-3d"
                  " %s" % (probe, kind, cells, disc, dis, dsz, tail))
    print()


# The three matched (internal, external) spellings of ONE body. Anything else
# in LINK_KINDS differs by more than the storage class.
LINK_PAIRS = [("sta-plain", "ext-plain"), ("sta-inline", "ext-inline"),
              ("sta-force", "ext-force")]


def linkage_split_summary(rname, seen, nmax, kmin, kmax):
    """Model-free: does the storage class move the verdict, and COULD it?

    §6.17.8 read "identical in every cell" as a negative. A cell in which both
    spellings are fully inlined at every one of `nmax` sites cannot express a
    split in either direction — it is §6.16.2's inert row, and it prints what
    a discriminating one prints. This counts the two separately, and says in
    words when there is nothing to read.
    """
    print("    --- LINKAGE SPLIT, model-free [rung: %s, k=%d..%d]"
          % (rname, kmin, kmax))
    print("        a cell DISCRIMINATES iff at least one of the two spellings")
    print("        is NOT fully inlined: while both read %d/%d the cell cannot"
          % (nmax, nmax))
    print("        express a split and agreeing costs nothing (§6.16.2).")
    for a, b in LINK_PAIRS:
        cells = disc = dis = 0
        first = None
        for k in range(kmin, kmax + 1):
            ra, rb = seen.get((rname, a, k)), seen.get((rname, b, k))
            if ra is None or rb is None:
                continue
            cells += 1
            if ra[1] < nmax or rb[1] < nmax:
                disc += 1
            if ra[1] != rb[1]:
                dis += 1
                if first is None:
                    first = (k, ra, rb)
        tail = ""
        if not cells:
            tail = "no cells"
        elif not disc:
            tail = ("<== NO DISCRIMINATING CELL — every cell is %d/%d in BOTH"
                    " classes; this rung says NOTHING about a linkage split"
                    % (nmax, nmax))
        elif dis:
            tail = ("<== *** THE CLASSES DIFFER: %d of %d ***  first at k=%d"
                    " (s=%s: %d vs %s: %d)"
                    % (dis, cells, first[0], a, first[1][1], b, first[2][1]))
        else:
            tail = "IDENTICAL on all %d cells, %d of them discriminating" \
                   % (cells, disc)
        print("        %-12s vs %-12s  cells %-3d disc %-3d disagree %-3d %s"
              % (a, b, cells, disc, dis, tail))
    print()


# --------------------------------------------------------------------------
# AXES — what else did every ladder in this document hold fixed?
#
# §6.17 found linkage only because a probe aimed at something else tripped over
# it, and its closing paragraph names why it could hide for two rounds: EVERY
# callee in EVERY ladder of §6.15 and §6.16 is a one-parameter, keyword-free
# `static int f(int)`. The instrument's own controls would not have noticed a
# term carried by any of the variables that design never varied. This mode
# varies them one at a time, each against a matched control:
#
#   return type   char/short/bool/unsigned/long long/double/void/ptr/ref/struct
#   language      `extern "C"`, with and without `static` and `inline`
#   storage       a static LOCAL, const-initialised and dynamically initialised
#   virtual       through a pointer, on a local object, and the non-virtual twin
#   template      an instantiation, plus its `static` and `inline` spellings
#
# GRADED BY MEASUREMENT, NEVER BY SPELLING. Every row reads the callee's own
# COFF storage class out of the symbol table and grades against THAT class's
# index rule — §6.17.3's anonymous-namespace row is the standing reason: it was
# pre-registered as internal linkage and this compiler emits it EXTERNAL.
#
# DISCRIMINATING CELL, defined before the run so the count cannot be tuned
# afterwards: a rung discriminates iff its class's predictor gives a DIFFERENT
# answer at `index` than at `index-8` or `index+8`. Eight bytes is not
# arbitrary — it is the size of the only extra term this document has ever
# found (`inline`, §6.17.5) and the size a new axis would most plausibly carry.
# A rung outside that window can agree with the model without the model ever
# having been at risk, and §6.16.2's lesson is that such a row PRINTS THE SAME
# THING as one that was.
AXIS_KMAX = 14                 # `fine` rungs: s = 48..104 on the framed base
AXIS_KMAX_OX = 22              # /Ox's threshold sits at 108/112, further out


def axis_predict(idx, sc, nmax):
    """(predicted Ndir, label) for one rung, from its MEASURED storage class."""
    if idx is None or sc is None:
        return None, "?"
    if sc == 2:
        ok = idx <= EXT_MAX
        return (nmax if ok else 0), ("inlined" if ok else "declined")
    p = law_d(idx)
    return (nmax if p is None else min(p, nmax)),           \
           ("unbounded" if p is None else "%d" % p)


def axis_discriminates(idx, sc, nmax):
    """Could an 8-byte term have shown up in this rung? See the header."""
    if idx is None or sc is None:
        return False
    base = axis_predict(idx, sc, nmax)[0]
    return any(axis_predict(idx + d, sc, nmax)[0] != base for d in (-8, 8))


# (name, control, nparams, inline?, force?, decls template, site, note)
# `nparams` is the count of parameters the MODEL says the external index
# subtracts for — the declared ones plus `this` (§6.17.6). A hidden sret
# pointer is deliberately NOT counted, so that if it is one the row refutes
# rather than being fitted.
AXIS_KINDS = [
    # ---- the two controls, re-measured by this instrument --------------
    ("sta-base", None, 1, False, False,
     "static int c1(int a){ int v=gs(a)+a; %s return v; }", "s=c1(s);",
     "CONTROL: §6.17.4's own static row (12/12/12/12/12/9/9/7/5/...)"),
    ("ext-base", None, 1, False, False,
     "int c1(int a){ int v=gs(a)+a; %s return v; }", "s=c1(s);",
     "CONTROL: §6.17.4's own external row (step at index 64/68)"),

    # ---- AXIS 1: the return type --------------------------------------
    ("sta-ret-char", "sta-base", 1, False, False,
     "static char c1(int a){ int v=gs(a)+a; %s return (char)v; }",
     "s+=c1(s);", "a narrower integer return"),
    ("ext-ret-char", "ext-base", 1, False, False,
     "char c1(int a){ int v=gs(a)+a; %s return (char)v; }",
     "s+=c1(s);", ""),
    ("sta-ret-short", "sta-base", 1, False, False,
     "static short c1(int a){ int v=gs(a)+a; %s return (short)v; }",
     "s+=c1(s);", ""),
    ("sta-ret-bool", "sta-base", 1, False, False,
     "static bool c1(int a){ int v=gs(a)+a; %s return v!=0; }",
     "s+=c1(s);", ""),
    ("sta-ret-uint", "sta-base", 1, False, False,
     "static unsigned c1(int a){ int v=gs(a)+a; %s return (unsigned)v; }",
     "s+=(int)c1(s);", ""),
    ("sta-ret-ll", "sta-base", 1, False, False,
     "static long long c1(int a){ int v=gs(a)+a; %s return (long long)v; }",
     "s+=(int)c1(s);", "the result is an r3:r4 PAIR, not a register"),
    ("ext-ret-ll", "ext-base", 1, False, False,
     "long long c1(int a){ int v=gs(a)+a; %s return (long long)v; }",
     "s+=(int)c1(s);", ""),
    ("sta-ret-dbl", "sta-base", 1, False, False,
     "static double c1(int a){ int v=gs(a)+a; %s return (double)v; }",
     "s+=(int)c1(s);", "an FPR result, `_fltused`, a different frame class"),
    ("ext-ret-dbl", "ext-base", 1, False, False,
     "double c1(int a){ int v=gs(a)+a; %s return (double)v; }",
     "s+=(int)c1(s);", ""),
    ("sta-ret-ptr", "sta-base", 1, False, False,
     "extern int* gp;\n"
     "static int* c1(int a){ int v=gs(a)+a; %s return gp+v; }",
     "s+=(int)(c1(s)-gp);", ""),
    ("sta-ret-ref", "sta-base", 1, False, False,
     "extern int* gp;\n"
     "static int& c1(int a){ int v=gs(a)+a; %s return gp[v]; }",
     "s+=c1(s);", "a reference return — a pointer the source does not spell"),
    # `void` needs its own control, because dropping the result also drops the
    # `return` statement: `sta-ret-void-ctl` is the same body WITH a result.
    ("sta-ret-void-ctl", "sta-base", 1, False, False,
     "extern int gv;\n"
     "static int c1(int a){ int v=gs(a)+a; %s gv=v; return v; }",
     "s=c1(s);", "the matched control for the `void` row: same store, a result"),
    ("sta-ret-void", "sta-ret-void-ctl", 1, False, False,
     "extern int gv;\n"
     "static void c1(int a){ int v=gs(a)+a; %s gv=v; }",
     "c1(s);", "…and the same body with NO result"),
    ("ext-ret-void", "ext-base", 1, False, False,
     "extern int gv;\n"
     "void c1(int a){ int v=gs(a)+a; %s gv=v; }",
     "c1(s);", ""),
    # A struct return is an sret HIDDEN POINTER PARAMETER. `nparams` stays 1
    # here on purpose: if the hidden pointer is charged like a real one the
    # external step moves 4 bytes and the row REFUTES rather than agreeing.
    ("sta-ret-struct", "sta-base", 1, False, False,
     "struct S2 { int x, y; };\n"
     "static S2 c1(int a){ int v=gs(a)+a; %s S2 r; r.x=v; r.y=v; return r; }",
     "s+=c1(s).x;", "sret: a hidden pointer parameter the source never spells"),
    ("ext-ret-struct", "ext-base", 1, False, False,
     "struct S2 { int x, y; };\n"
     "S2 c1(int a){ int v=gs(a)+a; %s S2 r; r.x=v; r.y=v; return r; }",
     "s+=c1(s).x;", ""),

    # ---- AXIS 2: `extern "C"` -----------------------------------------
    ("extc-ext", "ext-base", 1, False, False,
     'extern "C" int c1(int a){ int v=gs(a)+a; %s return v; }', "s=c1(s);",
     "C language linkage: an UNMANGLED external symbol"),
    ("extc-sta", "sta-base", 1, False, False,
     'extern "C" static int c1(int a){ int v=gs(a)+a; %s return v; }',
     "s=c1(s);", "…with `static`: C linkage, internal COFF linkage"),
    ("extc-inline", "ext-base", 1, True, False,
     'extern "C" inline int c1(int a){ int v=gs(a)+a; %s return v; }',
     "s=c1(s);", "…does the 8-byte `inline` term survive C linkage?"),

    # ---- AXIS 3: storage duration inside the callee ---------------------
    ("sta-glob-ctl", "sta-base", 1, False, False,
     "extern int gt;\n"
     "static int c1(int a){ int v=gs(a)+a; %s gt+=v; return v; }",
     "s=c1(s);", "CONTROL for the static-local rows: the same store, to a "
     "global"),
    ("sta-sloc-const", "sta-glob-ctl", 1, False, False,
     "static int c1(int a){ static int t=7; int v=gs(a)+a; %s t+=v;"
     " return v; }", "s=c1(s);",
     "a static LOCAL, constant-initialised — no guard"),
    ("ext-sloc-const", "ext-base", 1, False, False,
     "int c1(int a){ static int t=7; int v=gs(a)+a; %s t+=v; return v; }",
     "s=c1(s);", ""),
    ("sta-sloc-dyn", "sta-glob-ctl", 1, False, False,
     "static int c1(int a){ static int t=gs(7); int v=gs(a)+a; %s t+=v;"
     " return v; }", "s=c1(s);",
     "…DYNAMICALLY initialised: a guard variable and a one-shot branch"),
    ("ext-sloc-dyn", "ext-base", 1, False, False,
     "int c1(int a){ static int t=gs(7); int v=gs(a)+a; %s t+=v; return v; }",
     "s=c1(s);", ""),

    # ---- AXIS 4: virtual ------------------------------------------------
    # The control differs from the probe by exactly the `virtual` keyword: same
    # class, same out-of-class definition, same `this`, same call expression.
    ("mem-nonvirt", "ext-base", 2, False, False,
     "struct VB { int m; int c1(int a); };\n"
     "int VB::c1(int a){ int v=gs(a)+a; %s return v; }\n"
     "extern VB* vp;", "s=vp->c1(s);",
     "CONTROL: an out-of-class member through a pointer (== ext-plain, §6.17)"),
    ("mem-virt", "mem-nonvirt", 2, False, False,
     "struct VB { int m; virtual int c1(int a); };\n"
     "int VB::c1(int a){ int v=gs(a)+a; %s return v; }\n"
     "extern VB* vp;", "s=vp->c1(s);",
     "…the same, plus the `virtual` keyword and nothing else"),
    ("mem-virt-obj", "mem-virt", 2, False, False,
     "struct VB { int m; virtual int c1(int a); };\n"
     "int VB::c1(int a){ int v=gs(a)+a; %s return v; }",
     "{ VB o; s=o.c1(s); }",
     "…called on a LOCAL object: the static type is known, so it CAN be "
     "devirtualised"),
    # `VB gvb;` is not decoration: an in-class virtual that is only ever CALLED
    # virtually is never referenced by name, so it is NOT EMITTED and the row
    # has no `s` to grade — §6.5's "the callee's COMDAT is emitted whether or
    # not it was inlined" has an exception and this is it. A global instance
    # forces the vtable, and the vtable references the body.
    ("mem-virt-inclass", "mem-virt", 2, True, False,
     "struct VB { int m; virtual int c1(int a){ int v=gs(a)+a; %s"
     " return v; } };\nextern VB* vp;\nVB gvb;", "s=vp->c1(s);",
     "…defined in-class, hence implicitly `inline` as well"),

    # ---- AXIS 4b: two more keywords the ladders never varied ------------
    ("sta-vararg", "sta-base", 1, False, False,
     "static int c1(int a, ...){ int v=gs(a)+a; %s return v; }", "s=c1(s);",
     "a VARIADIC callee — the one shape a caller cannot simply substitute"),
    ("ext-vararg", "ext-base", 1, False, False,
     "int c1(int a, ...){ int v=gs(a)+a; %s return v; }", "s=c1(s);", ""),
    # A framed variadic callee cannot be built smaller than 76 bytes, which is
    # already past the `unbounded` band — so on the framed base "refused at
    # every size" cannot be told apart from "refused because of its size" at
    # the one place the pre-registration says to look. A LEAF body can: it
    # starts around 12 bytes and the ladder walks the whole band. The control
    # is the identical leaf body without the `...`.
    ("sta-leaf-ctl", "sta-base", 1, False, False,
     "static int c1(int a){ int v=a*3; %s return v; }", "s=c1(s);",
     "CONTROL: a LEAF body, so the ladder starts below the unbounded band"),
    ("sta-vararg-leaf", "sta-leaf-ctl", 1, False, False,
     "static int c1(int a, ...){ int v=a*3; %s return v; }", "s=c1(s);",
     "…the same leaf body, variadic: does it refuse BELOW the band too?"),
    ("ext-leaf-ctl", "ext-base", 1, False, False,
     "int c1(int a){ int v=a*3; %s return v; }", "s=c1(s);",
     "CONTROL: the same leaf body, external linkage"),
    ("ext-vararg-leaf", "ext-leaf-ctl", 1, False, False,
     "int c1(int a, ...){ int v=a*3; %s return v; }", "s=c1(s);", ""),
    # Separates "has a frame" from "contains a call": a stack array forces a
    # frame (`stwu`) with no `bl` and no LR save anywhere in the callee.
    ("sta-frameleaf", "sta-leaf-ctl", 1, False, False,
     "static int c1(int a){ int ar[4]; ar[a&3]=a; int v=ar[(a>>1)&3]*3;"
     " %s return v; }", "s=c1(s);",
     "a STACK FRAME but no call: does the 48-byte term key on the frame?"),
    # The one shape in this compiler that HAS a call and has NO frame and no
    # LR save: a tail call (`b gs`). It is the only probe that can separate the
    # 48-byte term's three coincident descriptions.
    ("sta-tailcall", "sta-leaf-ctl", 1, False, False,
     "static int c1(int a){ int v=a*3; %s return gs(v); }", "s=c1(s);",
     "a TAIL CALL: a call with no frame and no LR save"),
    ("sta-throw", "sta-base", 1, False, False,
     "static int c1(int a) throw(){ int v=gs(a)+a; %s return v; }",
     "s=c1(s);", "an empty exception specification"),
    ("ext-throw", "ext-base", 1, False, False,
     "int c1(int a) throw(){ int v=gs(a)+a; %s return v; }", "s=c1(s);", ""),

    # §6.17.6's parameter correction is an /O1 EXTERNAL-class rule. The /Ox
    # sweep found ONE cell suggesting the /Ox loop-free threshold carries it
    # too — both `mem-*` rows are 2-parameter callees. These are the free
    # functions that decide it without `this` in the way.
    ("ext-2arg", "ext-base", 2, False, False,
     "int c1(int q,int a){ int v=gs(a)+a; %s return v; }", "s=c1(1,s);",
     "an unused SECOND parameter: does /Ox carry §6.17.6's correction?"),
    ("ext-3arg", "ext-base", 3, False, False,
     "int c1(int q,int r,int a){ int v=gs(a)+a; %s return v; }",
     "s=c1(1,2,s);", "HELD OUT: a third parameter should move it twice"),

    # ---- AXIS 5: templates ----------------------------------------------
    ("tmpl-ext", "ext-base", 1, False, False,
     "template<class T> T c1(T a){ T v=gs(a)+a; %s return v; }",
     "s=c1<int>(s);", "an instantiation — a COMDAT, like an `inline` function"),
    ("tmpl-sta", "sta-base", 1, False, False,
     "template<class T> static T c1(T a){ T v=gs(a)+a; %s return v; }",
     "s=c1<int>(s);", "…spelled `static`"),
    ("tmpl-inline", "tmpl-ext", 1, True, False,
     "template<class T> inline T c1(T a){ T v=gs(a)+a; %s return v; }",
     "s=c1<int>(s);", "…spelled `inline`: does the term stack?"),
]


def axis_sweep(kind, mode, wd, nmax, kmax, scout=False):
    """One axis row, swept over 4-byte rungs. Returns the measured profile.

    The N sweep BREAKS at the first declined site: `/O1`'s decision is
    all-or-nothing per (caller, callee) over 2 904 objects (§6.15.1) and every
    row since has agreed, so the prefix is the whole answer. `nd` is still read
    per N, so a genuinely mixed row would print `MIXED` rather than be
    swallowed.
    """
    name, ctl, np_, inl, force, tmpl, site, note = kind
    o1 = "/O1" in mode
    rows, bad = [], 0
    for k in range(0, (1 if scout else kmax) + 1):
        leads = tmpl % stmts_fine(k)
        s, sc, ndir, mixed, ind, sref = None, None, 0, False, 0, None
        leaf = False
        for n in range(1, nmax + 1):
            src = src_of(GS, [leads],
                         "%s %s %s" % (INT_HEAD, " ".join([site] * n),
                                       INT_TAIL))
            o = capture(src, mode, wd, "ax_%s_%d_%d" % (name, k, n))
            r = None if o is None else read(o)
            if r is None or "error" in r:
                bad += 1
                break
            if n == 1:
                s = size_of(r["emit"], "c1")
                sc = (pressure_of(o, "c1") or {}).get("sc")
                leaf = callee_is_leaf(o, "c1")
            # A surviving `bl` and a surviving `bctrl` are both declines. The
            # reloc table sees only the first, and the whole point of the
            # `virtual` rows is that they take the second (§6.18).
            nd = surviving(o, r, n)
            ind = max(ind, r["nind"])
            if 0 < nd < n:
                mixed = True
            if nd:
                break
            ndir = n
        idx = sched_index(s, np_, inl, sc == 2, bool(leaf))
        if not o1 and s is not None:
            # §6.15.4 states the /Ox loop-free threshold on the callee's size
            # AS EMITTED AT /O1 — the /Ox-emitted size demonstrably does not
            # decide it — so the /Ox lane costs one extra reference capture
            # per rung. Without it this lane can only abstain.
            src = src_of(GS, [leads],
                         "%s %s %s" % (INT_HEAD, site, INT_TAIL))
            o = capture(src, "/O1 /GS- /c", wd, "axref_%s_%d" % (name, k))
            r = None if o is None else read(o)
            if r is not None and "error" not in r:
                sref = size_of(r["emit"], "c1")
        rows.append({"k": k, "s": s, "sc": sc, "idx": idx, "ndir": ndir,
                     "mixed": mixed, "ind": ind, "sref": sref,
                     "leaf": bool(leaf), "np": np_})
    return {"name": name, "ctl": ctl, "note": note, "force": force,
            "rows": rows, "bad": bad}


def axis_report(res, nmax, o1):
    """Print one axis row and return its summary dict."""
    name, rows = res["name"], res["rows"]
    disc = ref = disc_ref = 0
    lo = hi = None                # largest index inlined / smallest declined
    classes, prof = set(), []
    print("    %-18s %-3s %-6s %-6s %-6s %-9s %-10s %s"
          % ("kind", "k", "s", "index", "Ndir", "class", "predicted", ""))
    for r in rows:
        if r["s"] is None or r["sc"] is None:
            print("    %-18s %-3d %-6s %-6s %-6s %-9s %-10s %s"
                  % (name, r["k"], "-", "-", "-", "-", "-",
                     "not graded: NO CALLEE (did it compile?)"))
            continue
        cls = {2: "EXTERNAL", 3: "STATIC"}.get(r["sc"], "sc=%s" % r["sc"])
        classes.add(cls)
        prof.append(r["ndir"])
        # at /Ox the axis is the /O1-EMITTED size (§6.15.4), not the /O1 index
        key = r["idx"] if o1 else r["sref"]
        if key is not None and not r["ind"]:
            if r["ndir"]:
                lo = key if lo is None else max(lo, key)
            else:
                hi = key if hi is None else min(hi, key)
        if r["ind"]:
            # §6.18.4: the site is a `bctrl` through a vtable, so the callee's
            # identity is not known where the decision would be made. Grading
            # it against a rule about a NAMED callee would print a refutation
            # on every row of a class no rule here covers — the eighth
            # cry-wolf, avoided the way the other seven were, by an abstention
            # that is READ OUT OF THE OBJ.
            mark = ("not graded: INDIRECT CALL SITE (%d x `bctrl`) — there is"
                    " no callee to price (§6.18.4)" % r["ind"])
        elif not o1:
            if r["sref"] is None:
                mark = "not graded: NO /O1 REFERENCE SIZE"
            else:
                want, got = grade_ox(r["sref"], r["ndir"], nmax, r["np"],
                                     r["leaf"])
                base = r["sref"] - (OX_LEAF_BONUS if r["leaf"] else 0)
                d = not (OX_LOOPFREE[0] - 8 < base < OX_LOOPFREE[1] + 8)
                disc += 0 if d else 1
                if want == "?":
                    mark = "not graded: inside §6.15.4's own 108/112 gap"
                elif want == got:
                    mark = "%s OK%s" % (want,
                                        "   <== discriminating" if not d
                                        else "")
                else:
                    ref += 1
                    disc_ref += 0 if d else 1
                    mark = ("%s vs %s   <== *** REFUTES THE /Ox LOOP-FREE"
                            " THRESHOLD (s@/O1=%d) ***" % (want, got,
                                                           r["sref"]))
        elif cls.startswith("sc="):
            mark = "not graded: UNKNOWN CLASS"
        elif res["force"]:
            mark = "not graded: __forceinline"
        else:
            want, lbl = axis_predict(r["idx"], r["sc"], nmax)
            d = axis_discriminates(r["idx"], r["sc"], nmax)
            disc += 1 if d else 0
            if r["ndir"] == want:
                mark = "%s OK%s" % (lbl, "   <== discriminating" if d else "")
            else:
                ref += 1
                disc_ref += 1 if d else 0
                mark = ("%s vs %d   <== *** REFUTES THE %s RULE ***"
                        % (lbl, r["ndir"],
                           "EXTERNAL" if r["sc"] == 2 else "SCHEDULE D"))
        if r["mixed"]:
            mark += "   <== *** MIXED SITES ***"
        if "vararg" in name:
            # ANNOTATED, NOT SUPPRESSED, and read from the SPELLING because a
            # variadic signature leaves no signal in the obj. §6.18.5 measures
            # this class outside the size rule in BOTH linkage classes; the row
            # still prints its refutation so that a compiler which one day
            # inlined one would not be silently absorbed.
            mark += "   [VARIADIC — §6.18.5: outside the size rule]"
        print("    %-18s %-3d %-6d %-6s %-6d %-9s %-10s %s"
              % (name, r["k"], r["s"],
                 r["idx"] if o1 else ("@O1 %s" % r["sref"]), r["ndir"], cls,
                 axis_predict(r["idx"], r["sc"], nmax)[1] if o1 else "-",
                 mark))
    step = ("%s/%s" % (lo, hi)) if (lo is not None and hi is not None) \
        else "NO STEP LOCATED"
    if o1 and classes == {"STATIC"}:
        step = "n/a (SCHEDULE D)"
    print("    -> class %-9s step(%s) %-14s profile %s"
          % ("/".join(sorted(classes)) or "?", "index" if o1 else "s@/O1",
             step, "/".join(str(p) for p in prof)))
    print("       discriminating cells %-3d  refuting rows %-3d  (of which"
          " discriminating: %d)" % (disc, ref, disc_ref))
    if not disc:
        print("       NO DISCRIMINATING CELL — an 8-byte term could not have")
        print("       shown up in this row, so its agreement says NOTHING.")
    if res["note"]:
        print("       %s" % res["note"])
    print()
    return {"name": name, "ctl": res["ctl"], "step": step, "lo": lo, "hi": hi,
            "prof": prof, "disc": disc, "ref": ref, "disc_ref": disc_ref,
            "cls": "/".join(sorted(classes)),
            "byidx": {(r["idx"] if o1 else r["sref"]): r["ndir"]
                      for r in rows
                      if r["sc"] is not None
                      and (r["idx"] if o1 else r["sref"]) is not None}}


def run_axes(mode, wd, nmax, want=None, scout=False, kmax=None):
    """The five axes §6.17.11 named, each against a matched control."""
    o1 = "/O1" in mode
    print("=== axes   what every ladder in §6.15/§6.16/§6.17 held fixed")
    print("    Each rung is ONE instruction (4 bytes of `s`); N is swept to %d."
          % nmax)
    print("    `class` is READ FROM THE OBJ (COFF storage class), never from")
    print("    the spelling — §6.17.3's anonymous-namespace row is why.")
    print("    A rung is DISCRIMINATING iff its class's predictor differs at")
    print("    index+-8, the size of the only extra term ever found here.")
    print()
    bad, out = 0, {}
    for kind in AXIS_KINDS:
        if want and kind[0] not in want:
            continue
        res = axis_sweep(kind, mode, wd, nmax,
                         kmax or (AXIS_KMAX if o1 else AXIS_KMAX_OX), scout)
        bad += res["bad"]
        out[kind[0]] = axis_report(res, nmax, o1)
    print("=== axes summary — each probe against its matched control")
    print("    %-18s %-9s %-16s %-6s %-6s %s"
          % ("kind", "class", "step(index)", "disc", "refut", "vs control"))
    tot_d = tot_r = 0
    for nm, r in out.items():
        tot_d += r["disc"]
        tot_r += r["ref"]
        v = "-"
        c = out.get(r["ctl"])
        if c:
            # The model-free comparison: at every INDEX both rows reached, do
            # they take the same number of sites? Two kinds start at different
            # `s`, so the profiles are offset and only an index-matched compare
            # means anything. A disagreeing shared index is a term, full stop —
            # no schedule, no rule, no fitting.
            sh = sorted(set(r["byidx"]) & set(c["byidx"]))
            dis = [i for i in sh if r["byidx"][i] != c["byidx"][i]]
            if r["lo"] is not None and c["lo"] is not None \
                    and r["hi"] is not None and c["hi"] is not None:
                # Each row BRACKETS its step to (lo, hi]. A ladder whose rungs
                # are not 4 bytes apart — `long long` moves 8 at a time —
                # brackets it more loosely, and calling that a MOVE would be
                # the seventh cry-wolf's little brother. Two brackets are
                # compatible iff they intersect; only a disjoint pair is a
                # moved step.
                inter = max(r["lo"], c["lo"]) < min(r["hi"], c["hi"])
                if (r["lo"], r["hi"]) == (c["lo"], c["hi"]):
                    v = "SAME STEP as %s" % r["ctl"]
                elif inter:
                    v = ("step bracket (%d,%d] CONSISTENT with %s's (%d,%d]"
                         % (r["lo"], r["hi"], r["ctl"], c["lo"], c["hi"]))
                else:
                    v = ("<== *** STEP MOVED: (%d,%d] vs %s's (%d,%d] ***"
                         % (r["lo"], r["hi"], r["ctl"], c["lo"], c["hi"]))
            elif not sh:
                v = "not comparable: NO SHARED INDEX with %s" % r["ctl"]
            else:
                v = "-"
            v += ("   [%d shared index cells, %d disagree%s]"
                  % (len(sh), len(dis),
                     "" if not dis else ": " + ",".join(
                         "%d(%d vs %d)" % (i, r["byidx"][i], c["byidx"][i])
                         for i in dis[:4])))
        print("    %-18s %-9s %-16s %-6d %-6d %s"
              % (nm, r["cls"], r["step"], r["disc"], r["ref"], v))
    print()
    print("    TOTAL discriminating cells: %d   refuting rows: %d" %
          (tot_d, tot_r))
    if not tot_d:
        print("    NO DISCRIMINATING CELL ANYWHERE — this run says NOTHING")
        print("    about whether these axes carry a term.")
    return bad


# --------------------------------------------------------------------------
# CALLER — the other half of the pair, which has never been a variable
#
# The decision is a property of a (caller, callee) PAIR and across the whole of
# §6.15 to §6.18, without one exception, the caller is
#
#       int P(int a){ int s = gs(a) + a;  <N sites>  return s; }
#
# §6.15.3a measured that P's SIZE and P's EXISTING EXPANSION do not move the
# limit — by padding P's body with up to 40 statements of its own and by giving
# it a second, unrelated callee. Both are properties of P's BODY. What has never
# been touched is P's INTERFACE (linkage, parameter count, return type, `inline`,
# variadic, member-ness) and P's OWN LEAFNESS — and that last one matters
# because §6.18.6 measured the CALLEE's leafness at 48 bytes, six times the only
# other term this document has found.
#
# THE 2x2 IS THE POINT, not a list of caller spellings. With a NON-leaf callee a
# "leaf" P stops being one the instant it inlines, so only the leaf-callee
# column can produce a genuinely call-free caller; a probe that varied P's
# leafness against the framed callee alone would be measuring nothing and would
# PRINT THE SAME THING as one that was (§6.16.2).
#
# GRADED MODEL-FREE FIRST. The shipped model has no caller term at all, so its
# `REFUTES` column would fire on any shift whatsoever and says nothing about
# WHICH side moved. The load-bearing column is the index-matched comparison
# against the matched control: at every index both rows reached, do they take
# the same number of sites? A disagreeing shared index is a caller term, full
# stop. The `s` column is the confound control — the callee's COMDAT is emitted
# independently of the caller, so if `s` ever differs between a variant and its
# control at the same rung the comparison is not index-matched and the row says
# so instead of being read.
#
# THE RUNG RANGES ARE MODE-DEPENDENT, and that is not a detail. At `/O1` the
# interesting cells are SCHEDULE D's seven band boundaries, which the framed
# callee walks over index 48..104. At `/Ox` there is no schedule at all — one
# threshold, at 108/112 on the /O1-EMITTED size (§6.15.4), 152/156 for a leaf
# (§6.18.7a) — so the /O1 ranges never reach it and an /Ox run over them is
# VACUOUS: twelve of twelve at every rung, exactly what a passing run prints.
# The first /Ox capture of this round did precisely that and was thrown away.
CALLEE_KINDS = {
    # (name, template, nparams, inline?, kmin, kmax, kmin@/Ox, kmax@/Ox, note)
    "c-framed": ("c-framed",
                 "static int c1(int a){ int v=gs(a)+a; %s return v; }",
                 1, False, 0, 14, 8, 24,
                 "the callee EVERY ladder in §6.15-§6.17 uses: static,"
                 " non-`inline`, one parameter, CONTAINS A CALL"),
    "c-leaf": ("c-leaf",
               "static int c1(int a){ int v=a*3; %s return v; }",
               1, False, 22, 36, 30, 44,
               "§6.18.6's LEAF callee — indexed on s-48. The only column in"
               " which an inlining P can end up call-free."),
}

# (name, control, P's mangled prefix, extra decls, head, tail, sites(n), note)
CALLER_KINDS = [
    ("P-base", None, "P", "",
     "int P(int a){ int s=gs(a)+a;", "return s; }", None,
     "CONTROL: the caller of §6.15-§6.18, verbatim. External, 1 param,"
     " `int`, and NON-LEAF — its own body calls `gs`."),

    # ---- P1: the 2x2's caller axis -------------------------------------
    ("P-leaf", "P-base", "P", "",
     "int P(int a){ int s=a*3;", "return s; }", None,
     "THE PROBE: P's body contains NO call of its own. Against `c-leaf` an"
     " inlining P is call-free end to end; against `c-framed` it is not."),

    # ---- P2: P's declared interface ------------------------------------
    # `static` and `inline` callers need a reference or they may not be
    # emitted at all, so they take a keeper AND are graded against a matched
    # keeper control — comparing an address-taken P against a plain one would
    # be a confound and is registered against in the estimate.
    ("P-addr-ctl", "P-base", "P", "",
     "int P(int a){ int s=gs(a)+a;",
     "return s; }\nint (*pk)(int)=&P;", None,
     "CONTROL for the two rows that need a keeper: `P-base` with its address"
     " taken and nothing else"),
    ("P-static", "P-addr-ctl", "P", "",
     "static int P(int a){ int s=gs(a)+a;",
     "return s; }\nint (*pk)(int)=&P;", None,
     "the CALLER's linkage — the mirror of §6.17, which was the largest"
     " rescoping this document has had"),
    ("P-inline", "P-addr-ctl", "P", "",
     "inline int P(int a){ int s=gs(a)+a;",
     "return s; }\nint (*pk)(int)=&P;", None,
     "…and the mirror of §6.17.5's 8 bytes, on the caller"),
    ("P-2arg", "P-base", "P", "",
     "int P(int q,int a){ int s=gs(a)+a;", "return s+q; }", None,
     "the CALLER's parameter count — the mirror of §6.17.6"),
    ("P-void", "P-base", "P", "extern int gv;",
     "void P(int a){ int s=gs(a)+a;", "gv=s; }", None,
     "the CALLER's return type — §6.18.2 returned ten spellings of zero on"
     " the callee side"),
    ("P-ptrarg", "P-base", "P", "struct CP { int m; };\nextern CP* cpp;",
     "int P(int a){ int s=gs(a)+a;", "return s+cpp->m; }", None,
     "the MATCHED control for `P-member`: the same extra load, through a"
     " pointer instead of through `this`"),
    ("P-member", "P-ptrarg", "P@CP", "struct CP { int m; int P(int a); };",
     "int CP::P(int a){ int s=gs(a)+a;", "return s+m; }", None,
     "a MEMBER caller: `this` is live from entry. §6.17.2 needed exactly this"
     " control to show the callee's member-ness was never the variable."),
    ("P-vararg", "P-base", "P", "",
     "int P(int a, ...){ int s=gs(a)+a;", "return s; }", None,
     "a VARIADIC caller — §6.18.5's only categorical class, on the other"
     " side of the pair"),

    # ---- P3: the call SITE's form. No hypothesis; this is the control
    # tranche, and §6.18.11 records that all three of the last three
    # mechanisms were found by controls rather than by the probe.
    ("S-discard", "P-base", "P",
     "", "int P(int a){ int s=gs(a)+a;", "return s; }",
     lambda n: " ".join(["c1(s);"] * n) + " s^=a;",
     "the site's RESULT IS DISCARDED — the callee's return value is dead"),
    ("S-const", "P-base", "P", "",
     "int P(int a){ int s=gs(a)+a;", "return s; }",
     lambda n: " ".join("s+=c1(%d);" % (7 * (i + 1)) for i in range(n)),
     "a CONSTANT argument at every site, distinct per site so nothing can be"
     " CSE'd — the argument has never not been a live chained value"),
    ("S-if", "P-base", "P", "",
     "int P(int a){ int s=gs(a)+a;", "return s; }",
     lambda n: " ".join("if(a&%d){ s=c1(s); }" % (1 << i) for i in range(n)),
     "each site in its OWN basic block — every site so far is straight-line"),
    ("S-loop", "P-base", "P", "",
     "int P(int a){ int s=gs(a)+a;", "return s; }",
     lambda n: "for(int i=0;i<a;i++){ %s }"
               % " ".join(["s=c1(s);"] * n),
     "the sites inside a LOOP in the CALLER: N sites but not N executions"),
]


def _sites_default(n):
    return " ".join(["s=c1(s);"] * n)


def caller_sweep(ck, ce, mode, wd, nmax, krange=None):
    """One (caller variant, callee) sweep over the callee's 4-byte rungs."""
    name, _ctl, pfx, decls, head, tail, sites, _note = ck
    cname, ctmpl, cnp, cinl, kmin, kmax, kmin_ox, kmax_ox, _cnote = ce
    if "/O1" not in mode:
        kmin, kmax = kmin_ox, kmax_ox
    if krange:
        kmin, kmax = krange
    sites = sites or _sites_default
    if tail is None:                        # variadic caller: `...` needs a
        tail = "return s; }"                # tail of its own only if it moves
    rows, bad = [], 0
    for k in range(kmin, kmax + 1):
        leads = ctmpl % stmts_fine(k)
        s, sc, ndir, mixed, ind, leaf = None, None, 0, False, 0, False
        psc, pleaf = None, None
        for n in range(1, nmax + 1):
            probe = "%s %s %s" % (head, sites(n), tail)
            src = src_of(GS + (("\n" + decls) if decls else ""),
                         [leads], probe)
            o = capture(src, mode, wd,
                        "cl_%s_%s_%d_%d" % (name, cname, k, n))
            r = None if o is None else read(o, pfx)
            if r is None or "error" in r:
                bad += 1
                break
            if n == 1:
                s = size_of(r["emit"], "c1")
                sc = (pressure_of(o, "c1") or {}).get("sc")
                leaf = callee_is_leaf(o, "c1")
                # The CALLER's own class and leafness, read out of the obj and
                # never from the spelling — §6.17.3's anonymous-namespace row
                # is the standing reason. `pleaf` is measured at N=1, i.e.
                # AFTER whatever inlining happened, which is the only sense in
                # which "P is a leaf" is a fact about the emitted code.
                psc = (pressure_of(o, pfx) or {}).get("sc")
                pleaf = callee_is_leaf(o, pfx)
            nd = surviving(o, r, n)
            ind = max(ind, r["nind"])
            if 0 < nd < n:
                mixed = True
            if nd:
                break
            ndir = n
        rows.append({"k": k, "s": s, "sc": sc, "ndir": ndir, "mixed": mixed,
                     "ind": ind, "leaf": bool(leaf), "psc": psc,
                     "pleaf": pleaf,
                     "idx": sched_index(s, cnp, cinl, sc == 2, bool(leaf))})
    return {"name": name, "ctl": ck[1], "note": ck[7], "rows": rows,
            "bad": bad, "callee": cname}


def caller_report(res, nmax, o1):
    """Print one caller variant's sweep; return its summary."""
    name, rows = res["name"], res["rows"]
    disc = ref = disc_ref = 0
    prof, byidx, byk, sbyk, pcls = [], {}, {}, {}, set()
    print("    %-14s %-3s %-6s %-6s %-6s %-9s %-11s %-10s %s"
          % ("caller", "k", "s", "index", "Ndir", "callee", "CALLER",
             "predicted", ""))
    for r in rows:
        if r["s"] is None or r["sc"] is None:
            print("    %-14s %-3d %-6s %-6s %-6s %-9s %-11s %-10s %s"
                  % (name, r["k"], "-", "-", "-", "-", "-", "-",
                     "not graded: NO CALLEE or NO CALLER (did it compile?)"))
            continue
        pc = ("%s%s" % ({2: "EXT", 3: "STA"}.get(r["psc"], "sc=%s" % r["psc"]),
                        "/leaf" if r["pleaf"] else ""))
        pcls.add(pc)
        cls = {2: "EXTERNAL", 3: "STATIC"}.get(r["sc"], "sc=%s" % r["sc"])
        prof.append(r["ndir"])
        byidx[r["idx"]] = r["ndir"]
        byk[r["k"]] = r["ndir"]
        sbyk[r["k"]] = r["s"]
        if not o1:
            mark = "not graded: /Ox is a different mechanism"
        elif r["ind"]:
            mark = ("not graded: INDIRECT CALL SITE (%d x `bctrl`)" % r["ind"])
        else:
            want, lbl = axis_predict(r["idx"], r["sc"], nmax)
            d = axis_discriminates(r["idx"], r["sc"], nmax)
            disc += 1 if d else 0
            if r["ndir"] == want:
                mark = "%s OK%s" % (lbl, "   <== discriminating" if d else "")
            else:
                ref += 1
                disc_ref += 1 if d else 0
                mark = ("%s vs %d   <== *** the CALLEE-ONLY model misses this"
                        " cell ***" % (lbl, r["ndir"]))
        if r["mixed"]:
            mark += "   <== *** MIXED SITES ***"
        print("    %-14s %-3d %-6d %-6s %-6d %-9s %-11s %-10s %s"
              % (name, r["k"], r["s"], r["idx"], r["ndir"],
                 "%s%s" % (cls[:3], "/leaf" if r["leaf"] else ""), pc,
                 axis_predict(r["idx"], r["sc"], nmax)[1] if o1 else "-",
                 mark))
    if not o1:
        # /Ox has no schedule to differ at index+-8 (§6.15.4: one threshold, on
        # the /O1-emitted size), so the model cannot define the counter. What
        # CAN be defined, and measured out of this row alone: a rung is
        # discriminating iff the verdict differs two rungs away — the same
        # 8-byte window, expressed in rungs. A row whose profile never changes
        # never crossed the threshold and is VACUOUS, which is exactly what the
        # first /Ox capture of this round was.
        disc = sum(1 for i, v in enumerate(prof)
                   if (i >= 2 and prof[i - 2] != v)
                   or (i + 2 < len(prof) and prof[i + 2] != v))
    print("       CALLER as measured in the obj: %s"
          % ("/".join(sorted(pcls)) or "?"))
    print("       discriminating cells %-3d  cells the callee-only model"
          " misses %-3d  (of which discriminating: %d)"
          % (disc, ref, disc_ref))
    if not disc:
        print("       NO DISCRIMINATING CELL — a term could not have shown up")
        print("       in this row, so its agreement says NOTHING.")
    if res["note"]:
        print("       %s" % res["note"])
    print()
    return {"name": name, "ctl": res["ctl"], "prof": prof, "byidx": byidx,
            "byk": byk, "sbyk": sbyk, "disc": disc, "ref": ref,
            "disc_ref": disc_ref, "pcls": "/".join(sorted(pcls)) or "?"}


def run_caller(mode, wd, nmax, want=None, callees=None, krange=None):
    """Vary the CALLER. See the block comment above CALLEE_KINDS."""
    o1 = "/O1" in mode
    print("=== caller   the other half of the pair — never varied before")
    print("    Each rung moves the CALLEE by one instruction (4 bytes); the")
    print("    caller is held fixed within a block and varied between them.")
    print("    `index` is the CALLEE's, per §6.17/§6.18 — the shipped model")
    print("    has NO caller term, so a caller term shows up as a whole")
    print("    profile that disagrees with the control at a SHARED index.")
    print()
    bad, tot_d, tot_r = 0, 0, 0
    for cekey in (callees or ["c-framed", "c-leaf"]):
        ce = CALLEE_KINDS[cekey]
        print("=== callee %s   %s" % (ce[0], ce[-1]))
        print()
        out = {}
        for ck in CALLER_KINDS:
            if want and ck[0] not in want:
                continue
            if cekey == "c-leaf" and ck[0] not in ("P-base", "P-leaf"):
                continue        # the 2x2 only; the interface rows are §P2
            res = caller_sweep(ck, ce, mode, wd, nmax, krange)
            bad += res["bad"]
            out[ck[0]] = caller_report(res, nmax, o1)
        print("    --- summary, callee %s: each caller against its control"
              % ce[0])
        print("    %-14s %-11s %-6s %-6s %s"
              % ("caller", "CALLER", "disc", "miss",
                 "vs control (MODEL-FREE)"))
        for nm, r in out.items():
            tot_d += r["disc"]
            tot_r += r["ref"]
            c = out.get(r["ctl"])
            v = "-"
            if c:
                # confound control FIRST: the callee's COMDAT does not depend
                # on the caller, so a differing `s` at the same rung means the
                # two rows are not index-matched and nothing below is readable.
                sk = sorted(set(r["sbyk"]) & set(c["sbyk"]))
                sdiff = [k for k in sk if r["sbyk"][k] != c["sbyk"][k]]
                key = "byidx" if o1 else "byk"
                sh = sorted(set(r[key]) & set(c[key]))
                dis = [i for i in sh if r[key][i] != c[key][i]]
                if sdiff:
                    v = ("<== *** CONFOUND: the callee's own `s` differs from"
                         " %s at rung(s) %s — not index-matched ***"
                         % (r["ctl"], ",".join(str(k) for k in sdiff[:6])))
                elif not sh:
                    v = "not comparable: NO SHARED CELL with %s" % r["ctl"]
                elif not dis:
                    v = ("IDENTICAL to %s on all %d shared %s cells"
                         % (r["ctl"], len(sh), "index" if o1 else "rung"))
                else:
                    v = ("<== *** THE CALLER MOVES IT: %d of %d shared"
                         " cells disagree: %s ***"
                         % (len(dis), len(sh),
                            ",".join("%s%d(%d vs %d)"
                                     % ("" if o1 else "k=", i,
                                        r[key][i], c[key][i])
                                     for i in dis[:6])))
            print("    %-14s %-11s %-6d %-6d %s"
                  % (nm, r["pcls"], r["disc"], r["ref"], v))
        print()
    print("    TOTAL discriminating cells: %d   cells the callee-only model"
          " misses: %d" % (tot_d, tot_r))
    if not o1:
        print("    /Ox: the index means nothing here (§6.15.4 states its")
        print("    threshold on the /O1-emitted size), so the model columns")
        print("    abstain and the MODEL-FREE rung comparison carries the")
        print("    whole claim — which is what it was written to do.")
    elif not tot_d:
        print("    NO DISCRIMINATING CELL ANYWHERE — this run says NOTHING")
        print("    about whether the caller carries a term.")
    return bad


# --------------------------------------------------------------------------
# HELPER — does `bl __savegprlr_N` count as "a call"?
#
# §6.18.10 ships this as an open risk and names it precisely: `callee_is_leaf()`
# counts the allocator's save/restore helper because it IS a REL24, and no probe
# in this document has a callee whose only call is that helper. A 48-byte term —
# the largest in the index — therefore rests on an unmeasured answer.
#
# THE SHAPE. Liveness without a call needs an opaque barrier that is not a call.
# `volatile` is one: m ordered volatile READS, then the same m values written
# back to a volatile global in REVERSE order. Neither the reads nor the writes
# may be reordered or elided, and the reverse order means every value is live
# across the whole write sequence, so the allocator must take nonvolatiles —
# and past two of them this compiler switches to the out-of-line helper pair
# (§6.16.3, measured, six instructions cheaper).
#
# It is also the cheapest test yet available of §6.16.5a's (A) vs (B): the
# helper does not exist until after allocation and is not in the source at all.
# (A) — the decider measures a COMPILED callee — predicts the helper counts.
# (B) — the decider works from something upstream — predicts it does not.
#
# A SECOND untested branch of the same shipped predicate: `callee_is_leaf()`
# also counts a `bcctrl`, so a callee whose ONLY call is INDIRECT — through a
# function pointer — is charged the 48 bytes as well, and no probe tests that
# either. §6.18.4 measured what an indirect CALL SITE does to the decision (it
# removes it: there is no callee to price); an indirect call INSIDE the callee
# is a different question and is untouched.
HELPER_DECLS = "extern volatile int gvv;\nextern int (*fp)(int);"


def helper_body(m, k):
    """A callee with m simultaneously live values and NO call in its source.

    The rung chain starts from `t0`, a VOLATILE read, and not from `a`. The
    first spelling started `int v=a;` and the whole ladder folded — `s` took
    two values, 96 and 100, over twenty-one rungs — because `v^=a; v+=a;` on
    `v=a` collapses. A ladder that does not move is not a ladder, and it prints
    exactly what a moving one does; the `distinct s reached` counter exists so
    that failure is a printed line rather than a silently mis-read verdict.
    """
    rd = " ".join("int t%d=gvv;" % i for i in range(m))
    wr = " ".join("gvv=t%d;" % i for i in range(m - 1, -1, -1))
    return ("static int c1(int a){ %s %s int v=t0; %s return v; }"
            % (rd, wr, stmts_fine(k)))


def helper_ctl_body(m, k):
    """The same shape with ONE real call, so BOTH kinds of REL24 are present.

    Without this the `helper` row's verdict could be a fact about volatile
    bodies rather than about the helper: this row holds the volatile reads, the
    volatile writes, the liveness and the helper pair fixed and adds a call.
    """
    rd = " ".join("int t%d=gvv;" % i for i in range(m))
    wr = " ".join("gvv=t%d;" % i for i in range(m - 1, -1, -1))
    return ("static int c1(int a){ %s int u=gs(a); %s int v=t0^u; %s"
            " return v; }" % (rd, wr, stmts_fine(k)))


def indirect_body(_m, k):
    """A callee whose ONLY call is through a function pointer (`bctrl`)."""
    return ("static int c1(int a){ int v=fp(a)+a; %s return v; }"
            % stmts_fine(k))


def tailind_body(_m, k):
    """…and the same as a TAIL call through the pointer: a `bctr`, no frame."""
    return ("static int c1(int a){ int v=a*3; %s return fp(v); }"
            % stmts_fine(k))


def call_ctl_body(_m, k):
    """§6.15's own callee — the row where H3 is known to be the answer."""
    return ("static int c1(int a){ int v=gs(a)+a; %s return v; }"
            % stmts_fine(k))


def leafshape_ctl_body(_m, k):
    """§6.18.6's own leaf callee — the row where H4 is known to be it."""
    return "static int c1(int a){ int v=a*3; %s return v; }" % stmts_fine(k)




def callee_calls(o, want):
    """Every REL24 target inside one planted callee, by name."""
    for g in groups(o):
        if name_matches(g["name"], want):
            break
    else:
        return None
    sec, lo, hi = extent(o, g)
    out = []
    for va, symidx, ty in o.relocs(sec):
        if lo <= va < hi and ty == REL24:
            s = o.sym_by_index(symidx)
            out.append(s["name"] if s else "?")
    return out


HELPER_SHAPES = [
    # (name, body(m,k), m, note)
    ("helper", helper_body, 10,
     "THE PROBE: the callee's ONLY REL24 is `bl __savegprlr_N`, a call the"
     " SOURCE does not contain and that does not exist until after register"
     " allocation"),
    ("helper-ctl", helper_ctl_body, 10,
     "CONTROL: the same volatile shape with ONE real call added, so the helper"
     " AND a source-level call are both present. Without this row, `helper`"
     " landing on the leaf schedule could be a fact about volatile bodies."),
    ("indirect", indirect_body, 0,
     "the second untested branch of the same predicate: the callee's only"
     " call is INDIRECT (`bctrl`), which leaves no REL24 at all"),
    ("tailind", tailind_body, 0,
     "…and the same indirect call in TAIL position: `bctr`, no frame, no LR"
     " save (§6.18.7's tail-call shape, through a pointer)"),
    ("call-ctl", call_ctl_body, 0,
     "CONTROL: §6.15's own callee, so H3 is graded on a row where it is known"
     " to be the right answer"),
    ("leaf-ctl", leafshape_ctl_body, 0,
     "CONTROL: §6.18.6's own leaf callee, so H4 is graded on a row where IT is"
     " known to be the right answer"),
]


def run_helper(mode, wd, nmax, want=None):
    """Which REL24s and which `bcctrl`s does the 48-byte term actually count?

    `callee_is_leaf()` counts BOTH the allocator's save/restore helper and an
    indirect `bcctrl`, and §6.18.10 records that no probe tests either. Both are
    graded here against the two rival readings, on rungs where they DIFFER, with
    a control at each end so the grader is known to be able to print either
    answer.
    """
    o1 = "/O1" in mode
    print("=== helper   what counts as 'a call' for §6.18.6's 48 bytes?")
    print("    H3 'it counts'      -> SCHEDULE D on raw `s`      (the SHIPPED")
    print("                            rule: `callee_is_leaf()` says non-leaf)")
    print("    H4 'it does not'    -> SCHEDULE D on `s`-48")
    print("    A cell counts only where the two DIFFER; the rest are printed")
    print("    and not counted, per §6.16.2.")
    print()
    bad = 0
    print("    --- scout: what is actually in each callee, read out of the obj")
    print("    %-12s %-4s %-6s %-7s %-7s %-6s %s"
          % ("shape", "m", "s", "nsave", "is_leaf", "bctrl/any",
             "REL24 targets"))
    live = []
    for name, fn, m, note in HELPER_SHAPES:
        if want and name not in want:
            continue
        src = src_of(GS + "\n" + HELPER_DECLS, [fn(m, 0)],
                     "%s %s %s" % (INT_HEAD, "s=c1(s);", INT_TAIL))
        o = capture(src, mode, wd, "hlp_scout_%s" % name)
        r = None if o is None else read(o)
        if r is None or "error" in r:
            bad += 1
            print("    %-12s capture failed" % name)
            continue
        p = pressure_of(o, "c1") or {}
        calls = callee_calls(o, "c1") or []
        nb = bcctr_count(o, "c1")
        print("    %-12s %-4d %-6s %-7s %-7s %-6s %s"
              % (name, m, size_of(r["emit"], "c1"), p.get("nsave"),
                 callee_is_leaf(o, "c1"),
                 "%d/%d" % (bcctr_count(o, "c1", True), nb),
                 ",".join(calls) or "(none)"))
        live.append((name, fn, m, note))
    print()
    if not o1:
        print("    not graded: /Ox is a different mechanism (§6.15.4).")
        return bad
    tot_d = 0
    for name, fn, m, note in live:
        print("    --- %s   %s" % (name, note))
        print("    %-12s %-4s %-6s %-6s %-8s %-9s %-14s %s"
              % ("shape", "k", "s", "Ndir", "H3(s)", "H4(s-48)",
                 "in the callee", ""))
        nd3 = nd4 = disc = neither = 0
        seen, ungraded = set(), 0
        for k in range(0, 22):
            leads = fn(m, k)
            s, ndir, calls, nb = None, 0, [], 0
            for n in range(1, nmax + 1):
                src = src_of(GS + "\n" + HELPER_DECLS, [leads],
                             "%s %s %s" % (INT_HEAD,
                                           " ".join(["s=c1(s);"] * n),
                                           INT_TAIL))
                o = capture(src, mode, wd, "hlp_%s_%d_%d" % (name, k, n))
                r = None if o is None else read(o)
                if r is None or "error" in r:
                    bad += 1
                    break
                if n == 1:
                    s = size_of(r["emit"], "c1")
                    # PER RUNG, not once. The first spelling of this probe read
                    # the callee's calls from a single scout capture and the
                    # ladder MOVED THE CONDITION under it: `a` is dead at k=0
                    # and live from k=1, which is the difference between ten
                    # live values (all volatile registers, no helper) and
                    # eleven (a helper pair). A row graded on a condition it
                    # does not have is the same fault as §6.16.10a's inert
                    # cells, and it prints the same thing.
                    calls = callee_calls(o, "c1") or []
                    nb = bcctr_count(o, "c1")
                if surviving(o, r, n):
                    break
                ndir = n
            if s is None:
                continue
            has = {("HELPER" if "gprlr" in c or "fpr" in c else "CALL")
                   for c in calls}
            if nb:
                has.add("BCTRL")
            has = ",".join(sorted(has)) or "nothing"
            p3 = law_d(s)
            p4 = law_d(s - LEAF_BONUS)
            p3 = nmax if p3 is None else min(p3, nmax)
            p4 = nmax if p4 is None else min(p4, nmax)
            d = p3 != p4
            tag = ""
            if d and has == "nothing" and name in ("helper", "helper-ctl"):
                # The rung does not have the condition this shape is about, so
                # it is NOT evidence about it — measured, printed, not counted.
                ungraded += 1
                tag = "not graded: NO HELPER IN THIS RUNG (an ordinary leaf)"
                d = False
            if d:
                disc += 1
                if ndir == p3:
                    nd3 += 1
                    tag = "<== discriminating: H3 — it IS a call (shipped OK)"
                elif ndir == p4:
                    nd4 += 1
                    tag = ("<== discriminating: H4 — NOT a call"
                           " (*** the shipped rule is WRONG ***)")
                else:
                    neither += 1
                    tag = "<== discriminating: *** NEITHER — a third offset ***"
            seen.add(s)
            print("    %-12s %-4d %-6d %-6d %-8d %-9d %-14s %s"
                  % (name, k, s, ndir, p3, p4, has, tag))
        tot_d += disc
        print("    -> distinct `s` reached: %d over 22 rungs%s"
              % (len(seen),
                 "" if len(seen) > 4 else
                 "   <== *** THE LADDER IS NOT MOVING — rungs are folding ***"))
        print("    -> discriminating cells %d   H3 %d   H4 %d   neither %d"
              "   ungraded (condition absent) %d"
              % (disc, nd3, nd4, neither, ungraded))
        if not disc:
            print("    -> NO DISCRIMINATING CELL — the two readings agree on")
            print("       every cell reached. VACUOUS, not a pass.")
        print()
    print("    TOTAL discriminating cells: %d" % tot_d)
    return bad


def run_lawd(mode, wd, nmax):
    """Grade LAW Dc, and take its ONE real hold-out — the >=260 ceiling.

    Rungs are `k` calls (12 bytes each) plus `j` one-instruction statements (4
    bytes each), so any 4-byte target in 48..300 is reachable. `sta-inline`
    carries the whole schedule 8 bytes lower (§6.17.5), which puts a callee of
    268 EMITTED bytes at index 260 — the ceiling clamp's cell, and one no
    version of this table has measured.
    """
    print("=== lawd   the clamped form against the measured table, and the")
    print("    ceiling clamp against cells the table never reached.")
    miss, fits = law_dc_selfcheck()
    print("    self-check, 4-byte cells 4..396 where LAW Dc disagrees with")
    print("    the MEASURED LAW_D_TABLE: %s"
          % ("none" if not miss else miss))
    print("    (budget, cap) pairs the measured table admits, searched over")
    print("    1..59 x 1..39: %s" % (fits,))
    print()
    print("    %-12s %-6s %-6s %-6s %-6s %-8s %s"
          % ("kind", "s", "index", "Ndir", "LAW Dc", "uncapped", "verdict"))
    bad = wrong = held = 0
    targets = list(range(48, 116, 4)) + [140, 144, 148,
                                         248, 252, 256, 260, 264, 268, 272]
    for kind, inl, tmpl in (
            ("sta-plain", False, "static int c1(int a){ int v=gs(a)+a;"
             " %s return v; }"),
            ("sta-inline", True, "static inline int c1(int a){ int v=gs(a)+a;"
             " %s return v; }")):
        seen = set()
        for t in targets:
            d = max(0, t - 48)
            body = stmts_call(d // 12) + " " + stmts_fine((d % 12) // 4)
            leads = tmpl % body
            s, sc, ndir = None, None, 0
            for n in range(1, nmax + 1):
                src = src_of(GS, [leads], "%s %s %s"
                             % (INT_HEAD, " ".join(["s=c1(s);"] * n),
                                INT_TAIL))
                o = capture(src, mode, wd, "lawd_%s_%d_%d" % (kind, t, n))
                r = None if o is None else read(o)
                if r is None or "error" in r:
                    bad += 1
                    break
                if n == 1:
                    s = size_of(r["emit"], "c1")
                    sc = (pressure_of(o, "c1") or {}).get("sc")
                if surviving(o, r, n):
                    break
                ndir = n
            if s is None or sc is None:
                continue
            idx = sched_index(s, 1, inl, sc == 2)
            if (kind, idx) in seen:
                continue
            seen.add((kind, idx))
            want = law_dc(idx)
            unc = None if idx <= 64 else 1 + 79 // (idx - 64)
            got = nmax if want is None else min(want, nmax)
            v = "LAW Dc %s OK" % ("unbounded" if want is None else want)
            if got != ndir:
                v = ("LAW Dc %s vs %d   <== *** REFUTES LAW Dc ***"
                     % (want, ndir))
                wrong += 1
            elif want is not None and unc is not None \
                    and min(unc, nmax) != got:
                v += "   [uncapped LAW D said %d — the clamp is what moved]" \
                     % min(unc, nmax)
            # the cells no earlier run reached: the ceiling clamp, seen
            # through the `inline` shift
            if inl and idx >= 248:
                held += 1
                v += "   <== HELD OUT (index %d reached only via `inline`)" \
                     % idx
            print("    %-12s %-6d %-6d %-6d %-6s %-8s %s"
                  % (kind, s, idx, ndir,
                     "unbnd" if want is None else want,
                     "-" if unc is None else unc, v))
        print()
    print("    rows refuting LAW Dc: %d   held-out ceiling cells graded: %d"
          % (wrong, held))
    return bad


def run_thisctl(mode, wd, nmax):
    """Is `this` an ordinary first parameter to the BACK END?

    The port numbers `this` LAST among a function's parameters even though it
    is `params[0]` (measured: reverting the order costs 272 mismatches). That
    is a statement about the label/parameter numbering the IL carries, and it
    says nothing about what the register allocator does with the pointer. This
    compiles the SAME body twice — once as `MB::mf1(int)` and once as the free
    `pf1(MB*,int)` — and compares the callee's own bytes.

    Equal bytes = the back end treats `this` as parameter 0 and nothing else,
    and the member class is then a new callee SHAPE rather than a new
    allocation regime. A difference is the more interesting outcome and would
    be the first place this document has seen `this` cost anything of its own.
    """
    print("=== thisctl   MB::mf1(int) vs pf1(MB*,int) — the same body twice")
    print("    %-3s %-4s %-6s %-6s %-14s %-14s %s"
          % ("k", "sp", "s_mem", "s_ptr", "press_mem", "press_ptr", "verdict"))
    bad = 0
    for k in range(0, 7):
        for sp, gen in (("lo", stmts_clive_lo), ("hi", stmts_clive_hi)):
            row = {}
            for nm, tree in (("mem", tree_d1_member), ("ptr", tree_d1_thisparam)):
                src, watch = ladder_source(tree, gen, k, 1)
                o = capture(src, mode, wd, "thisctl_%s_%s_%d" % (nm, sp, k))
                if o is None:
                    bad += 1
                    row[nm] = None
                    continue
                r = read(o)
                row[nm] = (size_of(r["emit"], watch[0]),
                           pressure_of(o, watch[0]),
                           callee_bytes(o, watch[0]))
            if row.get("mem") is None or row.get("ptr") is None:
                continue
            (sm, pm, bm), (sp_, pp, bp) = row["mem"], row["ptr"]
            if bm == bp:
                v = "BYTE-IDENTICAL — `this` is an ordinary parameter 0"
            elif sm == sp_:
                v = "same size, DIFFERENT BYTES"
            else:
                v = ("<== *** `this` COSTS %+d BYTES over an explicit MB* ***"
                     % (sm - sp_))
            print("    %-3d %-4s %-6s %-6s %-14s %-14s %s"
                  % (k, sp, sm, sp_, pcol(pm), pcol(pp), v))
    return bad


def run_order(mode, wd, nmax):
    """Does the callee's DEFINITION ORDER move the schedule?

    The pressure pair (§6.16) shows the decision tracking the __savegprlr_
    idiom threshold — a quantity that does not exist until after register
    allocation. Whatever makes the decision therefore either IS the back end
    or is reading a number the back end produced, which is only possible if
    the callee has already been compiled. This asks the cheapest consequence
    of that: put the DEFINITION after the caller and see whether anything
    moves.

    `s` itself is printed for both orders as the control — if the callee
    compiles differently, the comparison is confounded and the row says so.
    """
    print("=== order   callee defined BEFORE vs AFTER the caller")
    print("    both are the same TU with the same text; only the position of")
    print("    the DEFINITION moves (a forward declaration stands in).")
    print("    %-3s %-7s %-7s %-6s %-6s %s"
          % ("k", "s_before", "s_after", "N_bef", "N_aft", "verdict"))
    bad = 0
    for k in (1, 2, 3, 5, 8):
        body = stmts_live_lo(k)
        defn = "static int in1(int a){ int v=gs(a)+a; %s return v; }" % body
        got = {}
        for which in ("before", "after"):
            leads = defn if which == "before" else "static int in1(int);"
            tail = "" if which == "before" else "\n" + defn
            ndir, s = 0, None
            for n in range(1, nmax + 1):
                probe = ("%s %s %s%s"
                         % (INT_HEAD, " ".join(["s=in1(s);"] * n), INT_TAIL,
                            tail))
                o = capture(src_of(GS, [leads], probe), mode, wd,
                            "ord_%s_%d_%d" % (which, k, n))
                r = None if o is None else read(o)
                if r is None or "error" in r:
                    bad += 1
                    continue
                if n == 1:
                    s = size_of(r["emit"], "in1")
                d = declined(r["rel"], ["in1"])
                if sum(d.values()) == 0 and ndir == n - 1:
                    ndir = n
            got[which] = (ndir, s)
        (nb, sb), (na, sa) = got["before"], got["after"]
        if sb != sa:
            v = "CONFOUNDED: the callee itself compiled differently"
        elif nb == na:
            pr = law_d(sb)
            v = ("order does not move it (sched D %s%s)"
                 % ("unbounded" if pr is None else pr,
                    ", capped at N=%d here" % nmax
                    if pr is None or pr > nmax else ""))
        else:
            v = "<== *** DEFINITION ORDER MOVES THE SCHEDULE ***"
        print("    %-3d %-7s %-7s %-6d %-6d %s" % (k, sb, sa, nb, na, v))
    return bad


def run_ends(mode, wd):
    """The two round boundaries, with allocation pressure in play."""
    print("=== ends   the two ROUND boundaries, under allocation pressure")
    print("    press = savedGPRs/frame/postpush-stores+loads. nsave=18 means")
    print("    every nonvolatile GPR is gone; postpush stores after that are")
    print("    SPILLS (every callee here is int f(int), so no store to r1 is")
    print("    an outgoing argument).")
    print()
    bad = 0
    for name, nmax, prereg, leads, note in END_BODIES:
        tree = lambda _e, _l=leads: (_l, "s=in1(s);", ["in1"])
        nfull, ndir, s, p, trace, b = sweep_rung(
            tree, lambda _k: "", 0, mode, wd, nmax, "end_" + name)
        bad += b
        pred = law_d(s)
        sched = "unbounded" if pred is None else str(pred)
        sched_ok = ((ndir >= nmax) if pred is None
                    else (ndir == min(pred, nmax)))
        spill = ("SPILLING (%d stores past 18 nonvolatiles)" % p["st"]
                 if p and p["nsave"] >= 18 and p["st"] else
                 ("%d postpush stores" % p["st"] if p and p["st"] else
                  "no spill"))
        print("    %-10s s=%-5s press=%-14s %s" % (name, s, pcol(p), spill))
        print("    %-10s Ndir=%-3d over N=1..%-3d trace %-26s"
              " sched D says %-9s %s"
              % ("", ndir, nmax, trace, sched,
                 "OK" if sched_ok else "<== *** REFUTES SCHEDULE D ***"))
        print("    %-10s pre-registered %-10s %s" % (
            "", prereg,
            "landed" if prereg == (sched if sched_ok else "?") or
            (prereg == "unbounded" and ndir >= nmax) or
            (prereg == str(ndir)) else "MISSED (recorded, not an alarm)"))
        print("    %-10s %s" % ("", note))
        print()

    print("    the SPILL FLOOR — how small can a callee that spills be?")
    print("    If the smallest spilling body is already past the 260-byte")
    print("    ceiling, then at /O1 a spilling callee is ARITHMETICALLY")
    print("    confined to the never-inlined row and the ceiling is the only")
    print("    place spilling can be tested at all.")
    print("    %-4s %-6s %-6s %-6s %-5s %-5s %-10s %s"
          % ("live", "s", "nsave", "frame", "st", "ld", "spill?", "sched D"))
    for k in range(12, 25):
        tree = lambda _e, _l=live_body(k): (_l, "s=in1(s);", ["in1"])
        src, watch = ladder_source(tree, lambda _k: "", 0, 1)
        o = capture(src, mode, wd, "spillfloor_%d" % k)
        if o is None:
            bad += 1
            continue
        r = read(o)
        if "error" in r:
            bad += 1
            continue
        s = size_of(r["emit"], "in1")
        p = pressure_of(o, "in1")
        pred = law_d(s)
        print("    %-4d %-6s %-6d %-6d %-5d %-5d %-10s %s"
              % (k, s, p["nsave"], p["frame"], p["st"], p["ld"],
                 "SPILL" if (p["nsave"] >= 18 and p["st"]) else "-",
                 "unbounded" if pred is None else pred))
    return bad


def run_ladder(name, mode, wd, nmax):
    tree, gen, kmax, note = LADDERS[name]
    o1 = "/O1" in mode
    print("=== %-18s %s" % (name, note))
    print("    %-3s %-5s %-5s %-6s %-6s %-6s  %-24s %s"
          % ("k", "Nfull", "Ndir", "s(dir)", "s(in)", "P@N=1",
             "declined per N (1..%d)" % nmax, "SCHED D"))
    bad, out, refuted = 0, [], 0
    for k in range(kmax + 1):
        per_n, nfull, ndir, ptext = [], 0, 0, None
        dsz = isz = dsc = None
        watch = tree(gen(k))[2]
        for n in range(1, nmax + 1):
            src, watch = ladder_source(tree, gen, k, n)
            r = sweep_cell(src, watch, mode, wd,
                           "%s_k%d_n%d" % (name.replace("-", "_"), k, n))
            if r is None:
                per_n.append("!")
                bad += 1
                continue
            if n == 1:
                ptext = r["tsize"]
                dsz = size_of(r["emit"], watch[0])
                isz = size_of(r["emit"], watch[-1])
                dsc = r.get("sc")
            d = r["declined"]
            # SCHEDULE D is a claim about ONE (caller, callee) pair. When a
            # DEEPER instance is refused — a constructor inside an inlined
            # wrapper, say — that is a different pair and grading it here
            # would cry wolf: the `ctor-*` ladders inline their wrapper at
            # every one of 12 sites and have the ctor declined at all of them.
            nd = sum(c for w, c in d.items() if w == watch[0])
            if nd == 0 and ndir == n - 1:
                ndir = n
            tot = sum(d.values())
            per_n.append("." if tot == 0
                         else "".join("%s%d" % (w[0], c) for w, c in
                                      sorted(d.items())))
            if tot == 0 and nfull == n - 1:
                nfull = n
        v = grade_d(ndir, dsz, nmax, o1, dsc)
        if nfull != ndir:
            v += "   INNER-DECLINED (a different pair)"
        if "REFUTES" in v:
            refuted += 1
        print("    %-3d %-5d %-5d %-6s %-6s %-6s  %-24s %s"
              % (k, nfull, ndir, dsz, isz, ptext, " ".join(per_n), v))
        out.append((k, nfull, ndir, dsz, isz, ptext))
    print()
    return bad, refuted


# --------------------------------------------------------------------------
# the existing gt_label_inline families, read with the new detector
# --------------------------------------------------------------------------
def direct_callee(site):
    """The identifier P actually calls at its site — `s=lsb(s);` -> `lsb`."""
    out, cur = [], ""
    for ch in site:
        if ch.isalnum() or ch == "_":
            cur += ch
        else:
            if ch == "(" and cur and not cur.isdigit():
                out.append(cur)
            cur = ""
    # the barrier `gs` is present in both variants and is never the subject
    return [c for c in out if c not in ("gs", "gt", "gu", "ga")]


def run_family(fam, mode, wd, nmax):
    """One gt_label_inline family, read with the relocation detector.

    Grading these against SCHEDULE D is a HELD-OUT test: the schedule was
    fitted entirely on `int` ladders, and these are the destructor,
    constructor and depth-3 shapes §6.9-§6.14 were built on.
    """
    print("=== %-18s %s" % (fam.name, fam.note[:66]))
    print("    %-3s %-7s %-7s %-7s  %-26s %s"
          % ("N", "P.text", "dtext", "hand-dt", "declined (reloc)", "verdicts"))
    bad = 0
    want = direct_callee(fam.site)
    prev = prevh = None
    ndir, dsz = 0, None
    for n in range(0, nmax + 1):
        o = capture(fam.source(n, "inl"), mode, wd,
                    "f_%s_i_%d" % (fam.name.replace("-", "_"), n))
        oh = capture(fam.source(n, "hand"), mode, wd,
                     "f_%s_h_%d" % (fam.name.replace("-", "_"), n))
        if o is None or oh is None:
            print("    %-3d capture failed" % n)
            bad += 1
            continue
        r, rh = read(o), read(oh)
        if "error" in r or "error" in rh:
            bad += 1
            continue
        # everything P relocates against that is NOT the opaque barrier or a
        # runtime helper is a call the front end did not inline
        keep = {nm: c for nm, c in r["rel"].items()
                if not nm.startswith(("gs", "gt", "gu", "ga", "__",
                                      "_fltused", "$", ".", "?a0@", "?a1@",
                                      "?a2@"))
                and demangle_ish(nm) not in ("gs", "gt", "gu", "ga")}
        if n == 1 and want:
            dsz = size_of(r["emit"], want[0])
        if n >= 1:
            nd = sum(c for nm, c in keep.items()
                     if want and (demangle_ish(nm) == want[0]
                                  or want[0] in nm))
            if nd == 0 and ndir == n - 1:
                ndir = n
        dt = None if prev is None else r["tsize"] - prev
        hdt = None if prevh is None else rh["tsize"] - prevh
        old = ""
        if n >= 1 and dt is not None:
            if dt <= 0 or (hdt and hdt > 0 and dt * 2 <= hdt):
                old = "OLD:DECLINED?"
        new_ = "NEW:declined" if keep else "NEW:all-inlined"
        agree = "" if (bool(keep) == bool(old)) else "   <== DETECTORS DISAGREE"
        print("    %-3d %-7d %-7s %-7s  %-26s %s %s%s"
              % (n, r["tsize"], dt, hdt,
                 ",".join("%s*%d" % (demangle_ish(k)[:14], v)
                          for k, v in sorted(keep.items())) or "-",
                 old or "OLD:ok", new_, agree))
        prev, prevh = r["tsize"], rh["tsize"]
    print("    -> direct callee %s  s=%s   Ndirect=%d   %s"
          % (want[0] if want else "?", dsz, ndir,
             grade_d(ndir, dsz, nmax, "/O1" in mode)))
    print()
    return bad


def main(argv):
    if "--list" in argv:
        for k, (_, _, kmax, note) in sorted(LADDERS.items()):
            print("%-20s k=0..%-3d %s" % (k, kmax, note))
        return 0
    mode, nmax, kcap, kset = "/O1 /GS- /c", 6, 14, False
    fams, pairs = [], []
    if "--mode" in argv:
        i = argv.index("--mode"); mode = argv[i + 1]; del argv[i:i + 2]
    if "--max" in argv:
        i = argv.index("--max"); nmax = int(argv[i + 1]); del argv[i:i + 2]
    if "--kmax" in argv:
        i = argv.index("--kmax"); kcap = int(argv[i + 1]); del argv[i:i + 2]
        kset = True
        for nm in list(LADDERS):
            t, g, km, nt = LADDERS[nm]
            LADDERS[nm] = (t, g, min(km, kcap), nt)
    kmin = None
    if "--kmin" in argv:
        i = argv.index("--kmin"); kmin = int(argv[i + 1]); del argv[i:i + 2]
    callees = []
    while "--callee" in argv:
        i = argv.index("--callee")
        callees.append(argv[i + 1]); del argv[i:i + 2]
    while "--pair" in argv:
        i = argv.index("--pair"); pairs.append(argv[i + 1]); del argv[i:i + 2]
    while "--family" in argv:
        i = argv.index("--family"); fams.append(argv[i + 1]); del argv[i:i + 2]
    want = [a for a in argv[1:] if not a.startswith("--")]

    print("mode: %s   N = call sites of the SAME body, chained" % mode)
    print("  Nfull  = the largest N at which EVERY site was inlined.")
    print("           0 means the front end declined from the FIRST site.")
    print("  the per-N column is the number of `bl`s P kept against each")
    print("  planted callee: `.` = none, i.e. the whole tree was inlined.")
    print("  in.text/ou.text = the callee COMDATs' own sizes (§6.5: they are")
    print("           emitted whether or not they were inlined), a free")
    print("           code-size proxy for the front end's own cost estimate.")
    print()
    wd = tempfile.mkdtemp(prefix="gtdec")
    bad = 0
    if "--pressure" in argv:
        bad += run_pressure(mode, wd, nmax, kcap, pairs)
        print("captures failed: %d" % bad)
        return 1 if bad else 0
    if "--linkage" in argv:
        bad += run_linkage(mode, wd, nmax,
                           (kmin if kmin is not None else 0, kcap)
                           if (kmin is not None or kset) else None,
                           "--deadpad" in argv, want)
        print("captures failed: %d" % bad)
        return 1 if bad else 0
    if "--axes" in argv:
        bad += run_axes(mode, wd, nmax, want, "--scout" in argv,
                        kcap if kset else None)
        print("captures failed: %d" % bad)
        return 1 if bad else 0
    if "--caller" in argv:
        kr = None
        if kmin is not None:
            kr = (kmin, kcap)
        bad += run_caller(mode, wd, nmax, want, callees or None, kr)
        print("captures failed: %d" % bad)
        return 1 if bad else 0
    if "--helper" in argv:
        bad += run_helper(mode, wd, nmax, want)
        print("captures failed: %d" % bad)
        return 1 if bad else 0
    if "--lawd" in argv:
        bad += run_lawd(mode, wd, nmax)
        print("captures failed: %d" % bad)
        return 1 if bad else 0
    if "--thisctl" in argv:
        bad += run_thisctl(mode, wd, nmax)
        print("captures failed: %d" % bad)
        return 1 if bad else 0
    if "--order" in argv:
        bad += run_order(mode, wd, nmax)
        print("captures failed: %d" % bad)
        return 1 if bad else 0
    if "--ends" in argv:
        bad += run_ends(mode, wd)
        print("captures failed: %d" % bad)
        return 1 if bad else 0
    if "--padp" in argv:
        bad += run_padp(mode, wd)
        print("captures failed: %d" % bad)
        return 1 if bad else 0
    if "--sibling" in argv:
        bad += run_sibling(mode, wd)
        print("captures failed: %d" % bad)
        return 1 if bad else 0
    if "--cases" in argv:
        bad += run_cases(mode, wd, nmax, want)
        print("captures failed: %d" % bad)
        return 1 if bad else 0
    if fams:
        byname = {f.name: f for f in FAMILIES}
        for f in fams:
            if f not in byname:
                print("no such family: %s" % f)
                continue
            bad += run_family(byname[f], mode, wd, nmax)
    else:
        for k in sorted(LADDERS):
            if want and k not in want:
                continue
            b, _ = run_ladder(k, mode, wd, nmax)
            bad += b
    print("captures failed: %d" % bad)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
