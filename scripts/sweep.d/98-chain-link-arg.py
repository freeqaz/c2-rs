# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter, and the driver fails if a fragment emits
# zero cases.
#
# ---- WCL: `p->a()->b(k)` — an argument on a LATER chain link ----------------
#
# `97-chained-call.py` sweeps the chain with every link nullary; its section 5
# holds the link-argument cells, which refused when it was written. This is that
# row built out, and the axes it exists for are the ones nothing else can reach:
#
#   * **THE MARSHALLING ORDER, which is the OPPOSITE of every other call's.** A
#     call whose argument list starts at slot 0 emits its moves highest
#     destination first; a chain link, whose slot 0 is the receiver a `bl` has
#     just left in r3, emits them LOWEST first. Every cell below with two or more
#     link arguments grades that directly, and the cells with two DISTINCT
#     formals grade it twice over — reversing the order and transposing the
#     source list are different mutations that agree on `gb2(j,j)` and disagree
#     everywhere else, which is why the same call appears with `(j,k)`, `(k,j)`
#     and `(j,j)`.
#   * **THE SLOT BASE.** The first explicit argument goes to r4, not r3. A grid
#     whose link argument is always `params[1]` cannot see this, because r4 is
#     both "argument slot 1" and "the register params[1] arrives in" — so the
#     leading-formal padding below moves the SOURCE register while the
#     destination stays put, and the `int z` first-parameter rows move it the
#     other way.
#   * **WHICH FORMAL BECOMES WHICH SAVED REGISTER**, crossed with the slot it
#     lands in. `r31, r30` is assigned in *parameter* order and the slots are
#     filled in *source* order, and those two orders are independent: `gb2(k,j)`
#     transposes one without the other.
#   * **LITERALS INTERLEAVED** rather than grouped — a constant argument is
#     `li r<slot>,k` in its own slot position, between the moves, and it costs no
#     callee-saved register at all, so a link whose arguments are all literals
#     leaves the body Class A with WCH's three-word prologue.
#
# and the axes that vary something changing no operator and no shape, which is
# the class that found six live mis-emits:
#
#   * the last link's RESULT TYPE and whether it is returned or discarded;
#   * cv-qualification on the receiver and on the argument;
#   * chain DEPTH, which moves the marshalling to a later `bl` without changing
#     it;
#   * an argument on the innermost link **as well**, which is the shipped
#     permutation path running in the same body as this one.

DECL = (
    'struct I {\n'
    '  int gi(); void vv(); unsigned gu(); bool gb(); char gc();\n'
    '  const I* self(); I* other();\n'
    '  int a1(int); void v1(int);\n'
    '  int a2(int, int); void v2(int, int);\n'
    '  int a3(int, int, int);\n'
    '  int a7(int, int, int, int, int, int, int);\n'
    '  I* ap(I*); int ac(const int&);\n'
    '  unsigned au(int); bool ab(int); char ah(int);\n'
    '  const I* as(int); I* ao(int); void av(int);\n'
    '};\n'
    'struct O {\n'
    '  I* Next(); I* Prev(); I* Last();\n'
    '  I* NextA(int); I* NextB(int, int);\n'
    '  O* Self(); O* Other(); O* SelfA(int);\n'
    '};\n'
)

# The leading formals that move the argument's SOURCE register without moving
# anything else about the body.
PADS = ('', 'int z, ', 'int z, int y, ', 'int z, int y, int x, ')


def cases(emit):
    # 1. ONE formal argument on the outer link, crossed with the formals in front
    #    of it. The destination is r4 in every one of these; the source walks
    #    r4, r5, r6, r7. A one-parameter fixture cannot tell the two apart.
    for pad in PADS:
        for meth in ('a1', 'v1'):
            ret = 'int' if meth == 'a1' else 'void'
            kw = 'return ' if meth == 'a1' else ''
            emit('%s%s f(%sO* p, int k) { %sp->Next()->%s(k); }\n'
                 % (DECL, ret, pad, kw, meth))
    # …and the receiver AFTER the argument, which swaps their registers.
    for pad in ('', 'int z, '):
        emit('%sint f(%sint k, O* p) { return p->Next()->a1(k); }\n' % (DECL, pad))
        emit('%sint f(%sint k, int j, O* p) { return p->Next()->a1(j); }\n' % (DECL, pad))

    # 2. WHICH of several formals the link takes — the save assignment is in
    #    parameter order and this varies which parameter is the live one.
    for n in (2, 3, 4):
        for pick in range(n):
            names = ', '.join('int k%d' % i for i in range(n))
            emit('%sint f(O* p, %s) { return p->Next()->a1(k%d); }\n'
                 % (DECL, names, pick))

    # 3. TWO arguments on the link — the order axis. Every ordered pair over two
    #    and three formals, plus the repeated one, which is the cell that
    #    separates "reverse the emission order" from "transpose the slot list".
    for pad in ('', 'int z, '):
        for i, j in ((0, 1), (1, 0), (0, 0), (1, 1)):
            emit('%sint f(%sO* p, int k0, int k1)'
                 ' { return p->Next()->a2(k%d, k%d); }\n' % (DECL, pad, i, j))
        for i, j in ((0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)):
            emit('%sint f(%sO* p, int k0, int k1, int k2)'
                 ' { return p->Next()->a2(k%d, k%d); }\n' % (DECL, pad, i, j))
        # …and the void form of each, so the tail cannot mask the setup.
        for i, j in ((0, 1), (1, 0)):
            emit('%svoid f(%sO* p, int k0, int k1)'
                 ' { p->Next()->v2(k%d, k%d); }\n' % (DECL, pad, i, j))

    # 4. LITERALS — alone, mixed, and at every position of a two- and
    #    three-argument list. The all-literal rows are CLASS A and must keep the
    #    three-word prologue; the mixed ones interleave the `li` between moves.
    for pad in ('', 'int z, '):
        for k in ('0', '1', '7', '-1', '-32768', '32767'):
            emit('%sint f(%sO* p) { return p->Next()->a1(%s); }\n' % (DECL, pad, k))
        for a, b in (('9', '8'), ('0', '1'), ('-1', '2')):
            emit('%sint f(%sO* p) { return p->Next()->a2(%s, %s); }\n'
                 % (DECL, pad, a, b))
        for slots in (('k0', '5'), ('5', 'k0')):
            emit('%sint f(%sO* p, int k0)'
                 ' { return p->Next()->a2(%s, %s); }\n' % ((DECL, pad) + slots))
        for slots in (('k0', 'k1', '5'), ('k0', '5', 'k1'), ('5', 'k0', 'k1'),
                      ('k0', '5', '6'), ('5', 'k0', '6'), ('5', '6', 'k0'),
                      ('k1', 'k0', '5'), ('5', 'k1', 'k0')):
            emit('%sint f(%sO* p, int k0, int k1)'
                 ' { return p->Next()->a3(%s, %s, %s); }\n' % ((DECL, pad) + slots))
    # The widest slot list that still fits: slot 0 is the receiver, so seven
    # explicit arguments reach r10 and an eighth would be stack-homed.
    emit('%sint f(O* p) { return p->Next()->a7(1, 2, 3, 4, 5, 6, 7); }\n' % DECL)
    emit('%sint f(O* p, int k) { return p->Next()->a7(k, 2, 3, 4, 5, 6, 7); }\n' % DECL)
    emit('%sint f(O* p, int k) { return p->Next()->a7(1, 2, 3, 4, 5, 6, k); }\n' % DECL)

    # 5. DEPTH moves the marshalling to a later `bl` and changes nothing else,
    #    and an argument on the INNERMOST link is the shipped permutation path
    #    running in the same body as this one.
    HEADS = ('p->Next()', 'p->Self()->Next()', 'p->Self()->Other()->Next()',
             'p->Other()->Self()->Next()', 'p->Self()->Self()->Other()->Next()')
    for head in HEADS:
        emit('%sint f(O* p, int k) { return %s->a1(k); }\n' % (DECL, head))
        emit('%sint f(O* p, int j, int k) { return %s->a2(j, k); }\n' % (DECL, head))
        emit('%sint f(O* p, int j, int k) { return %s->a2(k, j); }\n' % (DECL, head))
        emit('%svoid f(O* p, int k) { %s->v1(k); }\n' % (DECL, head))
    # arguments on the innermost link as well as on the outer one
    for inner, outer in (('NextA(j)', 'a1(k)'), ('NextA(k)', 'a1(j)'),
                         ('NextA(j)', 'a1(j)'), ('NextB(j, k)', 'gi()'),
                         ('NextB(k, j)', 'gi()')):
        emit('%sint f(O* p, int j, int k) { return p->%s->%s; }\n'
             % (DECL, inner, outer))
    emit('%sint f(O* p, int k) { return p->SelfA(k)->Next()->gi(); }\n' % DECL)
    emit('%sint f(O* p, int j, int k) { return p->SelfA(j)->Next()->a1(k); }\n' % DECL)

    # 6. THE LAST LINK'S RESULT TYPE, returned and discarded — the axis that
    #    found the `_fltused` mis-emit, now with an argument in front of it.
    for meth, ret in (('a1', 'int'), ('au', 'unsigned'), ('ab', 'bool'),
                      ('ah', 'char'), ('as', 'const I*'), ('ao', 'I*')):
        emit('%s%s f(O* p, int k) { return p->Next()->%s(k); }\n' % (DECL, ret, meth))
        emit('%svoid f(O* p, int k) { p->Next()->%s(k); }\n' % (DECL, meth))
        emit('%s%s f(O* p, int j, int k) { return p->Self()->Next()->%s(k); }\n'
             % (DECL, ret, meth))
    emit('%sI* f(O* p, I* q) { return p->Next()->ap(q); }\n' % DECL)
    emit('%sI* f(O* p, I* q, int z) { return p->Next()->ap(q); }\n' % DECL)
    emit('%svoid f(O* p, I* q) { p->Next()->ap(q); }\n' % DECL)

    # 7. THE RECEIVER'S AND THE ARGUMENT'S SPELLING — cv-qualification, which
    #    emits no `2C` at all, against a conversion that does. Neither changes an
    #    operator or a shape, which is what makes the axis worth running.
    for parm, expr in (('O* p', 'p'), ('const O* p', 'p'), ('O* const p', 'p'),
                       ('volatile O* p', 'p'), ('void* v', '((O*)v)')):
        emit('%sint f(%s, int k) { return %s->Next()->a1(k); }\n' % (DECL, parm, expr))
        emit('%sint f(%s, int j, int k) { return %s->Next()->a2(j, k); }\n'
             % (DECL, parm, expr))
    for aparm, aexpr in (('int k', 'k'), ('const int k', 'k'),
                         ('unsigned k', '(int)k'), ('short k', 'k'),
                         ('char k', 'k'), ('bool k', 'k'), ('int* k', '*k')):
        emit('%sint f(O* p, %s) { return p->Next()->a1(%s); }\n' % (DECL, aparm, aexpr))

    # 8. `this` AS THE RECEIVER. It is params[0] like any other formal, so the
    #    argument's source register still follows the formals in front of it —
    #    but it arrives from the `this` binding rather than from a `2D` formal.
    for args, call in ((('int k'), 'k'), (('int j, int k'), 'k'),
                       (('int j, int k'), 'j')):
        emit('%sstruct H { O* Nx(); int q(%s); void w(%s); };\n'
             'int H::q(%s) { return Nx()->Next()->a1(%s); }\n'
             % (DECL, args, args, args, call))
        emit('%sstruct H { O* Nx(); int q(%s); void w(%s); };\n'
             'void H::w(%s) { Nx()->Next()->v1(%s); }\n'
             % (DECL, args, args, args, call))
    emit('%sstruct H { O* Nx(); int q(int j, int k); };\n'
         'int H::q(int j, int k) { return Nx()->Next()->a2(j, k); }\n' % DECL)
    emit('%sstruct H { O* Nx(); int q(int j, int k); };\n'
         'int H::q(int j, int k) { return Nx()->Next()->a2(k, j); }\n' % DECL)

    # 9. THE REFUSALS, so the gates are graded too: a computed argument, a
    #    non-formal one, three live formals (the `__savegprlr_29` helper class),
    #    nine formals, a wide literal, and a permuted innermost call beside a
    #    save. Each must be NotImplemented, never wrong bytes.
    emit('%sint f(O* p, int k) { return p->Next()->a1(k + 1); }\n' % DECL)
    emit('%sint f(O* p, int j, int k) { return p->Next()->a2(j, k + 1); }\n' % DECL)
    emit('%sextern int g;\nint f(O* p) { return p->Next()->a1(g); }\n' % DECL)
    emit('%sint f(O* p, int i, int j, int k) { return p->Next()->a3(i, j, k); }\n' % DECL)
    emit('%sint f(O* p) { return p->Next()->a1(70000); }\n' % DECL)
    emit('%sint f(O* p) { return p->Next()->a1(-70000); }\n' % DECL)
    emit('%sint f(O* p) { return p->Next()->a7(1, 2, 3, 4, 5, 6, 7) + 1; }\n' % DECL)
    emit('%sint f(O* p, int j, int k) { return p->NextB(k, j)->a1(k); }\n' % DECL)
    emit('%sint f(O* p, int j, int k) { return p->NextA(j + 1)->a1(k); }\n' % DECL)
    emit('%sint f(int a, int b, int c, int d, int e, int g, int h, O* p)'
         ' { return p->Next()->a1(a); }\n' % DECL)
    emit('%sint f(O* p, int k) { return p->Next()->a1(k) + 1; }\n' % DECL)
    emit('%sint f(O* p, int k) { return p->Next()->ac(k); }\n' % DECL)

    # 10. SOURCE LINES and brace scopes — `GAPS.md` §6 instance #1's axis, which
    #     moves the one-byte line marker under the body without changing it.
    for pad in (0, 1, 3, 70):
        nl = '\n' * pad
        emit('%s%sint f(O* p, int k) { return p->Next()->a1(k); }\n' % (DECL, nl))
        emit('%s%sint f(O* p, int j, int k) { return p->Next()->a2(j, k); }\n'
             % (DECL, nl))
        emit('%s%svoid f(O* p, int k) { { p->Next()->v1(k); } }\n' % (DECL, nl))
