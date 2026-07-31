# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.
#
# Floating-point leaves: the FP register model is entirely separate from the
# integer one, so operand order and operator mix have to be swept again rather
# than assumed.


def cases(emit):
    for ty in ('float', 'double'):
        for o1 in ['+', '-', '*', '/']:
            emit("%s f(%s a, %s b) { return a %s b; }\n" % (ty, ty, ty, o1))
            emit("%s f(%s a, %s b) { return b %s a; }\n" % (ty, ty, ty, o1))
            for o2 in ['+', '-', '*', '/']:
                for perm in ['a %s b %s c', 'a %s c %s b', 'b %s a %s c', 'c %s b %s a']:
                    emit("%s f(%s a, %s b, %s c) { return %s; }\n"
                             % (ty, ty, ty, ty, perm % (o1, o2)))
