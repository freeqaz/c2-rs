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

# The FP **parameter list** axis, which no fixture and no sweep case had varied:
# every FP body above (and every `w13*` fixture) declares an all-`float` or
# all-`double` parameter list, which is exactly the shape where a parameter's
# positional index and its FP-register number coincide. They are two facts, and
# they come apart two ways — a non-FP parameter ahead of the floats, and an FP
# parameter the body never mentions. Both were live wrong-bytes emits
# (`fixtures/cpp/w13_fparam_neg.cpp`); a mismatch here is the alarm.
for ty in ('float', 'double'):
    for lead in ('int a', 'unsigned a', 'char a', 'long long a', 'int* a',
                 '%s a' % ty):
        for body in ('b * c', 'b + c', 'b - c', 'b / c'):
            emit_raw("%s f(%s, %s b, %s c) { return %s; }\n"
                     % (ty, lead, ty, ty, body))
        # …and with the non-FP parameter in the middle and at the end.
        emit_raw("%s f(%s b, %s, %s c) { return b * c; }\n" % (ty, ty, lead, ty))
        emit_raw("%s f(%s b, %s c, %s) { return b * c; }\n" % (ty, ty, ty, lead))
    # A bare return of an FP parameter, at every position: `fmr f1,fN` for any
    # position but the first, and nothing at all for the first.
    # (`nparam`, not `n` — `n` is this generator's file counter, and a `for n in`
    # here silently rewound it and overwrote 1,233 already-written cases.)
    for nparam in range(1, 5):
        ps = ', '.join('%s p%d' % (ty, i) for i in range(nparam))
        for i in range(nparam):
            emit_raw("%s f(%s) { return p%d; }\n" % (ty, ps, i))
    # An unused FP parameter: undecidable from `.ex` alone, so it must refuse.
    emit_raw("%s f(%s a, %s b) { return b * b; }\n" % (ty, ty, ty))
    emit_raw("%s f(%s a, %s b, %s c) { return b * c; }\n" % (ty, ty, ty, ty))
    # A member function — `this` takes a GPR and never appears in the FP file.
    emit_raw("struct C { %s m(%s x) const; };\n"
             "%s C::m(%s x) const { return x * x; }\n" % (ty, ty, ty, ty))
    emit_raw("struct C { %s m(%s x, %s y) const; };\n"
             "%s C::m(%s x, %s y) const { return x + y; }\n"
             % (ty, ty, ty, ty, ty, ty))

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

# ---- the REGISTER MOVE: `return <a formal that is not the first>` ---------------
# One `mr r3,rN` and a `blr`, where N is the formal's argument slot. The axis that
# matters is **position x value class**: the whole class rests on a formal's index
# in the list being its argument-register number, and that identity is exactly what
# a by-value aggregate or a stack-homed ninth argument breaks. Swept as a cross
# product rather than one axis at a time, because the two facts coincide for every
# scalar and only come apart when they are varied together.
MOVE_STRUCTS = (
    "struct S { int a; int b; int arr[3]; };\n"
    "struct Pair { int x, y; };\n"
    "struct Big { int a[8]; };\n"
)
# Every argument slot at every arity, so the move's source register is swept over
# the whole of r3..r10 and not only its ends.
for nargs in range(1, 9):
    ps = ', '.join('int p%d' % i for i in range(nargs))
    for i in range(nargs):
        emit_raw("int f(%s) { return p%d; }\n" % (ps, i))
        emit_raw("int g(int);\nint f(%s) { return g(p%d); }\n" % (ps, i))
# Value class x position. One GPR word is one GPR word — int, unsigned and every
# pointer spelling share the instruction — while the narrow, wide and FP classes
# refuse on their operand type ahead of any question about the move.
for ty in ('int', 'unsigned', 'short', 'long long', 'char', 'bool', 'float',
           'double', 'int*', 'const int*', 'char*', 'void*', 'int**', 'S*'):
    for pos in range(3):
        ps = ', '.join(['int a%d' % k for k in range(pos)] + ['%s v' % ty])
        emit_raw(MOVE_STRUCTS + "%s f(%s) { return v; }\n" % (ty, ps))
    emit_raw(MOVE_STRUCTS + "%s f(%s a, %s v) { return v; }\n" % (ty, ty, ty))
# The zero-offset sub-object address against its nonzero neighbour, at each
# position: the pair that separates the register move from the `addi`.
for pos in range(3):
    lead = ''.join('int a%d, ' % k for k in range(pos))
    for expr in ('&s->a', '&s->b', 's->arr', '&s->arr[1]'):
        emit_raw(MOVE_STRUCTS + "int* f(%sS* s) { return %s; }\n" % (lead, expr))
    emit_raw(MOVE_STRUCTS + "S* f(%sconst S* s) { return (S*)s; }\n" % lead)
    emit_raw(MOVE_STRUCTS + "void* f(%sS* s) { return s; }\n" % lead)
# Member functions: `this` takes r3, so the first explicit formal is r4 and every
# later one shifts with it. The off-by-one `il_this_line70.cpp` pins.
for nargs in range(1, 4):
    ps = ', '.join('int p%d' % i for i in range(nargs))
    for i in range(nargs):
        emit_raw("struct C { int m(%s) const; };\n"
                 "int C::m(%s) const { return p%d; }\n" % (ps, ps, i))
emit_raw(MOVE_STRUCTS + "struct C { S* p(S* q) const; };\n"
         "S* C::p(S* q) const { return q; }\n")
emit_raw(MOVE_STRUCTS + "struct C { int* p(S* q) const; };\n"
         "int* C::p(S* q) const { return &q->a; }\n")
# The neighbours that must NOT emit a move. A by-value aggregate wider than one
# GPR makes the index stop being the register number — `docs/GAPS.md` §6's fourth
# instance, and the reason the move is gated behind `.sy`'s declared widths — and
# an 8-byte one does not, so both must be swept or the gate is untested. A ninth
# argument is not in a register at all, and a global is not an argument.
for agg in ('Big', 'Pair'):
    emit_raw(MOVE_STRUCTS + "int f(%s v, int b) { return b; }\n" % agg)
    emit_raw(MOVE_STRUCTS + "int f(int a, %s v, int b) { return b; }\n" % agg)
    emit_raw(MOVE_STRUCTS + "%s* f(%s v, %s* p) { return p; }\n" % (agg, agg, agg))
    emit_raw(MOVE_STRUCTS + "int g(int);\nint f(%s v, int b) { return g(b); }\n" % agg)
emit_raw("int f(int a,int b,int c,int d,int e,int h,int i,int j,int k)"
         "{ return k; }\n")
emit_raw("int gv;\nint f(int a, int b) { return gv; }\n")
emit_raw("static int sv;\nint f(int a, int b) { return sv; }\n")
emit_raw("int f(int a, int b) { return b + 1; }\n")
emit_raw(MOVE_STRUCTS + "S* f(int a, S* s) { return s + 1; }\n")
emit_raw("int f(int a, int b, int c) { return a ? b : c; }\n")

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

# ---- D5: DATA-SYMBOL ADDRESSES ---------------------------------------------------
# `docs/IL_CALL_IN_EXPR.md` §17. The port emits NOTHING for this class — a data
# symbol's address needs a REFHI/REFLO pair, and two of them in one call need a
# `.rdata`-pool-relative selection that is not modeled — so every case below must be
# `NotImplemented`. That is exactly what makes the sweep worth having here: a
# relocation is the place where a wrong byte is invisible in a probe and fatal in a
# real TU, and the failure mode this guards is the port emitting a 5-section obj for
# a TU whose reference obj carries a `.rdata` string pool, a `.data`/`.bss` for a
# defined global, or four extra relocation records.
#
# The cross product is over the axes the capture showed matter: how many symbols one
# call materializes (1 / 2 / 3 — the workload is entirely 2), where they sit among
# the other arguments, whether the symbol is a string literal (its own `$SG…`
# `.rdata` entry) or a named object (an undefined external, no section at all), the
# object's LINKAGE (extern / defined here / static / const), and whether the address
# is at offset 0 or through a subscript.
DECLS = (
    "struct T;\n"
    "extern void s1(const char*);\n"
    "extern void s2(const char*, const char*);\n"
    "extern void s3(const char*, const char*, const char*);\n"
    "extern void ps(T*, const char*);\n"
    "extern void sp(const char*, T*);\n"
    "extern void psls(T*, const char*, int, const char*);\n"
    "extern void isli(int, const char*, int);\n"
    "extern int  rs(const char*);\n"
    "extern void ui(int*);\n"
    "extern void uii(int*, int*);\n"
    "extern int  gi(int);\n"
)
for body in (
    # one symbol, at every position, string and named object, void and int result
    'void f(){ s1("a"); }',
    'void f(T* p){ ps(p, "a"); }',
    'void f(T* p){ sp("a", p); }',
    'int  f(){ return rs("a"); }',
    'void f(){ ui(gA); }',
    'void f(){ ui(&gA[2]); }',
    'void f(){ ui(&gS.b); }',
    # two symbols in one call — the shape the whole workload row is
    'void f(){ s2("a", "b"); }',
    'void f(T* p){ psls(p, "expr", 42, "file"); }',
    'void f(){ uii(gA, gB); }',
    'void f(){ uii(gA, &gA[4]); }',
    'void f(){ s2("a", gC); }',
    # three
    'void f(){ s3("a", "b", "c"); }',
    # the same symbol twice: one `.rdata` entry or two?
    'void f(){ s2("a", "a"); }',
    'void f(){ uii(gA, gA); }',
    # a symbol also read elsewhere in the same body / TU
    'void f(){ ui(gA); }\nint  h(){ return gA[0]; }',
    'int  f(){ return gi(gA[1]); }',
    # literals interleaved, the assert-macro spellings
    'int  f(int a){ return gi(a + gA[0]); }',
    'void f(){ isli(1, "a", 2); }',
    # a symbol whose address is stored rather than passed
    'extern const char* gp;\nvoid f(){ gp = "a"; }',
):
    for prefix in (
        "extern int gA[8];\nextern int gB[4];\nstruct S{int a;int b;};\nextern S gS;\n"
        "extern const char gC[4];\n",
        # …and the same bodies with the objects DEFINED here, which puts a `.data`
        # or `.bss` section in the middle of the reference obj's section table.
        "int gA[8];\nint gB[4];\nstruct S{int a;int b;};\nS gS;\nconst char gC[4]={'a','b','c',0};\n",
        # …and static, which mangles to an undecorated name and is `.bss` too.
        "static int gA[8];\nstatic int gB[4];\nstruct S{int a;int b;};\nstatic S gS;\n"
        "static const char gC[4]={'a','b','c',0};\n"
        "int keep(){ return gA[0]+gB[0]+gS.a+gC[0]; }\n",
    ):
        emit_raw(DECLS + prefix + body + "\n")

# The shapes one byte away that must keep refusing: the same calls with no symbol at
# all (already in class as tail calls — these must stay `Match`, and a regression
# here would show up as a Mismatch), and the neighbours that differ from a symbol
# address by one construct.
for src in (
    # in class today: no data symbol anywhere
    "extern void s1(const char*);\nvoid f(){ s1(0); }\n",
    "extern void v1(int);\nvoid f(int a){ v1(a); }\n",
    "extern int gi(int);\nint f(int a){ return gi(a); }\n",
    # a string literal used for its LENGTH, not its address
    "extern void v1(int);\nvoid f(){ v1(sizeof(\"abcd\")); }\n",
    # a function's address, not a data object's
    "extern void h();\nextern void fp(void(*)());\nvoid f(){ fp(h); }\n",
    # a local array's address: a frame, not a relocation
    "extern void ui(int*);\nvoid f(){ int a[4]; ui(a); }\n",
    # a wide string literal: a different `.rdata` entry width
    "extern void uw(const wchar_t*);\nvoid f(){ uw(L\"ab\"); }\n",
    # an empty string literal, and one exactly at the 4-byte pool alignment
    "extern void uc(const char*);\nvoid f(){ uc(\"\"); }\n",
    "extern void uc(const char*);\nvoid f(){ uc(\"abc\"); }\n",
    "extern void uc(const char*);\nvoid f(){ uc(\"abcd\"); }\n",
):
    emit_raw(src)

# ---- address leaves: `return &s->m;` at both designators -------------------------
# `docs/IL_CALL_IN_EXPR.md` §19. The newest accepted class, and the one whose gate
# is *loosest by design*: the member's own type never reaches the emitted `addi`,
# so the address path admits every pointer TYPE where the load path beside it picks
# `lbz`/`lhz`/`lwz`/`ld` from exactly that field. Two productions therefore share
# one designator decoder and disagree about what it may carry — which is precisely
# the shape of the bug `docs/GAPS.md` §6 keeps recording, and only the cross product
# separates "the width does not matter for an address" from "the width was never
# varied in the fixtures".
#
# The axes: designator (plain `27` add vs intrinsic 2117) x member width x offset
# (including 0, which emits NOTHING, and both sides of the signed 16-bit edge) x
# base argument position (r3 vs r4 vs r5 — the `addi`'s rA field) x cv-qualification
# x the `28` subscript add x array decay.
ADDR_S = (
    "struct S1 { char a, b, c, d; };\n"
    "struct S2 { short a, b; };\n"
    "struct S4 { int a, b, c, d; };\n"
    "struct S8 { double a; int b; };\n"
    "struct SA { int h; int arr[4]; };\n"
    "struct A4 { int a0, a1; };\n"
    "struct B4 { int b0, b1, b2; };\n"
    "struct D4 : A4, B4 { int d; };\n"
    "struct AR { int t[4]; };\n"
    "struct DR : B4, AR { };\n"
)
# The plain designator: member x cv x argument position. `&r.m` through a reference
# is the same production reached from a different source spelling.
for st, mem, ty in (('S1','a','char'), ('S1','d','char'), ('S2','a','short'),
                    ('S2','b','short'), ('S4','a','int'), ('S4','d','int'),
                    ('S8','a','double'), ('S8','b','int')):
    for q in ('', 'const ', 'volatile '):
        emit_raw(ADDR_S + "%s%s* f(%s%s* p) { return &p->%s; }\n" % (q, ty, q, st, mem))
        emit_raw(ADDR_S + "%s%s* f(%s%s& r) { return &r.%s; }\n" % (q, ty, q, st, mem))
    emit_raw(ADDR_S + "%s* f(int x, %s* p) { return &p->%s; }\n" % (ty, st, mem))
    emit_raw(ADDR_S + "%s* f(int x, int y, %s* p) { return &p->%s; }\n" % (ty, st, mem))
    emit_raw(ADDR_S + "void* f(%s* p) { return &p->%s; }\n" % (st, mem))
# The subscript add, at every index including the ones that make the total zero, and
# the bare array (a `2C` decay). Two adds in a row must FOLD, where the load leaf
# beside them admits only one.
for ix in ('0', '1', '3', '-1'):
    emit_raw(ADDR_S + "int* f(SA* p) { return &p->arr[%s]; }\n" % ix)
    emit_raw(ADDR_S + "int* f(int x, SA* p) { return &p->arr[%s]; }\n" % ix)
emit_raw(ADDR_S + "int* f(SA* p) { return p->arr; }\n")
emit_raw(ADDR_S + "int* f(int x, SA* p) { return p->arr; }\n")
# The signed-16-bit edge, from both sides and at both designators. 32764 is one
# `addi`; 32768 is `addis`+`addi` and must refuse.
for pad in ('32756', '32760', '32764', '32765', '32768', '40000'):
    emit_raw("struct P { char pad[%s]; int t; };\nint* f(P* p) { return &p->t; }\n" % pad)
    emit_raw("struct BP { char pad[%s]; };\nstruct DP : BP { int t; };\n"
             "int* f(DP* p) { return &p->t; }\n" % pad)
# The intrinsic-2117 designator: every member of a two-base derived class, so the
# two literals are exercised at (0,0), (nonzero,0), (0,nonzero) and (nonzero,nonzero)
# — the only cross that separates a SUM from "whichever one is nonzero".
for mem in ('a0', 'a1', 'b0', 'b1', 'b2', 'd'):
    emit_raw(ADDR_S + "int* f(D4* p) { return &p->%s; }\n" % mem)
    emit_raw(ADDR_S + "int* f(int x, D4* p) { return &p->%s; }\n" % mem)
    emit_raw(ADDR_S + "const int* f(const D4* p) { return &p->%s; }\n" % mem)
    emit_raw(ADDR_S + "void* f(D4* p) { return &p->%s; }\n" % mem)
    # …and the LOAD of the same member, which shares the designator decoder and
    # must keep picking its instruction from the width the address path ignores.
    emit_raw(ADDR_S + "int f(D4* p) { return p->%s; }\n" % mem)
# The same through `this`, const and non-const, plus a second inheritance step
# (class descriptor `66 03` rather than `66 02`).
for mem in ('a0', 'b1'):
    emit_raw(ADDR_S + "struct C : D4 { int* g(); const int* gc() const; };\n"
             "int* C::g() { return &%s; }\n" % mem)
    emit_raw(ADDR_S + "struct C : D4 { const int* gc() const; };\n"
             "const int* C::gc() const { return &%s; }\n" % mem)
# An inherited ARRAY member: the `28` add lands AFTER the intrinsic rather than
# after a `B9`, which is the one ordering the plain form never produces.
for ix in ('0', '1', '3'):
    emit_raw(ADDR_S + "int* f(DR* p) { return &p->t[%s]; }\n" % ix)
emit_raw(ADDR_S + "int* f(DR* p) { return p->t; }\n")
# Inherited members of every width — the axis the address path deliberately drops
# and the load path must not.
ADDR_W = ("struct BW { int b0, b1; };\n"
          "struct W { char wc; short ws; int wi; long long wl; float wf; double wd; };\n"
          "struct DW : BW, W { };\n")
for mem, ty in (('wc','char'), ('ws','short'), ('wi','int'),
                ('wl','long long'), ('wf','float'), ('wd','double')):
    emit_raw(ADDR_W + "%s* f(DW* p) { return &p->%s; }\n" % (ty, mem))
    emit_raw(ADDR_W + "const %s* f(const DW* p) { return &p->%s; }\n" % (ty, mem))
    emit_raw(ADDR_W + "void* f(DW* p) { return &p->%s; }\n" % mem)
    emit_raw(ADDR_W + "%s f(DW* p) { return p->%s; }\n" % (ty, mem))
    emit_raw(ADDR_W + "int f(DW* p) { return (int)p->%s; }\n" % mem)
    emit_raw(ADDR_W + "void f(DW* p, %s v) { p->%s = v; }\n" % (ty, mem))
# The refusing neighbours. Each is one token from an accepted shape and each costs
# an instruction the single `addi` does not: a MISMATCH here is the alarm.
ADDR_V = ("struct VA { int v0, v1; };\n"
          "struct VD : virtual VA { int d2; };\n")
for src in (
    # A VIRTUAL base: intrinsic 2118, a vbtable indirection, not a constant offset.
    ADDR_V + "int* f(VD* p) { return &p->v1; }\n",
    ADDR_V + "int f(VD* p) { return p->v1; }\n",
    ADDR_V + "int* f(VD* p) { return &p->d2; }\n",
    # A variable index: the offset is not a literal at all.
    ADDR_S + "int* f(SA* p, int i) { return &p->arr[i]; }\n",
    ADDR_S + "int* f(DR* p, int i) { return &p->t[i]; }\n",
    # The address of a GLOBAL's member: a relocation pair, not an argument register.
    ADDR_S + "S4 g;\nint* f() { return &g.b; }\n",
    ADDR_S + "D4 g;\nint* f() { return &g.b1; }\n",
    # The address CONVERTED to an integer, and pointer arithmetic on the result.
    ADDR_S + "int f(S4* p) { return (int)&p->b; }\n",
    ADDR_S + "int* f(S4* p) { return &p->b + 1; }\n",
    ADDR_S + "int* f(S4* p, int i) { return &p->b + i; }\n",
    # A second statement: the production must reach the end of the segment.
    ADDR_S + "int* f(S4* p, int* q) { *q = 1; return &p->b; }\n",
    ADDR_S + "int* f(S4* p) { int* r = &p->b; return r; }\n",
    # A member of a member, and a member of a base of a member.
    ADDR_S + "struct O { int h; S4 s; };\nint* f(O* p) { return &p->s.b; }\n",
    ADDR_S + "struct O { int h; D4 d; };\nint* f(O* p) { return &p->d.b1; }\n",
    # The address of the object itself, and of a base sub-object (an upcast, which
    # is intrinsic 2114 and null-guarded — an `addi` AND a branch).
    ADDR_S + "D4* f(D4* p) { return p; }\n",
    ADDR_S + "B4* f(D4* p) { return p; }\n",
    ADDR_S + "A4* f(D4* p) { return p; }\n",
    # A member function POINTER's address, and a reference-typed member.
    ADDR_S + "struct FP { int h; void (*f)(); };\nvoid (**g(FP* p))() { return &p->f; }\n",
    # A bitfield: not addressable in C++, but the neighbouring plain member is, and
    # the layout the bitfield forces is what makes the offsets interesting.
    "struct BF { int a : 3; int b : 5; int c; };\nint* f(BF* p) { return &p->c; }\n",
):
    emit_raw(src)

# ---- pointer OPERANDS: the type gate at the LOAD, LIT, argument and result slots --
# `docs/IL_CALL_IN_EXPR.md` §21. `parse_expr` now admits a 4-byte pointer TYPE
# wherever it admits an int-like one, and the `55` formal type and the `41` result
# type were widened with it — without those two positions the operand widening
# admits no real call site at all (measured: 1.2 M functions changed census key and
# the numerator moved by exactly 0).
#
# The claim being swept is "a 4-byte pointer in a register is a 4-byte int in a
# register", and the axes are the ones that could falsify it independently: the
# POINTEE (whose width is what pointer arithmetic scales by, and which the tag
# carries in the *other* type position), the cv-spelling (which moves the tag `86`
# → `A6`/`96`/`B6`, and `A6` is a const-qualified POINTER — measured, not the
# const-qualified pointee, which stays `86`), and the ARGUMENT SLOT (because
# "already in the right register" is exactly what makes these free, and position is
# what breaks it).
PTR_S = "struct PS1 { char a; };\nstruct PS8 { double a; int b; };\n"
PTEES = ('int', 'const int', 'volatile int', 'unsigned', 'long', 'char',
         'const char', 'short', 'float', 'double', 'long long', 'void',
         'int*', 'PS1', 'PS8')
for pte in PTEES:
    ty = '%s*' % pte
    # the LOAD position, with the result staying int (only the operand is a pointer)
    emit_raw(PTR_S + "int g(%s);\nint f(%s p){ return g(p); }\n" % (ty, ty))
    # …and with the RESULT position a pointer too
    emit_raw(PTR_S + "%s g(%s);\n%s f(%s p){ return g(p); }\n" % (ty, ty, ty, ty))
    # the LIT position: a null pointer constant in an argument, and as a whole body
    emit_raw(PTR_S + "int g(%s);\nint f(){ return g(0); }\n" % ty)
    emit_raw(PTR_S + "%s f(){ return 0; }\n" % ty)
    # a const-qualified POINTER (tag `A6`), at the load and at the formal
    emit_raw(PTR_S + "int g(%s const);\nint f(%s const p){ return g(p); }\n" % (ty, ty))
# The four tag spellings `is_ptr4_kind` admits, as the loaded value itself, plus the
# code-pointer kind (`44`) that shares the predicate with the data one (`43`).
for decl, arg in (('int* p', 'p'), ('const int* p', 'p'), ('volatile int* p', 'p'),
                  ('int* const p', 'p'), ('int (*p)(int)', 'p'), ('void (*p)()', 'p'),
                  ('int (**p)(int)', 'p')):
    emit_raw("int g1(int);\n" +
             "int f(%s){ return (int)(long)%s; }\n" % (decl, arg))
for cal, decl in (('int g(int (*)(int));', 'int (*p)(int)'),
                  ('int g(void (*)());', 'void (*p)()'),
                  ('int g(int**);', 'int** p'),
                  ('int g(const int*);', 'const int* p'),
                  ('int g(volatile int*);', 'volatile int* p')):
    emit_raw("%s\nint f(%s){ return g(p); }\n" % (cal, decl))

# Every ARGUMENT SLOT, pointer against int, at every arity the class accepts. A gate
# written for "the pointer is the first argument" passes the one-argument case and
# every all-pointer case, and fails only here.
for n_args in (1, 2, 3, 4):
    for slot in range(n_args):
        tys = ['int'] * n_args
        tys[slot] = 'int*'
        params = ', '.join('%s a%d' % (t, i) for i, t in enumerate(tys))
        args = ', '.join('a%d' % i for i in range(n_args))
        emit_raw("int g(%s);\nint f(%s){ return g(%s); }\n"
                 % (', '.join(tys), params, args))
        # the same slot taking a null pointer LITERAL instead of a passed-in value
        pars2 = ', '.join('%s a%d' % (t, i) for i, t in enumerate(tys) if i != slot)
        args2 = ', '.join('0' if i == slot else 'a%d' % i for i in range(n_args))
        emit_raw("int g(%s);\nint f(%s){ return g(%s); }\n"
                 % (', '.join(tys), pars2 if pars2 else 'void', args2))
    # all pointers, and pointers of MIXED pointee width in one call
    allp = ', '.join(['int*'] * n_args)
    emit_raw("int g(%s);\nint f(%s){ return g(%s); }\n"
             % (allp, ', '.join('int* a%d' % i for i in range(n_args)),
                ', '.join('a%d' % i for i in range(n_args))))
    mix = [('char*', 'short*', 'int*', 'double*')[i % 4] for i in range(n_args)]
    emit_raw("int g(%s);\nint f(%s){ return g(%s); }\n"
             % (', '.join(mix),
                ', '.join('%s a%d' % (t, i) for i, t in enumerate(mix)),
                ', '.join('a%d' % i for i in range(n_args))))

# ---- the ARITHMETIC BOUNDARY, which is the whole reason the guard exists ----------
# `p + 1` is `addi r3,r3,4` for an `int*` and `addi r3,r3,1` for a `char*`, so the
# increment is the POINTEE's width and an add chain that used the literal would be
# wrong bytes for every width but one. MEASURED (§21.1): c1xx pre-scales, so the IL
# literal is already 4 — but that is a *second* rule, and until it is graded on its
# own axis every one of these must be `NotImplemented`. The sweep is what turns
# "must refuse" into a fact: a MISMATCH here is the alarm.
for pte in ('char', 'short', 'int', 'long', 'long long', 'double', 'int*', 'PS8'):
    ty = '%s*' % pte
    for e in ('p + 1', 'p - 1', 'p + 3', 'p + k', 'p - k', '1 + p', 'p + 0',
              'p + (k * 2)', 'p - (k + 1)'):
        emit_raw(PTR_S + "%s f(%s p, int k){ return %s; }\n" % (ty, ty, e))
    # the same arithmetic in an ARGUMENT position, where a different `parse_expr`
    # call sees it, and in a RETURNED-through-a-call position
    emit_raw(PTR_S + "int g(%s);\nint f(%s p){ return g(p + 1); }\n" % (ty, ty))
    emit_raw(PTR_S + "int g(%s, int);\nint f(%s p, int k){ return g(p + k, k); }\n"
             % (ty, ty))
    # pointer DIFFERENCE: the front end divides by the pointee width, which for a
    # power of two is an arithmetic shift the operand vocabulary refuses anyway —
    # so this class fails closed twice, and the sweep says so rather than assuming.
    emit_raw(PTR_S + "int f(%s p, %s q){ return (int)(p - q); }\n" % (ty, ty))
# A pointer and an int in one expression with the arithmetic on the INT — the guard
# is on the whole value, so these refuse too, and that cost is measured not argued.
for e in ('g(p, a + 1)', 'g(p, a * b)', 'g(p, 1)'):
    emit_raw("int g(int*, int);\nint f(int* p, int a, int b){ return %s; }\n" % e)

# ---- the refusing NEIGHBOURS of the widened gate ---------------------------------
# Each is one token from an admitted shape and each costs an instruction the tail
# call and the identity do not emit.
for src in (
    # `this` reached through a cv-strip: the A6-tagged LOAD is admitted and the `2C`
    # after it is not, which is where 98.6 % of the pointer-type population went.
    "struct C { int v; int m(); };\nint gC(C*);\nint C::m(){ return gC(this); }\n",
    "struct C { int v; int m() const; };\nint gC(const C*);\nint C::m() const { return gC(this); }\n",
    # a pointer COMPARED rather than passed: a relational opcode, not an operand
    "int f(int* p){ return p != 0; }\n",
    "int f(int* p, int* q){ return p == q; }\n",
    # a pointer DEREFERENCED in an argument: a `30` load, gated separately
    "int g(int);\nint f(int* p){ return g(*p); }\n",
    # the ADDRESS of a local passed as an argument: a frame, and a `27` with no base
    "int g(int*);\nint f(int a){ return g(&a); }\n",
    # an 8-byte operand that is NOT a pointer: the width gate must still refuse
    "int g(long long);\nint f(long long a){ return g(a); }\n",
    "long long f(long long a){ return a; }\n",
    # a REFERENCE parameter, which is a pointer in the IL but is dereferenced on use
    "int g(int&);\nint f(int& r){ return g(r); }\n",
    # a pointer through a varargs callee: the calling-convention byte is `40`
    "int g(int*, ...);\nint f(int* p){ return g(p, 1); }\n",
    # a pointer to an AGGREGATE returned by value: an sret bind, not an operand
    "struct BigA { int a[8]; };\nBigA g(int*);\nBigA f(int* p){ return g(p); }\n",
    # a FLOAT beside a pointer: two register files, and only one of them is modeled
    "int g(int*, float);\nint f(int* p, float x){ return g(p, x); }\n",
    # nine pointer arguments: past the eighth the class refuses on the frame
    "int g(int*,int*,int*,int*,int*,int*,int*,int*,int*);\n"
    "int f(int* a,int* b,int* c,int* d,int* e,int* h,int* i,int* j,int* k)"
    "{ return g(a,b,c,d,e,h,i,j,k); }\n",
):
    emit_raw(src)


# ---- W19: the CONSTRUCTOR EPILOGUE, `return this` after the RETURN ---------------
# The cross product the hand-written fixture cannot be: every member layout against
# every parameter list. A constructor's body carries a value expression between the
# `29` RETURN and the function tail, and the claim this rung makes is that it costs
# no instruction at all — so these must grade `Match`, not merely refuse. Loop
# variables are named for what they hold; `n` is the generator's own file counter
# and rebinding it silently overwrites already-written cases (docs/GAPS.md §6).
ctor_members = [
    '',
    'int m;',
    'int m, n2;',
    'double d;',
    'float f2;',
    'int arr[4];',
    'char c;',
    'void *vp; const char *cp;',
    'long long ll;',
]
ctor_params = [
    '',
    'int a',
    'int a, int b',
    'int a, int b, int c, int d',
    'float x',
    'double x',
    'const char *s',
    'int a, float x',
    'float x, int a',
    'int a, int b, int c, int d, int e, int g2, int h, int i',
]
for mi, mem in enumerate(ctor_members):
    for pi, plist in enumerate(ctor_params):
        emit_raw("struct S%d_%d { %s S%d_%d(%s); };\nS%d_%d::S%d_%d(%s) {}\n"
                 % (mi, pi, mem, mi, pi, plist, mi, pi, mi, pi, plist))
# The copy constructor, an 8-byte by-value aggregate parameter, and the shapes a
# per-class loop cannot reach.
for extra in (
    "struct CpA { int m; CpA(); CpA(const CpA &); };\nCpA::CpA() {}\nCpA::CpA(const CpA &o) {}\n",
    "struct CpB { double d; CpB(); CpB(const CpB &); };\nCpB::CpB() {}\nCpB::CpB(const CpB &o) {}\n",
    "struct PairS { int x, y; };\nstruct AgA { int m; AgA(PairS); };\nAgA::AgA(PairS v) {}\n",
    "struct PairS { int x, y; };\nstruct AgB { int m; AgB(PairS, int); };\nAgB::AgB(PairS v, int b) {}\n",
    "struct RefA { int m; RefA(int &); };\nRefA::RefA(int &r) {}\n",
    "struct PtrA { int m; PtrA(int *, int *); };\nPtrA::PtrA(int *p, int *q) {}\n",
    # several byte-identical bodies in ONE translation unit: the locality tell
    "struct L1 { int m; L1(); };\nstruct L2 { int m; L2(); };\nstruct L3 { int m; L3(); };\n"
    "L1::L1() {}\nL2::L2() {}\nL3::L3() {}\n",
    # the same, interleaved with the empty bodies that have NO epilogue
    "struct L4 { int m; L4(); };\nvoid e1() {}\nL4::L4() {}\nvoid e2() {}\n",
    # a nested class and a class in a namespace: the mangled name changes, the body
    # does not
    "struct Out { struct In { int m; In(); }; };\nOut::In::In() {}\n",
    "namespace ns { struct NsA { int m; NsA(); }; }\nns::NsA::NsA() {}\n",
    # a destructor with an empty body: the control, it has no epilogue
    "struct DtA { int m; ~DtA(); };\nDtA::~DtA() {}\n",
    # an empty member function and an empty static member: the other controls
    "struct MfA { int m; void v() const; static void s(); };\nvoid MfA::v() const {}\n"
    "void MfA::s() {}\n",
) + tuple(
    # every argument slot r4..r10 filled ahead of nothing being read, at each arity
    "struct Ar%d { int m; Ar%d(%s); };\nAr%d::Ar%d(%s) {}\n"
    % (k, k, ', '.join('int a%d' % j for j in range(k)),
       k, k, ', '.join('int a%d' % j for j in range(k)))
    for k in range(1, 9)
):
    emit_raw(extra)
# The refusing NEIGHBOURS of this gate. A call spills `this` to a nonvolatile and
# restores it (`mr r31,r3` … `mr r3,r31`), which is the frame axis; a store through
# `this` is the `27` designator. Both must stay `NotImplemented`, and a MISMATCH
# here is the alarm this block exists to raise.
for neighbour in (
    "struct NB { int b; NB(); };\nstruct ND : NB { ND(); };\nND::ND() {}\n",
    "void sfx();\nstruct NC { int m; NC(); };\nNC::NC() { sfx(); }\n",
    "struct NM { int m; NM(); };\nstruct NH { NM sub; NH(); };\nNH::NH() {}\n",
    "struct NS1 { int m; NS1(int); };\nNS1::NS1(int a) { m = a; }\n",
    "struct NS2 { int m; NS2(int); };\nNS2::NS2(int a) : m(a) {}\n",
    "struct NS3 { int m, n2; NS3(int, int); };\nNS3::NS3(int a, int b) : m(a), n2(b) {}\n",
    "struct NV { int m; virtual void f(); NV(); };\nNV::NV() {}\n",
    "struct NVi { int m; virtual void f(); NVi(); };\nvoid NVi::f() {}\nNVi::NVi() {}\n",
    # a returned object that is NOT `this`: the epilogue names another token
    "struct NRv { int m; NRv(); };\nNRv mk();\nNRv mk() { NRv v; return v; }\n",
    # a constructor that returns early: the body is not empty, it branches
    "struct NBr { int m; NBr(int); };\nNBr::NBr(int a) { if (a) return; }\n",
    # virtual inheritance: the epilogue is there, the body installs a vbtable
    "struct VB { int b; VB(); };\nstruct VD : virtual VB { VD(); };\nVD::VD() {}\n",
):
    emit_raw(neighbour)

# ---- W20: the `2C` CONVERSION in a general expression operand position -------
#
# `docs/IL_CALL_IN_EXPR.md` §24. A conversion whose target is the value's own
# 4-byte class emits NOTHING, so admitting it must leave the emitted bytes of the
# surrounding chain exactly as they were. The axis that matters — and the one no
# leaf shape could reach — is WHERE the conversion sits relative to the operands
# and the operator, because that is the layer the reassociation mis-emits lived
# in. Every combination below is generated rather than picked.
INT4 = ['int', 'unsigned', 'long', 'unsigned long']
# 1. the whole spelling matrix, source x target, as a one-operand body.
for src_t in INT4:
    for dst_t in INT4:
        emit_raw("%s f(%s a) { return (%s)a; }\n" % (dst_t, src_t, dst_t))
# 2. the conversion at each position of a two-operand chain, over every operator
#    and both operand orders (a literal on either side included).
for op in ['+', '-', '*']:
    for l, r in [('a', 'b'), ('b', 'a'), ('a', '3'), ('3', 'a'), ('a', 'a')]:
        emit_raw("unsigned f(int a, int b) { return (unsigned)%s %s %s; }\n" % (l, op, r))
        emit_raw("unsigned f(int a, int b) { return %s %s (unsigned)%s; }\n" % (l, op, r))
        emit_raw("unsigned f(int a, int b) { return (unsigned)(%s %s %s); }\n" % (l, op, r))
# 3. three-operand chains with the conversion at each of the four slots.
for op1 in ['+', '-', '*']:
    for op2 in ['+', '-', '*']:
        emit_raw("unsigned f(int a, int b, int c) { return (unsigned)a %s b %s c; }\n" % (op1, op2))
        emit_raw("unsigned f(int a, int b, int c) { return a %s (unsigned)b %s c; }\n" % (op1, op2))
        emit_raw("unsigned f(int a, int b, int c) { return a %s b %s (unsigned)c; }\n" % (op1, op2))
        emit_raw("unsigned f(int a, int b, int c) { return (unsigned)(a %s b) %s c; }\n" % (op1, op2))
        emit_raw("unsigned f(int a, int b, int c) { return (unsigned)(a %s b %s c); }\n" % (op1, op2))
# 4. nested conversions: the round trip, and back again.
for depth in ['(unsigned)a', '(unsigned)(int)a', '(int)(unsigned)a', '(unsigned)(int)(unsigned)a']:
    emit_raw("unsigned f(int a) { return %s; }\n" % depth)
    emit_raw("unsigned f(int a, int b) { return %s + b; }\n" % depth)
# 5. the converted formal at every argument slot, at every arity — the D10
#    register move underneath the conversion.
for k in range(1, 9):
    params = ', '.join('int a%d' % j for j in range(k))
    emit_raw("unsigned f(%s) { return (unsigned)a%d; }\n" % (params, k - 1))
# 6. the conversion inside a CALL-ARGUMENT region, which is `parse_expr`'s other
#    caller and where the workload's `calls-1` half lives.
emit_raw("int g1(int);\nint f(unsigned a) { return g1((int)a); }\n")
emit_raw("int g1(int);\nint f(int a, int b) { return g1((int)(a + b)); }\n")
emit_raw("int g1(int);\nint f(int a, int b) { return g1((int)a + b); }\n")
emit_raw("int g2(int, int);\nint f(unsigned a, unsigned b) { return g2((int)a, (int)b); }\n")
emit_raw("int g2(int, int);\nint f(unsigned a, int b) { return g2((int)a, b); }\n")
emit_raw("int g2(int, int);\nint f(int a, unsigned b) { return g2(a, (int)b); }\n")
emit_raw("int g3(int, int, int);\nint f(unsigned a, int b, int c) { return g3((int)a, b, c); }\n")
emit_raw("int g3(int, int, int);\nint f(int a, unsigned b, int c) { return g3(a, (int)b, c); }\n")
emit_raw("int g3(int, int, int);\nint f(int a, int b, unsigned c) { return g3(a, b, (int)c); }\n")
# 7. the POINTER half: every pointee width against every target pointer spelling,
#    as a tail-call argument (the workload shape) and as a whole body.
PTEE = ['char', 'short', 'int', 'double', 'void', 'S']
PDST = ['void *', 'const void *', 'const S *', 'S *']
for ptee in PTEE:
    emit_raw("struct S { int m; };\nint gv(void *);\n"
             "int f(%s *p) { return gv((void *)p); }\n" % ptee)
    emit_raw("struct S { int m; };\n"
             "void *f(%s *p) { return (void *)p; }\n" % ptee)
for dst_t in PDST:
    emit_raw("struct S { int m; };\nint gq(%s);\n"
             "int f(S *p) { return gq((%s)p); }\n" % (dst_t, dst_t))
    emit_raw("struct S { int m; };\n%s f(S *p) { return (%s)p; }\n" % (dst_t, dst_t))
# a pointer conversion at each argument slot of a multi-argument tail call
for k in range(1, 4):
    for j in range(k):
        args = ', '.join('void *' if i == j else 'int' for i in range(k))
        params = ', '.join(('S *p%d' % i) if i == j else ('int a%d' % i) for i in range(k))
        actual = ', '.join(('(void *)p%d' % i) if i == j else ('a%d' % i) for i in range(k))
        emit_raw("struct S { int m; };\nint gp%d(%s);\nint f(%s) { return gp%d(%s); }\n"
                 % (k, args, params, k, actual))
# 8. a member function, where `this` is a const pointer in r3
emit_raw("struct C { int m; unsigned u(int a) const; };\n"
         "unsigned C::u(int a) const { return (unsigned)a; }\n")
emit_raw("struct C { int m; unsigned u(int a, int b) const; };\n"
         "unsigned C::u(int a, int b) const { return (unsigned)b; }\n")
emit_raw("struct C { int m; int c() const; };\nint gv(void *);\n"
         "int C::c() const { return gv((void *)this); }\n")
# 9. the REFUSING neighbours. Every one of these emits an instruction the modeled
#    chain cannot produce, or is a reinterpret that has never been graded — a
#    MISMATCH here is the alarm this block exists to raise.
for dst_t in ['char', 'signed char', 'unsigned char', 'short', 'unsigned short',
              'long long', 'unsigned long long', 'float', 'double', 'bool']:
    emit_raw("%s f(int a) { return (%s)a; }\n" % (dst_t, dst_t))
    emit_raw("%s f(int a, int b) { return (%s)(a + b); }\n" % (dst_t, dst_t))
    emit_raw("int g1(int);\n%s f(int a) { return (%s)g1(a); }\n" % (dst_t, dst_t))
for src_t in ['char', 'short', 'long long', 'float', 'double']:
    emit_raw("int f(%s a) { return (int)a; }\n" % src_t)
    emit_raw("unsigned f(%s a) { return (unsigned)a; }\n" % src_t)
# the cross-class reinterpret, both directions, body and argument
emit_raw("struct S { int m; };\nint f(S *p) { return (int)p; }\n")
emit_raw("struct S { int m; };\nunsigned f(S *p) { return (unsigned)p; }\n")
emit_raw("struct S { int m; };\nS *f(int a) { return (S *)a; }\n")
emit_raw("struct S { int m; };\nS *f(unsigned a) { return (S *)a; }\n")
emit_raw("struct S { int m; };\nint g1(int);\nint f(S *p) { return g1((int)p); }\n")
emit_raw("struct S { int m; };\nint f(S *p, int k) { return (int)p + k; }\n")
# a conversion whose value then does pointer arithmetic — the §21 guard, reached
# through a conversion rather than straight off a LOAD
emit_raw("struct S { int m; };\nvoid *f(S *p) { return (void *)(p + 1); }\n")
emit_raw("struct S { int m; };\nint f(S *p, S *q) { return (int)(p - q); }\n")
emit_raw("struct S { int m; };\nvoid *f(S *p, int k) { return (void *)(p + k); }\n")
# a cv-qualified operand TYPE, which blocks at the operand and never reaches the
# conversion — a different key, kept here so a regression cannot merge the two
emit_raw("int f(const int a) { return (int)a; }\n")
emit_raw("unsigned f(const int a, int b) { return (unsigned)(a + b); }\n")


# ---- W-UNW-1: framed functions, `.pdata`, and the compiler label counter --------
# The framed class was single-function-per-TU for its whole life, so nothing here
# had ever been swept: the `.pdata` record, its relocation, the `$M`/`$T` label
# numbers and — the part that actually broke — the `bl` displacement, which
# follows `disp = −(own .text offset)` and was hardcoded to the offset of the one
# body the port could emit.
#
# Two axes, and the cross product is the point. **Position** decides the `bl`
# word and the `.text` offset of both `$M` labels; **the kinds of the preceding
# functions** decide the label numbers, because the counter is consumed by every
# function whether or not it emits a label. A sweep over position alone with one
# leaf kind would grade neither.
FRAMED_LEAFMATES = [
    'int L%d(int a) { return a + %d; }',
    'int L%d(int a) { return a - %d; }',
    'int L%d(int a) { return a; }',
    'int L%d(int a, int b) { return b; }',
    'int L%d(int a, int b) { return a + b; }',
    'int L%d(int a) { return g(a); }',
    'int L%d(int *p) { return *p; }',
    'struct S%d { int m; };\nint L%d(S%d *p) { return p->m; }',
]
def leafmate(kind, ix):
    t = FRAMED_LEAFMATES[kind]
    if t.startswith('struct'):
        return t % (ix, ix, ix)
    return t % ((ix, ix + 1) if '%d;' in t or t.count('%d') == 2 else (ix,))
def framed_fn(ix, k, callee):
    return 'int F%d(int a) { return %s(a) + %d; }' % (ix, callee, k)

# 1. Runs of framed functions on their own: 1..3 of them, shared and distinct
#    callees, and `+k` values that move only the `addi` immediate.
for count in (1, 2, 3):
    for distinct in (0, 1):
        callees = ['g%d' % j if distinct else 'g' for j in range(count)]
        decls = ''.join('int %s(int);\n' % c for c in sorted(set(callees)))
        body = '\n'.join(framed_fn(j, j + 1, callees[j]) for j in range(count))
        emit_raw(decls + body + '\n')

# 2. One framed function at every position among 0..3 leafmates of one kind.
for kind in range(len(FRAMED_LEAFMATES)):
    for mates in range(4):
        for pos in range(mates + 1):
            parts = []
            for j in range(mates + 1):
                parts.append(framed_fn(j, 1, 'g') if j == pos else leafmate(kind, j))
            emit_raw('int g(int);\n' + '\n'.join(parts) + '\n')

# 3. Two framed functions with leafmates of MIXED kinds between and around them —
#    the shape where a per-kind counter stride error and a position error can
#    cancel in one arrangement and not another.
for k1 in range(len(FRAMED_LEAFMATES)):
    for k2 in range(len(FRAMED_LEAFMATES)):
        for layout in ('FLFL', 'LFLF', 'FLLF', 'LFFL'):
            parts = []
            leaf_kinds = [k1, k2]
            li = 0
            for j, ch in enumerate(layout):
                if ch == 'F':
                    parts.append(framed_fn(j, j + 1, 'g'))
                else:
                    parts.append(leafmate(leaf_kinds[li % 2], j))
                    li += 1
            emit_raw('int g(int);\n' + '\n'.join(parts) + '\n')

# 4. The neighbours whose LABEL STRIDE decides the framed function's `$M`
#    numbers. The counter is advanced by every function in the TU whether or not
#    it emits a label, so a neighbour with a stride the emitter models wrongly
#    gives the framed function `$M` numbers that link and are wrong.
#
#    The stride is 1 for every class the port emits EXCEPT the comparison leaf,
#    which is 1 or 3 by relation, and the floating-point leaf, which is 2 (4 or 6
#    with pooled constants). Both lists are swept: the first must MATCH, the
#    second must refuse. A mismatch in either is the gate having a hole, and a
#    *refusal* in the first list is the gate over-refusing — cheaper, but it is
#    what this axis was added to measure.
FRAMED_STRIDE1 = [
    'int R(int x) { return x < 0; }',
    'int R(int x) { return x >= 0; }',
    'int R(int x) { return x == 0; }',
    'int R(int x) { return x != 0; }',
    'int R(int x) { return x == 5; }',
    'int R(int x) { return x != -5; }',
    'int R(int x) { return x == 32767; }',
    'int R(unsigned x) { return x < 5u; }',
    'int R(unsigned x) { return x >= 5u; }',
    'int R(unsigned x) { return x > 5u; }',
    'int R(unsigned x) { return x <= 5u; }',
]
FRAMED_REFUSERS = [
    'float R(float x, float y) { return x * y; }',
    'double R(double x, double y) { return x + y; }',
    'float R(float x) { return x * 2.5f; }',
    'int R(int x, int y) { return x < y; }',
    'int R(int x, int y) { return x >= y; }',
    'int R(int x) { return x < 5; }',
    'int R(int x) { return x > 0; }',
    'int R(int x) { return x <= 0; }',
    'int R(int x, int y) { return x == y; }',
]
for r in FRAMED_REFUSERS + FRAMED_STRIDE1:
    emit_raw('int g(int);\n%s\nint F(int a) { return g(a) + 1; }\n' % r)
    emit_raw('int g(int);\nint F(int a) { return g(a) + 1; }\n%s\n' % r)
    emit_raw('int g(int);\nint F1(int a) { return g(a) + 1; }\n%s\n'
             'int F2(int a) { return g(a) + 2; }\n' % r)

# 5. THE FRAMED CALL'S ARGUMENT REGISTER — the axis every case above holds
#    fixed. `framed_fn` is `int F(int a) { return g(a) + 1; }`: one parameter,
#    necessarily in r3, so the argument's *index* and its *register* were the
#    same number in all 363 framed cases and in every framed fixture. c2 emits
#    `or r3,rN,rN` when they differ and the port emitted nothing — a live
#    wrong-bytes emit found 2026-07-30 by compiling the neighbours rather than
#    by any instrument (`docs/GAPS.md` §6). Two things shift the register: the
#    argument's position among the formals, and the ABI footprint of whatever
#    precedes it — including a leading `float`/`double`/`long long`, which take
#    a GPR slot each on this ABI even though they are passed elsewhere.
for nf in range(1, 6):
    ps = ', '.join('int p%d' % i for i in range(nf))
    for i in range(nf):
        emit_raw('int g(int);\nint F(%s) { return g(p%d) + %d; }\n' % (ps, i, i + 1))
        # …and with a leaf ahead of it, so the `bl` displacement and the label
        # counter move at the same time as the argument register.
        emit_raw('int g(int);\nint L(int a) { return a + 1; }\n'
                 'int F(%s) { return g(p%d) + %d; }\n' % (ps, i, i + 1))
#    Past the eighth formal the argument is stack-homed (`lwz r3,180(r1)`), which
#    the register-move model cannot express and which the constant-body emitter
#    used to answer with no instruction at all. Refused; a MISMATCH here is that
#    gate having a hole.
for nf in (8, 9, 10):
    ps = ', '.join('int p%d' % i for i in range(nf))
    for i in (0, nf - 1):
        emit_raw('int g(int);\nint F(%s) { return g(p%d) + 1; }\n' % (ps, i))
FRAMED_ARG_LEADERS = ['float x', 'double x', 'long long x', 'int *x', 'char x',
                      'short x', 'unsigned x', 'float x, float y', 'int *x, int *y']
for lead in FRAMED_ARG_LEADERS:
    emit_raw('int g(int);\nint F(%s, int a) { return g(a) + 1; }\n' % lead)
    emit_raw('int g(int);\nint F(%s, int a, int b) { return g(b) + 1; }\n' % lead)
# Member functions: `this` is r3, so every formal is shifted by one.
for nf in range(1, 4):
    ps = ', '.join('int p%d' % i for i in range(nf))
    for i in range(nf):
        emit_raw('int g(int);\nstruct S { int m; int F(%s); };\n'
                 'int S::F(%s) { return g(p%d) + %d; }\n' % (ps, ps, i, i + 1))


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
