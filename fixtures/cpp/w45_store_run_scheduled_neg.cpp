// **Negative** — the boundary of the SCHEDULED store run. Every function here
// MUST be out of class, and the file must never mismatch.
//
// The positive class is `fixtures/cpp/w45_store_run_scheduled.cpp`: a run of
// stores through **one** base symbol, carrying up to **three** distinct
// single-word literals mixed freely with formals, emitted in
// `c2_core::codegen::order`'s permutation with `codegen::alloc`'s registers.
// Board **#642**.
//
// Every function below is one step outside that, and **each step is a measured
// different regime rather than caution** — which is the standard `docs/ALLOC.md`
// and `docs/ORDER.md` hold their own refusals to. The observations are
// `work/w-wire/boundary_probe.py`, real `c2.dll` under wibo, and where two modes
// are quoted they were compiled at both.
//
// ## 1. Four distinct literals — c2 REUSES a freed register, and the modes disagree
//
//   void n_four(S* s) { s->m0=1; s->m1=2; s->m2=3; s->m3=4; }
//
//     /O1   li r11 ; li r10 ; stw r11,0 ; li r9 ; li r11 ; stw r10,4 ; stw r9,8 ; stw r11,12
//     /Ox   li r11 ; li r10 ; li r9 ; stw r11,0 ; li r8 ; stw r10,4 ; stw r9,8 ; stw r8,12
//
// `/O1` hands the fourth value **r11 again**, reusing the register its first
// store has just freed; `/Ox` takes a fresh **r8**. That is board **#541**, and
// the *mode split* is board **#641**: the models agree at `/O1` and `/Ox` on
// every cell inside the domain (18 of 18) and are known to disagree here. So
// `MAX_MODELLED_PRODUCERS = 3` is not a round number — it is where the two
// compilers this port must satisfy stop giving the same answer.
//
// ## 2. A multi-word literal beside another producer — THE COUNTEREXAMPLE THAT FIRED
//
//   void n_wide(S* s) { s->m0 = 100000u; s->m1 = 1; }
//
//     lis r11 ; li r10 ; ori r11 ; stw r10,4(r3) ; stw r11,0(r3)
//
//   void n_wide2(S* s) { s->m0 = 100000u; s->m1 = 200000u; }
//
//     lis r11 ; lis r10 ; ori r11 ; ori r10 ; stw r11,0 ; stw r10,4
//
// Two independent facts break at once. The `lis`/`ori` pair is **SPLIT** — c2
// interleaves the halves of two wide loads — so a producer is not one
// contiguous instruction and `layout_slots`, which places producers by *index*,
// cannot express the sequence at all. And the first cell's **store order is
// `[1, 0]`** where `store_order` says source order.
//
// Every grid behind ORDER and ALLOC used single-word `li` values and **neither
// document said so**; both now carry the restriction as a banner (board #644).
// `docs/rungs/_2026-08-05-w-wire-prereg.md` §4 registered this cell predicting
// it was NOT a boundary. It is. Had the widening shipped without the probe, it
// would have been a live wrong emit — board #232's exact shape.
//
// A run whose **only** producer is wide is NOT here: it is in the positive
// fixture as `t_wide`, because one live range has nothing to interleave with.
//
// ## 3. The pool boundary — c2 descends into a formal's own register
//
//   void n_pool(S* s, unsigned a,b,c,d,e,f,g) { s->m0 = 1; s->m1 = 2; }
//
//     li r11 ; li r10 ; stw r11,0 ; stw r10,4
//
// Eight formals hold r3..r10, so the pool above them is **r11 alone** and two
// producers are wanted. c2 takes r10 anyway — the register its own seventh
// formal is sitting in — because that formal is dead. Reconstructing this needs
// a liveness model, which `docs/ALLOC.md` §3 names as open, so the port refuses
// rather than allocating one register short.
//
// ## 4. Two base symbols with two distinct literals
//
//   void n_2sym(S* s, S* t) { s->m0 = 1; t->m1 = 2; }
//
// The store order and the cross-symbol PIN both answer here — `w-sym` measured
// them over 7,589 cells — but the **LAYOUT** is only exact under
// `nsw <= MAX_SYMBOL_CROSSINGS`, and the parser's gate is drawn at *one symbol*
// so that `nsw` is identically 0 by construction rather than by a check the
// parser cannot make (it cannot see `c2-core`). Board **#621** measured a rival
// clause that answers the whole multi-symbol population at 99.44 % fit /
// 97.30 % holdout and **deliberately did not ship it**: 99 % is a rule with a
// residual, and an emitter fed a 99 % layout emits wrong bytes on the other 1 %.
// This lane did not resurrect it to widen coverage, and this function is the
// standing check that it did not.
//
// ## 5. A literal in company the schedule does not claim
//
//   void n_mixload(S* d, S* s) { d->m0 = s->m0; d->m1 = 2; }
//
//     lwz r11,0(r4) ; li r10,2 ; stw r10,4(r3) ; stw r11,0(r3)
//
// A *loaded* value is a different regime — it is hoisted, its store sinks past
// the next statement, and the literal in its company gets a second scratch
// register where a pure run uses only r11. The scheduled path claims a stream
// only when **every** group is value-simple, so this falls through to the walk,
// which refuses it. `docs/IL_STORE_LEAF.md`; the parser draws the same gate.

struct S {
    unsigned m0, m1, m2, m3, m4, m5, m6, m7;
};

// 1. four distinct literals — board #541, and the two modes disagree
void n_four(S* s) { s->m0 = 1; s->m1 = 2; s->m2 = 3; s->m3 = 4; }

// 2. a multi-word literal beside another producer — board #644
void n_wide(S* s) { s->m0 = 100000u; s->m1 = 1; }
void n_wide2(S* s) { s->m0 = 100000u; s->m1 = 200000u; }

// 3. the pool boundary — c2 reuses a dead formal's register
void n_pool(S* s, unsigned a, unsigned b, unsigned c, unsigned d, unsigned e,
            unsigned f, unsigned g) {
    s->m0 = 1;
    s->m1 = 2;
}

// 4. two base symbols with two distinct literals — board #621's refused region
void n_2sym(S* s, S* t) { s->m0 = 1; t->m1 = 2; }

// 5. a literal in a loaded value's company
void n_mixload(S* d, S* s) { d->m0 = s->m0; d->m1 = 2; }
