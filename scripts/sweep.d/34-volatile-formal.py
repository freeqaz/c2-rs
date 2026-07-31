# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# W32, the `volatile`-qualified formal: the thirteenth live wrong-bytes emit, and
# the one axis in this directory whose whole content is a *position*.
#
# A volatile parameter is a volatile object, so c2 homes the incoming argument
# register in the frame and reads it back from memory at every use — while the
# same two bytes at the `27`/`30` designator positions, and on a formal the body
# never reads, cost nothing at all. So the sweep has to generate BOTH halves: the
# refusals alone would license refusing the qualifier everywhere, and a blanket
# gate is a coverage loss with no fact behind it.
#
# `const` is the control. It differs from `volatile` in one bit of one tag byte
# and in a whole stack frame, and every row below is generated for both.


def cases(emit):
    QUALS = ('', 'const ', 'volatile ', 'const volatile ')
    SCALARS = ('int', 'unsigned', 'long', 'float', 'double', 'char', 'short')
    # ---- the qualifier on a formal the body READS, in every shape ----------
    for q in QUALS:
        for t in SCALARS:
            d = '%s%s y' % (q, t)
            # the identity / straight-line leaf
            emit('%s f(int x, %s) { return y; }\n' % (t, d))
            emit('%s f(%s) { return y; }\n' % (t, d))
            # arithmetic over it
            if t not in ('float', 'double'):
                emit('%s f(%s%s x, %s) { return x + y; }\n' % (t, q, t, d))
            # a tail call in the matching register file
            emit('%s g(%s);\n%s f(%s x, %s) { return g(y); }\n' % (t, t, t, t, d))
            emit('void g(%s);\nvoid f(%s x, %s) { g(y); }\n' % (t, t, d))
            # a framed post-op
            if t not in ('float', 'double'):
                emit('%s g(%s);\n%s f(%s x, %s) { return g(y) + 1; }\n'
                     % (t, t, t, t, d))
            # the two-argument permutation
            emit('%s g(%s, %s);\n%s f(%s%s x, %s y) { return g(y, x); }\n'
                 % (t, t, t, t, q, t, t))
    # ---- the qualifier on a formal the body NEVER reads: free ---------------
    for q in QUALS:
        for t in SCALARS:
            emit('int f(int x, %s%s y) { return x; }\n' % (q, t))
            emit('int g(int);\nint f(int x, %s%s y) { return g(x); }\n' % (q, t))
            emit('int f(%s%s y) { return 7; }\n' % (q, t))
    # ---- pointers: the pointer itself, the pointee, and the member ---------
    for q in QUALS:
        for t in ('int', 'char', 'short', 'long long', 'float', 'double'):
            # a qualified POINTER formal (the pointer is the volatile object)
            emit('%s f(int x, %s* %sp) { return *p; }\n' % (t, t, q))
            emit('%s* f(int x, %s* %sp) { return p; }\n' % (t, t, q))
            # a pointer to a qualified object (the POINTEE is qualified)
            emit('%s f(int x, %s%s* p) { return *p; }\n' % (t, q, t))
            # a qualified MEMBER through a plain pointer
            emit('struct S%d { %s%s m; };\n%s f(S%d* p) { return p->m; }\n'
                 % (len(q), q, t, t, len(q)))
            # the address of a qualified member
            emit('struct A%d { %s%s m; };\n%s%s* f(A%d* p) { return &p->m; }\n'
                 % (len(q), q, t, q, t, len(q)))
            # a store through a qualified designator
            emit('struct B%d { %s%s m; };\nvoid f(B%d* p, %s v) { p->m = v; }\n'
                 % (len(q), q, t, len(q), t))
