# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.
#
# W6 comparisons: relation x signedness x a spread of k including both i16
# boundaries. The cross product is the point — `w6_rel_k.cpp` tests every
# relation and both boundaries, but never a boundary-sensitive relation AT a
# boundary, which is how `a == -32768` stayed broken.


def cases(emit):
    for r in ['<', '<=', '>', '>=', '==', '!=']:
        for k in ['0', '1', '-1', '5', '-5', '2', '32767', '-32768']:
            emit("int f(int a) { return a %s %s; }\n" % (r, k))
            if not k.startswith('-'):
                emit("int f(unsigned a) { return a %s %su; }\n" % (r, k))
