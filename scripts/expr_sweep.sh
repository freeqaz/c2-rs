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
# `emit` is supplied by the LOADER (`scripts/sweep_gen.py`), which owns the
# counter and namespaces the output by fragment (`10-int-chains-0007.cpp`). A
# fragment therefore cannot see or touch another fragment's counter: the
# `n`-shadowing trap that silently rewound the file counter and overwrote 1,233
# already-written cases is now *unrepresentable*, not merely fixed. The loader
# prints a per-fragment count, **fails if any fragment emits zero cases** — the
# observable symptom of that bug is now a hard error — and **fails if what it
# counted is not what is on disk**, which is that bug's actual damage.
#
# The loader is a module rather than an inline block because it has a second
# consumer: `scripts/cross_sweep.py` grades the CROSS PRODUCT of the shape
# families these cases exercise, and two copies of the enumeration is the
# "one rule, two implementations" shape `docs/GAPS.md` §6 keeps recording.
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

python3 "$repo_root/scripts/sweep_gen.py" "$out" "$repo_root/scripts/sweep.d"

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
