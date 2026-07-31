# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- floating point BESIDE a framed function: the label counter --------------
    # A framed function's `$M`/`$T` labels come from a counter every function in the
    # TU consumes, so a function ahead of it with the wrong stride is six wrong bytes
    # in an obj that still links. An FP-touching function consumes **2** and an
    # integer leaf **1**, and the FP store leaf was given 1 — a live mis-emit that
    # neither the FP fixtures (no framed function) nor the framed fixtures (no
    # floating point) could contain. It exists only in the cross product, which is
    # the argument for generating one rather than adding a case.
    FRAMED_LEAD = (
        'struct LS { int i; float f; double d; };\nvoid q1();\nvoid q2();\n')
    FRAMED_KINDS = (
        ('void L(LS* s, int v)      { s->i = v; }\n'),      # stride 1, the control
        ('void L(LS* s, float v)    { s->f = v; }\n'),      # stride 2
        ('void L(LS* s, double v)   { s->d = v; }\n'),      # stride 2
        ('float L(float a, float b) { return a * b; }\n'),  # stride 2 (arithmetic)
        ('float L(float a, float b) { return b; }\n'),      # stride 2 (the fmr)
        ('int L(int a)              { return a + 1; }\n'),  # stride 1
    )
    FRAMED_BODIES = (
        'void F() { q1(); }\n',            # a void tail call — NOT framed
        'void F() { q1(); q2(); }\n',      # Class A many-calls — framed
        'int  F(int a) { return g(a) + 1; }\n',
    )
    for lead in FRAMED_KINDS:
        for framed in FRAMED_BODIES:
            pre = 'int g(int);\n' if 'g(a)' in framed else ''
            # the leaf before the framed function, and after it
            emit(FRAMED_LEAD + pre + lead + framed)
            emit(FRAMED_LEAD + pre + framed + lead)
            # …and with a stride-1 integer leaf between them, so an error in the
            # counter cannot be absorbed by an adjacent one
            emit(FRAMED_LEAD + pre + lead + 'int M(int a){return a+2;}\n' + framed)
