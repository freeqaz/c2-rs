#!/bin/sh
# Generated differential sweep over the shapes that claim byte-exactness.
#
# Enumerates small translation units over a set of axes, compiles each against the
# real toolchain, and reports every byte MISMATCH. This is the thing that found the
# reassociation and repeated-leaf mis-emits: ~20 wrong-bytes bugs in the
# straight-line class that the hand-written corpus had never separated, because
# every fixture in it happened to use distinct operands in ascending order.
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
# ---- the fragment contract (docs/ARCHITECTURE_SEAMS.md §2.4) -------------------
#
# The generator lives in `scripts/sweep.d/`, ONE FILE PER AXIS. A rung adds a NEW
# file; two rungs adding fragments never conflict, and two rungs claiming the same
# fragment name is an add/add conflict git flags loudly.
#
# Each fragment defines exactly:
#
#     def cases(emit):
#         emit("int f(int a) { return a + 1; }\n")   # one .cpp case
#
# `emit` is supplied by THIS driver, which owns the counter and namespaces the
# output by fragment (`10-int-chains-0007.cpp`). A fragment therefore cannot see
# or touch another fragment's counter: the `n`-shadowing trap that silently
# rewound the file counter and overwrote 1,233 already-written cases is now
# *unrepresentable*, not merely fixed. The driver prints a per-fragment count and
# **fails if any fragment emits zero cases** — the observable symptom of that bug
# is now a hard error.
#
# Usage:  scripts/expr_sweep.sh [outdir] [max-cases]
#         scripts/expr_sweep.sh /tmp/sweep 400     # a quick subset
#         C2RS_SWEEP_ONLY=fp scripts/expr_sweep.sh # only fragments matching "fp"
#
# `C2RS_SWEEP_ONLY` is for iterating on one axis; it makes the total meaningless
# by design, so the driver says so out loud and any gate run must be unfiltered.
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

python3 - "$out" "$repo_root/scripts/sweep.d" <<'PY'
import os, sys

out, frag_dir = sys.argv[1], sys.argv[2]
only = os.environ.get('C2RS_SWEEP_ONLY', '')

names = sorted(f for f in os.listdir(frag_dir)
               if f.endswith('.py') and not f.startswith('_'))
if not names:
    sys.exit('no sweep fragments in %s' % frag_dir)

selected = [f for f in names if only in f]
if only:
    print('C2RS_SWEEP_ONLY=%r: %d of %d fragments — THE TOTAL BELOW IS PARTIAL'
          % (only, len(selected), len(names)))
    if not selected:
        sys.exit('C2RS_SWEEP_ONLY=%r matched no fragment' % only)

total = 0
empty = []
for name in selected:
    stem = name[:-3]
    # The driver owns the counter. A fragment is handed `emit` and nothing else,
    # so it cannot reach another fragment's namespace or rewind its count.
    count = [0]

    def emit(src, _stem=stem, _count=count):
        _count[0] += 1
        path = os.path.join(out, '%s-%04d.cpp' % (_stem, _count[0]))
        with open(path, 'w') as fh:
            fh.write(src)

    path = os.path.join(frag_dir, name)
    ns = {'__name__': 'sweep_' + stem.replace('-', '_'), '__file__': path}
    exec(compile(open(path).read(), path, 'exec'), ns)
    if 'cases' not in ns:
        sys.exit('fragment %s defines no cases(emit)' % name)
    ns['cases'](emit)

    print('  fragment %-26s %5d cases' % (stem, count[0]))
    if count[0] == 0:
        empty.append(stem)
    total += count[0]

if empty:
    sys.exit('FRAGMENT EMITTED ZERO CASES: %s — a silent generator drop is a hard '
             'error here (docs/ARCHITECTURE_SEAMS.md §2.4)' % ', '.join(empty))
print('  %d fragments, %d cases total' % (len(selected), total))
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
