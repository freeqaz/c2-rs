// **w-bdnz — THE COUNTED-`for` LOOP CLASS**, and the first body class this port
// emits whose lowering comes from a *reading of c2's algorithm* rather than a
// transcription of one workload function.
//
// `docs/whitebox/WB_LOOP_FINDINGS.md` (lane `wb-loop`, board #1900–#1907) read
// c2's counted-loop lowering as **three independent passes**, each with its own
// `-QX` disable switch and each leaving the other two's output byte-identical —
// obj-confirmed by two counterfactual runs over a frozen 36-cell grid (§7.7):
//
//     1  the rotated pre-test GUARD      lur.c             -NoLUR
//     2  the mtctr/bdnz CONVERSION       p2\ppc\lower.c    -QXnobdnz
//     3  the lwzu/stwu UPDATE FORM       p2\misc.c         -QXnopreinc
//
// This file is passes **1 and 2 and not 3**. `wb-loop` §9 item 1 states exactly
// that increment — *"shipping only rule 1 + rule 2 reproduces c2's obj exactly
// for every loop where the update form does not apply"* — and pass 3 is
// **declined by name**: §4.4/§7.5 put four rivals on a frozen ten-cell grid and
// elected NONE (RU0′ 8/10 and RU2 8/10 with *disjoint* failures, RU0′-b
// retracted, RU-H filed unfrozen). Every cell here has **no memory reference at
// all**, which is the largest boundary provably outside that open question.
//
// # What c2 emits for every cell below
//
//     mr     r11, r3
//     li     r3, INIT
//     cmp{w,lw}i cr6, r11, 0        <- pass 1, in cr6, with the LOOP's signedness
//     bclr   {4,25 | 12,26}         <- realised as a CONDITIONAL RETURN
//     mtctr  r11                    <- pass 2
//     <OP>   r3, r3, r4
//     bdnz   .-4                    <- pass 2
//     blr
//
// Eight words, thirty-two bytes, and the whole TU is `Port=Match` at `/O1`
// **and** at `/Ox` — the second is a departure from `whash_ptr_walk_loop.cpp`
// and `wdata_static_scan.cpp`, which refuse `/Ox` because their lanes graded no
// `/Ox` cell. This one grades every cell at both.
//
// # The axes this file crosses, and why each is a cell and not an argument
//
// * **the accumulate opcode** — seven of them, and the map to the emitted word
//   is injective. Two neighbours are NOT here and are in the `_neg` file with
//   what c2 does instead: `+=` **deletes the loop** (the accumulation strength-
//   reduces to a single `mullw`, guard and all) and `/=` is a six-word spine
//   with two `twi` traps;
// * **the counter's signedness** — `p_uns` is board **#1788**'s cell. It differs
//   from `p_sub` in the IL by **exactly one TYPE byte** (`86 41 74` against
//   `86 42 75`); the relational opcode and the branch are byte-identical; and
//   the obj differs in **two words**, `cmplwi`/`bclr 12,26` for
//   `cmpwi`/`bclr 4,25`. A reader built on `eat_int_like` — which accepts both
//   spellings, by design — would be right on 30 of 32 bytes;
// * **the init literal** at both `simm16` edges. `32768` is in the `_neg` file
//   because c2 emits `lis`/`ori` for it **and puts the guard compare between the
//   two words** — a different block, not a wider field;
// * **the braced body**, which is a different IL production (`53` … `54 04`) for
//   byte-identical text.
//
// # The label counter
//
// `IlFunction::label_slots` returns `None` for this shape, so a TU pairing it
// with a framed function is refused whole — `wbdnz_ctr_then_framed_neg.cpp` is
// that TU and this file is its separating control. The reason is **measured and
// is this lane's own**: the charge is MODE-DEPENDENT and `label_slots` has no
// mode parameter, so any `Some(k)` would be six wrong bytes at one of the two
// modes.
//
// **w-counted, 2026-08-15 — the mode-dependence holds and both magnitudes were
// wrong.** `work/w-bdnz/LABEL_LEAD.md`'s `+7`/`+8` was differenced across two
// TUs whose source text differs, so it reads `Δcharge + Δseed` (board **#3148**).
// Seed-cancelled over a one/two/three-loop series the charge is **2 at `/O1`**
// and **3 at `/Ox` and `/O2`**, and the oracle confirms both end to end: with
// the charge installed on `label_lead`, `2` is `match` at `/O1` and a live
// `mismatch` at `/Ox`/`/O2`, and `3` is exactly the mirror, while this file
// stays `match` under all five constants tried. **There is no constant**, and
// that is the `None` demonstrated rather than argued
// (`work/w-counted/charge_probe.txt`).

int p_sub(int n, int k) { int s = 0; for (int i = 0; i < n; ++i) s -= k; return s; }
int p_mul(int n, int k) { int s = 1; for (int i = 0; i < n; ++i) s *= k; return s; }
int p_and(int n, int k) { int s = -1; for (int i = 0; i < n; ++i) s &= k; return s; }
int p_or (int n, int k) { int s = 0; for (int i = 0; i < n; ++i) s |= k; return s; }
int p_xor(int n, int k) { int s = 0; for (int i = 0; i < n; ++i) s ^= k; return s; }
int p_shl(int n, int k) { int s = 1; for (int i = 0; i < n; ++i) s <<= k; return s; }
int p_sar(int n, int k) { int s = 1; for (int i = 0; i < n; ++i) s >>= k; return s; }
// board #1788 — one IL TYPE byte apart from `p_sub`, two obj words apart.
int p_uns(unsigned n, int k) { int s = 0; for (unsigned i = 0; i < n; ++i) s -= k; return s; }
// a different IL production (the body opens its own scope) for identical text.
int p_braced(int n, int k) { int s = 0; for (int i = 0; i < n; ++i) { s -= k; } return s; }
// both `simm16` edges of the `li` immediate.
int p_hi(int n, int k) { int s = 32767; for (int i = 0; i < n; ++i) s *= k; return s; }
int p_lo(int n, int k) { int s = -32768; for (int i = 0; i < n; ++i) s *= k; return s; }

// ---------------------------------------------------------------------------
// **w-counted — THE ACCEPTED-SET CROSS.** Everything above grades the seven
// accumulate opcodes on a SIGNED counter and exactly ONE unsigned cell
// (`p_uns`, `-=`). The class's free axes are opcode x counter signedness, so
// **six of the fourteen cells had never been compiled at any mode** and the
// `/Ox` half of the accepted set was graded on one axis and not the cross.
// That is what `w-slots`' found-and-not-taken #5 meant by *"whether its `/Ox`
// acceptance is even correct appears UNGRADED"*.
//
// It is graded now, and it is CORRECT: 20 of 20 in-class cells `match` at
// `/O1`, `/Ox`, `/Ox /Gy`, `/Ox /EHsc /GR`, `/O2` and the workload's own
// `/O1 /Oi /EHsc /GR` — 120 gradings against real `c2.dll`, `mismatch` 0 —
// while the `+=` cell that is OUTSIDE the class refuses at every one
// (`work/w-counted/cross_grid.txt`). The grid is demonstrably able to go red:
// making the guard ignore `counter_unsigned` reddens exactly the unsigned
// column at both modes and leaves the signed column `match`
// (`work/w-counted/codegen_mutants.sh`, G1).
//
// The rows below are the six that were missing, plus the unsigned partners of
// the braced body and both `simm16` edges. They are here rather than in a new
// fixture because a cell graded only in a lane's scratch is a cell that is
// graded once; on this file they ride all 18 gate lanes for good.
int q_mul_u(unsigned n, int k) { int s = 1;  for (unsigned i = 0; i < n; ++i) s *= k;  return s; }
int q_and_u(unsigned n, int k) { int s = -1; for (unsigned i = 0; i < n; ++i) s &= k;  return s; }
int q_or_u (unsigned n, int k) { int s = 0;  for (unsigned i = 0; i < n; ++i) s |= k;  return s; }
int q_xor_u(unsigned n, int k) { int s = 0;  for (unsigned i = 0; i < n; ++i) s ^= k;  return s; }
int q_shl_u(unsigned n, int k) { int s = 1;  for (unsigned i = 0; i < n; ++i) s <<= k; return s; }
int q_sar_u(unsigned n, int k) { int s = 1;  for (unsigned i = 0; i < n; ++i) s >>= k; return s; }
// the braced body and both `simm16` edges, on an unsigned counter.
int q_braced_u(unsigned n, int k) { int s = 0; for (unsigned i = 0; i < n; ++i) { s ^= k; } return s; }
int q_hi_u(unsigned n, int k) { int s = 32767;  for (unsigned i = 0; i < n; ++i) s *= k; return s; }
int q_lo_u(unsigned n, int k) { int s = -32768; for (unsigned i = 0; i < n; ++i) s *= k; return s; }
