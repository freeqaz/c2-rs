# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# ---- WCO's ALARM: a statement call followed by an explicit `return;` ---------
#
# `void f() { g(); return; }` is the bare tail branch `b ?g@@YAXXZ`, identical to
# `void f() { g(); }`. c2 records the fallthrough as a SECOND `3A <label>` to the
# label the return plumbing then uses and emits nothing for it.
#
# The port emitted the 36-byte framed Class A body for it — a **live wrong-bytes
# emit on mainline**, `Port=Mismatch @ offset 2`. The statement-call
# production's tail-call probe runs the return plumbing at `BODY_SCOPE_DEPTH`,
# which cannot parse the double `3A`, so the body fell through to
# `parse_call_sequence`, where a `debug_assert` declared the state unreachable
# and was wrong.
#
# **No fragment in `sweep.d` had ever written an explicit `return;`.** That is
# the axis this file is: not an operator, not a shape, not a type — a redundant
# statement that changes no semantics and two bytes of IL. `docs/GAPS.md` §6's
# recurring class, and it is crossed here against every call arity, the argument
# setup, `this`-receivers, brace scopes and the call COUNT, because the frame
# boundary the defect sat on is "one call or more than one".
#
# The two-and-more-call rows are as load-bearing as the one-call rows: those
# really are framed sequences, and a repair that routed them to the tail call
# too would be the same defect mirrored.

DECL = (
    'struct O { void v(); void va(int); void vb(int,int); int gi(); int ga(int); };\n'
    'void g0(); void g1(int); void g2(int,int); void g3(int,int,int);\n'
    'void g4(int,int,int,int);\n'
    'int  r0(); int r1(int);\n'
)

# (body-with-return, formals) — one call, discarded. Each must be the bare `b`.
ONE = (
    ('g0();', ''),
    ('g1(a);', 'int a'),
    ('g1(7);', ''),
    ('g1(a + 1);', 'int a'),
    ('g1(a - 3);', 'int a'),
    ('g2(a, b);', 'int a, int b'),
    ('g2(b, a);', 'int a, int b'),
    ('g3(a, b, c);', 'int a, int b, int c'),
    ('g3(c, a, b);', 'int a, int b, int c'),
    ('g3(b, a, c);', 'int a, int b, int c'),
    ('g4(a, b, c, d);', 'int a, int b, int c, int d'),
    ('g4(d, c, b, a);', 'int a, int b, int c, int d'),
    ('g2(a, 5);', 'int a'),
    ('g2(5, a);', 'int a'),
    ('r0();', ''),
    ('r1(a);', 'int a'),
)

# Two or more calls — these keep the frame, and that is the other half of the
# boundary the defect sat on.
MANY = (
    ('g0(); g0();', ''),
    ('g1(a); g0();', 'int a'),
    ('g0(); g1(a);', 'int a'),
    ('g1(a); g1(b);', 'int a, int b'),
    ('g0(); g0(); g0();', ''),
    ('g2(a, b); g1(a);', 'int a, int b'),
)


def cases(emit):
    # 1. ONE call, with and without the explicit `return;`. The pair is the
    #    measurement: the two bodies must emit the SAME bytes, and only the
    #    `return;` half was wrong.
    for body, parms in ONE:
        emit('%svoid f(%s) { %s return; }\n' % (DECL, parms, body))
        emit('%svoid f(%s) { %s }\n' % (DECL, parms, body))

    # 2. TWO OR MORE calls, both ways — still the framed Class A sequence.
    for body, parms in MANY:
        emit('%svoid f(%s) { %s return; }\n' % (DECL, parms, body))
        emit('%svoid f(%s) { %s }\n' % (DECL, parms, body))

    # 3. A MEMBER call in the same position: `this` is argument zero, so the
    #    receiver walks the argument file with the formals in front of it.
    for body, parms in (('p->v();', 'O* p'), ('p->va(k);', 'O* p, int k'),
                        ('p->va(7);', 'O* p'),
                        ('p->vb(j, k);', 'O* p, int j, int k'),
                        ('p->vb(k, j);', 'O* p, int j, int k'),
                        ('p->gi();', 'O* p'), ('p->ga(k);', 'O* p, int k'),
                        ('p->v();', 'int z, O* p')):
        emit('%svoid f(%s) { %s return; }\n' % (DECL, parms, body))
        emit('%svoid f(%s) { %s }\n' % (DECL, parms, body))

    # 4. A BRACE SCOPE around the call, with and without the `return;` — the
    #    scope depth is what the tail-call probe gets wrong in the other
    #    direction, so the two interact.
    for body, parms in (('g0();', ''), ('g1(a);', 'int a'),
                        ('g2(a, b);', 'int a, int b')):
        emit('%svoid f(%s) { { %s } return; }\n' % (DECL, parms, body))
        emit('%svoid f(%s) { { %s return; } }\n' % (DECL, parms, body))
        emit('%svoid f(%s) { { %s } }\n' % (DECL, parms, body))
        emit('%svoid f(%s) { { { %s } } return; }\n' % (DECL, parms, body))

    # 5. A VALUE tail after the call — `return <literal>;` is the framed
    #    `bl` + `li r3,k` and is not this row.
    for parms, k in (('', '5'), ('int a', '0'), ('int a', '-1')):
        emit('%sint f(%s) { g0(); return %s; }\n' % (DECL, parms, k))
        emit('%sint f(%s) { g0(); g0(); return %s; }\n' % (DECL, parms, k))

    # 6. An EMPTY body with a bare `return;`, and a call-free one — the
    #    neighbouring shapes the double-`3A` plumbing also reaches.
    emit('%svoid f() { return; }\n' % DECL)
    emit('%svoid f(int a) { return; }\n' % DECL)
    emit('%sint f(int a) { return a; }\n' % DECL)
