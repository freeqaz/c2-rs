# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter, and the driver fails if a fragment emits
# zero cases.
#
# ---- WCH: `return p->a()->b();` — the chained member call -------------------
#
# `72-member-call.py` and `73-framed-member-call.py` sweep the member call with
# ONE call in the body; `71-call-sequence.py` sweeps the Class A statement
# sequence with no receivers in it. This is the two crossed: N calls in one
# expression where each call's *result* is the next call's `this`, which is Class
# A because that result is already in r3 and nothing has to survive a `bl`.
#
# The axis this fragment exists for, and which none of those three can reach:
#
#   * **THE EMISSION ORDER, which is the IL's push run REVERSED.** The method
#     symbols stack LIFO, so `p->a()->b()` is `26 <b> 26 <a> B9 <p>` and `?a` is
#     called first. Every cell below whose links call *distinct* methods grades
#     that directly — the two callees have different symbol-table positions, so
#     an emitter that walked the pushes forwards writes two REL24 targets the
#     other way round. Depth three separates "reverse the run" from "swap the
#     two", which agree at depth two and disagree at every depth above it, so the
#     grid runs to four links with all-distinct methods.
#
# and the axes that vary something changing no operator and no shape, which is
# the class that found six live mis-emits two sessions ago:
#
#   * **the innermost receiver's REGISTER**, moved by leading `int` formals
#     without moving anything else (`GAPS.md` §6 records four defects where a
#     formal's index and its register were the same number in every fixture);
#   * **cv-qualification and the receiver's `2C` conversion**, the axes
#     `34-volatile-formal.py` and `73-framed-member-call.py` found live mis-emits
#     on, now with a chain behind the receiver instead of one call;
#   * **the last link's RESULT TYPE**, and whether it is returned or discarded —
#     the axis that found the `_fltused` mis-emit, where a `float` result the
#     body throws away still changes the TU's symbol count;
#   * **how many symbols the TU declared first**, i.e. the token WIDTH. The
#     head-run walk reads one token per push with `read_token_var`, whose width
#     choice moves every cursor after it, and a real translation unit's tokens
#     are four bytes wide while every other generated case's are two.

DECL = (
    'struct E20 { int a, b, c, d, e; };\n'
    'struct I {\n'
    '  int gi(); unsigned gu(); bool gb(); char gc(); short gs();\n'
    '  float gf(); double gd(); E20* ge(); void vv();\n'
    '  const I* self(); I* other();\n'
    '  int ga(int); int gb2(int, int); int gb3(int, int, int);\n'
    '};\n'
    'struct O {\n'
    '  I* Next(); I* Prev(); I* Last();\n'
    '  I* NextA(int); I* NextB(int, int); I* NextC(int, int, int);\n'
    '  O* Self(); O* Other(); O* Third();\n'
    '  int oi();\n'
    '};\n'
)


def cases(emit):
    # 1. THE EMISSION ORDER at depth two — every ordered pair of three distinct
    #    outer methods over three distinct inner ones, so no cell can be read
    #    right by accident, crossed with leading formals that move the receiver's
    #    register but not the shape.
    INNER = ('Next', 'Prev', 'Last')
    OUTER = (('gi', 'int'), ('self', 'const I*'), ('other', 'I*'))
    for pad in ('', 'int z, ', 'int z, int y, '):
        for a in INNER:
            for b, ret in OUTER:
                emit('%s%s f(%sO* p) { return p->%s()->%s(); }\n'
                     % (DECL, ret, pad, a, b))

    # 2. DEPTH. Three and four links with ALL-DISTINCT methods is the cell that
    #    separates "reverse the push run" from "swap the last two" — the two
    #    agree at depth two and disagree at every depth above it. The repeated
    #    forms grade the one-external case, where a per-link symbol would be an
    #    entry too many.
    DEEP = (
        ('p->Self()->Next()->gi()', 'int'),
        ('p->Other()->Prev()->self()', 'const I*'),
        ('p->Self()->Other()->Next()->gi()', 'int'),
        ('p->Self()->Other()->Third()->Next()->gi()', 'int'),
        ('p->Self()->Self()->Next()->gi()', 'int'),
        ('p->Self()->Self()->oi()', 'int'),
        ('p->Self()->Self()->Self()->oi()', 'int'),
        ('p->Self()->Next()->other()->other()->gi()', 'int'),
    )
    for expr, ret in DEEP:
        emit('%s%s f(O* p) { return %s; }\n' % (DECL, ret, expr))
        emit('%svoid f(O* p) { %s; }\n' % (DECL, expr))
        for pad in ('int z, ', 'int z, int y, int x, '):
            emit('%s%s f(%sO* p) { return %s; }\n' % (DECL, ret, pad, expr))

    # 3. `this` AS THE INNERMOST RECEIVER. It is `params[0]` and already in r3,
    #    but it arrives from the `this` binding rather than from a `2D` formal —
    #    a different route to the same empty setup. Both the value and the
    #    statement form, at three arities. The chain's methods are declared on
    #    `H` itself rather than inherited: an INHERITED method reached through
    #    `this` is the base-adjust intrinsic 2113, a different receiver
    #    production with its own lowering.
    for args in ('', 'int k', 'int k, int j'):
        decl = args or 'void'
        for expr in ('Nx()->gi()', 'Sf()->Nx()->gi()', 'Sf()->Ot()->Nx()->gi()'):
            emit('%sstruct H {\n  I* Nx(); H* Sf(); H* Ot();\n'
                 '  int q(%s); void v(%s);\n};\n'
                 'int H::q(%s) { return %s; }\n'
                 % (DECL, decl, decl, args, expr))
            emit('%sstruct H {\n  I* Nx(); H* Sf(); H* Ot();\n'
                 '  int q(%s); void v(%s);\n};\n'
                 'void H::v(%s) { %s; }\n'
                 % (DECL, decl, decl, args, expr))

    # 4. ARGUMENTS ON THE INNERMOST LINK are in class — `this` is slot 0 and the
    #    explicit arguments follow it, so the marshalling is the ordinary
    #    permutation. Every ordering of one, two and three arguments, so the
    #    cycles the permutation walk has to break are all reached.
    for pad in ('', 'int z, '):
        emit('%sint f(%sO* p, int k) { return p->NextA(k)->gi(); }\n' % (DECL, pad))
        emit('%sint f(%sint k, O* p) { return p->NextA(k)->gi(); }\n' % (DECL, pad))
        for j, k in ((0, 1), (1, 0)):
            emit('%sint f(%sO* p, int a0, int a1)'
                 ' { return p->NextB(a%d, a%d)->gi(); }\n' % (DECL, pad, j, k))
        for perm in ((0, 1, 2), (0, 2, 1), (1, 0, 2), (1, 2, 0), (2, 0, 1), (2, 1, 0)):
            emit('%sint f(%sO* p, int a0, int a1, int a2)'
                 ' { return p->NextC(a%d, a%d, a%d)->gi(); }\n'
                 % ((DECL, pad) + perm))
        # …and a literal, and a computed argument (which must refuse).
        emit('%sint f(%sO* p) { return p->NextA(7)->gi(); }\n' % (DECL, pad))
        emit('%sint f(%sO* p, int k) { return p->NextA(k + 1)->gi(); }\n' % (DECL, pad))

    # 5. ARGUMENTS ON A LATER LINK must refuse, and the two cells are different
    #    lowerings: a formal is Class B (`std r31 ; mr r31,r4 ; bl ; mr r4,r31`)
    #    and a literal is Class A into **r4** (`li r4,7`). Both, at both depths,
    #    with and without an argument on the innermost link as well.
    for outer in ('ga(k)', 'ga(7)', 'gb2(k, j)', 'gb2(7, 8)'):
        emit('%sint f(O* p, int k, int j) { return p->Next()->%s; }\n' % (DECL, outer))
        emit('%sint f(O* p, int k, int j) { return p->Self()->Next()->%s; }\n'
             % (DECL, outer))
        emit('%sint f(O* p, int k, int j) { return p->NextA(k)->%s; }\n'
             % (DECL, outer))
        emit('%svoid f(O* p, int k, int j) { p->Next()->%s; }\n' % (DECL, outer))

    # 6. THE LAST LINK'S RESULT TYPE, returned and discarded. `int`, `unsigned`,
    #    a pointer and `void` are in class; `float`/`double` oblige the TU to
    #    carry `_fltused` **even discarded**, and the narrow types widen.
    for meth, ret in (('gi', 'int'), ('gu', 'unsigned'), ('gb', 'bool'),
                      ('gc', 'char'), ('gs', 'short'), ('gf', 'float'),
                      ('gd', 'double'), ('ge', 'E20*'), ('self', 'const I*')):
        emit('%s%s f(O* p) { return p->Next()->%s(); }\n' % (DECL, ret, meth))
        emit('%svoid f(O* p) { p->Next()->%s(); }\n' % (DECL, meth))
        emit('%s%s f(O* p) { return p->Self()->Next()->%s(); }\n' % (DECL, ret, meth))
    emit('%svoid f(O* p) { p->Next()->vv(); }\n' % DECL)
    emit('%svoid f(O* p) { p->Self()->Next()->vv(); }\n' % DECL)

    # 7. THE INNERMOST RECEIVER'S SPELLING — cv-qualification, which emits no
    #    `2C` at all, against a conversion that does, against the designators
    #    that are a different production and must decline (a global, a
    #    dereference, a sub-object, another call's result).
    for parm, expr in (('O* p', 'p'), ('const O* p', 'p'), ('O* const p', 'p'),
                       ('volatile O* p', 'p'), ('void* v', '((O*)v)'),
                       ('char* c', '((O*)c)'), ('O** pp', '(*pp)'),
                       ('O* p', 'static_cast<O*>(p)')):
        emit('%sint f(%s) { return %s->Next()->gi(); }\n' % (DECL, parm, expr))
        emit('%sint f(%s) { return %s->Self()->Next()->gi(); }\n' % (DECL, parm, expr))
    emit('%sextern O g_o;\nint f() { return g_o.Next()->gi(); }\n' % DECL)
    emit('%sextern O* g_p;\nint f() { return g_p->Next()->gi(); }\n' % DECL)
    emit('%sstruct W { int pad; O* o; };\n'
         'int f(W* w) { return w->o->Next()->gi(); }\n' % DECL)
    emit('%sstruct W { O o; };\nint f(W* w) { return w->o.Next()->gi(); }\n' % DECL)
    emit('%sO* mk();\nint f() { return mk()->Next()->gi(); }\n' % DECL)

    # 8. THE `-then-` BOUNDARY. A construct after the chain is a different tail
    #    and every one of them must refuse: a literal post-op, a dereference of
    #    the result, a comparison, a branch, a second statement, an assignment
    #    destination. These are the siblings the rung leaves behind by name.
    for tailexpr in ('p->Next()->gi() + 1', 'p->Next()->gi() - 1',
                     'p->Next()->gi() * 2', 'p->Next()->gi() == 3',
                     'p->Next()->gi() + p->Prev()->gi()'):
        emit('%sint f(O* p) { return %s; }\n' % (DECL, tailexpr))
    emit('%sint f(O* p) { if (p->Next()->gi()) return 1; return 2; }\n' % DECL)
    emit('%svoid g2();\nvoid f(O* p) { p->Next()->vv(); g2(); }\n' % DECL)
    emit('%svoid g2();\nvoid f(O* p) { g2(); p->Next()->vv(); }\n' % DECL)
    emit('%sint f(O* p) { int x; x = p->Next()->gi(); return x; }\n' % DECL)
    emit('%sint f(O* p) { int x = p->Next()->gi(); return x; }\n' % DECL)
    emit('%sextern int g_i;\nvoid f(O* p) { g_i = p->Next()->gi(); }\n' % DECL)

    # 9. NINE formals — one past the register file, where the receiver is
    #    stack-homed and reading it is `lwz`, not a register move.
    for n in (1, 7, 8, 9, 10):
        pre = ''.join('int q%d, ' % j for j in range(n - 1))
        emit('%sint f(%sO* p) { return p->Next()->gi(); }\n' % (DECL, pre))
        emit('%sint f(%sO* p) { return p->Self()->Next()->gi(); }\n' % (DECL, pre))

    # 10. SOURCE LINES and brace scopes, including a body past the one-byte line
    #     marker — `GAPS.md` §6 instance #1's axis. The brace form matters here
    #     for the same reason it does in every other whole-body shape: the scope
    #     closes BETWEEN the statement end and the return branch.
    for pad in (0, 1, 3, 70):
        nl = '\n' * pad
        emit('%s%sint f(O* p) { return p->Next()->gi(); }\n' % (DECL, nl))
        emit('%s%sint f(O* p) {\n  return p->Next()->gi();\n}\n' % (DECL, nl))
        emit('%s%svoid f(O* p) { { p->Next()->vv(); } }\n' % (DECL, nl))
        emit('%s%svoid f(O* p) { { { p->Self()->Next()->vv(); } } }\n' % (DECL, nl))

    # 11. A NEIGHBOUR in the same TU, which is where the `/Gy` label counter and
    #     the symbol-table order are actually graded — a wrong per-function
    #     stride is invisible in a one-function TU. The chain contributes N
    #     callee externals, one per link, which is what a per-link symbol model
    #     would get wrong here and nowhere else.
    NEIGH = ('int g(int);\nint nb(int a) { return g(a) + 1; }\n',
             'int nb(int a) { return a + 1; }\n',
             'int g(int);\nvoid nb(int a) { g(a); g(a); }\n',
             'int nb(int a) { return a == 3; }\n')
    for nb in NEIGH:
        for expr in ('p->Next()->gi()', 'p->Self()->Other()->Next()->gi()'):
            emit('%sint f(O* p) { return %s; }\n%s' % (DECL, expr, nb))
            emit('%s%sint f(O* p) { return %s; }\n' % (DECL, nb, expr))
            emit('%sint f(O* p) { return %s; }\n'
                 'int f2(O* p) { return p->Prev()->gi(); }\n%s'
                 % (DECL, expr, nb))

    # 12. **A FOUR-BYTE TOKEN.** Every other case here declares a few dozen
    #     symbols, so every IL token takes the two-byte form; a real translation
    #     unit declares tens of thousands. The head-run walk reads one token per
    #     method push and its WIDTH moves every cursor after it, so a chain is
    #     exactly the shape where getting that wrong compounds per link. Three
    #     cases only — they are ~1 MB of source each and the axis is binary.
    WIDE = 'struct P {\n%s};\n' % ''.join(
        '  int w%d() const;\n' % j for j in range(33000))
    emit('%s%sint f(O* p) { return p->Next()->gi(); }\n' % (DECL, WIDE))
    emit('%s%sint f(O* p) { return p->Self()->Other()->Next()->gi(); }\n'
         % (DECL, WIDE))
    emit('%s%svoid f(O* p) { p->Self()->Next()->vv(); }\n' % (DECL, WIDE))
