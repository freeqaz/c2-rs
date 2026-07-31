# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter, and the driver fails if a fragment emits
# zero cases.
#
# ---- what this fragment is for --------------------------------------------
#
# **Flat multi-statement locals**: a local defined by one statement and consumed
# by a COMPOUND expression in the next, with no brace scope between them.
#
#     int f(int a,int b){ int x=a+1; return x+b; }
#
# `43-locals-scopes.py` sweeps the neighbouring shapes densely but never
# generates this one. Its flat arm returns a bare local (`int x=a+1;return x;`),
# and every compound return it produces is wrapped in a brace scope
# (`int x=a+1;{return x+b;}`). So the flat form was reachable only by writing it
# out by hand, which is how it was found — and a class the enumeration cannot
# reach is a class whose disagreement count is unknown rather than zero. This
# fragment closes that hole.
#
# The shape matters because substitution CREATES streams the written source does
# not show. `int x=a+1; return x+b;` resolves to `[a, 1, Add, b, Add]`, which
# still owes an immediate when it reaches the reg-reg `add` — a stream the affine
# selector cannot lower in source order and c2 emits as `(a+b)+1`. That is the
# pending-immediate axis, and both of its outcomes live here: the additive forms
# canonicalize and are byte-graded, the forms mixing `*` with `+` have no
# canonical form and must be REFUSED. A fragment that generated only one of the
# two would grade a rule in the single operand order it was derived from, which
# is the failure `10-int-chains.py`'s commutation pairs already record.


def cases(emit):
    # ---- one local, then a compound return, FLAT -------------------------------
    # The right-hand side varies over the forms that leave a pending immediate,
    # one that leaves none, and the constant folds; the return varies the operator
    # and BOTH operand orders, so a rule can never be graded in one order only.
    for rhs in ('a', 'a+1', 'a-1', '1+a', 'a+b', 'b+a', 'a-b', 'a*b', 'a*2', '2*a', '0', '7'):
        for ret in ('x', 'x+b', 'b+x', 'x-b', 'b-x', 'x*b', 'b*x',
                    'x+1', 'x-1', '1+x', 'x*2', 'x+a', 'a+x'):
            emit("int f(int a,int b){int x=%s;return %s;}\n" % (rhs, ret))

    # ---- two locals in a row, then a compound return, FLAT ---------------------
    # A second definition is what makes the substituted stream longer than any
    # written expression in the corpus, so the term bound gets exercised from the
    # locals side rather than only from the flat-expression side.
    for rhs2 in ('x+1', 'x-1', 'x+b', 'b+x', 'x*b', 'x-b', 'b-x'):
        for ret in ('y', 'y+a', 'a+y', 'y*a', 'y+1', 'y-a'):
            emit("int f(int a,int b){int x=a+1;int y=%s;return %s;}\n" % (rhs2, ret))

    # ---- three formals, so the resolved chain reaches four terms ---------------
    # Four leaves is exactly the enumerated bound `canonicalize_chain` accepts up
    # to (MAX_SWEPT_TERMS), so this is the arm that grades the boundary rather
    # than extrapolating across it.
    for lhs in ('a+b', 'a+1', 'a*b', 'a-1'):
        for ret in ('x+c', 'c+x', 'x*c', 'c*x', 'x-c', 'c-x', 'x+b+c', 'x+c+b', 'x+c+1'):
            emit("int f(int a,int b,int c){int x=%s;return %s;}\n" % (lhs, ret))

    # ---- a dead first definition, flat -----------------------------------------
    # c2 register-allocates locals and drops the dead store, so these must resolve
    # to the same stream as the one-local forms above; a divergence would mean the
    # substitution environment is leaking a definition the return cannot see.
    for ret in ('x+b', 'b+x', 'x*b', 'x+1'):
        emit("int f(int a,int b){int x=0;x=a+1;return %s;}\n" % ret)
        emit("int f(int a,int b){int x=a+1;int y=b;return %s;}\n" % ret.replace('b', 'y'))
