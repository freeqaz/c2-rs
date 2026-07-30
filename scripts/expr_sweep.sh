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
