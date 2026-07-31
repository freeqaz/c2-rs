# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace.
#
# ---- what this axis is for ------------------------------------------------
#
# The control-flow rung is DECODE ONLY: it lowers nothing, so every case below
# must come back NotImplemented. That makes this fragment an unusual one — it is
# not enumerating a class that claims byte-exactness, it is enumerating the
# *boundary* of one that does not.
#
# It is worth running anyway, and for one reason: the failure this project keeps
# finding is a straight-line emitter accepting a body it did not understand. Any
# case here that reports MATCH instead of SKIP is a body with a branch in it that
# some shape recognizer took for a leaf, and that is a wrong-bytes emit waiting
# for the day the surrounding grammar widens by one token. The sweep is where a
# generated neighbour finds that, because a hand-written fixture is written by
# someone who already knows which shapes exist.
#
# The axes are (construct) x (what the branch guards) x (what the body computes),
# with the third axis crossed against the shapes that ARE emitted — a leaf, a
# tail call, an add chain — so every case is one construct away from something
# byte-exact.

# What the guarded/looped body computes. Every one of these ALONE is an emitted
# shape, so a case that matches has done so by ignoring the control flow.
BODIES = [
    'return 1;',
    'return a;',
    'return a + 1;',
    'return a + b;',
    'a = a - 1;',
]

# Conditions, spanning the three ways a condition reaches a branch: a bare value
# (`38`/`39` straight off a load), a relation (the W6 comparison feeding a branch
# with no `2C` convert), and the short-circuit forms (which the front end lowers
# to branches, so `1A`/`1B`/`1C` never appear).
CONDS = ['a', '!a', 'a > 0', 'a < b', 'a == 0', 'a != b', 'a && b', 'a || b']


def cases(emit):
    # ---- the diamond, with and without an else arm -------------------------
    for c in CONDS:
        for body in BODIES:
            emit('int f(int a, int b) { if (%s) { %s } return b; }\n' % (c, body))
        emit('int f(int a, int b) { if (%s) return a; else return b; }\n' % c)
        # …and the same condition as a returned VALUE, which for the relations is
        # the emitted W6 leaf. The pair is what separates "a comparison" from "a
        # branch": one of the two is byte-exact and the other must refuse.
        emit('int f(int a, int b) { return %s; }\n' % c)
        # the conditional expression — control flow inside an operand stream,
        # which the statement layer cannot see at all
        emit('int f(int a, int b) { return (%s) ? a : b; }\n' % c)

    # ---- back edges --------------------------------------------------------
    for c in CONDS:
        emit('int f(int a, int b) { while (%s) { a = a - 1; } return a; }\n' % c)
        emit('int f(int a, int b) { do { a = a - 1; } while (%s); return a; }\n' % c)
        emit('int f(int a, int b) { for (int i = 0; %s; i = i + 1) { b = b + i; } return b; }\n'
             % (c if c not in ('a && b', 'a || b') else 'i < a'))
    # break / continue / goto — all the same `3A <label>` as a return, so a
    # decoder that got one right got all four right, and a lowering that got one
    # wrong got all four wrong.
    emit('int f(int a) { while (a) { a = a - 1; if (a) break; } return a; }\n')
    emit('int f(int a) { while (a) { a = a - 1; if (a) continue; } return a; }\n')
    emit('int f(int a) { if (a) goto out; a = a + 1; out: return a; }\n')
    emit('int f(int a) { goto out; out: return a; }\n')

    # ---- switch ------------------------------------------------------------
    # Three more opcodes and a jump table. Swept at the arities where the front
    # end is known to change strategy (a single case, a dense run, a default) so
    # a future lowering has the boundary already enumerated.
    for n in (1, 2, 3, 5):
        arms = ''.join('case %d: return %d; ' % (k, k * 10) for k in range(1, n + 1))
        emit('int f(int a) { switch (a) { %sdefault: return 0; } }\n' % arms)
        emit('int f(int a) { switch (a) { %s} return 0; }\n' % arms)

    # ---- the control flow BESIDE an emitted shape --------------------------
    # The dangerous neighbourhood: one function in the TU is byte-exact and the
    # next has a branch. A gate that scanned a neighbourhood rather than parsing a
    # whole body would take the second for the first — which is the exact defect
    # the whole-body positive parser replaced, so it is swept rather than assumed
    # gone.
    NEIGHBOURS = [
        'int g(int a) { return a + 1; }',
        'int g(int a, int b) { return a + b; }',
        'void g();\nvoid h() { g(); }',
        'int g(int a) { return a > 0; }',
    ]
    for nb in NEIGHBOURS:
        for c in ('a', 'a > 0'):
            emit('%s\nint f(int a) { if (%s) return 1; return 2; }\n' % (nb, c))
            emit('int f(int a) { if (%s) return 1; return 2; }\n%s\n' % (c, nb))
            emit('%s\nint f(int a) { while (%s) { a = a - 1; } return a; }\n' % (nb, c))

    # ---- nesting -----------------------------------------------------------
    # Depth is what the scope-close invariant is checked against, and it is the
    # one field whose encoding above 0x7F is UNKNOWN. Sweep the shallow end where
    # the two readings agree, so that if a lowering ever lands the boundary is
    # already enumerated rather than discovered.
    for d in range(1, 7):
        opens = '{ ' * d
        closes = ' }' * d
        emit('int f(int a) { %sif (a) return 1;%s return 2; }\n' % (opens, closes))
        emit('int f(int a) { %swhile (a) { a = a - 1; }%s return a; }\n' % (opens, closes))
    # nested ifs, which allocate a label per level and join them in reverse
    for d in range(2, 6):
        head = ''.join('if (a > %d) { ' % k for k in range(d))
        emit('int f(int a) { %sreturn 1;%s return 0; }\n' % (head, ' }' * d))
