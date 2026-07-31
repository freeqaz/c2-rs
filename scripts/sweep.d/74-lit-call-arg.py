# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter, and the driver fails if a fragment emits
# zero cases.
#
# ---- WLA: a LITERAL argument in a multi-argument tail call ------------------
#
# `g3(a, b, 7)` is `li r5,7 ; b ?g3` — one instruction, no move, because every
# other slot's formal is already in the argument register it is being passed in.
# The row this takes is 4,792 functions on the 878-TU workload; what it declines
# is 733, and the two are separated by exactly one property, so the axes that
# matter here are the ones that move a formal off its own slot:
#
#   * **WHICH SLOT the literal occupies**, crossed with arity. The taken cell is
#     "the formals are in place"; a literal *before* a formal pushes that formal
#     out of its slot and the lowering grows a `mr`. Both sides are enumerated,
#     because the refusing side is where a widening would first emit wrong bytes
#     and `NotImplemented` is the only correct answer there.
#   * **THE `li` IMMEDIATE**, at both ends and one past each. 32767 is `li` and
#     32768 is `lis`+`ori`, and a fixture that only ever passes small constants
#     cannot see the boundary.
#   * **THE EMISSION ORDER of two and three literals**, which is DESCENDING
#     destination — the opposite of a chain link's. A single-literal grid agrees
#     with both rules; only the multi-literal cells separate them.
#   * **THE MEMBER FORM**, where `this` is the formal in slot 0. That is what
#     makes the row big (every one-argument member call is a two-argument list),
#     and it moves the literal's destination register without changing anything
#     else about the body.
#   * **A FORMAL THE CALL DOES NOT PASS**, sitting in the register the `li`
#     overwrites. It is dead across a tail branch; a live-value model that did
#     not know that would refuse or spill.
#   * the arity walk from 2 to 8 slots, which moves the destination through
#     r4..r10 without changing the rule.

DECL = (
    'void g2(int, int); void g3(int, int, int);\n'
    'void g4(int, int, int, int); void g5(int, int, int, int, int);\n'
    'void g6(int, int, int, int, int, int);\n'
    'void g7(int, int, int, int, int, int, int);\n'
    'void g8(int, int, int, int, int, int, int, int);\n'
    'int rg2(int, int); int rg3(int, int, int);\n'
    'int* pg2(int*, int); unsigned ug3(int, int, int);\n'
    'struct O {\n'
    '  int m;\n'
    '  int a1(int); void v1(int);\n'
    '  int a2(int, int); void v2(int, int);\n'
    '  int a3(int, int, int);\n'
    '  int c2(int, int) const;\n'
    '};\n'
)

# The trailing formals the call does not pass — one of them is the register the
# `li` overwrites, and it must be dead.
TAILS = ('', ', int t1', ', int t1, int t2')

LITS = ('0', '1', '7', '-1', '100', '-32768', '32767', '32768', '-32769',
        '65535', '70000')


def cases(emit):
    # 1. Arity 2..8, the literal in the LAST slot — the taken cell, walked
    #    across every argument register from r4 to r10.
    for n in range(2, 9):
        formals = ', '.join('int k%d' % i for i in range(n - 1))
        args = ', '.join('k%d' % i for i in range(n - 1)) + ', 7'
        emit('%svoid f(%s) { g%d(%s); }\n' % (DECL, formals, n, args))
        # …and with dead formals behind the passed ones.
        for tail in TAILS[1:]:
            emit('%svoid f(%s%s) { g%d(%s); }\n' % (DECL, formals, tail, n, args))

    # 2. THE LITERAL'S SLOT, at arity 2, 3 and 4 — every position, so the taken
    #    cell and the refused ones come from the same enumeration.
    for n in (2, 3, 4):
        for slot in range(n):
            formals = ', '.join('int k%d' % i for i in range(n - 1))
            parts = []
            src = 0
            for s in range(n):
                if s == slot:
                    parts.append('7')
                else:
                    parts.append('k%d' % src)
                    src += 1
            emit('%svoid f(%s) { g%d(%s); }\n'
                 % (DECL, formals, n, ', '.join(parts)))

    # 3. THE IMMEDIATE, at both ends of `li`'s field and one past each.
    for k in LITS:
        emit('%svoid f(int a, int b) { g3(a, b, %s); }\n' % (DECL, k))
        emit('%svoid f(int a) { g2(a, %s); }\n' % (DECL, k))

    # 4. TWO AND THREE LITERALS — the emission order. A single-literal grid
    #    cannot separate ascending from descending; these can.
    for a, b in (('5', '6'), ('6', '5'), ('0', '1'), ('-1', '2'), ('7', '7')):
        emit('%svoid f(int a) { g3(a, %s, %s); }\n' % (DECL, a, b))
        emit('%svoid f(int a, int b) { g4(a, b, %s, %s); }\n' % (DECL, a, b))
        emit('%svoid f() { g2(%s, %s); }\n' % (DECL, a, b))
    for a, b, c in (('4', '5', '6'), ('6', '5', '4'), ('0', '0', '0')):
        emit('%svoid f(int a) { g4(a, %s, %s, %s); }\n' % (DECL, a, b, c))
        emit('%svoid f() { g3(%s, %s, %s); }\n' % (DECL, a, b, c))
    # …and literals separated by a formal, which is the refusing side of the
    # same axis: the formal is out of its slot.
    emit('%svoid f(int a, int b) { g4(a, 7, b, 8); }\n' % DECL)
    emit('%svoid f(int a, int b) { g4(7, a, 8, b); }\n' % DECL)

    # 5. A REAL PERMUTATION beside the literal — refused, and the cell where a
    #    widening would first emit wrong bytes.
    emit('%svoid f(int a, int b) { g3(b, a, 7); }\n' % DECL)
    emit('%svoid f(int a, int b, int c) { g4(b, c, a, 7); }\n' % DECL)
    emit('%svoid f(int a, int b, int c) { g4(c, b, a, 7); }\n' % DECL)
    emit('%svoid f(int a, int b, int c) { g3(a, c, 7); }\n' % DECL)
    emit('%svoid f(int a, int b) { g3(b, b, 7); }\n' % DECL)

    # 6. THE MEMBER FORM — `this` is slot 0, so the literal's register moves by
    #    one and nothing else about the body changes.
    for meth, ret, kw in (('a1', 'int', 'return '), ('v1', 'void', ''),
                          ('a2', 'int', 'return '), ('v2', 'void', '')):
        n = 2 if meth in ('a1', 'v1') else 3
        formals = ', '.join('int k%d' % i for i in range(n - 2))
        sep = ', ' if formals else ''
        args = ', '.join(['k%d' % i for i in range(n - 2)] + ['7'])
        emit('%s%s f(O* p%s%s) { %sp->%s(%s); }\n'
             % (DECL, ret, sep, formals, kw, meth, args))
    emit('%sint f(const O* p, int j) { return p->c2(j, 7); }\n' % DECL)
    emit('%svoid f(O* p) { p->v2(3, 4); }\n' % DECL)
    emit('%sint f(O* p) { return p->a3(1, 2, 3); }\n' % DECL)
    emit('%sint f(O* p, int j) { return p->a2(7, j); }\n' % DECL)

    # 7. THE RESULT and the pointer parameter — a returned value and a pointer
    #    formal in slot 0 change nothing about the setup, and the cases that say
    #    so are the ones that would catch a rule keyed on either.
    emit('%sint f(int a, int b) { return rg3(a, b, 7); }\n' % DECL)
    emit('%sint f(int a) { return rg2(a, 7); }\n' % DECL)
    emit('%sunsigned f(int a, int b) { return ug3(a, b, 7); }\n' % DECL)
    emit('%sint* f(int* p) { return pg2(p, 7); }\n' % DECL)
    emit('%svoid f(int* p) { pg2(p, 7); }\n' % DECL)

    # 8. A COMPUTED argument, which is what the row's old key actually named,
    #    and a non-formal one. Both stay refused.
    emit('%svoid f(int a, int b) { g3(a, b, a + 1); }\n' % DECL)
    emit('%svoid f(int a, int b) { g3(a, a + b, 7); }\n' % DECL)
    emit('%sint gi; void f(int a, int b) { g3(a, b, gi); }\n' % DECL)
    emit('%sint gi; void f(int a, int b) { g3(a, gi, 7); }\n' % DECL)

    # 9. A FRAMED call with a literal — `callseq-multiarg-lit`, refused: the
    #    marshalling interleaves with the callee-saved copies there.
    emit('%svoid f(int a, int b) { g3(a, b, 7); g2(a, b); }\n' % DECL)
    emit('%svoid f(int a, int b) { g2(a, b); g3(a, b, 7); }\n' % DECL)
    emit('%svoid f(int a, int b) { g3(a, b, 7); g3(a, b, 8); }\n' % DECL)

    # 10. THE LOCALITY TELL — byte-identical bodies at varied file positions,
    #     with different neighbours between them.
    emit('%svoid d1(int a, int b) { g3(a, b, 7); }\n'
         'int p1(int a, int b) { return a + b; }\n'
         'void d2(int a, int b) { g3(a, b, 7); }\n'
         'void p2(int a, int b) { g2(a, b); }\n'
         'void d3(int a, int b) { g3(a, b, 7); }\n' % DECL)
