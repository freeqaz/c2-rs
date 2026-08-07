# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the loader; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter, and the loader fails if a fragment emits
# zero cases.
#
# ---- W-GEN2: the store-run/call family, crossed against LIVENESS -------------
#
# **Why this file exists, and it is not the reason `88-store-run-call.py` exists.**
#
# `88` was built by `w-gen` on 2026-08-08 *before* the widening it was meant to
# protect, exactly so board **#232** could not happen again in this family: 1,576
# generated cases, graded, landed ahead of the reader. That same evening lane
# `w-seam2` widened into the family and shipped an emitter that produced **two
# `Port=Mismatch` verdicts** — and `88` reported `checked=1576 mismatches=0`
# through both of them. The cell that caught the defect was a hand-written
# fixture.
#
# Board **#1174** is that result. It is a third failure mode beside *"no
# instrument"* (#232) and *"instrument removed"* (#871/#876): **the instrument
# was present, exhaustive, and looking at the wrong axis.**
#
# ---- THE AXIS `88` DOES NOT VARY --------------------------------------------
#
# **Liveness of a stored value across the call.** Measured by `w-seam2`
# (rung §4.2, `work/w-seam2/grid3/`, `grid4/`):
#
# ```text
#   void P::lf(unsigned a, unsigned b) { m0=0; m1=b; m2=a; }        LEAF
#       li 11,0 ; stw 5,4(3) ; stw 4,8(3) ; stw 11,0(3) ; blr
#   P::P(unsigned a, unsigned b) { m0=0; m1=b; m2=a; Alloc(a); }    FRAMED
#       li 11,0 ; stw 4,8(3) ; stw 5,4(3) ; mr 31,3 ; stw 11,0(3) ; bl
#   P::P(unsigned a, unsigned b) { m0=0; m1=b; m2=a; Reset(); }     NULLARY
#       li 11,0 ; stw 5,4(3) ; stw 4,8(3) ; mr 31,3 ; stw 11,0(3) ; bl
# ```
#
# The two unproduced stores **SWAP**, and only when the call passes `a`. So board
# #866 — *"the leaf schedule transfers unchanged into a framed body"*, true over
# 96 cells in `w-seam` and 34 more in `w-seam2`'s frozen GRID S — is **refuted in
# its general form**. It holds only while no store reads a value the call keeps
# alive.
#
# **Why `88` cannot produce that shape, stated mechanically rather than by
# apology.** Two independent reasons, and closing either one alone leaves the
# hole open:
#
#   1. `88`'s runs store `mSize = size` (the LAST formal), `mFreeHead = this`,
#      literals and interior addresses. **Not one of its 1,576 cases stores the
#      formal its call passes.** Its call setups are `Alloc(initSize)` and
#      `Reset()`; `initSize` is never a stored value anywhere in the file.
#   2. The one setup that *does* pass a stored formal — `c1b`, `Alloc(size)` —
#      is refused by the reader's REGIME gate one step earlier, because slot 1
#      would then hold `params[2]` and the call's argument setup is not empty
#      (`mr 4,5`). It never reaches the transfer gate at all.
#
# So the axis needs a callee whose argument list is a **prefix of the formals in
# identity order** at arity >= 2, and `88`'s `H` declares none.
#
# ---- why a SIBLING fragment and not an extension of `88` ---------------------
#
# `88`'s `PRE` is held FIXED across its whole cross **by construction** — its own
# comment: *"so a difference between two cells is a difference in the body and
# never in the type"*. The new axis needs three formals (see ARITY below) and a
# two-argument member callee, and adding either to `H` rewrites the source of all
# 1,576 existing cases. That would:
#
#   * re-baseline the port split `w-gen` and `w-seam2` both published for that
#     fragment (44 Match / 1,532 NotImplemented, then 83 / 1,493), so neither
#     number would still name a measured population; and
#   * make every one of `88`'s cells non-comparable with the cells `w-seam2`
#     graded against it, for a change that has nothing to do with `88`'s axes.
#
# A sibling keeps `88`'s 1,576 cases byte-identical and adds a disjoint family
# whose own `PRE` can be shaped for liveness. Two fragments also cannot conflict
# in git, which is the `sweep.d/` one-file-per-axis contract's whole point.
#
# ---- THE AXES, and why each is STRUCTURAL rather than a value ---------------
#
# The rule this project has now paid for four times is **enumerate STRUCTURAL
# axes and cross them; vary values inside each cell**. Both grids that missed
# #1174 varied values thoroughly and left one structural axis constant. So:
#
#   * **ARITY of the callee, at identity slot binding** — 0, 1, 2, 3, plus the
#     no-call LEAF CONTROL. `w-seam2`'s GRID S4 is why this is a level and not a
#     constant: every cell behind its derivation had arity 2, where *"slot 1 is
#     live"* and *"slot >= 1 is live"* are the SAME STATEMENT. Its `a2_break2` —
#     a two-argument callee whose run stores the **second** argument — swaps, so
#     a gate (or a generator) keyed on slot 1 alone is wrong and blind
#     respectively. Three formals are the minimum that separates the three
#     readings, which is why this family's struct has three and `88`'s two.
#   * **WHICH formal the run stores**, as a subset of `{a, b, c}` crossed with
#     the receiver `this`. `this` is argument slot 0 and is measured EXEMPT
#     (`w-seam2` §4.3), so it is a level of this axis and not an omission: a
#     generator that only stored formals could not show that the exemption is
#     real, and one that only stored `this` is `88`.
#   * **POSITION of the live store within the run.** `work/w-seam2/grid3/p3`
#     stores the live argument FIRST, where the hoist that would fix these is a
#     no-op and the framed run *is* the leaf's. So position decides whether the
#     defect is VISIBLE, and a generator that only ever put the live store last
#     would show the swap but could not show where it stops. §2 is the block that
#     walks every permutation of a three-store run.
#   * **the REGIME break** — a callee whose slots are NOT the formals in order
#     (`A1(b)`, `A2(b,a)`, `A2(a,c)`), a FREE callee (writes `r3`), and a member
#     call on another object. These are `88`'s axis C one level in: there the
#     question was what the setup writes, here it is what the setup writes *while
#     the run stores something the setup reads*.
#
# And the axes `88` already crosses are **crossed against the new one, not
# replaced**: producer count (held orthogonal to width — the confusion that
# produced #1099's wrong width), F2 (a member's address as a stored value), the
# reference bind (#839: the two spellings are DIFFERENT bodies), and the
# return-value use (framed constructor vs the three forms that tail-call).
#
# ---- what a MISMATCH here means, and the COUNTERFACTUAL that proved it -------
#
# Measured at master `62af9b75` with `w-seam2`'s live-argument refusal REVERTED
# in a scratch tree — the counterfactual board #1174 asks for, because a
# generator that adds cases and cannot demonstrate it would have caught the known
# defect has grown a number and closed nothing:
#
# ```text
#   landed tree        this fragment   checked=1270 mismatches=0
#   refusal reverted   this fragment   checked=1270 mismatches=186
#   refusal reverted   88 alone        checked=1576 mismatches=0    <- the CONTROL
# ```
#
# The third line is what makes the first two a measurement rather than a claim:
# on the SAME reverted tree `88` reports zero, which is exactly the reading board
# #1174 records from the evening two wrong emits went past it. The hole was in
# the generator and not in the tree.
#
# The 186 split **75 / 62 / 49** by callee arity 1 / 2 / 3, and by the set of
# argument slots the run stores: `{1}` 100, `{1,2}` 53, `{1,3}` 16, `{1,2,3}` 14,
# and **`{2}` 3** — the `a2_break2` cells, which an axis keyed on the call's
# FIRST argument does not reach. Zero at arity 0, zero on the leaf controls, zero
# on the regime breaks. The reverted tree is never committed;
# `docs/rungs/2026-08-08-w-gen2.md` §3 carries the patch and both runs.
#
# On the LANDED tree almost everything here refuses: the transfer gate is
# syntactic and refuses every live-argument store, including the ones where the
# hoist would have been a no-op. The leaf controls and the nullary-callee cells
# `Port=Match` today and are the half that cannot regress quietly; the refusals
# are the half that turns red the moment a hoist rule is fitted and is wrong one
# cell over.
#
# ---- the profile, stated rather than conflated (board #1112) ----------------
#
# The sweep compiles at **`/Ox /GS- /c`**. `w-seam2`'s grids and every `xboxheap`
# measurement quoted above were graded at the workload's **`/GR /O1 /Oi /EHsc`**.
# These are DIFFERENT populations and nothing here should be quoted as a
# statement about the workload profile. The cells are structural — shaped to
# reach the productions, not to reproduce any particular obj's bytes.


def cases(emit):
    # ---- the type, held FIXED across every block in this file ---------------
    #
    # THREE formals, because the arity axis needs to separate "slot 1", "slot
    # >= 1" and "every slot": at two formals a callee of arity 2 takes all of
    # them and the three readings coincide, which is the level GRID S4 had to
    # add after 63 frozen cells had already agreed.
    #
    # Offsets, all distinct so that no two axes ever write the same member (a
    # dead store makes c2 emit ONE, which silently shortens a run):
    #
    #   p0 0 · p1 4 · mList 8 (mNext 8, mPrev 12) · mA 16 · mB 20 · mC 24
    #   mSecond 28 (mNext 28, mPrev 32) · mD 36 · mE 40
    #
    # `A0/A1/A2/A3` are members on `this` whose argument lists are PREFIXES of
    # `(a, b, c)` in order — the only shape whose argument setup is empty, which
    # is what the reader's regime gate requires before liveness can matter at
    # all. `f0/f1/f2` are the free-function counterparts: same argument lists,
    # slot 0 no longer `this`, so the setup writes `r3`.
    PRE = (
        'struct BE { BE* mNext; BE* mPrev; };\n'
        'extern BE* f0();\n'
        'extern BE* f1(unsigned int);\n'
        'extern BE* f2(unsigned int, unsigned int);\n'
        'struct K {\n'
        '  K(unsigned int a, unsigned int b, unsigned int c);\n'
        '  void mv(unsigned int a, unsigned int b, unsigned int c);\n'
        '  BE* mr(unsigned int a, unsigned int b, unsigned int c);\n'
        '  void md(unsigned int a, unsigned int b, unsigned int c);\n'
        '  BE* A0();\n'
        '  BE* A1(unsigned int);\n'
        '  BE* A2(unsigned int, unsigned int);\n'
        '  BE* A3(unsigned int, unsigned int, unsigned int);\n'
        '  K* p0; K* p1; BE mList;\n'
        '  unsigned int mA; unsigned int mB; unsigned int mC;\n'
        '  BE mSecond; unsigned int mD; unsigned int mE;\n'
        '};\n'
    )

    # The run's atoms. `a`/`b`/`c` are the three formals (r4/r5/r6), `t`/`u` are
    # the receiver at slot 0, and the `q*` are PRODUCERS — a value that must be
    # materialised before the run. A formal store and a `this` store need none,
    # which is what keeps the width axis orthogonal to the producer count.
    SA, SB, SC = 'mA = a;', 'mB = b;', 'mC = c;'
    ST, SU = 'p0 = this;', 'p1 = this;'
    QL, QM = 'mD = 0;', 'mE = 7;'
    QA = 'mList.mNext = &mList;'

    seen = set()

    def case(src):
        """Emit once. Two axes can name the same body; two copies of one case is
        coverage the sweep did not buy (board #281), so the fragment dedupes
        rather than letting the count carry duplicates."""
        if src in seen:
            return
        seen.add(src)
        emit(src)

    HDRS = {
        'ctor': 'K::K(unsigned int a, unsigned int b, unsigned int c)',
        'void': 'void K::mv(unsigned int a, unsigned int b, unsigned int c)',
        'ret': 'BE* K::mr(unsigned int a, unsigned int b, unsigned int c)',
        'disc': 'void K::md(unsigned int a, unsigned int b, unsigned int c)',
    }

    def body(form, pro, run, callexpr):
        """One case. `form` selects the return-value use — only `ctor` frames
        (#869: a framed run needs one value live across one call, and the
        constructor's implicit `return this` is that value); the other three are
        frame words 0 and TAIL-CALL behind the run, so they are controls for the
        composition and not instances of it."""
        if callexpr is None:
            tail = ''
        elif form == 'ret':
            tail = '  return %s;\n' % callexpr
        elif form == 'disc':
            tail = '  BE* r = %s; (void)r;\n' % callexpr
        else:
            tail = '  %s;\n' % callexpr
        return PRE + '%s {\n%s%s%s}\n' % (
            HDRS[form], pro, ''.join('  %s\n' % s for s in run), tail)

    # ---- axis: the CALLEE, at IDENTITY slot binding -------------------------
    # Slot `i` holds `params[i]`, so the argument setup is EMPTY and the run's
    # base register is never written — the regime the reader accepts. `knone` is
    # the LEAF CONTROL: the identical run with no call at all. Without it "the
    # run transfers" is a claim about one cell rather than a comparison, which is
    # board #866's own construction and the thing this fragment exists to keep
    # honest now that #866 is refuted in general.
    ARITY = [
        ('knone', None),            # no call            — the leaf control
        ('k0', 'A0()'),             # arity 0            — nothing kept alive
        ('k1', 'A1(a)'),            # arity 1            — slot 1 live
        ('k2', 'A2(a, b)'),         # arity 2            — slots 1,2 live
        ('k3', 'A3(a, b, c)'),      # arity 3            — every slot live
    ]

    # ---- axis: WHICH formals the run stores, and in WHICH ORDER -------------
    # The subset is the liveness axis; the two orders at cardinality 2 and 3 are
    # the position axis at its cheapest. `t`/`u` are the receiver — argument slot
    # 0, measured EXEMPT — so they are a level here and not an omission.
    STORED = [
        ('z', []),                  # no formal stored at all — the control
        ('a', [SA]),
        ('b', [SB]),                # arity 2 + this = GRID S4's `a2_break2`
        ('c', [SC]),
        ('ab', [SA, SB]),
        ('ba', [SB, SA]),
        ('ac', [SA, SC]),
        ('ca', [SC, SA]),
        ('bc', [SB, SC]),
        ('cb', [SC, SB]),
        ('abc', [SA, SB, SC]),
        ('cba', [SC, SB, SA]),
        ('t', [ST]),                # slot 0 only — the exemption's own cell
        ('at', [SA, ST]),
        ('ta', [ST, SA]),
        ('tu', [ST, SU]),           # two exempt stores, nothing live
    ]

    # ---- axis: the PRODUCER COUNT, orthogonal to the width -----------------
    # Kept orthogonal on purpose: the STORED axis above varies the width at a
    # fixed producer count and this one varies the producers at a fixed width, so
    # "three stores" and "two producers" can never be the same number anywhere —
    # the confusion that produced #1099's wrong width. `qA` is F2, a member's
    # ADDRESS as a stored value, which is what makes the run mixed-kind.
    PROD = [
        ('q0', []),
        ('q1', [QL]),               # one literal
        ('q2', [QL, QM]),           # two DISTINCT literals — same kind, count 2
        ('qA', [QA]),               # F2: an interior address (`addi`)
    ]

    # ===== 1. THE CORE CROSS =================================================
    # arity x stored-formals x producer count, at the FRAMED form. This is the
    # cross board #1174 says is missing: `k1`/`k2`/`k3` crossed with a run that
    # stores `a`, `b` or `c` is exactly "the run stores a value the call keeps
    # alive", and `knone`/`k0` beside each of them is the comparison that says so.
    for kname, call in ARITY:
        for sname, stores in STORED:
            for pname, prod in PROD:
                case(body('ctor', '', prod + stores, call))

    # ===== 1b. THE SAME CROSS AT THE THREE TAIL-CALL FORMS ===================
    # `w-heap` §3.3: `void`, `return <call>` and the discarded-`int` form all
    # tail-call at frame words 0, so three of four look-alike cells are a
    # DIFFERENT shape. Crossed at reduced resolution rather than dropped — a
    # widening that starts emitting a composition for a tail-call body must turn
    # this red, and a fragment that graded only the framed form could not say so.
    for kname, call in ARITY[1:]:
        for sname, stores in STORED[1:12]:
            for form in ('void', 'ret', 'disc'):
                case(body(form, '', [QL] + stores, call))

    # ===== 2. THE POSITION OF THE LIVE STORE =================================
    # Every permutation of a three-store run, crossed with where the producer
    # sits. `work/w-seam2/grid3/p3` is why this is an axis: it stores the live
    # argument FIRST, the hoist is a no-op there and the framed run IS the leaf's
    # — so position decides whether the defect is visible at all, and a grid that
    # only ever put the live store last would show the swap without showing its
    # boundary.
    TRIPLES = [
        (SA, SB, ST),               # two formals + the exempt receiver
        (SA, SB, SC),               # three formals: at k3 EVERY store is live
        (SA, SC, SU),               # a live store and a dead one, `c` dead < k3
    ]
    PERMS = [(0, 1, 2), (0, 2, 1), (1, 0, 2), (1, 2, 0), (2, 0, 1), (2, 1, 0)]
    for trip in TRIPLES:
        for perm in PERMS:
            run = [trip[i] for i in perm]
            for kname, call in ARITY:
                for pos in ('none', 'front', 'mid', 'back'):
                    if pos == 'none':
                        r = run
                    elif pos == 'front':
                        r = [QL] + run
                    elif pos == 'mid':
                        r = run[:1] + [QL] + run[1:]
                    else:
                        r = run + [QL]
                    case(body('ctor', '', r, call))

    # ===== 3. THE REGIME BREAK ===============================================
    # A callee whose slots are NOT the formals in order, a FREE callee, and a
    # member call on another object — `88`'s axis C one level in. There the
    # question was what the argument setup WRITES; here it is what the setup
    # writes while the run stores something the setup READS. Every one of these
    # is refused by the reader's regime gate today, one step before liveness is
    # asked, so this block is the over-accept guard for a widening of THAT gate:
    # admit a non-empty setup and these become live-argument cells with a
    # register move interleaved into the run.
    BREAKS = [
        ('x1', 'A1(b)'),            # slot 1 = b   != params[1] -> `mr 4,5`
        ('x2s', 'A2(b, a)'),        # the two swapped           -> two moves
        ('x2c', 'A2(a, c)'),        # slot 2 = c   != params[2]
        ('x3', 'A3(a, c, b)'),      # slots 2,3 swapped
        ('xf0', 'f0()'),            # free, nullary — slot 0 is not `this`
        ('xf1', 'f1(a)'),           # free, one arg             -> WRITES r3
        ('xf2', 'f2(a, b)'),        # free, two args            -> WRITES r3
        ('xm', 'p0->A1(a)'),        # member on ANOTHER object; the receiver is
                                    # a LOAD through the run's own base
    ]
    for xname, call in BREAKS:
        for sname, stores in (STORED[1], STORED[2], STORED[3], STORED[5],
                              STORED[7], STORED[12], STORED[13]):
            for form in ('ctor', 'void'):
                case(body(form, '', [QL] + stores, call))

    # ===== 4. THE REFERENCE BIND, CROSSED WITH LIVENESS ======================
    # Board #839: `xboxheap`'s constructor written *without* `BE& lh = mList` is
    # a DIFFERENT body — both producers swap emission order and one store moves.
    # A widening that collapses the two spellings emits the other body's words,
    # which is #232's shape. `88` crosses the bind against everything it has; it
    # has no liveness to cross it against, and the two interact: the bind decides
    # the producers' emission order and the live store decides the unproduced
    # stores' order, so a rule fitted on one spelling is being asked about both.
    for bind in (False, True):
        pro = '  BE& lh = mList;\n' if bind else ''
        L = 'lh' if bind else 'mList'
        LA = '&lh' if bind else '&mList'
        for uses in (1, 2):
            addr = ['%s.mNext = %s;' % (L, LA)]
            if uses == 2:
                addr.append('%s.mPrev = %s;' % (L, LA))
            for sname, stores in (STORED[0], STORED[1], STORED[2], STORED[5],
                                  STORED[13]):
                for kname, call in ARITY:
                    case(body('ctor', pro, [QL] + addr + stores, call))
                    case(body('ctor', pro, [QL] + stores + addr, call))

    # ===== 5. THE TWO-FORMAL WITNESS, GENERATED ==============================
    # `w-seam2` §4.2's bodies verbatim, as generated cases rather than as a
    # hand-written fixture. Two formals and three members is the smallest body
    # that swaps, and it is the one the defect was actually found on; the K
    # family above is the same axis widened, and having both means a future
    # regression cannot hide behind "that is a different struct".
    #
    # `Alloc(b)` is here as the regime break at two formals — `88`'s `c1b` shape,
    # which is where this whole family's blindness came from.
    PRE_P = (
        'struct BE { BE* mNext; BE* mPrev; };\n'
        'struct P {\n'
        '  P(unsigned int a, unsigned int b);\n'
        '  void lf(unsigned int a, unsigned int b);\n'
        '  BE* Alloc(unsigned int);\n'
        '  BE* Alloc2(unsigned int, unsigned int);\n'
        '  BE* Reset();\n'
        '  unsigned int m0; unsigned int m1; unsigned int m2; P* mp;\n'
        '};\n'
    )
    P_CALLS = [
        ('pn', None),               # the LEAF control (`P::lf` when void)
        ('p0', 'Reset()'),          # nullary  — nothing live, does NOT swap
        ('p1', 'Alloc(a)'),         # arity 1  — `a` live, SWAPS
        ('p2', 'Alloc2(a, b)'),     # arity 2  — both live
        ('px', 'Alloc(b)'),         # regime break: slot 1 = b != params[1]
    ]
    P_HDR = {
        'ctor': 'P::P(unsigned int a, unsigned int b)',
        'void': 'void P::lf(unsigned int a, unsigned int b)',
    }
    P_LIT, P_A, P_B, P_T = 'm0 = 0;', 'm2 = a;', 'm1 = b;', 'mp = this;'
    P_RUNS = []
    for perm in PERMS:
        P_RUNS.append([(P_LIT, P_B, P_A)[i] for i in perm])
    for perm in PERMS:
        P_RUNS.append([(P_T, P_B, P_A)[i] for i in perm])
    P_RUNS.append([P_B, P_A])
    P_RUNS.append([P_A, P_B])
    for run in P_RUNS:
        for pname, call in P_CALLS:
            for form in ('ctor', 'void'):
                tail = '' if call is None else '  %s;\n' % call
                case(PRE_P + '%s {\n%s%s}\n' % (
                    P_HDR[form], ''.join('  %s\n' % s for s in run), tail))

    # ===== 6. THE OVER-ACCEPT GUARDS FOR THIS AXIS ===========================
    # The shapes a live-argument HOIST rule could widen INTO by accident. Each is
    # one statement away from an accepted cell above and none of them is the
    # accepted shape — which is what makes a future hoist safe rather than merely
    # watched. `88` §5 does this for the composition's SHAPE; this does it for
    # the composition's LIVENESS, and neither implies the other.
    GUARDS = [
        # the live store is AFTER the call, not in the run
        'mD = 0; mB = b; A1(a); mA = a;',
        # the live formal reaches the store through a LOCAL
        'unsigned int t = a; mD = 0; mB = b; mA = t; A1(a);',
        # ...and the CALL takes the local instead
        'unsigned int t = a; mD = 0; mB = b; mA = a; A1(t);',
        # the call's argument is the MEMBER just stored, not the formal
        'mD = 0; mB = b; mA = a; A1(mA);',
        # the same formal stored TWICE, to two members
        'mD = 0; mA = a; mB = a; A1(a);',
        # the live store's value is an EXPRESSION over the formal
        'mD = 0; mB = b; mA = a + 1; A1(a);',
        # a widening cast on the way in
        'mD = 0; mB = b; mA = (unsigned int)(unsigned char)a; A1(a);',
        # the live value is the RECEIVER's own member, not a formal
        'mD = 0; mB = b; mA = mC; A1(a);',
        # the call is nullary but a LATER call takes the live formal
        'mD = 0; mB = b; mA = a; A0(); A1(a);',
        # the live formal is stored and the call is under a BRANCH
        'mD = 0; mB = b; mA = a; if (c) A1(a);',
        # the run is split by a nested scope and the live store is inside it
        'mD = 0; { mB = b; mA = a; } A1(a);',
        # the live formal is also the loop bound of a loop AFTER the call
        'mD = 0; mB = b; mA = a; A1(a); while (a) { A0(); break; }',
        # the store is through a POINTER to the same object, not `this`
        'K* s = this; mD = 0; s->mB = b; s->mA = a; A1(a);',
        # the live store is a WIDE literal beside the live formal (w-seam2 §5.1)
        'mD = 70000; mB = b; mA = a; A1(a);',
    ]
    for g in GUARDS:
        for form in ('ctor', 'void'):
            case(PRE + '%s {\n  %s\n}\n' % (HDRS[form], g))

    # ===== 7. THE LIVE-ARGUMENT COMPOSITION BESIDE A NEIGHBOUR ===============
    # The per-TU label counter and the `_fltused` marker are decided by the WHOLE
    # translation unit (`GAPS.md` §6 #12/#13). A body that REFUSES still owns its
    # `$M`/`$T` labels, so a refusal is not a free pass here: the neighbour's
    # labels move whether or not this function emits.
    LIVE = ('K::K(unsigned int a, unsigned int b, unsigned int c)\n'
            '{ mD = 0; mB = b; mA = a; A1(a); }\n')
    DEAD = ('void K::mv(unsigned int a, unsigned int b, unsigned int c)\n'
            '{ mD = 0; mB = b; mA = a; A0(); }\n')
    NEIGHBOURS = (
        'int L(int a) { return a + 1; }\n',
        'float L(float a, float b) { return a * b; }\n',
        'void L() {}\n',
        'void L(K* k, unsigned int v) { k->mB = v; }\n',
    )
    for n in NEIGHBOURS:
        for c in (LIVE, DEAD):
            case(PRE + n + c)
            case(PRE + c + n)
    case(PRE + LIVE + DEAD)
    case(PRE + DEAD + LIVE)
