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
// is this lane's own**: the charge is MODE-DEPENDENT (+7 at `/O1`, +8 at `/Ox`,
// over `leaf-none`) and `label_slots` has no mode parameter, so any `Some(k)`
// would be six wrong bytes at one of the two modes. `work/w-bdnz/LABEL_LEAD.md`
// has the eight-row table, the straight-line control that prices the two locals
// at +2, and the `do/while` row that charges differently while emitting
// different text.

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
