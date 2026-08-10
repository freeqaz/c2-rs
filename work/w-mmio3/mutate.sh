#!/bin/sh
# w-mmio3 — THE MUTATION GRID. Committed BEFORE it is first run (#2668, #2699).
#
#   sh work/w-mmio3/mutate.sh
#
# Each cell deletes ONE clause from `crates/` — the WHOLE conjunction, never a
# term of it (#2698/#2699) — rebuilds, re-grades, and restores. A cell whose
# deletion changes nothing is a clause that is not load-bearing and the fixture
# that was supposed to grade it is vacuous.
#
# **It refuses outright on an uncommitted `crates/` or `fixtures/`**, because a
# grid that mutates a dirty tree cannot restore it and the restore is what makes
# the run repeatable. `w-decouple` §8.1 records the run where a `set -e` abort
# left the tree mutated and the guard caught it only on the NEXT invocation; the
# trap below is that fix.
#
# **`gap` exits non-zero when a TU mismatches — which is exactly what a cell is
# FOR — so every grading command is `|| true`.** Same trap, same lane.
set -eu

here=$(cd "$(dirname "$0")/../.." && pwd)
cd "$here"
out="$here/work/w-mmio3"
dc3="${C2RS_DC3:?set C2RS_DC3 to the dc3-decomp checkout}"
flags="$out/fx/one-o1/flags.txt"

if [ -n "$(git status --porcelain -- crates fixtures)" ]; then
    echo "REFUSING: crates/ or fixtures/ is dirty — commit first, the grid restores by checkout" >&2
    exit 3
fi
trap 'git checkout -- crates fixtures' EXIT

py() { python3 -c "$1"; }

grade_fixtures() {  # <tag> <fixture...>
    tag="$1"; shift
    sh "$out/fx.sh" target/release/c2rs "mut-$tag" "/nologo /c /GR /O1 /Oi /EHsc" "$@" >/dev/null 2>&1 || true
    grep -E '^  \[' "$out/fx/mut-$tag/out.txt" | sed 's/z:.*fixtures.cpp./  /'
}

grade_mmio() {      # <tag>
    tag="$1"
    ./target/release/c2rs gap --list "$out/one.txt" --flags-file "$out/../dc3-workload/flags.txt" \
        --cwd "$dc3" --jobs 1 > "$out/mut_$tag.txt" 2>&1 || true
    grep -E '^  \[1/1\]' "$out/mut_$tag.txt" | sed 's/  */ /g'
}

cell() {            # <tag> <python mutation> <what to grade>
    tag="$1"; mutation="$2"; shift 2
    echo
    echo "== $tag"
    py "$mutation"
    cargo build --release -p c2-harness >/dev/null 2>&1
    "$@"
    git checkout -- crates fixtures
    cargo build --release -p c2-harness >/dev/null 2>&1
}

echo "BASELINE (unmutated, for comparison)"
grade_fixtures base wmmio3_close_call_chain.cpp wmmio3_close_sibling_neg.cpp wmmio3_close_extern_neg.cpp
grade_mmio base

# ---- M1: the ELISION's whole conjunction ----------------------------------
# Both halves at once — "the callee is a sibling" AND "its body is pure" — so
# the mutation deletes the conjunction and not a term of it (#2699).
cell M1 '
p="crates/c2-il/src/func/bundle.rs"; s=open(p).read()
old="""            match sibling(&c.elided) {
                Some(g) if g.is_pure_expression_leaf() => {}
                _ => return None,
            }"""
assert s.count(old)==1
open(p,"w").write(s.replace(old,"            // M1",1))
' grade_fixtures M1 wmmio3_close_call_chain.cpp wmmio3_close_sibling_neg.cpp

# ---- M2: the VOID CALLEE IS EXTERNAL clause -------------------------------
cell M2 '
p="crates/c2-il/src/func/bundle.rs"; s=open(p).read()
old="""            if defined.contains(&c.void_call) {
                return None;
            }"""
assert s.count(old)==1
open(p,"w").write(s.replace(old,"            // M2",1))
' grade_fixtures M2 wmmio3_close_call_chain.cpp wmmio3_close_extern_neg.cpp

# ---- M3: the FENCE EXEMPTION DECOUPLING, reverted --------------------------
# Mechanism 7. Put the incumbent walk-based exemption back and both the fixture
# and the workload TU must stop matching, at `locally-defined-callee`.
cell M3 '
p="crates/c2-il/src/func/bundle.rs"; s=open(p).read()
old="super::gl::plain_external_names_among(gl, names.iter().map(String::as_str))"
assert s.count(old)==1
open(p,"w").write(s.replace(old,"super::gl::plain_external_defined_names(gl)",1))
' grade_fixtures M3 wmmio3_close_call_chain.cpp

# M3 again, on the workload TU this lane converted.
cell M3-mmio '
p="crates/c2-il/src/func/bundle.rs"; s=open(p).read()
old="super::gl::plain_external_names_among(gl, names.iter().map(String::as_str))"
assert s.count(old)==1
open(p,"w").write(s.replace(old,"super::gl::plain_external_defined_names(gl)",1))
' grade_mmio M3

# ---- M4: the class is FRAMED ----------------------------------------------
# Remove the `is_framed()` arm and the TU loses its `.pdata` COMDAT and its
# label triple — two sections short, `w-ifn`'s M3 shape one class over.
cell M4 '
p="crates/c2-il/src/func/mod.rs"; s=open(p).read()
old="            || self.close_call_chain.is_some()\n"
assert s.count(old)==1
open(p,"w").write(s.replace(old,"",1))
' grade_fixtures M4 wmmio3_close_call_chain.cpp

# ---- M5: the ELIDED call is EMITTED ---------------------------------------
# The emitter gains the `bl` the class exists to omit. This is the cell that
# says the elision is a real deletion and not an accident of the grammar.
cell M5 '
p="crates/c2-core/src/comdat.rs"; s=open(p).read()
old="""                coff::Call { reloc_offset: body.bl_offsets[1], callee: c.void_call.as_str() },"""
new="""                coff::Call { reloc_offset: body.bl_offsets[1], callee: c.void_call.as_str() },
                coff::Call { reloc_offset: body.bl_offsets[1], callee: c.elided.as_str() },"""
assert s.count(old)==1
open(p,"w").write(s.replace(old,new,1))
' grade_fixtures M5 wmmio3_close_call_chain.cpp

echo
echo "restored:"
git status --porcelain -- crates fixtures || true
