# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- member functions across source lines ---------------------------------------
    # `this` is bound from the pre-body region, and locating that region by a bare byte
    # search made a member function on source line 70 emit the wrong base register
    # (fixtures/cpp/il_this_line70.cpp). Line number is therefore a real axis, and the
    # only way to sweep it is to move the definition.
    for line in range(66, 74):
        pad = '\n'.join('// pad %d' % i for i in range(1, line - 4))
        emit("struct C { int m; int gp(int* q) const; int gv(int v,int* q) const; };\n"
                 + pad + "\nint C::gp(int* q) const { return *q; }\n"
                 "int C::gv(int v,int* q) const { return *q; }\n")
