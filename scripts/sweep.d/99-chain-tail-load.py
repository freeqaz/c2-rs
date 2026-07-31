# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter, and the driver fails if a fragment emits
# zero cases.
#
# ---- WCO: one DESIGNATOR STEP on a chained call's pointer result -------------
#
# `97-chained-call.py` sweeps the chain that ENDS at its outermost call and
# `98-chain-link-arg.py` the arguments on its links. This is the one instruction
# that may follow it, and the axes it exists for are the ones no other fragment
# can reach:
#
#   * **OFFSET 0 IS NOT THE SAME FACT IN THE TWO FORMS.** `&chain()->first` emits
#     nothing at all — the `addi` folds — while `chain()->first` still emits
#     `lwz r3,0(r3)`, because a memory read at displacement 0 is still a memory
#     read. Every `_a` (offset-0) row below is graded in BOTH forms so that the
#     fold cannot be moved from the add onto the designator.
#   * **THE OFFSET RUN FOLDS INTO ONE DISPLACEMENT.** A nested member is `27·27`
#     and a constant subscript is `27·28`; both sum. A recognizer that took one
#     add — which is exactly what the indirect-load leaf carried until W35, worth
#     5,161 functions there — emits the wrong displacement rather than refusing,
#     so this is a wrong-bytes axis and not a coverage one.
#   * **THE DISPLACEMENT IS THE SUM, at every position of the run.** The member
#     offsets below are deliberately distinct and non-zero in both orders, so
#     "take the first add", "take the last add" and "sum them" are three
#     different answers on the same case.
#   * **THE LOADED WIDTH.** `char`/`short`/`long long`/`float` members are
#     `lbz`/`lhz`/`ld`/`lfs`, not `lwz`. Those must be NotImplemented, never a
#     wrong `lwz` — the refusal rows are as load-bearing as the accepting ones.
#
# and the axes that vary something changing no operator and no shape, which is
# the class that has found six live mis-emits:
#
#   * cv-qualification on the receiver, on the member and on the method;
#   * the chain's DEPTH and whether its links carry arguments (Class A and B);
#   * a POINTER member against an `int` one — same `lwz`, different type bytes;
#   * `this` as the innermost receiver.

DECL = (
    'struct In { int p; int q; };\n'
    'struct M {\n'
    '  int a; int m; int b; int* pi; const int c; In in; int arr[8];\n'
    '  char ch; short sh; long long ll; float fl;\n'
    '};\n'
    'struct O {\n'
    '  int n;\n'
    '  O* Next(); O* Self(); O* SelfA(int);\n'
    '  M* gf(); const M* gcf(); M* gfa(int); M* gfb(int, int);\n'
    '  int* gpi(); In* gin();\n'
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
    ('p->Next()->gfb(j, k)', 'O* p, int j, int k'),
    ('p->SelfA(k)->Next()->gf()', 'O* p, int k'),
    ('p->Next()->gcf()', 'O* p'),
)

# The designators, with the type each yields. `a` is at offset 0 and every other
# member is not: the pair is what separates the fold from the designator.
MEMBERS = (
    ('a', 'int'),
    ('m', 'int'),
    ('b', 'int'),
    ('pi', 'int*'),
    ('c', 'int'),
)

# Multi-step designators — the offset RUN. Each has two adds at two distinct
# non-zero offsets, in both orders where the type allows.
RUNS = (
    ('in.p', 'int'),
    ('in.q', 'int'),
    ('arr[0]', 'int'),
    ('arr[2]', 'int'),
    ('arr[7]', 'int'),
)


def cases(emit):
    # 1. THE PAIR, over every head. The load form and the address form of the
    #    same member, so offset 0 is graded in both and the fold cannot migrate.
    for head, parms in HEADS:
        for mem, ty in MEMBERS:
            emit('%s%s f(%s) { return %s->%s; }\n' % (DECL, ty, parms, head, mem))
            if mem != 'c':
                emit('%s%s* f(%s) { return &%s->%s; }\n'
                     % (DECL, ty, parms, head, mem))
        # …and the chain with NO designator at all, which is the shipped
        # `-whole` row and must not move.
        emit('%sM* f(%s) { return %s; }\n' % (DECL, parms, head))

    # 2. THE OFFSET RUN, load and address, over the two simplest heads. `in.p`
    #    and `in.q` differ only in the SECOND add; `arr[0]` and `arr[7]` only in
    #    the second too, and `arr[0]`'s second add is ZERO, which is the cell
    #    that separates "sum the run" from "take the last add".
    for head, parms in (HEADS[0], HEADS[1], HEADS[4]):
        for expr, ty in RUNS:
            emit('%s%s f(%s) { return %s->%s; }\n' % (DECL, ty, parms, head, expr))
            emit('%s%s* f(%s) { return &%s->%s; }\n'
                 % (DECL, ty, parms, head, expr))
        # A whole sub-object's address is the run with no load at its end.
        emit('%sIn* f(%s) { return &%s->in; }\n' % (DECL, parms, head))

    # 3. NO OFFSET ADD AT ALL — a bare `30`, which is the same `lwz r3,0(r3)`.
    #    A recognizer anchored on the offset add alone cannot reach these.
    for head, parms in (('p->Next()->gpi()', 'O* p'),
                        ('p->Self()->Next()->gpi()', 'O* p'),
                        ('p->Next()->gin()', 'O* p')):
        if head.endswith('gpi()'):
            emit('%sint f(%s) { return *%s; }\n' % (DECL, parms, head))
            emit('%sint f(%s) { return %s[0]; }\n' % (DECL, parms, head))
            emit('%sint f(%s) { return %s[3]; }\n' % (DECL, parms, head))
            emit('%sint* f(%s) { return %s + 3; }\n' % (DECL, parms, head))
        else:
            emit('%sint f(%s) { return %s->p; }\n' % (DECL, parms, head))
            emit('%sint f(%s) { return %s->q; }\n' % (DECL, parms, head))

    # 4. THE RECEIVER'S SPELLING — cv-qualification emits no `2C` at all, a
    #    pointer conversion emits one and still costs nothing. Neither changes
    #    an operator or a shape.
    for parm, expr in (('O* p', 'p'), ('const O* p', 'p'), ('O* const p', 'p'),
                       ('void* v', '((O*)v)')):
        emit('%sint f(%s) { return %s->Next()->gf()->m; }\n' % (DECL, parm, expr))
        emit('%sint* f(%s) { return &%s->Next()->gf()->b; }\n' % (DECL, parm, expr))
        emit('%sint f(%s) { return %s->Next()->gf()->a; }\n' % (DECL, parm, expr))

    # 5. `this` AS THE INNERMOST RECEIVER — params[0] like any other formal, but
    #    arriving from the `this` binding rather than from a `2D` formal.
    for ret, expr in (('int', 'Nx()->Next()->gf()->m'),
                      ('int', 'Nx()->Next()->gf()->a'),
                      ('int*', '&Nx()->Next()->gf()->b'),
                      ('int*', '&Nx()->Next()->gf()->a'),
                      ('int', 'Nx()->Next()->gf()->in.q')):
        emit('%sstruct H { O* Nx(); %s r(); };\n%s H::r() { return %s; }\n'
             % (DECL, ret, ret, expr))
    emit('%sstruct H { O* Nx(); int r(int k); };\n'
         'int H::r(int k) { return Nx()->Next()->gfa(k)->m; }\n' % DECL)

    # 6. THE REFUSALS, so the gates are graded too. Each must be
    #    NotImplemented — never wrong bytes — and each is a DIFFERENT
    #    instruction from `lwz`.
    for mem, ty in (('ch', 'char'), ('sh', 'short'), ('ll', 'long long'),
                    ('fl', 'float')):
        emit('%s%s f(O* p) { return p->Next()->gf()->%s; }\n' % (DECL, ty, mem))
        emit('%sint f(O* p) { return p->Next()->gf()->%s; }\n' % (DECL, mem))
    # A post-op on the loaded value puts the load in r11 and adds.
    emit('%sint f(O* p) { return p->Next()->gf()->m + 1; }\n' % DECL)
    emit('%sint f(O* p) { return p->Next()->gf()->a + 1; }\n' % DECL)
    # A VARIABLE subscript is not a literal add at all — `slwi`+`lwzx`.
    emit('%sint f(O* p, int i) { return p->Next()->gf()->arr[i]; }\n' % DECL)
    emit('%sint* f(O* p, int i) { return &p->Next()->gf()->arr[i]; }\n' % DECL)
    # A displacement past the signed 16-bit immediate.
    emit('%sstruct W { char pad[40000]; int far; };\n'
         'struct O2 { O* Next(); W* gw(); };\n'
         'int f(O2* p) { return p->Next()->gw()->far; }\n' % DECL)
    emit('%sstruct W { char pad[40000]; int far; };\n'
         'struct O2 { O* Next(); W* gw(); };\n'
         'char* f(O2* p) { return &p->Next()->gw()->pad[39000]; }\n' % DECL)
