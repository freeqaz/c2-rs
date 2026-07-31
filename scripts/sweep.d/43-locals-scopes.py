# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- locals, substitution and lexical scopes ------------------------------------
    # Substitution is a *source* of operand orders and repeated leaves the written source
    # does not have, which is the exact mechanism behind the reassociation mis-emits
    # above — so the class needs sweeping for the same reason, not merely testing.
    for rhs1 in ('a', 'a+1', 'a+b', 'b+a', 'a*2', '0', '7'):
        for rhs2 in ('x', 'x+1', 'x+a', 'a+x', 'x+x', 'x*b', 'x-a'):
            emit("int f(int a,int b){int x=%s;int y=%s;return y;}\n" % (rhs1, rhs2))
            emit("int f(int a,int b){int x=%s;x=%s;return x;}\n"
                     % (rhs1, rhs2.replace('x', '(x)')))
    # The same, inside brace scopes at several depths, plus a close-then-continue.
    for body in ('int x=a+1;return x;', 'int x=a+1;{return x+b;}',
                 '{int x=a+1;}return a+b;', '{int x=a+1;{int y=x+b;return y;}}',
                 '{int x=a+1;}{int y=a+b;return y;}', '{}return a+1;'):
        emit("int f(int a,int b){%s}\n" % body)
