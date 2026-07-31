# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# ---- WLB: `g2(b, 7)` — the literal, and the formal that has to MOVE ---------
#
# WLA's neighbour and its whole residue at two slots. The order is not fixed:
# c2's default is highest destination first (the `li` in front) and it HOISTS the
# move ahead of the `li` exactly when the `li`'s destination is the register the
# move reads. One boolean, both values captured — and the axes here are the ones
# that can make that boolean flip without changing anything else:
#
#   * **THE SOURCE REGISTER**, walked across r4..r10 by padding the formals list.
#     r4 is the hoisted cell and every other value is the descending one, so a
#     grid that only ever passes the second parameter sees one of the two.
#   * **THE LITERAL'S SLOT.** In slot 1 it is this rung; in slot 0 the pair of
#     registers is different and the case is refused. Both are enumerated from
#     the same loop so the boundary comes out of the enumeration.
#   * **THREE SLOTS**, where a rule fitted to two mis-emits: `g3(c,b,7)` and
#     `g3(b,c,7)` follow the hoist and `g3(c,a,7)` breaks through r11 with the
#     `li` inside the walk. All three must refuse, and the sweep is where that is
#     checked against the real compiler rather than asserted.
#   * the `li` immediate at both ends and one past each, on both cells;
#   * the returned and void forms, and a pointer argument, none of which may
#     change the setup.

DECL = (
    'void g2(int, int); void g3(int, int, int);\n'
    'void g4(int, int, int, int);\n'
    'int rg2(int, int); int* pg2(int*, int);\n'
    'unsigned ug2(int, int);\n'
    'struct O { int m; void v(int, int); int a(int, int); };\n'
)

# Padding that walks the SOURCE register without touching anything else.
PADS = ('int a, ', 'int a, int z, ', 'int a, int z, int y, ',
        'int a, int z, int y, int x, ', 'int a, int z, int y, int x, int w, ',
        'int a, int z, int y, int x, int w, int v, ',
        'int a, int z, int y, int x, int w, int v, int u, ')

LITS = ('0', '1', '7', '-1', '-32768', '32767', '32768', '-32769', '70000')


def cases(emit):
    # 1. THE SOURCE REGISTER, r4 through r10. The first row is the hoisted cell
    #    and the rest are the descending one.
    for pad in PADS:
        emit('%svoid f(%sint b) { g2(b, 7); }\n' % (DECL, pad))
        emit('%sint f(%sint b) { return rg2(b, 7); }\n' % (DECL, pad))
    # …and the same with a trailing formal the call does not pass, so the moved
    # formal is not simply the last one.
    for pad in PADS[:4]:
        emit('%svoid f(%sint b, int t) { g2(b, 7); }\n' % (DECL, pad))

    # 2. THE LITERAL'S SLOT at two slots — slot 1 is this rung, slot 0 is not.
    for pad in PADS[:4]:
        emit('%svoid f(%sint b) { g2(7, b); }\n' % (DECL, pad))
        emit('%svoid f(%sint b) { g2(b, 7); }\n' % (DECL, pad))

    # 3. THE IMMEDIATE, on the hoisted cell and on a descending one.
    for k in LITS:
        emit('%svoid f(int a, int b) { g2(b, %s); }\n' % (DECL, k))
        emit('%svoid f(int a, int z, int b) { g2(b, %s); }\n' % (DECL, k))

    # 4. THREE SLOTS — every arrangement of two formals and a trailing literal
    #    over a three-parameter function. Two of these follow the hoist and one
    #    needs r11; all must refuse, and the enumeration is what makes that a
    #    boundary rather than a guess.
    for i, j in ((0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)):
        emit('%svoid f(int k0, int k1, int k2)'
             ' { g3(k%d, k%d, 7); }\n' % (DECL, i, j))
    # …and the literal in the middle and at the head of a three-slot list.
    for i, j in ((0, 1), (1, 0), (1, 2), (2, 1)):
        emit('%svoid f(int k0, int k1, int k2)'
             ' { g3(k%d, 7, k%d); }\n' % (DECL, i, j))
        emit('%svoid f(int k0, int k1, int k2)'
             ' { g3(7, k%d, k%d); }\n' % (DECL, i, j))
    # …and four slots, where the same question has more room.
    for i, j, k in ((1, 2, 3), (3, 2, 1), (2, 0, 1)):
        emit('%svoid f(int k0, int k1, int k2, int k3)'
             ' { g4(k%d, k%d, k%d, 7); }\n' % (DECL, i, j, k))

    # 5. THE RESULT TYPE and the pointer argument — neither may change the setup.
    emit('%sunsigned f(int a, int b) { return ug2(b, 7); }\n' % DECL)
    emit('%sunsigned f(int a, int z, int b) { return ug2(b, 7); }\n' % DECL)
    emit('%sint* f(int* p, int* q) { return pg2(q, 7); }\n' % DECL)
    emit('%svoid f(int* p, int* q) { pg2(q, 7); }\n' % DECL)
    emit('%svoid f(O* p, int j) { p->v(j, 7); }\n' % DECL)
    emit('%sint f(O* p, int j, int k) { return p->a(k, 7); }\n' % DECL)

    # 6. A COMPUTED argument beside the moved formal, and the framed form —
    #    both refused, and both one edit away from the taken cell.
    emit('%svoid f(int a, int b) { g2(b, a + 1); }\n' % DECL)
    emit('%svoid f(int a, int b) { g2(b + 1, 7); }\n' % DECL)
    emit('%sint gi; void f(int a, int b) { g2(gi, 7); }\n' % DECL)
    emit('%svoid f(int a, int b) { g2(b, 7); g2(a, b); }\n' % DECL)

    # 7. THE LOCALITY TELL — byte-identical bodies at varied file positions,
    #    on both cells.
    emit('%svoid d1(int a, int b) { g2(b, 7); }\n'
         'int p1(int a, int b) { return a + b; }\n'
         'void d2(int a, int b) { g2(b, 7); }\n'
         'void p2(int a, int b) { g2(a, b); }\n'
         'void d3(int a, int b) { g2(b, 7); }\n'
         'void e1(int a, int z, int b) { g2(b, 7); }\n'
         'void p3(int a) { g2(a, 1); }\n'
         'void e2(int a, int z, int b) { g2(b, 7); }\n' % DECL)
