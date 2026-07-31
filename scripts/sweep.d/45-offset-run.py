# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# ---- W34: the RUN of byte-offset adds --------------------------------------
#
# The indirect-load leaf used to admit exactly ONE `27`/`28` byte-offset add;
# it now folds an arbitrary run of literal ones into the single `lwz`
# displacement, which is what the address and store leaves have always done.
# 40-indirect-load.py sweeps the single-add class thoroughly and has no case
# with two literal adds in it at all except one it expected to refuse.
#
# The axes here are the ones that would separate "fold the sum" from the nearest
# plausible alternatives, none of which any hand-written fixture distinguishes:
#
#  * **depth** — a rule that folded only the first two, or that mis-associated
#    the sum, agrees with folding at depth 2 and disagrees at 3+;
#  * **cv-qualification at EVERY level of the chain**, independently. This is
#    the axis `docs/GAPS.md` §6's thirteenth live mis-emit hid behind: cv
#    changes no operator and no shape, so it is invisible to review, and it
#    *does* change the `27` tags the width cross-check reads;
#  * **which `27` in the run announces the loaded width.** Only the LAST one is
#    in a position to; a rule reading the first would agree whenever the chain
#    is all-4-byte, which is nearly every hand-written case, and disagree the
#    moment the tail is a `char`, a `short`, a `long long` or a pointer;
#  * **`27` and `28` interleaved in every order**, since only `27` re-types;
#  * **the 16-bit displacement crossed by the SUM rather than by any one add** —
#    a per-add gate passes every one of these and emits the wrong instruction;
#  * **the DS-form `ld`**, whose displacement must be a multiple of 4, reached
#    by a sum rather than by a single offset;
#  * **intrinsic 2117 (`base-member-addr`) at each position in the chain** —
#    the run rule has a second site there, and a base member reached in the
#    MIDDLE of a chain is a different production that must still refuse.


def cases(emit):
    # Widths for the tail of the chain. `float`/`double` must refuse (different
    # register file); the rest each pick a different load instruction.
    TAILS = ('int', 'unsigned', 'long', 'char', 'signed char', 'unsigned char',
             'short', 'unsigned short', 'long long', 'unsigned long long',
             'int*', 'const int*', 'float', 'double')
    CV = ('', 'const ', 'volatile ', 'const volatile ')

    # ---- depth ladder x tail width -----------------------------------------
    # `L0` wraps `L1` wraps … wraps a struct whose second member is the tail, so
    # every level contributes a NON-ZERO offset and a rule that dropped one is
    # visible in the displacement rather than hidden by a zero.
    for depth in (1, 2, 3, 4, 5, 6):
        for ty in TAILS:
            decls = ["struct N0 { int pad0; %s v; };" % ty]
            for k in range(1, depth + 1):
                decls.append("struct N%d { int pad%d; N%d inner; };" % (k, k, k - 1))
            path = ".".join(["inner"] * depth + ["v"]) if depth else "v"
            emit("%s\n%s f(N%d* p) { return p->%s; }\n"
                 % ("\n".join(decls), ty, depth, path))

    # ---- cv-qualification at every level, independently ---------------------
    # Three levels, each of the four cv spellings on the member at that level
    # plus the four on the pointer itself: 4^3 x 4 would be 256 cases, which is
    # more than this axis needs to separate anything, so the pointer axis is
    # crossed against the diagonal of the member axis and against each member
    # level varied alone.
    for cvp in CV:
        for cv0, cv1, cv2 in [(a, a, a) for a in CV] + \
                             [(a, '', '') for a in CV] + \
                             [('', a, '') for a in CV] + \
                             [('', '', a) for a in CV]:
            emit("struct C0 { int c0pad; %sint v; };\n"
                 "struct C1 { int c1pad; %sC0 in0; };\n"
                 "struct C2 { int c2pad; %sC1 in1; };\n"
                 "int f(%sC2* p) { return p->in1.in0.v; }\n"
                 % (cv2, cv1, cv0, cvp))

    # ---- `27` and `28` interleaved in every order ---------------------------
    # Only `27` re-types the address, so the order in which the two forms appear
    # decides which token last announced a pointee width.
    ARR = ("struct E { int e[3]; };\n"
           "struct R { int rpad; E rows[2]; };\n"
           "struct T { int tpad; R grid[2]; };\n")
    for i in (0, 1):
        for j in (0, 1):
            for k in (0, 1, 2):
                emit(ARR + "int f(T* p) { return p->grid[%d].rows[%d].e[%d]; }\n"
                     % (i, j, k))
    # …and the same chain with a struct member between the subscripts, so a `27`
    # follows a `28` rather than only preceding one.
    emit(ARR + "struct M { int mpad; T t; };\n"
         "int f(M* p) { return p->t.grid[1].rows[0].e[2]; }\n")

    # ---- the last `27` is the one that announces the loaded width -----------
    # A chain whose intermediate levels are 4-byte-aligned aggregates and whose
    # TAIL is narrow or wide. A rule reading the first `27`'s tag says 4 here.
    for ty in ('char', 'unsigned char', 'short', 'unsigned short',
               'long long', 'unsigned long long', 'int*', 'float', 'double'):
        emit("struct W0 { int w0pad; %s v; };\n"
             "struct W1 { int w1pad; W0 in0; };\n"
             "struct W2 { int w2pad; W1 in1; };\n"
             "%s f(W2* p) { return p->in1.in0.v; }\n"
             "int g(W2* p) { return (int)p->in1.in0.v; }\n" % (ty, ty))

    # ---- the displacement gate applies to the SUM ---------------------------
    # Each individual add is well inside the 16-bit field; only some totals are
    # not. A per-add gate emits a folded `lwz` for every one of these.
    for pad in (8188, 8189, 8190, 8191, 8192, 16382, 16384):
        for lead in (0, 1, 2):
            leadpad = "".join("int L%d;" % i for i in range(lead))
            emit("struct BP { int a[%d]; int last; };\n"
                 "struct BH { %s BP bp; };\n"
                 "int f(BH* p) { return p->bp.last; }\n" % (pad, leadpad))

    # ---- the DS-form `ld`, reached by a sum ---------------------------------
    # `ld` encodes its displacement in 14 bits and the low two are the form's,
    # so the SUM has to be a multiple of 4 — a property no single offset in the
    # chain need have on its own.
    for lead in (0, 1, 2, 3):
        leadpad = "".join("int D%d;" % i for i in range(lead))
        emit("struct QI { int qpad; long long q; };\n"
             "struct QO { %s QI qi; };\n"
             "long long f(QO* p) { return p->qi.q; }\n" % leadpad)

    # ---- intrinsic 2117 at each position in the chain -----------------------
    # A member inherited from a non-virtual base is `base-member-addr`, not a
    # `27`. At the HEAD of the chain the run folds after it; in the MIDDLE it is
    # a different production and must refuse rather than be folded through.
    BASE = ("struct BA { int bapad; };\n"
            "struct BB { int bbpad; };\n"
            "struct IN { int inpad; int v; };\n")
    emit(BASE + "struct DH : BA { int dpad; IN in; };\n"
         "int f(DH* p) { return p->bapad; }\n")
    emit(BASE + "struct DI { int dipad; IN in; };\n"
         "struct DJ : DI { int djpad; };\n"
         "int f(DJ* p) { return p->in.v; }\n")
    emit(BASE + "struct DK : BA, BB { int dkpad; IN in; };\n"
         "int f(DK* p) { return p->in.v; }\n")
    emit(BASE + "struct DL : BA { int dlpad; };\n"
         "struct DM { int dmpad; DL dl; };\n"
         "int f(DM* p) { return p->dl.bapad; }\n")

    # ---- the run behind `this`, at every argument position ------------------
    # `this` takes r3 and shifts every explicit formal up one, so a chain whose
    # base is the n-th formal of a member function is the one place the folded
    # displacement and the base register are decided by different facts.
    NEST = ("struct TI { int tipad; int v; };\n"
            "struct TO { int topad; TI ti; };\n")
    for n in range(0, 6):
        args = "".join("int a%d, " % i for i in range(n))
        emit(NEST + "struct K%d { int m(%sTO* q) const; };\n"
             "int K%d::m(%sTO* q) const { return q->ti.v; }\n" % (n, args, n, args))
    # …and the chain rooted at `this` itself, which is one deref shorter.
    for depth in (1, 2, 3):
        decls = ["struct H0 { int h0pad; int v; };"]
        for k in range(1, depth + 1):
            decls.append("struct H%d { int h%dpad; H%d inner; };" % (k, k, k - 1))
        path = ".".join(["inner"] * (depth - 1) + ["v"]) if depth >= 1 else "v"
        emit("%s\nstruct HC%d { H%d top; int m() const; };\n"
             "int HC%d::m() const { return top.%s; }\n"
             % ("\n".join(decls), depth, depth, depth, path))
