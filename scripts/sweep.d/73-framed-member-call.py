# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter, and the driver fails if a fragment emits
# zero cases.
#
# ---- W41: the FRAMED member call, and the receiver's pointer conversion -----
#
# `70-framed.py` sweeps the free-function framed call `return g(a) + k`.
# `72-member-call.py` sweeps the member TAIL call. This fragment sweeps their
# product, which is where W41's two widenings live and which neither of the two
# can reach:
#
#   * the post-op OPERATOR crossed with the sign and magnitude of the literal.
#     `03` (SUB) was refused from W4b2 to W41 on the stated ground that
#     "subtraction is non-commutative and off the verified ADD frame" — an
#     argument, not a capture, and false: `- k` is the same `addi` with a negated
#     immediate. `04` (MUL) really is out of class. The only way to keep those
#     two apart is to vary the operator against the value, including the two
#     signed-16-bit boundaries in BOTH directions and both spellings of zero —
#     because `± 0` is not a framed call at all but the folded tail branch, and a
#     rule that got the fold wrong would emit a whole spurious frame.
#   * the RECEIVER'S FORMAL POSITION crossed with the post-op. The framed path
#     emits `mr r3,rN` for the receiver and `addi r3,r3,k` for the post-op, and
#     both write r3 — so a body where the receiver is not already in r3 is the
#     only case that orders the two writes. `GAPS.md` §6 records four separate
#     defects (#4, #5, #6, #8) where a formal's INDEX and its REGISTER were the
#     same number in every fixture; here they are crossed to nine formals, which
#     is one past the register file.
#   * the RESULT'S TYPE crossed with the literal. A pointer result scales the
#     literal by the pointee size, so `p->ge() - 1` on a 20-byte element is
#     `addi r3,r3,-20` and on a 1-byte one is `addi r3,r3,-1`; a struct size that
#     pushes the scaled value past the immediate must refuse, and the value that
#     decides it is nowhere in the source.
#   * the RECEIVER'S CONVERSION. `61-conversion-2c.py` sweeps `2C` in value
#     positions; nobody had ever put one between a receiver LOAD and its `99`
#     bind. Which C++ spellings even produce one was measured rather than assumed
#     (`work/w41/probe/p2.cpp`, `p4.cpp`): a cast to the receiver's own type is
#     folded away, cv-qualification emits none, a base adjustment is
#     `intrinsic 2113`. So the grid varies the ones that DO — `void*`,
#     `const_cast`, an unrelated pointee — crossed with the post-op and with the
#     receiver's position, because the conversion sits on the same value the
#     register move reads.
#   * an ARGUMENT beside the receiver crossed with the post-op, which the framed
#     shape refuses and the tail shape accepts. That boundary is one byte apart
#     in the IL and the two productions are in the same function.


def cases(emit):
    DECL = (
        'struct E1 { char c; };\n'
        'struct E4 { int a; };\n'
        'struct E20 { int a, b, c, d, e; };\n'
        'struct A {\n'
        '  int gi(); unsigned gu(); long gl(); int gic() const;\n'
        '  E1* g1(); E4* g4(); E20* g20();\n'
        '  int ga(int); int g2(int, int); int g3(int, int, int);\n'
        '  char gc(); short gs(); bool gb(); float gf(); double gd();\n'
        '};\n'
        'struct S { int a; void v(); int g(); };\n'
        'struct U { int u; void v(); int g(); };\n'
    )

    # 1. The post-op operator crossed with the literal, at both immediate
    #    boundaries and both spellings of zero. `+0`/`-0` must FOLD to the bare
    #    tail branch; `*k` must refuse; everything else is one `addi`.
    for op in ('+', '-'):
        for k in (0, 1, 2, 20, 255, 256, 32766, 32767, 32768, 40000, 65535,
                  65536, 100000):
            emit('%sint f(A* p) { return p->gi() %s %d; }\n' % (DECL, op, k))
    for k in (0, 1, 2, 20, 32767):
        emit('%sint f(A* p) { return p->gi() * %d; }\n' % (DECL, k))

    # 2. The receiver's formal position crossed with the post-op, to nine — one
    #    past the register file. The `mr r3,rN` and the `addi r3,r3,k` both write
    #    r3 and only a non-zero position orders them.
    for slot in range(0, 9):
        pre = ''.join('int q%d, ' % j for j in range(slot))
        for k in ('- 20', '+ 20', '- 1', '+ 0'):
            emit('%sint f(%sA* p) { return p->gi() %s; }\n' % (DECL, pre, k))

    # 3. The result's own type crossed with the literal. A POINTER result scales
    #    the literal by the pointee size — the multiplier is nowhere in the
    #    source — so the same `- 2` is three different immediates here, and a
    #    scaled value past the immediate must refuse.
    for ret, meth in (('int', 'gi'), ('unsigned', 'gu'), ('long', 'gl'),
                      ('E1*', 'g1'), ('E4*', 'g4'), ('E20*', 'g20')):
        for k in ('- 1', '- 2', '+ 3', '- 1000', '- 2000', '+ 40000', '- 0'):
            emit('%s%s f(A* p) { return p->%s() %s; }\n' % (DECL, ret, meth, k))
    # …and the narrow / boolean / floating results, which annotate the `41`
    # differently and must refuse rather than take an integer `addi`.
    for ret, meth in (('char', 'gc'), ('short', 'gs'), ('bool', 'gb'),
                      ('float', 'gf'), ('double', 'gd')):
        for k in ('- 1', '+ 20'):
            emit('%s%s f(A* p) { return p->%s() %s; }\n' % (DECL, ret, meth, k))

    # 4. cv-qualification of the pointer, the pointee and the method, crossed
    #    with the post-op — the axis that decides whether the receiver's TYPE
    #    tag is `86`, `A6` or `96`, and the `96` (volatile) one costs a frame.
    for recv in ('A* p', 'const A* p', 'A* const p', 'const A* const p',
                 'volatile A* p', 'A* volatile p'):
        meth = 'gic' if 'const A*' in recv else 'gi'
        for k in ('- 20', '+ 20', ''):
            emit('%sint f(%s) { return p->%s()%s; }\n' % (DECL, recv, meth, k))

    # 5. The receiver's `2C` pointer conversion — the spellings that actually
    #    produce one — crossed with the post-op, the result type and the
    #    receiver's formal position. The conversion sits on the same value the
    #    argument register move reads.
    CASTS = (
        ('void* v', '((S*)v)'),
        ('const S* p', 'const_cast<S*>(p)'),
        ('U* u', '((S*)(void*)u)'),
        ('char* c', '((S*)c)'),
        ('int* i', 'reinterpret_cast<S*>(i)'),
    )
    for parm, expr in CASTS:
        emit('%svoid f(%s) { %s->v(); }\n' % (DECL, parm, expr))
        emit('%sint f(%s) { return %s->g(); }\n' % (DECL, parm, expr))
        for k in ('- 20', '+ 20', '- 0', '- 40000'):
            emit('%sint f(%s) { return %s->g() %s; }\n' % (DECL, parm, expr, k))
        for slot in range(1, 4):
            pre = ''.join('int q%d, ' % j for j in range(slot))
            emit('%sint f(%s%s) { return %s->g() - 20; }\n'
                 % (DECL, pre, parm, expr))

    # 6. An explicit ARGUMENT beside the receiver, crossed with the post-op. The
    #    tail form takes a whole permutation; the framed form carries one operand
    #    stream and must refuse. Every permutation at each arity, so the boundary
    #    is swept and not sampled.
    import itertools as _it
    for n, meth in ((1, 'ga'), (2, 'g2'), (3, 'g3')):
        args = ', '.join('int a%d' % j for j in range(n))
        for perm in _it.permutations(range(n)):
            call = '%s(%s)' % (meth, ', '.join('a%d' % j for j in perm))
            emit('%sint f(A* p, %s) { return p->%s; }\n' % (DECL, args, call))
            for k in ('- 20', '+ 20', '- 0'):
                emit('%sint f(A* p, %s) { return p->%s %s; }\n'
                     % (DECL, args, call, k))

    # 7. The caller is a MEMBER function, so its own `this` occupies formal 0 and
    #    the receiver's index and its register are different numbers for the
    #    first time — the single most repeated defect in this project.
    for recv in ('A* p', 'int q, A* p', 'int q, int r, A* p'):
        for k in ('- 20', '+ 20', ''):
            emit('%sstruct H { int m(%s); };\n'
                 'int H::m(%s) { return p->gi()%s; }\n'
                 % (DECL, recv, recv, k))
        emit('%sstruct H { void m(%s); };\n'
             'void H::m(%s) { p->gi(); }\n' % (DECL, recv, recv))

    # 8. Source lines and brace scopes around the whole thing — `GAPS.md` §6
    #    instance #1's axis, and the statement here now has two productions that
    #    end it. Includes a body at source line 70, past the one-byte marker.
    for pad in (0, 1, 3, 70):
        nl = '\n' * pad
        emit('%s%sint f(A* p) { return p->gi() - 20; }\n' % (DECL, nl))
        emit('%s%sint f(A* p) {\n  return p->gi() - 20;\n}\n' % (DECL, nl))
        emit('%s%sint f(A* p) { { return p->gi() - 20; } }\n' % (DECL, nl))
        emit('%s%svoid f(A* p) { { ((S*)(void*)p)->v(); } }\n' % (DECL, nl))
