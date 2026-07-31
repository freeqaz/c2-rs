# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# W31, the single-argument floating-point tail call. The axes are the ones that
# separate candidate rules rather than the ones that are easy to enumerate:
#
#   * the FP file's numbering — a non-FP formal BEFORE, BETWEEN and AFTER the FP
#     ones, and more than one of them. With a single non-FP leader the FP index
#     is `position - 1` everywhere and an off-by-one model still agrees on every
#     case, which is exactly how the W27 mis-emits survived their first sweep.
#   * the two widths interleaved in one parameter list, and the two conversions
#     at the boundary — `float`->`double` is free and `double`->`float` is an
#     `frsp` FUSED with the move, so a model that emitted `fmr` then `frsp` is
#     wrong by one instruction on every narrowing call from anything but f1.
#   * `this`, which takes r3 and displaces nothing in the FP file.
#   * the TU cross product with every other FP-touching shape, which is where
#     `_fltused` and the compiler-label counter become observable at all. That
#     coupling is what produced mis-emit #12 (`docs/CODEGEN_FP_ARGS.md` §4.1):
#     two green branches whose combination neither branch's fixtures contained.


def cases(emit):
    # ---- the register file: every FP/non-FP pattern, both widths, all four
    # return classes ---------------------------------------------------------
    for pat in ('F', 'GF', 'FG', 'FF', 'GFF', 'FGF', 'FFG', 'GGF', 'FGG',
                'GFGF', 'FGFG', 'FFF', 'GFFG'):
        for gty in ('int', 'int*', 'char', 'long long'):
            for fty in ('float', 'double'):
                decl = ', '.join(
                    ('%s p%d' % (gty if c == 'G' else fty, i))
                    for i, c in enumerate(pat))
                for i, c in enumerate(pat):
                    if c != 'F':
                        continue
                    for formal, ret in (('float', 'float'), ('double', 'double'),
                                        ('float', 'int'), ('double', 'void')):
                        body = ('g(p%d);' % i) if ret == 'void' \
                            else ('return g(p%d);' % i)
                        emit('%s g(%s);\n%s f(%s) { %s }\n'
                             % (ret, formal, ret, decl, body))
    # ---- the two widths in ONE parameter list, and the conversion pair ------
    # The FP file is width-agnostic (`double a, float b` is f1, f2), so a model
    # that separated the widths — or that counted a double as two registers,
    # which is true of some other PowerPC ABIs — mismatches on the second of
    # each pair. Crossed with the callee's own formal width, which is where the
    # free widening and the fused `frsp` live.
    for a in ('float', 'double'):
        for b in ('float', 'double'):
            for c in ('float', 'double'):
                decl = '%s x, %s y, %s z' % (a, b, c)
                for formal in ('float', 'double'):
                    for nm in ('x', 'y', 'z'):
                        emit('%s g(%s);\n%s f(%s) { return g(%s); }\n'
                             % (formal, formal, formal, decl, nm))
    # ---- `this` takes r3 and displaces nothing in the FP file ---------------
    for q in ('', ' const'):
        for fty in ('float', 'double'):
            for decl, arg in ((('%s a' % fty), 'a'),
                              (('int k, %s a' % fty), 'a'),
                              (('%s a, %s b' % (fty, fty)), 'b'),
                              (('%s a, int k, %s b' % (fty, fty)), 'b'),
                              (('int j, int k, %s a, %s b' % (fty, fty)), 'b')):
                emit('%s g(%s);\nstruct C { %s m(%s)%s; };\n'
                     '%s C::m(%s)%s { return g(%s); }\n'
                     % (fty, fty, fty, decl, q, fty, decl, q, arg))
    # ---- an FP formal the body never reads still occupies its register ------
    for k in range(2, 7):
        decl = ', '.join('float p%d' % i for i in range(k))
        for i in range(k):
            emit('float g(float);\nfloat f(%s) { return g(p%d); }\n' % (decl, i))
    # ---- the TU cross product: `_fltused` and the label counter -------------
    PRELUDE = ('struct S { int i; float f; double d; };\n'
               'float gf(float);\nint gi(int);\nvoid gv();\nvoid gv2();\n')
    SHAPE = {
        'fptail':   'float %s(float x, float y) { return gf(y); }',
        'fptailn':  'float %s(double x, double y) { return gf(y); }',
        'fptailv':  'void %s(float x, float y) { gf(y); }',
        'fpstore':  'void %s(S* p, float v) { p->f = v; }',
        'fpstored': 'void %s(S* p, double v) { p->d = v; }',
        'intleaf':  'int %s(int a, int b) { return a + b; }',
        'intstore': 'void %s(S* p, int v) { p->i = v; }',
        'framed':   'int %s(int a) { return gi(a) + 1; }',
        'seq':      'void %s() { gv(); gv2(); }',
        'empty':    'void %s() {}',
        'cmp':      'int %s(int a) { return a < 5; }',
        'voidtail': 'void %s() { gv(); }',
        'inttail':  'int %s(int a) { return gi(a); }',
        'load':     'int %s(S* p) { return p->i; }',
        'addr':     'int* %s(S* p) { return &p->i; }',
    }
    fps = ('fptail', 'fptailn', 'fptailv')
    for fp in fps:
        for other in SHAPE:
            for order in ((fp, other), (other, fp)):
                src = PRELUDE + ''.join(
                    (SHAPE[k] % ('f%d_%s' % (j, k))) + '\n'
                    for j, k in enumerate(order))
                emit(src)
    # …and a three-function TU with the FP tail call in each position beside a
    # framed function, which is the only shape whose `$M`/`$T` numbers make the
    # counter observable.
    for other in ('intleaf', 'intstore', 'empty', 'inttail', 'load', 'fpstore'):
        for pos in range(3):
            names = [other, 'framed']
            names.insert(pos, 'fptail')
            src = PRELUDE + ''.join(
                (SHAPE[k] % ('f%d_%s' % (j, k))) + '\n'
                for j, k in enumerate(names))
            emit(src)
