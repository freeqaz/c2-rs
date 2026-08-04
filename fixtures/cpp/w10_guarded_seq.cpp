// **W10 — the FRAMED × BRANCHING cell.** The port's first body that is both.
//
// `work/w-frame/RANKING.md` §4 measured the gap this file closes: over the 105
// functions the port emitted byte-exact, **28 were framed, 2 branched, and ZERO
// were both**. Its entire branching capability was two *leaf* bodies from W8's
// `cond_tail`. **Ten of the seventeen FRONTIER TUs need the product.**
//
// The minimum instance is `mvp_call_seq.cpp`'s shipped `void two(){v0();v1();}`
// with a compare and a branch inserted, and the reference is 44 bytes:
//
//     7d8802a6  mflr  r12          the shipped Class A 96-byte frame
//     9181fff8  stw   r12,-8(r1)
//     9421ffa0  stwu  r1,-96(r1)
//     2f030000  cmpwi cr6,r3,0     <- the guard, BETWEEN the prologue and the
//     419a0008  bt    26,+8           sequence
//     4bffffed  bl    ?v0           the GUARDED call, REL24
//     4bffffe9  bl    ?v1           the sequence continues, REL24
//     38210060  addi  r1,r1,96
//     8181fff8  lwz   r12,-8(r1)
//     7d8803a6  mtlr  r12
//     4e800020  blr
//
// **The STRUCTURAL axes are crossed, not the value axis.** Board #198 and
// w-frame's §4.5.3 record the same defect twice: a fixture family exhaustive on
// the axis it varies and blind on the one it holds fixed reads as complete
// (`w6_rel_k.cpp` has twenty bodies, every one against a NON-zero literal, and
// could never have caught the `Rel::Le` zero fold). W9 already varied the
// relation and the signedness exhaustively — twelve cells, all graded — so this
// file holds those nearly fixed and varies the **structure**:
//
//   * **the guarded arm's setup**: empty, one register move, one literal, one
//     computed argument (`g1`…`g4`);
//   * **which formal is the scrutinee**: the first (r3 — the register the arm's
//     setup wants) and the second (r4 — one it does not). `g5` against `g2`;
//   * **the join's length**: 1, 2 and 3 unguarded calls after the guard
//     (`g1`, `n2`, `n3`). This is the axis that turned out to matter most —
//     see the note below;
//   * **the tail**: void, `return <literal>`, and the last call's own value
//     (`t1`, `t2`).
//
// The one thing this file pins that nothing else in the corpus does: **the
// guarded call's setup stays INSIDE the guarded block.** `g2` emits
// `cmpwi cr6,r3,0 ; bt 26,+12 ; mr r3,r4 ; bl ?a1 ; bl ?v1` — the `mr` is after
// the branch. An emitter that hoisted it above the compare would be the right
// length and the wrong program, and hoisting IS what c2 does the moment the arm
// needs a scratch park (`work/w-cross/p/probe2.cpp::s4`), which is why a guarded
// arm here takes at most one argument.
//
// Ten framed functions also exercise the compiler-label counter, whose stride
// was measured before this rung was written: holding the body shape fixed and
// varying only the branch count over 0/1/2/4 targets, the stride is **5**
// throughout — the same as a straight-line framed body, so a guarded sequence
// carries no `label_lead` and `coff.rs` is untouched (`probe3.cpp`, `L0`…`L4`).
//
// ---- what is NOT here, and why ------------------------------------------
//
// **An `else` arm.** It was built, graded against the real `c2`, and taken back
// out. `void e(int a){ if(a) v0(); else v1(); v2(); }` is **52 B with an
// intra-section `48000008`** at `/O1` and **68 B with no `b` at all** at `/Ox`
// and `/O2`, where c2 **tail-duplicates the join block and all four epilogue
// words into both arms**. That refutes `docs/OPT_MODE.md`'s standing claim that
// the modes "differ in exactly one rule … only a register field": here they
// differ in block structure. And the duplication has a threshold this lane did
// not crack — at `/Ox` a one-call join duplicates and a two-call join does not
// (`probe5.cpp`, `j1`/`j2`/`j3`), so the boundary is bracketed by one cell
// either side of a c2 cost model. `w10_guarded_seq_neg.cpp` holds the shape and
// the refusal; board #191's intra-section `b` stays open.

extern void v0();
extern void v1();
extern void v2();
extern void a1(int);
extern int i0();

// ---- the guarded arm's SETUP ---------------------------------------------

void g1(int a) { if (a != 0) v0(); v1(); }          // empty setup:      44 B
void g2(int a, int b) { if (a != 0) a1(b); v1(); }  // one `mr r3,r4`:   48 B
void g3(int a) { if (a != 0) a1(7); v1(); }         // one `li r3,7`
void g4(int a) { if (a != 0) a1(a + 1); v1(); }     // one `addi r3,r3,1`

// The scrutinee is the SECOND formal, so the arm's setup does not want its
// register — the separating cell against `g2`, where it does. The compare must
// read r4 and the setup must still be r3's.
void g5(int a, int b) { if (b != 0) a1(a); v1(); }

// ---- the JOIN's length ----------------------------------------------------
// The axis the `else` form turned on: at `/Ox` a one-call join is the cell that
// tail-duplicates. A one-armed guard does not duplicate at any join length —
// measured at 1, 2 and 3 in both modes — so the three cells here are the
// control that says the mode-dependence belongs to the `else` and not to the
// guard.

void n2(int a) { if (a != 0) v0(); v1(); v2(); }
void n3(int a) { if (a != 0) v0(); v1(); v2(); v1(); }

// ---- the SIGNEDNESS of the guard's compare -------------------------------
// Added AFTER `work/w-frame/sweep.py` was run on this rung's first draft, which
// is the whole point of running it: over 3,418 coverage regions it found
// exactly one EMISSION line this rung added with no coverage under the GRADED
// profile — `seq_guard_emit`'s `encode_cmplwi` arm. Every cell above compares
// an `int`, so the guard had been graded on `cmpwi` (2f……) and never once on
// `cmplwi` (2b……). That is `branch_sense`'s shape exactly — written, passing a
// unit test that compares the port's table to itself, never seen by the oracle
// — in the rung that cites it. w-frame's row F-c, applied to its author's
// successor and firing on the first try.
//
// `p1` is a pointer scrutinee, which is UNSIGNED because the operand's TYPE
// triple says so and not because of the relational opcode (docs/CFG_SHAPE.md
// §3.2). `u1` is an `unsigned` against a NON-ZERO literal, so the immediate
// field of `cmplwi` is graded here too — the same cell W9 added for the
// tail-call form and the one that would catch a `u > 7` -> `u >= 8`
// canonicalization.

void p1(void *p) { if (p != 0) v0(); v1(); }
void u1(unsigned a) { if (a != 7) v0(); v1(); }
void u2(unsigned a) { if (a >= 7) v0(); v1(); }

// ---- the TAIL, crossed with a guard ---------------------------------------
// `SeqTail` is orthogonal to the guard and is therefore varied separately: a
// literal return (`li r3,k` after the last `bl`) and the last call's own value
// (no post-op at all).

int t1(int a) { if (a != 0) v0(); v1(); return 5; }
int t2(int a) { if (a != 0) v0(); return i0(); }
