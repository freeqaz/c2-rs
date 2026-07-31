# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- generated empty destructors: the base-delegation skeleton -------------------
    # The newest accepted class (`docs/IL_CALL_IN_EXPR.md` §5). It is admitted on a rigid
    # byte skeleton that includes two UNDECODED trailers, `5C <int> <f>` and `5E <n> <g>`,
    # and the reason it needs sweeping rather than testing is that two of those three
    # payload fields turned out to vary — `<n>` with the number of destroyed sub-objects
    # and `<f>`/`<g>` with `/EH`. A third co-varying field would pass the gate and emit a
    # bare branch where the reference emits an `addi` and two `bl`s.
    DBASES = (('B0', ''), ('B1', 'int b0;'), ('B4', 'int b0,b1,b2,b3;'),
              ('B8', 'double b0; char b1;'))
    DMEMS = ('', 'int d;', 'double d;', 'char d;', 'int d0,d1,d2;')
    for bn, bdata in DBASES:
        for dmem in DMEMS:
            emit("struct %s { ~%s(); %s };\nstruct D : %s { ~D(); %s };\nD::~D() {}\n"
                     % (bn, bn, bdata, bn, dmem))
            # Two inheritance levels: the delegation is still ONE step, so the class-pair
            # descriptor must still be `66 02` — the count this grammar requires literally.
            emit("struct %s { ~%s(); %s };\nstruct M : %s { ~M(); %s };\n"
                     "struct D : M { ~D(); %s };\nD::~D() {}\n"
                     % (bn, bn, bdata, bn, dmem, dmem))
    # The definition's SOURCE LINE, for the same reason the member-function loop below
    # sweeps it: `this` is bound from the pre-body region, and the closing brace's own
    # `4F 01 <line>` marker lands inside the return plumbing, which a one-line probe
    # never shows. Line 70's marker is `4F 01 46` — the known-bad formals anchor.
    for line in range(64, 77):
        pad = '\n'.join('// pad %d' % k for k in range(1, line - 2))
        emit("struct B { ~B(); int x; };\nstruct D : B { ~D(); int y; };\n" + pad
                 + "\nD::~D()\n{\n}\n")
