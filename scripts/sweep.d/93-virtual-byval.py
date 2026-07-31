# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace.
#
# ---- what this axis is for ------------------------------------------------
#
# WDR decoded `67 <varint slot> <tok>` (virtual dispatch), `9A <TYPE>` (the
# vtable-slot bind) and `64 <TYPE>` (the by-value return's materialize). It
# lowers none of them, so like `95-control-flow.py` this fragment enumerates the
# *boundary* of a class rather than the class: every case must come back SKIP.
#
# A case that reports MATCH is the alarm, and it is a specific one. Decoding
# these three opcodes moved 188,794 bodies into `cflow-straight` — a shape whose
# name the emitter's recognizers also use — so the hazard this fragment exists to
# catch is a recognizer that reaches a tail branch through a virtual call or a
# materialized temporary and emits a direct `b`. The pairing is what makes it
# findable: `near_direct` in `fixtures/cpp/wdr_neighbours.cpp` differs from
# `virt_ptr` in `wdr_virtual_byval.cpp` only by the word `virtual`, and exactly
# one of the two is byte-exact today.
#
# The axes, and what each separates (counts measured over the emitted corpus,
# `work/WDR/SEPARATION.md`):
#
#   1. virtual vs direct dispatch — 2 keys, and one side is EMITTED
#   2. vtable slot below/at/above byte offset 0x80 — the varint escape
#   3. the returned type's category (int / by value / by value with a
#      destructor / pointer / reference) — whether `64` appears at all, and for
#      the destructor case which side of the EH boundary the body lands on
#   4. what the result is used for — where in the statement `64` sits
#   5. a virtual call BESIDE an emitted shape — the neighbourhood check

# Slot indices spanning the varint boundary. 31 is the last byte offset the
# short form reaches (0x7C); 32 is the first that escapes (0x80).
SLOTS = [0, 1, 2, 31, 32, 33, 39]

# What the call's result is used for. Each puts the `64` (or the call's result)
# in a different statement position, which is the mis-attribution guard: the
# construct must file under the same family wherever it sits.
USES = [
    ('%s;', 'discard'),
    ('return %s;', 'return'),
    ('return %s + 1;', 'operand'),
    ('return sink(%s);', 'argument'),
    ('int t = %s; return t;', 'assign'),
]


def _wide(n):
    """A class with `n` virtual functions, so slot `n-1` is at byte offset 4(n-1)."""
    return 'struct W {\n' + ''.join(
        '  virtual int v%02d();\n' % k for k in range(n)
    ) + '};\n'


def cases(emit):
    sink = 'int sink(int);\n'

    # ---- axis 1 + 2: virtual dispatch, and the slot's encoding -------------
    for s in SLOTS:
        w = _wide(s + 1)
        for tmpl, _ in USES:
            body = tmpl % ('p->v%02d()' % s)
            emit('%s%sint f(W* p) { %s }\n' % (sink, w, body))
        # a reference receiver, byte-identical to the pointer one per
        # IL_CALL_IN_EXPR §3 — if it ever is not, this is where it shows
        emit('%s%sint f(W& r) { return r.v%02d(); }\n' % (sink, w, s))
        # the DIRECT neighbour at the same slot index: same class, same call
        # site, no `virtual`. One of the two is emitted and the other refuses.
        d = 'struct D {\n' + ''.join('  int v%02d();\n' % k for k in range(s + 1)) + '};\n'
        emit('%sint f(D* p) { return p->v%02d(); }\n' % (d, s))
    # two dispatches in one statement: the second `67` is met with the cursor
    # already past the first, so a wrong slot width is caught twice
    w40 = _wide(40)
    for s in (0, 32, 39):
        emit('%sint f(W* p, W* q) { return p->v%02d() + q->v%02d(); }\n' % (w40, s, s))

    # ---- axis 3: what the callee returns -----------------------------------
    # `64` appears iff the return is a class BY VALUE. The destructor variant is
    # the one that also crosses docs/EH_RECORDS.md's boundary, so the two must be
    # swept together or the EH class of the row is unmeasured.
    RETS = [
        ('int', 'int', 'r'),
        ('V', 'struct V { int a, b, c; };\n', 'r.a'),
        ('U', 'struct U { int a, b, c; ~U(); };\n', 'r.a'),
        ('V*', 'struct V { int a, b, c; };\n', 'r->a'),
        ('const V&', 'struct V { int a, b, c; };\n', 'r.a'),
    ]
    for ty, decl, acc in RETS:
        for virt in ('', 'virtual '):
            s = ('%s%sstruct S { %s%s Make(); int m; };\n' % (sink, decl, virt, ty))
            emit('%sint f(S* p) { %s r = p->Make(); return %s; }\n' % (s, ty, acc))
            emit('%sint f(S* p) { return p->Make() %s; }\n'
                 % (s, '' if ty == 'int' else ('->a' if ty.endswith('*') else '.a')))
            emit('%svoid f(S* p) { p->Make(); }\n' % s)
            emit('%sint f(S* p) { return sink(p->Make() %s); }\n'
                 % (s, '' if ty == 'int' else ('->a' if ty.endswith('*') else '.a')))

    # ---- axis 4: the by-value return in every statement position -----------
    byval = 'struct V { int a, b, c; };\nstruct S { V Make(); int m; };\nint sink(int);\nint use(const V&);\n'
    emit('%svoid f(S* p) { p->Make(); }\n' % byval)
    emit('%sint f(S* p) { return p->Make().a; }\n' % byval)
    emit('%sint f(S* p) { return use(p->Make()); }\n' % byval)
    emit('%sV f(S* p) { return p->Make(); }\n' % byval)
    emit('%sint f(S* p, S* q) { return p->Make().a + q->Make().b; }\n' % byval)
    emit('%sint f(S* p) { return sink(p->Make().a + 1); }\n' % byval)
    emit('%sint f(S* p) { V v = p->Make(); return v.a + v.b; }\n' % byval)
    emit('%sint f(S* p) { return p->Make().a + p->Make().b; }\n' % byval)

    # ---- axis 5: the neighbourhood ------------------------------------------
    # One function in the TU is byte-exact and the next has a virtual call or a
    # materialized temporary. A recognizer that scanned a neighbourhood rather
    # than a whole body would take the second for the first.
    NEIGHBOURS = [
        'int g(int a) { return a + 1; }',
        'int g(int a, int b) { return a + b; }',
        'int g(int* p) { return *p; }',
        'int h(int);\nint g(int a) { return h(a); }',
    ]
    w1 = _wide(1)
    for nb in NEIGHBOURS:
        emit('%s%s\nint f(W* p) { return p->v00(); }\n' % (w1, nb))
        emit('%sint f(W* p) { return p->v00(); }\n%s\n' % (w1, nb))
        emit('%s%s\nint f(S* p) { return p->Make().a; }\n' % (byval, nb))
        emit('%sint f(S* p) { return p->Make().a; }\n%s\n' % (byval, nb))
