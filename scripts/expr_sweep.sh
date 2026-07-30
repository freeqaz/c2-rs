#!/bin/sh
# Generated differential sweep over the integer-expression class.
#
# Enumerates small expressions over three parameters and a few literals, compiles
# each against the real toolchain, and reports every byte MISMATCH. This is the
# thing that found the reassociation and repeated-leaf mis-emits: ~20 wrong-bytes
# bugs in the straight-line class that the hand-written corpus had never separated,
# because every fixture in it happened to use distinct operands in ascending order.
#
# The lesson is in `docs/GAPS.md`: a green fixture run is only as strong as the
# corpus's ability to *separate* the candidate rules, and a hand-picked corpus is
# systematically biased toward the shapes whoever wrote it was already thinking
# about. Enumeration has no such bias.
#
# A MISMATCH is an alarm, not a gap — the port emitted bytes and they were wrong.
# Either fix the lowering or tighten the gate until it refuses. NotImplemented is
# fine and expected for most cases here.
#
# Usage:  scripts/expr_sweep.sh [outdir] [max-cases]
#         scripts/expr_sweep.sh /tmp/sweep 400     # a quick subset
#
# Needs the toolchain (see CLAUDE.md); without it every case reports SKIP and the
# sweep is vacuous, so it checks for that up front.
set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-/tmp/c2rs-expr-sweep}"
limit="${2:-0}"
c2rs="$repo_root/target/release/c2rs"

if [ ! -x "$c2rs" ]; then
    echo "building the harness first"
    (cd "$repo_root" && cargo build --release -p c2-harness)
fi

mkdir -p "$out"
rm -f "$out"/*.cpp "$out"/cases.txt 2>/dev/null || true

python3 - "$out" <<'PY'
import sys, os
out = sys.argv[1]
ops = ['+', '-', '*']
leaves = ['a', 'b', 'c', '1', '2', '7', '0']
n = 0
def emit(body):
    global n
    n += 1
    with open(os.path.join(out, 'f%04d.cpp' % n), 'w') as fh:
        fh.write("int f(int a, int b, int c) { return %s; }\n" % body)
# Two-leaf forms: every leaf/operator/leaf combination.
for l1 in leaves:
    for o1 in ops:
        for l2 in leaves:
            emit("%s %s %s" % (l1, o1, l2))
# Three-leaf left-associative chains. This is the layer that matters: operand
# ORDER and operator MIX are exactly what the hand-written corpus never varied.
for l1 in leaves:
    for o1 in ops:
        for l2 in leaves:
            for o2 in ops:
                for l3 in ['a', 'b', 'c', '1', '3']:
                    emit("%s %s %s %s %s" % (l1, o1, l2, o2, l3))

# ---- the other classes that claim byte-exactness --------------------------------
# Each of these found real mis-emits the fixtures had missed, for the same reason:
# the corpus varied one axis at a time rather than the cross product.
def emit_raw(src):
    global n
    n += 1
    with open(os.path.join(out, 'f%04d.cpp' % n), 'w') as fh:
        fh.write(src)

# W6 comparisons: relation x signedness x a spread of k including both i16
# boundaries. The cross product is the point — `w6_rel_k.cpp` tests every relation
# and both boundaries, but never a boundary-sensitive relation AT a boundary, which
# is how `a == -32768` stayed broken.
for r in ['<', '<=', '>', '>=', '==', '!=']:
    for k in ['0', '1', '-1', '5', '-5', '2', '32767', '-32768']:
        emit_raw("int f(int a) { return a %s %s; }\n" % (r, k))
        if not k.startswith('-'):
            emit_raw("int f(unsigned a) { return a %s %su; }\n" % (r, k))

# Floating-point leaves: the FP register model is entirely separate from the integer
# one, so operand order and operator mix have to be swept again rather than assumed.
for ty in ('float', 'double'):
    for o1 in ['+', '-', '*', '/']:
        emit_raw("%s f(%s a, %s b) { return a %s b; }\n" % (ty, ty, ty, o1))
        emit_raw("%s f(%s a, %s b) { return b %s a; }\n" % (ty, ty, ty, o1))
        for o2 in ['+', '-', '*', '/']:
            for perm in ['a %s b %s c', 'a %s c %s b', 'b %s a %s c', 'c %s b %s a']:
                emit_raw("%s f(%s a, %s b, %s c) { return %s; }\n"
                         % (ty, ty, ty, ty, perm % (o1, o2)))

# Tail calls: argument count, argument permutation, and computed arguments.
emit_raw("int g1(int);\nint f(int a){return g1(a);}\n")
for p in ['a,b', 'b,a']:
    emit_raw("int g2(int,int);\nint f(int a,int b){return g2(%s);}\n" % p)
for p in ['a,b,c', 'a,c,b', 'b,a,c', 'b,c,a', 'c,b,a', 'c,a,b']:
    emit_raw("int g3(int,int,int);\nint f(int a,int b,int c){return g3(%s);}\n" % p)
for e in ['a+1', 'a-1', 'a+b', 'b+a', 'a-b', '1']:
    emit_raw("int g1(int);\nint f(int a,int b){return g1(%s);}\n" % e)

# ---- indirect loads: deref, member, subscript ------------------------------------
# The newest accepted class and, until now, entirely unswept. Two of its gates rest
# on fields whose *meaning* is unproven — the `28` subscript payload is `00 00` at
# every captured site with no known semantics, and the `2C` cv strip is treated as
# free on the same "always observed" basis (docs/GAPS.md §6: a field the port skips
# is indistinguishable from a field that is always the same). A co-varying semantic
# — a scaled rather than byte index, a qualification strip that is not free — would
# pass those gates and emit. Only the cross product separates that from safety.
STRUCTS = (
    "struct S1 { char a, b, c, d; };\n"
    "struct S2 { short a, b; };\n"
    "struct S4 { int a, b, c, d; };\n"
    "struct S8 { double a; int b; };\n"
    "struct A4 { int a0, a1; };\n"
    "struct B4 { int b0, b1, b2; };\n"
    "struct D4 : A4, B4 { int d; };\n"
)
# Element size x index: the axis that would expose a scaled-vs-byte index rule.
for ty in ('int', 'unsigned', 'long', 'char', 'short', 'float', 'double', 'int*'):
    for ix in ('0', '1', '3', '-1', '-4', '8191', '8192', '100000'):
        emit_raw("int f(%s* p) { return (int)p[%s]; }\n" % (ty, ix))
        emit_raw("%s f(%s* p) { return p[%s]; }\n" % (ty, ty, ix))
# Member offsets across widths, and the same member reached by `.` and `->`.
for st, mem in (('S1','a'),('S1','d'),('S2','a'),('S2','b'),('S4','a'),('S4','d'),
                ('S8','a'),('S8','b')):
    emit_raw(STRUCTS + "int f(%s* p) { return (int)p->%s; }\n" % (st, mem))
    emit_raw(STRUCTS + "int f(%s& r) { return (int)r.%s; }\n" % (st, mem))
# cv-qualification on the pointee: the axis the `2C` strip claims is free.
for q in ('', 'const ', 'volatile ', 'const volatile '):
    for ty in ('int', 'unsigned', 'char', 'short'):
        emit_raw("int f(%s%s* p) { return (int)*p; }\n" % (q, ty))
        emit_raw("int f(%s%s* p) { return (int)p[2]; }\n" % (q, ty))
# Inherited members: the two literals of intrinsic 2117 must ADD, and only a member
# at a nonzero offset inside a base at a nonzero offset separates that from
# "whichever is nonzero".
for mem in ('a0', 'a1', 'b0', 'b1', 'b2', 'd'):
    emit_raw(STRUCTS + "int f(D4* p) { return p->%s; }\n" % mem)
# Two adds chained, which must refuse rather than fold to one.
emit_raw(STRUCTS + "int f(S4* p) { return p[2].c; }\n")
emit_raw(STRUCTS + "int f(int** p) { return *p[1]; }\n")

# ---- pointer-VALUED leaves: the class T1 admitted, +88,116 functions -------------
# The gate here is on the loaded value's OWN width, never the pointee's — loading a
# `char*` member is `lwz` while loading THROUGH a `char*` is `lbz`, and both spell
# `char` somewhere in the type. Two predicates, `is_ptr4_kind` and `is_ptr_to_4`,
# for those two questions. Sweeping the cross is what separates them: the fixtures
# pin roughly a dozen shapes, and this class is the largest single admission the
# project has made.
PSTRUCT = (
    "struct H {\n"
    "  int i; char c;\n"
    "  int* pi; const int* pci; int* const cpi; char* pc; void* pv;\n"
    "  int** ppi; void (*pf)(); int (*pfi)(int);\n"
    "};\n"
)
# Pointer-valued member getters: pointee type x cv-spelling x through `this` or not.
# A member's own declared type is returned where C++ can spell it as a return type;
# the two function-pointer members are reached through the cast forms below, which
# is also the interesting case (a function pointer is kind class 4, not 3, and both
# must lower to the same bare `lwz`).
for mem, ret in (('pi', 'int*'), ('pci', 'const int*'), ('cpi', 'int*'),
                 ('pc', 'char*'), ('pv', 'void*'), ('ppi', 'int**')):
    emit_raw(PSTRUCT + "%s f(H* h) { return h->%s; }\n" % (ret, mem))
for mem in ('pi', 'pci', 'cpi', 'pc', 'pv', 'ppi', 'pf', 'pfi'):
    emit_raw(PSTRUCT + "void* f(H* h) { return (void*)h->%s; }\n" % mem)
    emit_raw(PSTRUCT + "void* f(const H* h) { return (void*)h->%s; }\n" % mem)
# The same read through `this`, const and non-const — `this` is A6-tagged in BOTH,
# so a gate written for one tag would refuse the commoner spelling.
for mem in ('pi', 'pc', 'pv', 'ppi'):
    emit_raw(PSTRUCT + "struct C : H { void* g(); };\n"
             "void* C::g() { return (void*)%s; }\n" % mem)
    emit_raw(PSTRUCT + "struct C : H { void* gc() const; };\n"
             "void* C::gc() const { return (void*)%s; }\n" % mem)
# Pointer IDENTITIES, which emit no instruction at all: the value is already in its
# argument register. Swept across argument position, because "already in r3" is the
# whole reason they are free and position is what breaks it.
for ty in ('int', 'char', 'void', 'H'):
    star = '%s*' % ty
    emit_raw(PSTRUCT + "%s f(%s p) { return p; }\n" % (star, star))
    emit_raw(PSTRUCT + "void* f(%s p) { return p; }\n" % star)
    emit_raw(PSTRUCT + "%s f(int a, %s p) { return p; }\n" % (star, star))
    emit_raw(PSTRUCT + "%s f(int a, int b, %s p) { return p; }\n" % (star, star))
emit_raw(PSTRUCT + "struct C : H { C* self(); const C* cself() const; };\n"
         "C* C::self() { return this; }\n")
emit_raw(PSTRUCT + "struct C : H { const C* cself() const; };\n"
         "const C* C::cself() const { return this; }\n")
# The neighbours that MUST refuse rather than emit. Each is one byte or one
# production away from an admitted shape, and each costs a real instruction the
# identity/getter lowerings do not emit: an `addi` for an address-of, element-size
# scaling for pointer arithmetic, an extra load for a double deref, an `mr` when the
# result is not already in r3. A mismatch here is the alarm; NotImplemented is right.
for expr in ('&h->i', '&h->pi', 'h->pi + 1', 'h->pi - 1', '*h->ppi',
             'h->ppi[1]', '(char*)h + 4'):
    emit_raw(PSTRUCT + "void* f(H* h) { return (void*)(%s); }\n" % expr)
emit_raw(PSTRUCT + "H* f(int a, H* h) { return a ? h : h; }\n")
emit_raw(PSTRUCT + "int* f(H* h) { return 0; }\n")

# ---- locals, substitution and lexical scopes ------------------------------------
# Substitution is a *source* of operand orders and repeated leaves the written source
# does not have, which is the exact mechanism behind the reassociation mis-emits
# above — so the class needs sweeping for the same reason, not merely testing.
for rhs1 in ('a', 'a+1', 'a+b', 'b+a', 'a*2', '0', '7'):
    for rhs2 in ('x', 'x+1', 'x+a', 'a+x', 'x+x', 'x*b', 'x-a'):
        emit_raw("int f(int a,int b){int x=%s;int y=%s;return y;}\n" % (rhs1, rhs2))
        emit_raw("int f(int a,int b){int x=%s;x=%s;return x;}\n"
                 % (rhs1, rhs2.replace('x', '(x)')))
# The same, inside brace scopes at several depths, plus a close-then-continue.
for body in ('int x=a+1;return x;', 'int x=a+1;{return x+b;}',
             '{int x=a+1;}return a+b;', '{int x=a+1;{int y=x+b;return y;}}',
             '{int x=a+1;}{int y=a+b;return y;}', '{}return a+1;'):
    emit_raw("int f(int a,int b){%s}\n" % body)

# ---- member functions across source lines ---------------------------------------
# `this` is bound from the pre-body region, and locating that region by a bare byte
# search made a member function on source line 70 emit the wrong base register
# (fixtures/cpp/il_this_line70.cpp). Line number is therefore a real axis, and the
# only way to sweep it is to move the definition.
for line in range(66, 74):
    pad = '\n'.join('// pad %d' % i for i in range(1, line - 4))
    emit_raw("struct C { int m; int gp(int* q) const; int gv(int v,int* q) const; };\n"
             + pad + "\nint C::gp(int* q) const { return *q; }\n"
             "int C::gv(int v,int* q) const { return *q; }\n")

# ---- generated empty destructors: the base-delegation skeleton -------------------
# The newest accepted class (`docs/IL_CALL_IN_EXPR.md` §5). It is admitted on a rigid
# byte skeleton that includes two UNDECODED trailers, `5C <int> <f>` and `5E <n> <g>`,
# and the reason it needs sweeping rather than testing is that two of those three
# payload fields turned out to vary — `<n>` with the number of destroyed sub-objects
# and `<f>`/`<g>` with `/EH`. A third co-varying field would pass the gate and emit a
# bare branch where the reference emits an `addi` and two `bl`s.
DBASES = (('B0', ''), ('B1', 'int b0;'), ('B4', 'int b0,b1,b2,b3;'),
          ('B8', 'double b0; char b1;'))
DMEMS = ('', 'int d;', 'double d;', 'char d;', 'int d0,d1,d2;')
for bn, bdata in DBASES:
    for dmem in DMEMS:
        emit_raw("struct %s { ~%s(); %s };\nstruct D : %s { ~D(); %s };\nD::~D() {}\n"
                 % (bn, bn, bdata, bn, dmem))
        # Two inheritance levels: the delegation is still ONE step, so the class-pair
        # descriptor must still be `66 02` — the count this grammar requires literally.
        emit_raw("struct %s { ~%s(); %s };\nstruct M : %s { ~M(); %s };\n"
                 "struct D : M { ~D(); %s };\nD::~D() {}\n"
                 % (bn, bn, bdata, bn, dmem, dmem))
# The definition's SOURCE LINE, for the same reason the member-function loop below
# sweeps it: `this` is bound from the pre-body region, and the closing brace's own
# `4F 01 <line>` marker lands inside the return plumbing, which a one-line probe
# never shows. Line 70's marker is `4F 01 46` — the known-bad formals anchor.
for line in range(64, 77):
    pad = '\n'.join('// pad %d' % k for k in range(1, line - 2))
    emit_raw("struct B { ~B(); int x; };\nstruct D : B { ~D(); int y; };\n" + pad
             + "\nD::~D()\n{\n}\n")
# ---- generated empty destructors: the MEMBER sub-object form ---------------------
# The second generated destructor (`docs/IL_CALL_IN_EXPR.md` §14.3, §15): no
# destructible base, exactly one destructible **member**, receiver = `this + k`
# through a plain `27` add with no class-layout intrinsic. It is one production with
# the base form above and differs in one literal — the member's byte offset — which
# is also the entire codegen difference: nothing at 0, one `addi r3,r3,k` otherwise.
#
# So the axis to sweep is the OFFSET, and it has to be swept against everything that
# could move it independently: the padding that produces it, the member's own size
# and alignment, cv-qualification on the member (which moves the receiver's TYPE tag
# from `86` to `A6`), and the number of members (which moves `5E <n>` and turns the
# whole lowering into a frame). A fixture set cannot separate "the offset is the
# literal `k`" from "the offset is the member's index" or "…is always 4" without the
# cross product, and c2's own switch from one `addi` to `addis`+`addi` at the signed
# 16-bit edge is only visible if the sweep crosses it.
MEMS = (('MemI', '~MemI(); int a;'),          # 4 bytes
        ('MemD', '~MemD(); double a;'),       # 8, aligned 8
        ('MemC', '~MemC(); char a;'),         # 1, aligned 1
        ('MemB', '~MemB(); char b[100];'),    # large, aligned 1
        ('MemE', '~MemE();'))                 # empty: 1 byte, still destructible
# Leading padding, chosen to land the member at 0 and at a spread of offsets on both
# sides of every alignment rule, plus both sides of the `addi` immediate boundary.
PADS = ('', 'char p0;', 'char p0, p1;', 'int p0;', 'int p0, p1;', 'double p0;',
        'char p0[3];', 'char p0[7];', 'char p0[32760];', 'char p0[32764];',
        'char p0[32765];', 'char p0[32768];', 'char p0[40000];', 'char p0[65536];')
for mn, mbody in MEMS:
    for pad in PADS:
        emit_raw("struct %s { %s };\nstruct D { ~D(); %s %s m; };\nD::~D() {}\n"
                 % (mn, mbody, pad, mn))
        # The same member `const` and `volatile`: the receiver's TYPE tag picks up the
        # cv bits, and `ValueClass::Ptr4` admits four tag spellings on the claim that
        # they are all the same pointer. Only sweeping them tests that claim.
        for q in ('const', 'volatile'):
            emit_raw("struct %s { %s };\nstruct D { ~D(); %s %s %s m; };\nD::~D() {}\n"
                     % (mn, mbody, pad, q, mn))
# A NON-destructible base contributing only to the member's offset: the base's own
# size is the offset, so this is the same rule reached a different way.
for bdata in ('int b;', 'double b;', 'char b[3];', 'char b[32764];', 'char b[32768];'):
    emit_raw("struct M{~M();int a;};\nstruct B{%s};\nstruct D:B{~D();M m;};\nD::~D(){}\n" % bdata)
# A member sub-object that itself has a member sub-object: two generated destructors
# in one TU, each destroying one thing, at independent offsets.
for pad in ('', 'int p;', 'double p;'):
    emit_raw("struct M{~M();int a;};\nstruct I{~I();%s M m;};\nstruct O{~O();%s I i;};\n"
             "I::~I(){}\nO::~O(){}\n" % (pad, pad))
# A member whose own destructor is VIRTUAL. Destroying a member sub-object of known
# type is still DIRECT dispatch (`99`, not `67`/`9A`), so this must emit a bare
# branch to `??1…@@UAA@XZ` — the licence to branch comes from the bind, not from the
# callee, and that is exactly the kind of claim a sweep is for.
for pad in ('', 'int p;'):
    emit_raw("struct V{virtual ~V();int a;};\nstruct D{~D();%s V m;};\nD::~D(){}\n" % pad)
# The definition's SOURCE LINE again, for this receiver: the closing brace's
# `4F 01 <line>` marker lands inside the return plumbing, and line 70's marker is the
# known-bad formals anchor `4F 01 46`.
for line in range(64, 77):
    pad = '\n'.join('// pad %d' % k for k in range(1, line - 2))
    emit_raw("struct M{~M();int a;};\nstruct D{~D();int q; M m;};\n" + pad
             + "\nD::~D()\n{\n}\n")

# The refusing neighbours. Each is one production or one payload byte from the
# accepted shape and each costs instructions the bare branch does not emit, so a
# MISMATCH here is the alarm and NotImplemented is the right answer.
for src in (
    # Two bases: two calls, the second at a nonzero adjust, and `5E 02 21`.
    "struct M1{~M1();int a;};struct M2{~M2();int b;};\nstruct D:M1,M2{~D();};\nD::~D(){}\n",
    # A destructible MEMBER as well as a base — two calls again.
    "struct M1{~M1();int a;};struct M2{~M2();int b;};\nstruct D:M1{~D();M2 m;};\nD::~D(){}\n",
    # TWO destructible members. `5E 02`, two statements, and the reference emits a
    # FRAME: `or r31,r3,r3`, two `bl`s in REVERSE declaration order, `or r3,r31,r31`
    # between them, because `this` is live across the first call. These are the 574
    # bodies §14.3 measured as lost to the offset split, and they are lost for a real
    # reason — grammar-complete with both offsets, codegen-complete under neither.
    # Swept over the offset pair, because "the first member is at 0" is the one case
    # where a single-branch lowering would look plausible.
    "struct M1{~M1();int a;};struct M2{~M2();int b;};\nstruct D{~D();M1 m;M2 n;};\nD::~D(){}\n",
    "struct M1{~M1();int a;};struct M2{~M2();int b;};\nstruct D{~D();int q;M1 m;M2 n;};\nD::~D(){}\n",
    "struct M1{~M1();int a;};\nstruct D{~D();M1 m,n;};\nD::~D(){}\n",
    "struct M1{~M1();int a;};\nstruct D{~D();M1 m,n,o;};\nD::~D(){}\n",
    # An ARRAY of destructible members: a destruct LOOP plus the `??_I` helper, and it
    # blocks on a different opcode entirely (`5C` in an unexpected place).
    "struct M1{~M1();int a;};\nstruct D{~D();M1 m[2];};\nD::~D(){}\n",
    "struct M1{~M1();int a;};\nstruct D{~D();M1 m[3];};\nD::~D(){}\n",
    "struct M1{~M1();int a;};\nstruct D{~D();int q;M1 m[3];};\nD::~D(){}\n",
    # A member with NO destructor: nothing to destroy, so the body is empty.
    "struct M1{int a;};\nstruct D{~D();M1 m;};\nD::~D(){}\n",
    "struct M1{int a;};\nstruct D{~D();int q;M1 m;};\nD::~D(){}\n",
    # A member POINTER and a member REFERENCE to a destructible type: neither is a
    # sub-object, so neither is destroyed.
    "struct M1{~M1();int a;};\nstruct D{~D();M1* m;};\nD::~D(){}\n",
    "struct M1{~M1();int a;};\nstruct D{~D();M1& m;D(M1&);};\nD::~D(){}\n",
    # The member's destructor DEFINED in this TU: c2 may inline it rather than branch.
    "struct M1{~M1(){}int a;};\nstruct D{~D();M1 m;};\nD::~D(){}\n",
    "struct M1{~M1(){}int a;};\nstruct D{~D();int q;M1 m;};\nD::~D(){}\n",
    # A destructible member and a real statement in the body: two calls.
    "void h();\nstruct M1{~M1();int a;};\nstruct D{~D();int q;M1 m;};\nD::~D(){h();}\n",
    # A VIRTUAL destructor on the enclosing class: `??_E`/`??_G` thunks appear and the
    # body is no longer the only function emitted.
    "struct M1{~M1();int a;};\nstruct D{virtual ~D();int q;M1 m;};\nD::~D(){}\n",
    # The member sits inside a VIRTUAL base: intrinsic 2116 through a vbtable.
    "struct M1{~M1();int a;};\nstruct V{~V();M1 m;};\nstruct D:virtual V{~D();};\nD::~D(){}\n",
    # A destructible member of a TEMPLATE class, and a template member: the `.ex`
    # segment of an instantiation ends `47 54 01 54 00 4D`, which every shape refuses
    # on the module framing alone (`docs/IL_CALL_IN_EXPR.md` §13.5).
    "struct M1{~M1();int a;};\ntemplate<class T> struct D{~D();T q;M1 m;};\n"
    "template<class T> D<T>::~D(){}\ntemplate struct D<int>;\n",
    # A CONSTRUCTOR of the same class: same `0x0100` optimization-word bit, and it
    # calls the member's constructor rather than its destructor.
    "struct M1{M1();~M1();int a;};\nstruct D{D();~D();int q;M1 m;};\nD::D(){}\n",
    # A real statement in the body.
    "void h();\nstruct M1{~M1();int a;};\nstruct D:M1{~D();};\nD::~D(){h();}\n",
    # A VIRTUAL destructor: opcode `67`/`9A` dispatch, plus the `??_E`/`??_G` thunks.
    "struct M1{virtual ~M1();int a;};\nstruct D:M1{virtual ~D();};\nD::~D(){}\n",
    # A VIRTUAL base: intrinsic 2116 through a vbtable, not 2113.
    "struct V{~V();int v;};\nstruct D:virtual V{~D();};\nD::~D(){}\n",
    # The base destructor DEFINED in this TU: c2 may inline it rather than branch.
    "struct M1{~M1(){}int a;};\nstruct D:M1{~D();};\nD::~D(){}\n",
    # A base with NO destructor: nothing to delegate to.
    "struct M1{int a;};\nstruct D:M1{~D();};\nD::~D(){}\n",
    # A destructor with nothing at all to destroy: `EmptyBody`, a bare `blr`.
    "struct D{~D();int a;};\nD::~D(){}\n",
    # CONSTRUCTORS. They carry the same `0x0100` optimization-word bit that this rung
    # started masking off, so admitting that bit put them in front of the emitter too.
    "struct M1{M1();int a;};\nstruct D:M1{D();};\nD::D(){}\n",
    "struct D{D();int a;};\nD::D(){}\n",
    "struct D{D(int);int a;};\nD::D(int v){a=v;}\n",
    "struct D{D(const D&);int a;};\nD::D(const D& o){a=o.a;}\n",
    "struct M1{M1();int a;};\nstruct M2{M2();int b;};\nstruct D:M1,M2{D();};\nD::D(){}\n",
):
    emit_raw(src)

print(n)
PY

ls "$out"/*.cpp | sort > "$out/cases.txt"
total=$(wc -l < "$out/cases.txt")
if [ "$limit" -gt 0 ] 2>/dev/null; then
    head -n "$limit" "$out/cases.txt" > "$out/cases.run"
else
    cp "$out/cases.txt" "$out/cases.run"
fi
run=$(wc -l < "$out/cases.run")

# Bail out loudly rather than reporting a vacuous pass.
first=$(head -1 "$out/cases.run")
if "$c2rs" diff "$first" 2>&1 | grep -q "SKIP"; then
    echo "SKIP: toolchain absent — the sweep would be vacuous"
    exit 0
fi

echo "sweeping $run of $total generated cases"
mismatch=0
checked=0
while read -r f; do
    checked=$((checked + 1))
    verdict=$("$c2rs" diff "$f" 2>&1 | tail -1)
    case "$verdict" in
        *Mismatch*)
            mismatch=$((mismatch + 1))
            echo "MISMATCH  $(head -1 "$f")"
            ;;
    esac
done < "$out/cases.run"

echo "checked=$checked mismatches=$mismatch"
[ "$mismatch" -eq 0 ] || exit 1
