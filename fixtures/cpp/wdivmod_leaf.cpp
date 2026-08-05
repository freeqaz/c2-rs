// **w-divmod** — the integer divide/modulo leaf, lifted out of the loop class.
//
// `codegen::ptr_walk_loop` already emitted the signed-`%` spine, but only as
// eight words welded into a twenty-word loop transcription. `w-hash` §9.1
// recorded why it could not be lifted: it had measured **two distinct `twi 6`
// placements** and could not name the discriminator, and "a leaf lowering owes
// it an answer".
//
// The answer is in `codegen::div_mod_leaf`'s module docs and it is a
// measurement, not a shipped rule — every body in this fixture has **both
// operands live-in**, which is the far side of the hoisting clause, so each of
// the four is a constant schedule with no free field at all.
//
// The four shapes are four **lengths**, which is the point of putting all four
// in one TU: 9 / 7 / 5 / 3 words. `/` is not `%` with a flag — it needs neither
// the `mullw` nor the `subf` — and unsigned needs neither the three-instruction
// `INT_MIN`/`-1` predicate nor the `twi 5` that reads it, because that overflow
// cannot arise.
//
//     smod  rotlwi r11,r3,1 · divw r10,r3,r4 · addi r11,r11,-1 ·
//           mullw r10,r10,r4 · andc r11,r4,r11 · twi 6,r4,0 ·
//           subf r3,r10,r3 · twi 5,r11,-1 · blr
//     sdiv  rotlwi r11,r3,1 · divw r3,r3,r4 · addi r11,r11,-1 ·
//           twi 6,r4,0 · andc r11,r4,r11 · twi 5,r11,-1 · blr
//     umod  divwu r11,r3,r4 · twi 6,r4,0 · mullw r11,r11,r4 ·
//           subf r3,r11,r3 · blr
//     udiv  divwu r3,r3,r4 · twi 6,r4,0 · blr
//
// **This fixture is in class at BOTH modes**, unlike `whash_ptr_walk_loop.cpp`,
// and that is its second job. `/Ox` emits the same mnemonics in the same order
// with a different register assignment (`mullw r8,r10,r4` for `/O1`'s
// `mullw r10,r10,r4`), so the emitter carries two mode tables — except for
// `udiv`, which is **byte-identical across the modes** and is the control that
// says the difference is a real allocation and not a capture artefact.
//
// The refusals are graded next door in `wdivmod_leaf_refuse.cpp`.
int smod(int a, int b) { return a % b; }
int sdiv(int a, int b) { return a / b; }
unsigned umod(unsigned a, unsigned b) { return a % b; }
unsigned udiv(unsigned a, unsigned b) { return a / b; }
