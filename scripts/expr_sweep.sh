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
