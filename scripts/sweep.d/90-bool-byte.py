# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- W26: the one-byte-unsigned value class -------------------------------
    # `bool` and `unsigned char` share the operand TYPE `82 12`, and inside the
    # class a value costs no instruction: `li r3,k`, a bare `blr`, or the W18
    # register move. The axes are (spelling) x (literal value) x (argument slot),
    # crossed against the conversions OUT of the class, which are a real `rlwinm`.
    BOOL_T = ['bool', 'unsigned char']
    for t in BOOL_T:
        for k in ('0', '1', '2', '127', '200', '255'):
            emit('%s f() { return (%s)%s; }\n' % (t, t, k))
        for slot in range(8):
            pre = ''.join('int p%d, ' % j for j in range(slot))
            emit('%s f(%s%s v) { return v; }\n' % (t, pre, t))
        # the class beside every accepted neighbour that shares its bytes
        emit('struct S { int i; %s m; };\n%s g(S* s) { return s->m; }\n'
                 'void h(S* s, %s v) { s->m = v; }\n%s f(%s v) { return v; }\n'
                 % (t, t, t, t, t))
        # ...and the REFUSING conversions, each alone and beside an emitted leaf
        for target in ('int', 'unsigned', 'char', 'short', 'long long'):
            emit('%s f(%s v) { return v; }\n' % (target, t))
            emit('%s f(%s v) { return v; }\nint h(int a) { return a + 1; }\n' % (target, t))
        for op in ('+', '-', '*'):
            emit('%s f(%s a, %s b) { return a %s b; }\n' % (t, t, t, op))
        emit('%s f(%s a) { return !a; }\n' % (t, t))
        emit('%s f(%s a, %s b) { return a && b; }\n' % (t, t, t))
        emit('%s f(%s a) { %s x = a; return x; }\n' % (t, t, t))
        emit('%s g();\n%s f() { return g(); }\n' % (t, t))
    # the OTHER one-byte class, which must keep refusing at every one of the same
    # positions — `char`/`signed char` are `82 11`, and a signed narrow value parts
    # company from an unsigned one exactly one token later.
    for t in ('char', 'signed char'):
        emit('%s f(%s v) { return v; }\n' % (t, t))
        emit('%s f(int k, %s v) { return v; }\n' % (t, t))
        emit('%s f() { return (%s)7; }\n' % (t, t))
