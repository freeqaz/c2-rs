# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter, and the driver fails if a fragment emits
# zero cases.
#
# ---- WFL: the designator step whose member is FLOATING POINT ----------------
#
# `99-chain-tail-load.py` sweeps the integer form of this step (`lwz r3,k(r3)`)
# and carries its `float` member only as a REFUSAL row. This fragment is that
# row promoted to the accepting side, and the axes it exists for are the ones no
# other fragment can reach:
#
#   * **THE WIDTH BIT FOLLOWS THE MEMBER, NOT THE RESULT.** `lfs` loads AND
#     converts, so a `float` member returned as a `double` is byte-identical to
#     the unpromoted body: the same `lfs f1,k(r3)`. Every promotion row below is
#     graded beside its unpromoted twin, so "read the result type" and "read the
#     load type" are two different answers on one pair of cases.
#   * **THE OTHER DIRECTION IS NOT FREE and must stay refused.** A `double`
#     member narrowed to a `float` result is `lfd f0 ; frsp f1,f0` — two words,
#     and the load's destination is f0. Wrong bytes, not missing coverage.
#   * **THE DESTINATION IS f1, NOT r3.** The instruction that would be emitted
#     by the integer sibling at the same displacement (`lwz r3,k(r3)`) is a
#     different register file entirely, and nothing about the surrounding body
#     differs, so no shape gate can see the mistake.
#   * **`_fltused`.** Every case here obliges the TU to carry the undefined
#     external, which is a SYMBOL and not an instruction: the failure is
#     `Port=Mismatch @ offset 12`, the COFF header's `NumberOfSymbols`, and it
#     is invisible to a `.text` compare. W36 lost one exactly this way. The
#     mixed TUs at the end of this fragment are where its PLACEMENT is graded —
#     it goes after the first FP-touching function's complete symbol group, and
#     "first FP-touching" now includes a framed body whose only FP is one load.
#   * **OFFSET 0 DOES NOT FOLD**, exactly as in the integer form; every `_a`
#     (offset-0) row is graded beside the address form of the same member, so
#     the fold cannot migrate from the add onto the designator.
#
# and the axes that vary something changing no operator and no shape, which is
# the class that has found seven live mis-emits:
#
#   * cv-qualification on the FP member and on the getter;
#   * the chain's DEPTH and whether its links carry arguments (Class A and B);
#   * `this` as the innermost receiver;
#   * an FP member of a NESTED sub-object, where the run folds.

DECL = (
    'struct In { float u; double v; float w; };\n'
    'struct M {\n'
    '  int a; float f; float g; double d; const float cf; const double cd;\n'
    '  In in; float arr[8]; double darr[4];\n'
    '};\n'
    'struct O {\n'
    '  int n;\n'
    '  O* Next(); O* Self(); O* SelfA(int);\n'
    '  M* gf(); const M* gcf(); M* gfa(int); M* gfb(int, int);\n'
    '  float* gpf(); double* gpd(); In* gin();\n'
    '};\n'
)

# The chain heads: depth, and link arguments (Class A with a literal, Class B
# with a formal). Each is followed by `->` and a designator below.
HEADS = (
    ('p->Next()->gf()', 'O* p'),
    ('p->Self()->Next()->gf()', 'O* p'),
    ('p->Self()->Self()->Next()->gf()', 'O* p'),
    ('p->Next()->gfa(7)', 'O* p'),
    ('p->Next()->gfa(k)', 'O* p, int k'),
    ('p->Next()->gfb(k, 3)', 'O* p, int k'),
    ('p->SelfA(k)->Next()->gf()', 'O* p, int k'),
    ('p->Next()->gcf()', 'O* p'),
)

# The FP members, with each one's own type. `f` is NOT at offset 0 and `arr[0]`
# is; the pair is what separates the fold from the designator.
MEMBERS = (
    ('f', 'float'),
    ('g', 'float'),
    ('d', 'double'),
    ('cf', 'float'),
    ('cd', 'double'),
    ('arr[0]', 'float'),
    ('arr[3]', 'float'),
    ('darr[0]', 'double'),
    ('darr[2]', 'double'),
)

# Multi-step designators — the offset RUN through a nested sub-object.
RUNS = (
    ('in.u', 'float'),
    ('in.v', 'double'),
    ('in.w', 'float'),
)


def cases(emit):
    # 1. THE MEMBER, over every head, at its own width — and the ADDRESS form of
    #    the same member beside it, which is the shipped `CallValue` row and
    #    folds at 0 where the load does not.
    for head, parms in HEADS:
        for mem, ty in MEMBERS:
            emit('%s%s f(%s) { return %s->%s; }\n' % (DECL, ty, parms, head, mem))
            if 'c' not in mem[:1]:
                emit('%s%s* f(%s) { return &%s->%s; }\n'
                     % (DECL, ty, parms, head, mem))

    # 2. THE PROMOTION, beside its unpromoted twin. A `float` member returned as
    #    a `double` is the IDENTICAL `lfs`; taking the width off the result type
    #    emits `lfd` here and is byte-wrong with nothing else changed.
    for head, parms in (HEADS[0], HEADS[1], HEADS[4]):
        for mem in ('f', 'g', 'cf', 'arr[0]', 'arr[3]'):
            emit('%sfloat  f(%s) { return %s->%s; }\n' % (DECL, parms, head, mem))
            emit('%sdouble f(%s) { return %s->%s; }\n' % (DECL, parms, head, mem))
        for mem in ('in.u', 'in.w'):
            emit('%sdouble f(%s) { return %s->%s; }\n' % (DECL, parms, head, mem))

    # 3. THE OFFSET RUN through a nested sub-object, load and address. A
    #    single-add recognizer emits the wrong displacement rather than
    #    refusing, so this is a wrong-bytes axis and not a coverage one.
    for head, parms in (HEADS[0], HEADS[1], HEADS[4]):
        for expr, ty in RUNS:
            emit('%s%s f(%s) { return %s->%s; }\n' % (DECL, ty, parms, head, expr))
            emit('%s%s* f(%s) { return &%s->%s; }\n' % (DECL, ty, parms, head, expr))
        emit('%sIn* f(%s) { return &%s->in; }\n' % (DECL, parms, head))

    # 4. NO OFFSET ADD AT ALL — a bare `30`, the same load at displacement 0.
    for head, parms in (('p->Next()->gpf()', 'O* p'),
                        ('p->Self()->Next()->gpf()', 'O* p'),
                        ('p->Next()->gpd()', 'O* p'),
                        ('p->Next()->gin()', 'O* p')):
        if head.endswith('gpf()'):
            emit('%sfloat  f(%s) { return *%s; }\n' % (DECL, parms, head))
            emit('%sfloat  f(%s) { return %s[0]; }\n' % (DECL, parms, head))
            emit('%sfloat  f(%s) { return %s[3]; }\n' % (DECL, parms, head))
            emit('%sdouble f(%s) { return *%s; }\n' % (DECL, parms, head))
            emit('%sfloat* f(%s) { return %s + 3; }\n' % (DECL, parms, head))
        elif head.endswith('gpd()'):
            emit('%sdouble f(%s) { return *%s; }\n' % (DECL, parms, head))
            emit('%sdouble f(%s) { return %s[0]; }\n' % (DECL, parms, head))
            emit('%sdouble f(%s) { return %s[2]; }\n' % (DECL, parms, head))
        else:
            emit('%sfloat  f(%s) { return %s->u; }\n' % (DECL, parms, head))
            emit('%sdouble f(%s) { return %s->v; }\n' % (DECL, parms, head))
            emit('%sfloat  f(%s) { return %s->w; }\n' % (DECL, parms, head))

    # 5. THE RECEIVER'S SPELLING — cv-qualification emits no `2C` at all, a
    #    pointer conversion emits one and still costs nothing.
    for parm, expr in (('O* p', 'p'), ('const O* p', 'p'), ('O* const p', 'p'),
                       ('void* v', '((O*)v)')):
        emit('%sfloat  f(%s) { return %s->Next()->gf()->f; }\n' % (DECL, parm, expr))
        emit('%sdouble f(%s) { return %s->Next()->gf()->d; }\n' % (DECL, parm, expr))
        emit('%sfloat  f(%s) { return %s->Next()->gf()->arr[0]; }\n' % (DECL, parm, expr))
        emit('%sfloat* f(%s) { return &%s->Next()->gf()->f; }\n' % (DECL, parm, expr))

    # 6. `this` AS THE INNERMOST RECEIVER.
    for ret, expr in (('float', 'Nx()->Next()->gf()->f'),
                      ('double', 'Nx()->Next()->gf()->d'),
                      ('double', 'Nx()->Next()->gf()->f'),
                      ('float', 'Nx()->Next()->gf()->arr[0]'),
                      ('float', 'Nx()->Next()->gf()->in.u'),
                      ('float*', '&Nx()->Next()->gf()->f')):
        emit('%sstruct H { O* Nx(); %s r(); };\n%s H::r() { return %s; }\n'
             % (DECL, ret, ret, expr))
    emit('%sstruct H { O* Nx(); float r(int k); };\n'
         'float H::r(int k) { return Nx()->Next()->gfa(k)->f; }\n' % DECL)

    # 7. `_fltused` AND ITS PLACEMENT. The symbol is emitted once per TU,
    #    immediately after the FIRST FP-touching function's complete symbol
    #    group — and this rung makes a FRAMED body FP-touching for the first
    #    time, so the group it follows is six symbols rather than three. Each
    #    ordering below puts the FP body in a different position, and an integer
    #    chain-tail load beside it is the neighbour that must NOT produce the
    #    symbol.
    INT = 'int  i1(O* p) { return p->Next()->gf()->a; }\n'
    FPL = 'float f1(O* p) { return p->Next()->gf()->f; }\n'
    FPD = 'double f2(O* p) { return p->Next()->gf()->d; }\n'
    LEAF = 'float lf(float a, float b) { return a * b; }\n'
    STORE = 'void sf(M* m, float v) { m->f = v; }\n'
    for order in ((FPL,), (FPL, FPD), (INT, FPL), (FPL, INT), (INT, FPL, INT),
                  (INT, INT, FPL), (FPL, LEAF), (LEAF, FPL), (FPL, STORE),
                  (STORE, FPL), (INT, FPL, LEAF), (INT, LEAF, FPL),
                  (FPD, INT, FPL), (INT, FPD, INT, FPL)):
        emit(DECL + ''.join(order))

    # 8. THE REFUSALS, so the gates are graded too. Each must be
    #    NotImplemented — never wrong bytes.
    #    The NARROWING: `lfd f0 ; frsp f1,f0`, two words into a scratch.
    for head, parms in (HEADS[0], HEADS[4]):
        emit('%sfloat f(%s) { return %s->d; }\n' % (DECL, parms, head))
        emit('%sfloat f(%s) { return (float)%s->d; }\n' % (DECL, parms, head))
        emit('%sfloat f(%s) { return %s->cd; }\n' % (DECL, parms, head))
        emit('%sfloat f(%s) { return %s->darr[1]; }\n' % (DECL, parms, head))
        emit('%sfloat f(%s) { return %s->in.v; }\n' % (DECL, parms, head))
    #    OUT of the FP file — `fctiwz` plus a spill through the frame.
    for mem in ('f', 'd', 'arr[0]', 'in.u'):
        emit('%sint f(O* p) { return p->Next()->gf()->%s; }\n' % (DECL, mem))
        emit('%sunsigned f(O* p) { return p->Next()->gf()->%s; }\n' % (DECL, mem))
    #    INTO the FP file from an integer member — WCO's `-load-convert`, which
    #    that rung's header records as having no witness. It has these.
    emit('%sfloat  f(O* p) { return p->Next()->gf()->a; }\n' % DECL)
    emit('%sdouble f(O* p) { return p->Next()->gf()->a; }\n' % DECL)
    #    An FP POST-OP pools a constant and adds — a whole second production.
    emit('%sfloat f(O* p) { return p->Next()->gf()->f + 1.0f; }\n' % DECL)
    emit('%sfloat f(O* p) { return p->Next()->gf()->f * 2.0f; }\n' % DECL)
    emit('%sdouble f(O* p) { return p->Next()->gf()->d + 1.0; }\n' % DECL)
    #    A VARIABLE subscript is not a literal add at all.
    emit('%sfloat f(O* p, int i) { return p->Next()->gf()->arr[i]; }\n' % DECL)
    #    A displacement past the signed 16-bit immediate.
    emit('%sstruct W { char pad[40000]; float far; };\n'
         'struct O2 { O* Next(); W* gw(); };\n'
         'float f(O2* p) { return p->Next()->gw()->far; }\n' % DECL)
    #    A `volatile` FP member — c2 emits the identical single `lfs`, and this
    #    port refuses it because the predicate it asks is the shared
    #    `is_fp_type`, whose volatile refusal belongs to the FORMAL position.
    #    Graded as a refusal so that the choice is a measured row and not a
    #    silent one.
    emit('%sstruct V { volatile float vf; volatile double vd; };\n'
         'struct O3 { O* Next(); V* gv(); };\n'
         'float f(O3* p) { return p->Next()->gv()->vf; }\n' % DECL)
    emit('%sstruct V { volatile float vf; volatile double vd; };\n'
         'struct O3 { O* Next(); V* gv(); };\n'
         'double f(O3* p) { return p->Next()->gv()->vd; }\n' % DECL)
