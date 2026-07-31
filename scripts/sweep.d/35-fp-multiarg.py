# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# W34, the multi-argument floating-point tail call. The axes are the ones that
# separate candidate rules, not the ones that are easy to enumerate:
#
#   * **the complete permutation grid at n = 2, 3 and 4.** The whole rung is a
#     permutation solver, and `docs/CODEGEN_ARG_PERM.md` §2 is the record of what
#     sampling three cells out of a grid costs: the published "one temp breaks
#     the cycle" rule is true of every cycle up to length three and false at
#     four, and the corpus that carried it had no cycle longer than three.
#     Enumerating the grid is the only way that class of error shows up.
#   * **two numberings that are both wrong to read positionally.** The SOURCE is
#     the FP file over the formals and the DESTINATION is the FP file over the
#     arguments, so a non-FP formal moves the first and not the second, and a
#     non-FP formal that is not passed moves neither. `GF`/`FGF`/`GGFF` with the
#     call permuting only the FP ones is where a positional model dies.
#   * **`this`**, which takes r3 and is outside the FP file — the case the
#     integer multi-argument rung cannot reach at all, because its `arg_sources`
#     indexes a formals list with `this` at index 0.
#   * **the two widths interleaved and the free widening**, crossed with the
#     permutation: a `double` takes one FP register, not two, so a width-pair
#     model mismatches on the second of every pair *and* on every move after it.
#   * **the TU cross product with every other FP-touching shape**, where
#     `_fltused` and the compiler-label counter become observable at all. That
#     coupling is mis-emit #12 (`docs/CODEGEN_FP_ARGS.md` §4.1): two green
#     branches whose combination neither branch's fixtures contained.
#
# The negative half is enumerated too, deliberately. A refusal that silently
# became an acceptance is how W31's `p1` row stopped being a refusal, and the
# only thing that catches the reverse — an acceptance that emits the wrong bytes
# — is compiling it.

import itertools


def cases(emit):
    # ---- the complete permutation grid, n = 2, 3, 4 -------------------------
    # Every cell, both widths, and both the value and the discarded-result form.
    # The in-class subset is the cells with at most one local minimum; the rest
    # must refuse, and compiling them is what proves the gate is on the minimum
    # count and not on the cycle length.
    for n in (2, 3, 4):
        names = [chr(ord('a') + i) for i in range(n)]
        for perm in itertools.permutations(names):
            for fty in ('float', 'double'):
                decl = ', '.join('%s %s' % (fty, x) for x in names)
                args = ', '.join(perm)
                ctys = ', '.join([fty] * n)
                emit('%s g(%s);\n%s f(%s) { return g(%s); }\n'
                     % (fty, ctys, fty, decl, args))
            decl = ', '.join('float %s' % x for x in names)
            args = ', '.join(perm)
            emit('void g(%s);\nvoid f(%s) { g(%s); }\n'
                 % (', '.join(['float'] * n), decl, args))
    # …and the five-argument cycles, where the GPR file's own grid first split
    # 8/16 on the local-minimum count. Only the single cycles, to keep the case
    # count honest.
    for perm in itertools.permutations('abcde'):
        # a single 5-cycle: no element is a fixed point and the map is one orbit
        ix = {c: i for i, c in enumerate('abcde')}
        sigma = [ix[c] for c in perm]
        seen, at, k = set(), 0, 0
        while at not in seen:
            seen.add(at)
            at = sigma[at]
            k += 1
        if k != 5:
            continue
        emit('float g(float,float,float,float,float);\n'
             'float f(float a, float b, float c, float d, float e)'
             ' { return g(%s); }\n' % ', '.join(perm))

    # ---- the two numberings, which come apart in opposite directions --------
    # `G` is a non-FP formal. The call permutes the FP formals only, so the GPR
    # file never moves — but the FP SOURCES are renumbered by every `G` removed
    # from the count while the DESTINATIONS are not.
    for pat in ('GFF', 'FGF', 'FFG', 'GGFF', 'FGFG', 'GFFG', 'FFGG', 'GFFF',
                'FGFF', 'FFGF', 'FFFG'):
        fps = [i for i, c in enumerate(pat) if c == 'F']
        for gty in ('int', 'int*', 'char', 'long long'):
            for fty in ('float', 'double'):
                decl = ', '.join(
                    ('%s p%d' % (gty if c == 'G' else fty, i))
                    for i, c in enumerate(pat))
                for perm in itertools.permutations(fps):
                    args = ', '.join('p%d' % i for i in perm)
                    emit('%s g(%s);\n%s f(%s) { return g(%s); }\n'
                         % (fty, ', '.join([fty] * len(fps)), fty, decl, args))

    # ---- the widths interleaved in ONE parameter list, crossed with the
    # permutation, and the free `float`->`double` widening at the boundary -----
    for a in ('float', 'double'):
        for b in ('float', 'double'):
            for c in ('float', 'double'):
                decl = '%s x, %s y, %s z' % (a, b, c)
                for cty in ('float', 'double'):
                    for perm in itertools.permutations('xyz'):
                        emit('%s g(%s,%s,%s);\n%s f(%s) { return g(%s); }\n'
                             % (cty, cty, cty, cty, cty, decl, ', '.join(perm)))

    # ---- `this` is outside the FP file entirely ----------------------------
    for q in ('', ' const'):
        for fty in ('float', 'double'):
            for decl, names in ((('%s a, %s b' % (fty, fty)), 'ab'),
                                (('int k, %s a, %s b' % (fty, fty)), 'ab'),
                                (('%s a, int k, %s b' % (fty, fty)), 'ab'),
                                (('%s a, %s b, %s c' % (fty, fty, fty)), 'abc')):
                n = len(names)
                for perm in itertools.permutations(names):
                    emit('%s g(%s);\nstruct C { %s m(%s)%s; };\n'
                         '%s C::m(%s)%s { return g(%s); }\n'
                         % (fty, ', '.join([fty] * n), fty, decl, q,
                            fty, decl, q, ', '.join(perm)))

    # ---- the result class is not this rung's business ----------------------
    for rty in ('float', 'double', 'int', 'void', 'char', 'int*'):
        for fty in ('float', 'double'):
            for perm in ('ab', 'ba'):
                body = ('g(%s);' if rty == 'void' else 'return g(%s);') \
                    % ', '.join(perm)
                emit('%s g(%s,%s);\n%s f(%s a, %s b) { %s }\n'
                     % (rty, fty, fty, rty, fty, fty, body))

    # ---- the OTHER register file, which must not move -----------------------
    # The gate is not "every argument is floating-point", it is "no GPR argument
    # moves" — and the GPR destination is `r(2+slot)` counting the FP arguments'
    # slots while the source is `r(base+ix)` in the caller's own numbering, with
    # `base` = r4 for a member. Both numberings count an FP parameter as
    # occupying a slot it does not fill, which is the half of §0 that a packed
    # model gets wrong. Enumerate every GPR/FP pattern up to five arguments,
    # forward the whole list unchanged (so the GPRs must all stay put), and then
    # permute the FP ones only.
    for k in range(2, 6):
        for bits in range(1, (1 << k) - 1):
            pat = ''.join('F' if (bits >> i) & 1 else 'G' for i in range(k))
            fps = [i for i, c in enumerate(pat) if c == 'F']
            for gty in ('int', 'void*', 'char*'):
                for fty in ('float', 'double'):
                    decl = ', '.join(
                        ('%s p%d' % (gty if c == 'G' else fty, i))
                        for i, c in enumerate(pat))
                    ctys = ', '.join((gty if c == 'G' else fty) for c in pat)
                    args = ', '.join('p%d' % i for i in range(k))
                    emit('void g(%s);\nvoid f(%s) { g(%s); }\n'
                         % (ctys, decl, args))
                    # …and with the FP arguments permuted in place. Only the FP
                    # file moves, and the capture says its schedule is then the
                    # pure-FP one.
                    for perm in itertools.permutations(fps):
                        if tuple(perm) == tuple(fps):
                            continue
                        it = iter(perm)
                        args = ', '.join(
                            ('p%d' % next(it)) if c == 'F' else ('p%d' % i)
                            for i, c in enumerate(pat))
                        emit('void g(%s);\nvoid f(%s) { g(%s); }\n'
                             % (ctys, decl, args))
    # …and the member form, where `this` takes r3 and every explicit GPR formal
    # sits one register above the slot the call wants it in — so forwarding a
    # member's own integer formal MOVES and must refuse, while its floating-point
    # formals do not.
    for pat in ('GF', 'FG', 'GFF', 'FGF', 'FFG', 'FF', 'FFF'):
        for fty in ('float', 'double'):
            decl = ', '.join(
                ('int p%d' % i) if c == 'G' else ('%s p%d' % (fty, i))
                for i, c in enumerate(pat))
            ctys = ', '.join('int' if c == 'G' else fty for c in pat)
            args = ', '.join('p%d' % i for i in range(len(pat)))
            for q in ('', ' const'):
                emit('void g(%s);\nstruct C { void m(%s)%s; };\n'
                     'void C::m(%s)%s { g(%s); }\n'
                     % (ctys, decl, q, decl, q, args))

    # ---- refusals that must STAY refusals -----------------------------------
    # Each of these is a shape c2 emits differently, and each is one token away
    # from an accepted one. They are here because the failure mode of a gate is
    # silent: `w31_fp_tail_neg.cpp`'s two-file permutation row stopped being a
    # refusal the moment this rung landed, and only a census run said so.
    NEG = [
        # the two files at once, in both orders and at three arities
        'int g(int,int,float,float);\n'
        'int f(int a,int b,float c,float d) { return g(b,a,d,c); }\n',
        'int g(float,float,int,int);\n'
        'int f(float a,float b,int c,int d) { return g(b,a,d,c); }\n',
        'int g(int,float,int,float);\n'
        'int f(int a,float b,int c,float d) { return g(c,d,a,b); }\n',
        'int g(int,int,int,float,float,float);\n'
        'int f(int a,int b,int c,float d,float e,float h)'
        ' { return g(c,a,b,h,d,e); }\n',
        # only the GPR file moves — no interleaving at all, and still refused
        'int g(int,float,int,float);\n'
        'int f(int a,int b,float c,float d) { return g(a,c,b,d); }\n',
        # a narrowing inside a permutation
        'float g(float,float);\nfloat f(double a,double b) { return g(b,a); }\n',
        'float g(float,float,float);\n'
        'float f(double a,double b,double c) { return g(b,c,a); }\n',
        'float g(float,double);\nfloat f(double a,double b) { return g(b,a); }\n',
        # a computed argument beside a bare one, both ways round
        'float g(float,float);\nfloat f(float a,float b) { return g(a+b,a); }\n',
        'float g(float,float);\nfloat f(float a,float b) { return g(a,a*b); }\n',
        # an FP literal in argument position
        'float g(float,float);\nfloat f(float a) { return g(a,1.5f); }\n',
        'float g(float,float);\nfloat f(float a) { return g(2.0f,a); }\n',
        # a value passed twice
        'float g(float,float);\nfloat f(float a,float b) { return g(a,a); }\n',
        'float g(float,float,float);\n'
        'float f(float a,float b) { return g(b,a,b); }\n',
        # a source outside the destination range (a shift, not a permutation)
        'float g(float,float);\n'
        'float f(float a,float b,float c) { return g(b,c); }\n',
        'float g(float,float);\n'
        'float f(float a,float b,float c) { return g(c,b); }\n',
        # a conversion applied to the RESULT
        'float g(float,float);\ndouble f(float a,float b) { return g(b,a); }\n',
        # a global rather than a formal
        'float gv;\nfloat g(float,float);\n'
        'float f(float a) { return g(gv,a); }\n',
        # across the files: an int formal converted to float in the FP file
        'float g(float,float);\n'
        'float f(int a,float b) { return g(b,a); }\n',
    ]
    for src in NEG:
        emit(src)

    # ---- the TU cross product: `_fltused` and the compiler-label counter ----
    PRELUDE = ('struct S { int i; float f; double d; };\n'
               'float gf(float);\nfloat gf2(float,float);\n'
               'float gf3(float,float,float);\n'
               'int gi(int);\nvoid gv();\nvoid gv2();\n')
    SHAPE = {
        'fpmulti':   'float %s(float x, float y) { return gf2(y, x); }',
        'fpmultiid': 'float %s(float x, float y) { return gf2(x, y); }',
        'fpmulti3':  'float %s(float x, float y, float z) { return gf3(y, z, x); }',
        'fpmultiv':  'void %s(float x, float y) { gf2(y, x); }',
        'fptail':    'float %s(float x, float y) { return gf(y); }',
        'fpstore':   'void %s(S* p, float v) { p->f = v; }',
        'fpleaf':    'float %s(float a, float b) { return a * b; }',
        'intleaf':   'int %s(int a, int b) { return a + b; }',
        'framed':    'int %s(int a) { return gi(a) + 1; }',
        'seq':       'void %s() { gv(); gv2(); }',
        'empty':     'void %s() {}',
        'voidtail':  'void %s() { gv(); }',
        'inttail':   'int %s(int a) { return gi(a); }',
        'load':      'int %s(S* p) { return p->i; }',
        'cmp':       'int %s(int a) { return a < 5; }',
    }
    mine = ('fpmulti', 'fpmultiid', 'fpmulti3', 'fpmultiv')
    for fp in mine:
        for other in SHAPE:
            for order in ((fp, other), (other, fp)):
                emit(PRELUDE + ''.join(
                    (SHAPE[k] % ('f%d_%s' % (j, k))) + '\n'
                    for j, k in enumerate(order)))
    # …and in each of three positions beside a framed function, which is the only
    # shape whose `$M`/`$T` numbers render the label counter at all.
    for other in ('intleaf', 'empty', 'inttail', 'load', 'fpstore', 'fptail'):
        for pos in range(3):
            names = [other, 'framed']
            names.insert(pos, 'fpmulti')
            emit(PRELUDE + ''.join(
                (SHAPE[k] % ('f%d_%s' % (j, k))) + '\n'
                for j, k in enumerate(names)))
