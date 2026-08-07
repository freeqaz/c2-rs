# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the loader; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter, and the loader fails if a fragment emits
# zero cases.
#
# ---- W-GEN: the STORE RUN followed by a CALL — the composition seam ----------
#
# **Why this file exists.** Board #844 says the store-run emitter is leaf-only by
# construction: `scheduled_gpr_run_text` is reached only from `store_leaf_text`
# and terminates by appending `encode_blr()` unconditionally, so there is no seam
# that emits a scheduled run as the MIDDLE of a framed body. `xboxheap.cpp` is the
# cheapest TU on the frontier and three lanes have now circled it (#1097,
# `w-front2`, `w-heap`), so this is the next place a reader is going to widen.
#
# Board **#232** is what happens when a reader widens into a shape no instrument
# can generate: a clean refusal became a wrong emit, it survived **255 commits**
# on master, and the 878-TU workload scan read `mismatch 0` the entire time
# **because it could not generate that shape**. It was caught only by this sweep,
# on an axis somebody had enumerated first. `w-heap` §6.3 names the same gap for
# this family in as many words — *"the 878-TU scan cannot generate this shape at
# all… the instrument that would catch it is the generated sweep, and the sweep
# has no store-run-before-a-call generator today"* — and declines to build it.
# This fragment is that generator, landed BEFORE the widening rather than after.
#
# **This family is not one of board #283's 56 zero-marker rows.** Every marker in
# `scripts/sweep_shapes.py` is LEXICAL — a keyword, a type, an operator — and
# `constructor/destructor`, `reference type &` and `pointer` are all long since
# non-zero. A store run *followed by a call in the same body* is a composition of
# two productions, which no source-token regex can see. So it is not an unreached
# marker; it is a shape the marker table structurally cannot represent. Absent,
# not unreached — a different repair, and this is it.
#
# ---- the axes, and why these ------------------------------------------------
#
# `w-heap`'s GRID F3 graded 27 frozen cells and collapsed six candidate axes to
# two that matter. Those measurements are consumed here, not re-derived:
#
#   * **C — the call's ARGUMENT SETUP. This is the regime boundary and it is
#     finer than board #870 recorded.** #870 states it as *"a trailing call that
#     takes an ARGUMENT breaks the run"*. Measured, that is one level too coarse:
#     `Alloc(size)`, whose setup is `mr 4,5`, **still transfers** — one base
#     throughout, `mr 31,3` additive (#866). What breaks the run is the setup
#     writing **`r3`**, which destroys `this`: a FREE callee forces `mr 3,4`, the
#     store base switches `r3 -> r31` mid-run and the setup interleaves into it
#     (#870). So axis C is enumerated by *what the setup writes*, not by how many
#     arguments the call has, and `c1b` is the cell that separates them.
#   * **F — the RETURN-VALUE USE, because only the constructor is framed.** The
#     `void`, `return <call>` and discarded-`int` forms all **tail-call at frame
#     words 0** (#869: a framed run needs one value live across one call, and the
#     ctor's implicit `return this` is that value). Three of four look-alike cells
#     are a different shape, so a generator that varied only the store run would
#     be grading tail calls and reporting frames.
#   * **the PRODUCER COUNT, not the store count.** #1099's evidence for *"the
#     schedule is paid at `xboxheap`'s own width"* was cell `x7`, which has **one**
#     producer where `xboxheap` has **two** — it matched the store count and not
#     the producer count, and the producer count is the axis the schedule is
#     about (`w-heap` §5). A producer here is a value that must be materialised
#     into a register before the run: a literal (`li`) or an interior member
#     address (`addi`). A `this` store and a formal store need none.
#   * **the REFERENCE BIND, because it is load-bearing at this width.**
#     `xboxheap`'s constructor written *without* `BE& listHead = mListHead` is a
#     **different body** — both producers swap emission order and one store moves
#     (`w-heap` §4.2). A widening that collapses the two spellings to one emits
#     the other body's words, which is #232's exact shape. So the bind is crossed,
#     never held.
#   * **F2 — a member's ADDRESS as a stored value**, present vs absent, which is
#     the reader refusal item 1 of `w-heap`'s five and the thing that makes the
#     run mixed-kind (`addi` beside `li`) and therefore reaches `alloc`'s
#     mixed-kind refusal (#836/#868).
#   * **the STORE-RUN WIDTH**, held orthogonal to the producer count by padding
#     with no-producer stores, so "six stores" and "two producers" cannot be the
#     same number anywhere — the confusion that produced #1099's wrong width.
#
# Axes `w-heap` measured as NOT free-standing are not crossed and the coupling is
# recorded instead of pretended away: **D** (callee kind) and **E** (receiver slot
# 0) are *determined by* C — a free function or a member on another object both
# put a non-`this` value in slot 0, so both force C = writes-`r3`. They get their
# own small block (§4) rather than a factor in the cross.
#
# ---- what a MISMATCH here would mean ----------------------------------------
#
# Everything in this fragment is expected to refuse today: the reader has neither
# F2 nor F3 and #844's seam does not exist. `Port=NotImplemented` is the right
# answer and the sweep grades it as good. What these cases buy is the **next**
# lane's widening: the moment a reader admits an `AddrOf` in a store value or a
# `BodyShape` for a run-then-call, ~1.4k cases across all six axes start emitting,
# and a widening that is right on `xboxheap` and wrong on its neighbours — which
# is how all six refuted allocation keys got written — turns this row red instead
# of turning the workload scan silent.
#
# ---- the profile, stated rather than conflated (board #1112) ----------------
#
# The sweep compiles at **`/Ox /GS- /c`**. `xboxheap` and every `w-heap` grid cell
# were graded at the workload's **`/GR /O1 /Oi /EHsc`**. These are DIFFERENT
# populations: a refusal can read as paid at one and be unpaid at the other. The
# cells here are structural — they are shaped to reach the productions, not to
# reproduce `xboxheap`'s bytes — and nothing in this file should be quoted as a
# statement about the workload profile.


def cases(emit):
    # The field layout is `xboxheap`'s own CXboxHeap, extended with the members
    # this cross needs so that no two axes ever write the same offset (a dead
    # store makes c2 emit ONE, which would silently shorten a run):
    #
    #   mFreeHead 0 · mUsedHead 4 · mListHead 8 (mNext 8, mPrev 12) · mSize 16
    #   mCount 20 · mSecond 24 (mNext 24, mPrev 28) · mFlags 32 · mPeak 36
    #
    # The struct is held FIXED across the whole cross, including the declarations
    # a given cell does not define, so a difference between two cells is a
    # difference in the body and never in the type.
    PRE = (
        'struct BE { BE* mNext; BE* mPrev; };\n'
        'extern BE* g1(unsigned int);\n'
        'extern BE* g2(unsigned int, unsigned int);\n'
        'extern BE* ga(BE*);\n'
        'struct H {\n'
        '  H(unsigned int initSize, unsigned int size);\n'
        '  H(unsigned int initSize, unsigned int size, H* q);\n'
        '  void mv(unsigned int initSize, unsigned int size);\n'
        '  BE* mr(unsigned int initSize, unsigned int size);\n'
        '  void md(unsigned int initSize, unsigned int size);\n'
        '  BE* Alloc(unsigned int);\n'
        '  BE* Reset();\n'
        '  H* mFreeHead; H* mUsedHead; BE mListHead;\n'
        '  unsigned int mSize; unsigned int mCount; BE mSecond;\n'
        '  unsigned int mFlags; unsigned int mPeak;\n'
        '};\n'
    )

    # ---- axis: the RUN, by producer count and by F2 -------------------------
    #
    # `(name, bindable, builder)`. `builder(L, LA)` takes the spelling of the
    # bound sub-object and of its address, so the same structural run is emitted
    # once through `mListHead` / `&mListHead` and once through a reference bind
    # `lh` / `&lh`. `bindable` is False for runs that never touch `mListHead`, so
    # the bind axis is not faked on cells where it has no expression.
    #
    #   producers · F2 · what it separates
    #   0 · no  · the control: a run that materialises nothing
    #   1 · no  · one literal, one use  |  one literal, TWO uses (shared `li`)
    #   2 · no  · two DISTINCT literals — two producers of the SAME kind
    #   1 · yes · the interior address alone, at one use and at two
    #   2 · yes · address(2 uses) + literal(1)  <- `xboxheap`'s own mix
    #   2 · yes · literal(2 uses) + address(1)  <- the INVERSE, which is
    #             `w-heap` §4.1.1's `j1_lit2` counterexample: c2 still hands the
    #             top of the pool to the address, so a use-count clause fitted on
    #             `xboxheap` alone is wrong here
    #   2 · yes · TWO interior addresses at different offsets, no literal
    #   3 · yes · two addresses + a literal
    RUNS = [
        ('p0',    False, lambda L, LA: []),
        ('pL1',   False, lambda L, LA: ['mCount = 0;']),
        ('pLu2',  False, lambda L, LA: ['mCount = 0;', 'mFlags = 0;']),
        ('pL2',   False, lambda L, LA: ['mCount = 0;', 'mFlags = 7;']),
        ('pZ1',   True,  lambda L, LA: ['%s.mNext = 0;' % L]),
        ('pZ2',   True,  lambda L, LA: ['%s.mNext = 0;' % L, '%s.mPrev = 0;' % L]),
        ('pA1',   True,  lambda L, LA: ['%s.mNext = %s;' % (L, LA)]),
        ('pA2',   True,  lambda L, LA: ['%s.mNext = %s;' % (L, LA),
                                        '%s.mPrev = %s;' % (L, LA)]),
        ('pAL',   True,  lambda L, LA: ['mCount = 0;',
                                        '%s.mNext = %s;' % (L, LA),
                                        '%s.mPrev = %s;' % (L, LA)]),
        ('pALl2', True,  lambda L, LA: ['mCount = 0;', 'mFlags = 0;',
                                        '%s.mNext = %s;' % (L, LA)]),
        ('pAA',   True,  lambda L, LA: ['%s.mNext = %s;' % (L, LA),
                                        '%s.mPrev = %s;' % (L, LA),
                                        'mSecond.mNext = &mSecond;']),
        ('pAAL',  True,  lambda L, LA: ['mCount = 0;',
                                        '%s.mNext = %s;' % (L, LA),
                                        '%s.mPrev = %s;' % (L, LA),
                                        'mSecond.mNext = &mSecond;',
                                        'mSecond.mPrev = &mSecond;']),
    ]

    # ---- axis: the WIDTH, padded with stores that need NO producer ----------
    # Held orthogonal to the producer count on purpose: `mSize = size` is a formal
    # already in a register and `mFreeHead = this` is `r3`, so these lengthen the
    # run without adding anything to the constant pool. A grid where width and
    # producer count move together cannot attribute a schedule change to either.
    WIDTHS = [
        ('w0', []),
        ('w1', ['mSize = size;']),
        ('w3', ['mSize = size;', 'mFreeHead = this;', 'mUsedHead = this;']),
    ]

    # ---- axis C: what the call's ARGUMENT SETUP writes ----------------------
    # `cnone` is the LEAF CONTROL — the identical run with no call at all. Without
    # it "the run transfers" is a claim about one cell rather than a comparison,
    # which is board #866's own construction.
    SETUPS = [
        ('cnone', None),                  # no call: the leaf control
        ('c0',    'Alloc(initSize)'),     # member on `this`, arg already in r4 -> EMPTY setup
        ('c0n',   'Reset()'),             # member on `this`, nullary           -> EMPTY setup
        ('c1b',   'Alloc(size)'),         # member on `this`, arg is formal 2   -> `mr 4,5`: writes r4, STILL TRANSFERS
        ('c1r3',  'g1(initSize)'),        # free fn, one arg                    -> `mr 3,4`: WRITES r3
        ('c2r3',  'g2(size, initSize)'),  # free fn, two args swapped           -> two moves, WRITES r3
    ]

    # ---- axis F: the return-value use, i.e. FRAMED vs TAIL-CALL -------------
    # Only `fctor` frames. The other three are frame words 0 and tail-call behind
    # the run (`w-heap` §3.3), so this axis is what keeps the fragment from
    # grading three tail calls and calling them a composition.
    FORMS = [
        ('fctor',    'H::H(unsigned int initSize, unsigned int size)',
         lambda c: '  %s;\n' % c, False),
        ('fvoid',    'void H::mv(unsigned int initSize, unsigned int size)',
         lambda c: '  %s;\n' % c, False),
        ('fretcall', 'BE* H::mr(unsigned int initSize, unsigned int size)',
         lambda c: '  return %s;\n' % c, True),
        ('fdiscard', 'void H::md(unsigned int initSize, unsigned int size)',
         lambda c: '  BE* r = %s; (void)r;\n' % c, True),
    ]

    def body(header, prologue, stmts, tail):
        return '%s {\n%s%s%s}\n' % (
            header, prologue, ''.join('  %s\n' % s for s in stmts), tail)

    # ===== 1. THE CROSS ======================================================
    # run-kind x bind x width x argument-setup x return-value-use. The bind axis
    # only exists where `mListHead` is touched, so it is 20 run/bind pairs rather
    # than 24 — a level that has no expression is dropped, never emitted twice
    # under two names.
    for rname, bindable, build in RUNS:
        for bind in ((False, True) if bindable else (False,)):
            L = 'lh' if bind else 'mListHead'
            LA = '&lh' if bind else '&mListHead'
            pro = '  BE& lh = mListHead;\n' if bind else ''
            run = build(L, LA)
            for wname, pad in WIDTHS:
                stmts = pad + run
                for cname, callexpr in SETUPS:
                    for fname, header, tailof, needs_call in FORMS:
                        if callexpr is None and needs_call:
                            continue    # `return <call>` with no call is not a cell
                        tail = '' if callexpr is None else tailof(callexpr)
                        emit(PRE + body(header, pro, stmts, tail))

    # ===== 2. AXIS B — TWO STORE BASES =======================================
    # `w-heap`'s `f3_b2` emits and is left refused, matching the parser's existing
    # multi-symbol gate. Swept against the setup axis because the second base and
    # the setup compete for the same volatiles once the setup writes `r3`.
    for cname, callexpr in SETUPS:
        for hdr, is_ctor in (
            ('H::H(unsigned int initSize, unsigned int size, H* q)', True),
            ('void H::mv(unsigned int initSize, unsigned int size)', False),
        ):
            q = 'q' if is_ctor else 'mFreeHead'
            for run in (
                ['mSize = size;', '%s->mCount = 0;' % q, 'mUsedHead = this;'],
                ['mListHead.mNext = &mListHead;', '%s->mCount = 0;' % q,
                 '%s->mFlags = 0;' % q],
                ['mCount = 0;', 'mListHead.mNext = &mListHead;',
                 '%s->mSecond.mNext = &%s->mSecond;' % (q, q)],
            ):
                tail = '' if callexpr is None else '  %s;\n' % callexpr
                emit(PRE + body(hdr, '', run, tail))

    # ===== 3. AXES D/E — CALLEE KIND AND RECEIVER SLOT 0 =====================
    # Enumerated as their own block because `w-heap` measured them to be
    # DETERMINED by C rather than free beside it: a member call on another object
    # and a free function both put a non-`this` value in slot 0, so both force the
    # setup to write `r3`. Recording the coupling is the point; a cross would
    # claim to separate two axes that are one.
    RECEIVERS = [
        ('r_this',   'Alloc(initSize)'),              # slot 0 = this        (C=0)
        ('r_formalq','q->Alloc(initSize)'),           # slot 0 = a formal    (C>=1)
        ('r_member', 'mFreeHead->Alloc(initSize)'),   # slot 0 = a LOAD      (C>=1, and the load is in the run's own base)
        ('r_addr',   'ga(&mListHead)'),               # slot 0 = the F2 VALUE ITSELF
        ('r_addr2',  'ga(&mSecond)'),                 # slot 0 = a second interior address
    ]
    for rname, callexpr in RECEIVERS:
        needs_q = 'q->' in callexpr
        for hdr in (
            ('H::H(unsigned int initSize, unsigned int size, H* q)' if needs_q
             else 'H::H(unsigned int initSize, unsigned int size)'),
            ('void H::mv(unsigned int initSize, unsigned int size)'
             if not needs_q else None),
        ):
            if hdr is None:
                continue
            for run in (
                ['mSize = size;', 'mFreeHead = this;', 'mUsedHead = this;'],
                ['mCount = 0;', 'mListHead.mNext = &mListHead;',
                 'mListHead.mPrev = &mListHead;'],
                ['mSize = size;', 'mCount = 0;', 'mFreeHead = this;',
                 'mUsedHead = this;', 'mListHead.mNext = &mListHead;',
                 'mListHead.mPrev = &mListHead;'],
            ):
                for pro in ('', '  BE& lh = mListHead;\n'):
                    r = ([s.replace('mListHead', 'lh') for s in run]
                         if pro else run)
                    emit(PRE + body(hdr, pro, r, '  %s;\n' % callexpr))

    # ===== 4. AXIS G — THE F2 VALUE'S MEMBER OFFSET ==========================
    # The address's own offset decides whether an `addi` exists at all: offset 0
    # is `this` and emits none, offset 8 is `xboxheap`'s, offset 24 is a second
    # sub-object further in. Crossed with the bind and with the setup, because the
    # producer the offset creates is the one whose emission order the bind swaps.
    OFFSETS = [
        ('g0',  '(BE*)this',   'mListHead'),   # offset 0  -> no addi
        ('g8',  '&mListHead',  'mListHead'),   # offset 8  -> `addi r,r3,8`
        ('g24', '&mSecond',    'mSecond'),     # offset 24 -> `addi r,r3,24`
    ]
    for gname, addr, dest in OFFSETS:
        for bind in (False, True):
            if bind:
                pro = '  BE& lh = %s;\n' % dest
                d, a = 'lh', ('&lh' if addr.startswith('&') else addr)
            else:
                pro, d, a = '', dest, addr
            for cname, callexpr in SETUPS:
                for uses in (1, 2):
                    run = ['mCount = 0;', '%s.mNext = %s;' % (d, a)]
                    if uses == 2:
                        run.append('%s.mPrev = %s;' % (d, a))
                    tail = '' if callexpr is None else '  %s;\n' % callexpr
                    emit(PRE + body(
                        'H::H(unsigned int initSize, unsigned int size)',
                        pro, run, tail))

    # ===== 5. THE OVER-ACCEPT GUARDS =========================================
    # The shapes a #844 composition seam could widen INTO by accident. Every one
    # of them is one statement away from an accepted cell above and none of them
    # is the accepted shape, so this block is what makes the widening safe rather
    # than merely watched. A `Port=Mismatch` here after a reader lands is the
    # alarm the 878-TU scan cannot raise (#232's mechanism, #871's for `fnbyte`).
    RUN3 = 'mSize = size; mFreeHead = this; mUsedHead = this;'
    RUNF2 = ('mCount = 0; mListHead.mNext = &mListHead; '
             'mListHead.mPrev = &mListHead;')
    for run in (RUN3, RUNF2):
        for guard in (
            # the call is in the MIDDLE of the run, not after it
            '%s Alloc(initSize); mFlags = 1;',
            # a store AFTER the call — the run is split by the `bl`
            '%s Alloc(initSize); mPeak = 0;',
            # the call comes FIRST and the run follows it
            'Alloc(initSize); %s',
            # TWO calls after the run
            '%s Alloc(initSize); Reset();',
            # a call whose result feeds a store
            '%s BE* r = Alloc(initSize); mListHead.mPrev = r;',
            # a BRANCH between the run and the call
            '%s if (size) Alloc(initSize);',
            # a LOOP after the run
            '%s while (size) { Alloc(initSize); break; }',
            # the argument is a MEMBER LOAD, so the setup is an `lwz` not a move
            '%s Alloc(mSize);',
            # the argument is the F2 value itself and the callee is free
            '%s ga(&mListHead);',
            # the call is through a member function POINTER-like indirection
            '%s mFreeHead->Reset();',
            # the run is under a nested scope and the call is outside it
            '{ %s } Alloc(initSize);',
            # the call is under a nested scope and the run is outside it
            '%s { Alloc(initSize); }',
        ):
            src = guard % run
            emit(PRE + '%s {\n  %s\n}\n' % (
                'H::H(unsigned int initSize, unsigned int size)', src))
            emit(PRE + '%s {\n  %s\n}\n' % (
                'void H::mv(unsigned int initSize, unsigned int size)', src))

    # ===== 6. SOURCE LINES AND STATEMENT BOUNDARIES ==========================
    # `GAPS.md` §6 instance #1's axis, on a body that now has TWO productions
    # ending it. Includes the run at source line 70, where the first `0x46` is a
    # line marker's payload rather than the `this` group — the exact byte that was
    # once read as a token. The composition is what is new: the call's own line
    # marker sits after the run's, and nothing has ever varied their distance.
    for pad in (0, 1, 3, 62):
        nl = '\n' * pad
        emit(PRE + nl + 'H::H(unsigned int initSize, unsigned int size)\n'
             '{ mCount = 0; mListHead.mNext = &mListHead;'
             ' mListHead.mPrev = &mListHead; Alloc(initSize); }\n')
        emit(PRE + nl + 'H::H(unsigned int initSize, unsigned int size)\n{\n'
             '  mCount = 0;\n  mListHead.mNext = &mListHead;\n'
             '  mListHead.mPrev = &mListHead;\n\n\n  Alloc(initSize);\n}\n')
        emit(PRE + nl + 'H::H(unsigned int initSize, unsigned int size)\n{\n'
             '  BE& lh = mListHead;\n  mCount = 0;\n  lh.mNext = &lh;\n'
             '  lh.mPrev = &lh;\n  Alloc(initSize);\n}\n')
        emit(PRE + nl + 'H::H(unsigned int initSize, unsigned int size)\n{\n'
             '  mCount = 0;\n  BE& lh = mListHead;\n  lh.mNext = &lh;\n'
             '  lh.mPrev = &lh;\n  Alloc(initSize);\n}\n')

    # ===== 7. THE COMPOSITION BESIDE A NEIGHBOUR =============================
    # The per-TU label counter and the `_fltused` marker are decided by the WHOLE
    # translation unit (`GAPS.md` §6 #12/#13), and a framed composition is a new
    # way to own `$M`/`$T` labels. Both orders, against leaves of every stride the
    # port emits, so a counter error cannot be absorbed by its neighbour.
    COMP = ('H::H(unsigned int initSize, unsigned int size)\n'
            '{ mCount = 0; mListHead.mNext = &mListHead;'
            ' mListHead.mPrev = &mListHead; Alloc(initSize); }\n')
    COMP_TAIL = ('void H::mv(unsigned int initSize, unsigned int size)\n'
                 '{ mCount = 0; mListHead.mNext = &mListHead;'
                 ' mListHead.mPrev = &mListHead; Alloc(initSize); }\n')
    NEIGHBOURS = (
        'int L(int a) { return a + 1; }\n',
        'int L(int* p) { return *p; }\n',
        'float L(float a, float b) { return a * b; }\n',
        'double L(double a, double b) { return a + b; }\n',
        'void L() {}\n',
        'int L(int x) { return x < 0; }\n',
        'void L(H* h, unsigned int v) { h->mSize = v; }\n',
    )
    for n in NEIGHBOURS:
        for c in (COMP, COMP_TAIL):
            emit(PRE + n + c)
            emit(PRE + c + n)
    # …and the two compositions in one TU, which is the only configuration that
    # separates a per-function label rule from a per-TU one (§6 #13: at n = 1 the
    # wrong formulation and the right one are indistinguishable).
    emit(PRE + COMP + COMP_TAIL)
    emit(PRE + COMP_TAIL + COMP)
