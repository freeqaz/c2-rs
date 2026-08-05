// **Positive** — the SCHEDULED store run. Every function here must emit, and
// the whole obj must be byte-exact.
//
// This is the first fixture in the project whose emitted **order** is decided by
// a model rather than by the source. `w25_store_leaf.cpp` and its neighbours
// grade a lowering that walks the statements as written; every function below
// comes out in an order the source does not give, or with its `li`s placed
// somewhere the source cannot suggest, or both.
//
// Board **#642**. The models it composes, each with the holdout that licenses
// it (`docs/ORDER.md`, `docs/ALLOC.md`, `docs/SYMBOL.md`):
//
//   store_order    which store goes where           561/561   (w-order2)
//   producer_order which producer is emitted first   #582      (w-sym)
//   layout_slots   where producers sit among stores 24,891/24,891 (w-frame2)
//   allocate       which register each producer takes 250/250  (w-alloc)
//
// ## Why this file is not a list of cells
//
// The four models disagree with each other in sign, and a fixture set that did
// not separate them would pass on any one of them alone:
//
//   * the **rank** (which producer is emitted first) breaks a use-count tie by
//     FORWARD source order; the **allocation** breaks the same tie among
//     constants by REVERSE source order. `t_perm2` is the cell where the two
//     orders disagree — `1` is emitted first and takes `r10`, `2` is emitted
//     second and takes `r11`. A port that used one order for both is wrong here
//     and right everywhere a count is unique.
//   * the **store order** and the **producer order** are different questions.
//     `t_disp` moves a store and leaves the producers alone; `t_lead2` moves a
//     producer and leaves the stores alone.
//   * `u` — how many head store slots a producer may be interleaved into — is
//     the LEADING RUN of unproduced stores in the FINAL order, capped at 2
//     (board #584), and NOT the count of them. `t_ilv2` and `t_ilv3` sit either
//     side of the cap.
//
// ## What each function discriminates
//
// `t_two` / `t_three` — the base case: two and three distinct literals, no
//   filler. `u` is 0, so the producers come out CONTIGUOUSLY ahead of every
//   store, in rank order, taking r11/r10/r9 descending. A port that interleaved
//   them unconditionally fails here.
//
// `t_perm2` / `t_perm3` — the ALLOCATION permutations. `{1,2,1,2}` puts the
//   *second* value in r11 (`docs/ALLOC.md` clause 4, the reverse tiebreak among
//   shared constants), and `{1,2,3,2,1}` puts `2` in r11 with `1` in r10 and
//   `3` in r9. These are two of the four cells that refuted the four fitted
//   allocation rules `leaf_store.rs` used to carry.
//
// `t_disp` / `t_disp2` — the STORE ORDER moving. `{1,1,2}` leaves source order
//   alone but `{1,2,2}` does not: the count-2 value takes rank 0, so the
//   count-1 store is displaced out of position 0 and the run comes back
//   `S1 S0 S2`. `t_disp2` is `{1,1,2,2,2}`, which comes back `S2 S0 S1 S3 S4`.
//
// `t_mix1` / `t_mix1r` — one literal beside one formal, both orders. This is
//   the cell the file this fixture supersedes named as the reason to refuse:
//   `{ s->a=1; s->b=u; }` is `li r11,1 ; stw r4,4(r3) ; stw r11,0(r3)` — the two
//   statements in the OPPOSITE order to the source. The reverse spelling comes
//   back in source order, so the pair separates "always reorder" from the rule.
//
// `t_ilv2` / `t_ilv3` / `t_ilv4` — the INTERLEAVE, which is `layout_slots` and
//   nothing else. With two unproduced stores in the lead, `u` is 2 and the first
//   two producers go one apiece immediately before store slots 0 and 1;
//   everything past the second producer is emitted contiguously before slot 2.
//   `t_ilv4` has three fillers and `u` is still 2 — the cap, not the count.
//
// `t_wide` — a run whose ONLY producer is wide. `lis`+`ori` is two words for one
//   producer and they stay whole here. Its neighbour — a wide literal BESIDE a
//   narrow one — is refused, and that refusal is a measured regime rather than
//   caution: c2 interleaves the halves (`lis r11 ; li r10 ; ori r11`) and emits
//   the stores in the opposite order to the model's. See
//   `docs/rungs/_2026-08-05-w-wire-prereg.md` §4.1; it is the counterexample
//   that fired.
//
// `t_widths` — the widths are per statement and the schedule does not care:
//   `stb`/`sth`/`stw`/`std` pick their opcode from the stored TYPE while ORDER
//   picks the position from the producer. A port that keyed either on the other
//   passes every uniform-width cell above and fails this one.
//
// `t_share` — the pre-existing one-value class, kept here on purpose: one
//   literal shared by every store is `li r11,k ; stw ; stw ; stw` in SOURCE
//   order, and the widening must not disturb it. This is the byte-graded
//   regression check on the class that was already in.
//
// The exhaustive version of this file is `work/w-wire/grid.py` — every word of
// length 2..4 over {formal, 1, 2, 3} plus the length-5/6 shapes, 360 probe
// functions, `Port=Match` at `/Ox` and at the workload's own `/O1 /Oi /EHsc /GR`.
// It is a lane instrument and is not committed as a fixture: the gate grades
// each fixture in 18 mode lanes, and the cells above are the ones that
// discriminate.

struct S {
    unsigned m0, m1, m2, m3, m4, m5, m6, m7;
};

struct W {
    unsigned char c;
    unsigned short h;
    unsigned a;
    unsigned long long q;
};

// --- the base case: producers contiguous, r11/r10/r9 descending -------------
void t_two(S* s) { s->m0 = 1; s->m1 = 2; }
void t_three(S* s) { s->m0 = 1; s->m1 = 2; s->m2 = 3; }

// --- the ALLOCATION permutations (reverse tiebreak among shared constants) ---
void t_perm2(S* s) { s->m0 = 1; s->m1 = 2; s->m2 = 1; s->m3 = 2; }
void t_perm3(S* s) { s->m0 = 1; s->m1 = 2; s->m2 = 3; s->m3 = 2; s->m4 = 1; }

// --- the STORE ORDER moving --------------------------------------------------
void t_disp(S* s) { s->m0 = 1; s->m1 = 2; s->m2 = 2; }
void t_disp2(S* s) { s->m0 = 1; s->m1 = 1; s->m2 = 2; s->m3 = 2; s->m4 = 2; }

// --- one literal beside one formal, both orders -------------------------------
void t_mix1(S* s, unsigned u) { s->m0 = 1; s->m1 = u; }
void t_mix1r(S* s, unsigned u) { s->m0 = u; s->m1 = 1; }

// --- the INTERLEAVE: u is the LEADING RUN, capped at 2 ------------------------
void t_ilv2(S* s, unsigned u, unsigned v) { s->m0 = 1; s->m1 = 2; s->m2 = u; s->m3 = v; }
void t_ilv3(S* s, unsigned u, unsigned v) { s->m0 = u; s->m1 = 1; s->m2 = 2; s->m3 = v; }
void t_ilv4(S* s, unsigned u, unsigned v) { s->m0 = u; s->m1 = v; s->m2 = 1; s->m3 = 2; }

// --- a wide literal, alone: lis+ori stays whole -------------------------------
void t_wide(S* s) { s->m0 = 100000u; s->m1 = 100000u; }

// --- the widths are per statement and the schedule does not care --------------
void t_widths(W* w) { w->c = 1; w->h = 2; w->a = 3; w->q = 1; }

// --- the pre-existing one-value class, unchanged ------------------------------
void t_share(S* s) { s->m0 = 9; s->m1 = 9; s->m2 = 9; }
