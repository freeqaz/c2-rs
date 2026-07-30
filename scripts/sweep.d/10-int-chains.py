# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.
#
# Integer-expression chains: every two-leaf form and every three-leaf
# left-associative chain over three parameters and a spread of literals.
# This is the axis that found the reassociation and repeated-leaf mis-emits:
# operand ORDER and operator MIX, which the hand-written corpus never varied.


def cases(emit):
    ops = ['+', '-', '*']
    leaves = ['a', 'b', 'c', '1', '2', '7', '0']

    def chain(body):
        emit("int f(int a, int b, int c) { return %s; }\n" % body)

    # Two-leaf forms: every leaf/operator/leaf combination.
    for l1 in leaves:
        for o1 in ops:
            for l2 in leaves:
                chain("%s %s %s" % (l1, o1, l2))
    # Three-leaf left-associative chains. This is the layer that matters: operand
    # ORDER and operator MIX are exactly what the hand-written corpus never varied.
    for l1 in leaves:
        for o1 in ops:
            for l2 in leaves:
                for o2 in ops:
                    for l3 in ['a', 'b', 'c', '1', '3']:
                        chain("%s %s %s %s %s" % (l1, o1, l2, o2, l3))
