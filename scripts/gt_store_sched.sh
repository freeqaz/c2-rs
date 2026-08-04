#!/bin/sh
# gt_store_sched.sh — ground-truth probe grid for the SCHEDULE of a mixed store
# run: where c2 places the instruction that MATERIALIZES a stored value
# relative to the store that consumes it.
#
# Why this exists (lane w-pair, 2026-08-04). `src/xdk/nuispeech/xboxheap.cpp` is
# one of exactly two straight-line-only TUs on the pre-Phase-7 frontier, and its
# constructor is a store run whose value-producing instructions are interleaved
# between the stores. `crates/c2-il/src/func/body/shapes/leaf_store.rs` already
# records four allocation rules fitted to this family and each refuted by
# another cell of the same grid (`GAPS.md` §6 instance #10 — measure at the
# edge, do not fit the scheduler). This grid adds the PLACEMENT axis those four
# did not cover, and refutes a fifth and a sixth.
#
# The measure is the GAP: slots between a value-producing instruction and the
# first store that consumes it. Two cell groups end the exercise:
#
#   E1/E2  the same source shape with the referenced sub-object moved between
#          the two pointer parameters. Gap 1 vs gap 3 — so the placement is not
#          a function of the dataflow, and it suggests the producer's base
#          register decides.
#   F1/F2  the CONTROLLED swap of that suggestion: identical statement
#          structure, the two parameters exchanged. BOTH gap 1, and F2's
#          producer reads r4 exactly as E2's does. The register rule dies here.
#
# Six candidate rules have now been refuted by a cell of this grid. See
# docs/rungs/_2026-08-04-w-pair-findings.md §4 for the table, and do not fit a
# seventh without a cell that survives all 23.
#
# Read-only measurement tooling. Outside the std-only Rust workspace on purpose,
# same status as scripts/gt_capture.sh, which it drives.
#
# Usage:
#   scripts/gt_store_sched.sh [outdir]        # default: work/store-sched
#
# Env: as scripts/gt_capture.sh (C2RS_WIBO, C2RS_COMPILERS).
#
# Prints one block per probe function: its name and its disassembly, so the
# emitted slot order can be read directly against the source order below.
set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-$repo_root/work/store-sched}"
mkdir -p "$out"

# The workload's optimization flags. NOT the CLI default /Ox /GS- /c — the
# frontier TU this grid is about compiles at /O1, and a cross-flag comparison
# here would be measuring the wrong compiler mode.
FLAGS="/nologo /c /GR /O1 /Oi /EHsc"

cat > "$out/sched.cpp" <<'EOF'
// Probe grid — placement of a value-materializing instruction in a store run.
// Every cell's SOURCE order is the declaration order of its statements; read it
// against the emitted slot order to see what moved.
struct B { B* n; B* p; };
struct H { H* fh; H* uh; B lh; unsigned sz; unsigned ct; };
struct S8 { int a,b,c,d,e,f,g,h; };

// --- controls: no producer at all -----------------------------------------
// C0  KNOWN-ANSWER CONTROL. The shape try_parse_store_run already accepts and
//     the port already grades byte-exact. Must be three stores, source order,
//     no setup instruction. If this cell is not that, the grid is void.
void c0(S8* s, int u, int v, int w) { s->a=u; s->b=v; s->c=w; }

// --- one `li` producer, varying the consumer's position and the run length --
void c1 (S8* s, int u)                 { s->a=u; s->b=0; }
void c2f(S8* s, int u, int v)          { s->a=u; s->b=0; s->c=v; }
void d1 (S8* s, int u)                 { s->a=0; s->b=u; }
void d2 (S8* s, int u, int v)          { s->a=0; s->b=u; s->c=v; }
void d3 (S8* s, int u, int v, int w)   { s->a=0; s->b=u; s->c=v; s->d=w; }
void d7 (S8* s, int u, int v, int w)   { s->a=u; s->b=0; s->c=v; s->d=w; }
void d8 (S8* s, int u, int v, int w, int x) { s->a=0; s->b=u; s->c=v; s->d=w; s->e=x; }
void c7 (S8* s, int a, int b, int c, int d, int e, int f)
       { s->a=a; s->b=b; s->c=c; s->d=d; s->e=e; s->f=f; s->g=0; }
void c8 (S8* s, int a, int b, int c, int d, int e, int f)
       { s->a=0; s->b=a; s->c=b; s->d=c; s->e=d; s->f=e; s->g=f; }

// --- the producer is an `addi` from a FORMAL, not a literal ----------------
void d6(S8* s, int u, int v, int w) { s->a=u+1; s->b=v; s->c=w; s->d=u; }

// --- two producers with distinct values ------------------------------------
void e5(S8* s, int u, int v) { s->a=1; s->b=2; s->c=u; s->d=v; }

// --- the producer is a sub-object ADDRESS (xboxheap's `auto& l = mListHead`) -
void c5(H* h) { B& l = h->lh; l.n=&l; l.p=&l; }
void d5(H* h, unsigned u, unsigned v) { B& l=h->lh; l.n=&l; h->sz=u; h->ct=v; l.p=&l; }
void e3(H* h, unsigned u, unsigned v)
     { B& l=h->lh; l.n=&l; h->sz=u; h->ct=v; h->fh=h; h->uh=h; l.p=&l; }

// --- THE DECISIVE CELLS -----------------------------------------------------
// E1/E2 differ only in which parameter owns the referenced sub-object.
// E1 emits producer→consumer gap 1, E2 emits gap 3.
void e1(H* h, H* g, unsigned u, unsigned v)
     { B& l = h->lh; g->fh=(H*)&l; h->sz=u; h->ct=v; g->uh=(H*)&l; }
void e2(H* h, H* g, unsigned u, unsigned v)
     { B& l = g->lh; h->fh=(H*)&l; h->sz=u; h->ct=v; h->uh=(H*)&l; }
// F1/F2 are the CONTROLLED swap: identical statement structure, the two
// pointer parameters exchanged, so the only difference is which architectural
// register each role lands in. BOTH emit gap 1 — which refutes the rule E1/E2
// suggest ("the producer's base register decides"), because F2's producer
// reads r4 exactly as E2's does and is not delayed. F3/F4 are the same swap
// with a `li` producer, where the producer has no base register at all.
void f1(H* a, H* b, unsigned u, unsigned v) { B& l = a->lh; l.n=&l; b->sz=u; b->ct=v; l.p=&l; }
void f2(H* a, H* b, unsigned u, unsigned v) { B& l = b->lh; l.n=&l; a->sz=u; a->ct=v; l.p=&l; }
void f3(H* a, H* b, unsigned u) { a->sz=0; a->ct=u; b->sz=u; b->ct=u; }
void f4(H* a, H* b, unsigned u) { b->sz=0; b->ct=u; a->sz=u; a->ct=u; }

// --- xboxheap.cpp's constructor, minus the frame and the trailing call ------
void c3(H* h, unsigned size)
     { h->sz=size; h->fh=h; h->ct=0; h->uh=h; B& l=h->lh; l.n=&l; l.p=&l; }
void c9(H* h, unsigned u1, unsigned u2)
     { h->sz=u1; h->ct=u2; h->fh=h; h->uh=h; B& l=h->lh; l.n=&l; l.p=&l; }
EOF

GT_OUT="$out/sched.obj" "$repo_root/scripts/gt_capture.sh" "$out/sched.cpp" \
    $FLAGS /FAsc "/Fa$out/sched.cod" >/dev/null

grep -E "PROC NEAR|^  [0-9a-f]{5}" "$out/sched.cod"
echo
echo "listing: $out/sched.cod"
