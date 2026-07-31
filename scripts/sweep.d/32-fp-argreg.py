# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- W27: the FP argument register file, numbered over FP parameters ALONE -------
    # The block above sweeps ONE non-FP leader in front of a uniform FP tail, which was
    # enough to catch the two mis-emits but not enough to grade the numbering: with one
    # leader the FP index is `position - 1` everywhere, so an off-by-one model and the
    # right one still agree on every case. What separates them is a non-FP parameter
    # *between* the FP ones, more than one of them, and the two FP widths interleaved —
    # none of which the old block generates. The cross product below does, and each
    # case is a body whose emission is now a positive claim rather than a refusal.
    FP_SLOTS = ('int', 'unsigned', 'char', 'long long', 'int*', 'float', 'double')
    for pat in ('FGF', 'GFF', 'FFG', 'GFGF', 'FGFG', 'GGFF', 'FGGF', 'GFGFG'):
        for gty in ('int', 'int*', 'char', 'long long'):
            for fty in ('float', 'double'):
                ps, fps = [], []
                for i, ch in enumerate(pat):
                    nm = 'p%d' % i
                    ps.append('%s %s' % (fty if ch == 'F' else gty, nm))
                    if ch == 'F':
                        fps.append(nm)
                decl = ', '.join(ps)
                # a bare return of each FP parameter — one `fmr`, or nothing
                for nm in fps:
                    emit("%s f(%s) { return %s; }\n" % (fty, decl, nm))
                # …and arithmetic over the FP ones, in both operand orders
                for op in ('+', '-', '*', '/'):
                    emit("%s f(%s) { return %s %s %s; }\n"
                             % (fty, decl, fps[0], op, fps[1]))
                    emit("%s f(%s) { return %s %s %s; }\n"
                             % (fty, decl, fps[1], op, fps[0]))
    # The two FP widths in ONE parameter list: the FP file is numbered width-agnostically
    # (`double a, float b` is f1, f2), so a model that separated the widths — or that
    # counted doubles as two registers, which is true of some other PPC ABIs — mismatches
    # on the second of each pair.
    for a in ('float', 'double'):
        for b in ('float', 'double'):
            for c in ('float', 'double'):
                emit("%s f(%s x, %s y, %s z) { return y; }\n" % (b, a, b, c))
                emit("%s f(%s x, %s y, %s z) { return z; }\n" % (c, a, b, c))
                emit("%s f(%s x, %s y, %s z) { return x + y; }\n" % (a, a, b, c))
    # A member function with a mixed list: `this` takes r3, so the GPR file shifts and
    # the FP file must not.
    for fty in ('float', 'double'):
        for decl, ret in ((('int a, %s b' % fty), 'b'),
                          (('%s a, int b, %s c' % (fty, fty)), 'c'),
                          (('int a, int b, %s c, %s d' % (fty, fty)), 'd')):
            emit("struct C { %s m(%s) const; };\n%s C::m(%s) const { return %s; }\n"
                     % (fty, decl, fty, decl, ret))
    # An FP parameter the body never reads still occupies its register and still
    # advances the count — the case the old gate refused outright.
    # (`nfp`, not `n` — `n` is this generator's own file counter, and a `for n in`
    # here silently rewinds it and overwrites already-written cases. That bug shipped
    # once, cost 1,233 cases, and reported a green sweep over the survivors; it is
    # recorded in `docs/GAPS.md` §6 and it recurred while this block was being
    # written. The printed case count is the only tell.)
    for fty in ('float', 'double'):
        for nfp in range(2, 6):
            ps = ', '.join('%s p%d' % (fty, i) for i in range(nfp))
            for i in range(nfp):
                emit("%s f(%s) { return p%d; }\n" % (fty, ps, i))
                if i + 1 < nfp:
                    emit("%s f(%s) { return p%d + p%d; }\n" % (fty, ps, i, i + 1))
    # The 13-register boundary, from both sides.
    for nfp in (12, 13, 14):
        ps = ', '.join('float p%d' % i for i in range(nfp))
        emit("float f(%s) { return p%d; }\n" % (ps, nfp - 1))
        emit("float f(%s) { return p0 + p1; }\n" % ps)

    # Tail calls: argument count, argument permutation, and computed arguments.
    emit("int g1(int);\nint f(int a){return g1(a);}\n")
    for p in ['a,b', 'b,a']:
        emit("int g2(int,int);\nint f(int a,int b){return g2(%s);}\n" % p)
    for p in ['a,b,c', 'a,c,b', 'b,a,c', 'b,c,a', 'c,b,a', 'c,a,b']:
        emit("int g3(int,int,int);\nint f(int a,int b,int c){return g3(%s);}\n" % p)
    for e in ['a+1', 'a-1', 'a+b', 'b+a', 'a-b', '1']:
        emit("int g1(int);\nint f(int a,int b){return g1(%s);}\n" % e)
