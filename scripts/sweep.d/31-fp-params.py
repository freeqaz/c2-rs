# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.
#
# The FP **parameter list** axis, which no fixture and no sweep case had varied:
# every FP body (and every `w13*` fixture) declares an all-`float` or
# all-`double` parameter list, which is exactly the shape where a parameter's
# positional index and its FP-register number coincide. They are two facts, and
# they come apart two ways — a non-FP parameter ahead of the floats, and an FP
# parameter the body never mentions. Both were live wrong-bytes emits
# (`fixtures/cpp/w13_fparam_neg.cpp`); a mismatch here is the alarm.


def cases(emit):
    for ty in ('float', 'double'):
        for lead in ('int a', 'unsigned a', 'char a', 'long long a', 'int* a',
                     '%s a' % ty):
            for body in ('b * c', 'b + c', 'b - c', 'b / c'):
                emit("%s f(%s, %s b, %s c) { return %s; }\n"
                         % (ty, lead, ty, ty, body))
            # …and with the non-FP parameter in the middle and at the end.
            emit("%s f(%s b, %s, %s c) { return b * c; }\n" % (ty, ty, lead, ty))
            emit("%s f(%s b, %s c, %s) { return b * c; }\n" % (ty, ty, ty, lead))
        # A bare return of an FP parameter, at every position: `fmr f1,fN` for any
        # position but the first, and nothing at all for the first.
        # (`nparam`, not `n` — `n` is this generator's file counter, and a `for n in`
        # here silently rewound it and overwrote 1,233 already-written cases.)
        for nparam in range(1, 5):
            ps = ', '.join('%s p%d' % (ty, i) for i in range(nparam))
            for i in range(nparam):
                emit("%s f(%s) { return p%d; }\n" % (ty, ps, i))
        # An unused FP parameter: undecidable from `.ex` alone, so it must refuse.
        emit("%s f(%s a, %s b) { return b * b; }\n" % (ty, ty, ty))
        emit("%s f(%s a, %s b, %s c) { return b * c; }\n" % (ty, ty, ty, ty))
        # A member function — `this` takes a GPR and never appears in the FP file.
        emit("struct C { %s m(%s x) const; };\n"
                 "%s C::m(%s x) const { return x * x; }\n" % (ty, ty, ty, ty))
        emit("struct C { %s m(%s x, %s y) const; };\n"
                 "%s C::m(%s x, %s y) const { return x + y; }\n"
                 % (ty, ty, ty, ty, ty, ty))
